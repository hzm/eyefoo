use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::state::AppState;

// ─── 窗口创建 / 保活 ──────────────────────────────────────────────────────────

pub fn ensure_rest_windows(app: &AppHandle) -> tauri::Result<()> {
    let old_labels = app
        .state::<AppState>()
        .rest_window_labels
        .lock()
        .unwrap()
        .clone();

    if let Ok(monitors) = app.available_monitors() {
        let mut labels = Vec::new();
        for (i, monitor) in monitors.iter().enumerate() {
            let label = format!("rest-{}", i);
            let size = monitor.size();
            let position = monitor.position();

            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.hide();
            } else {
                let win =
                    WebviewWindowBuilder::new(app, &label, WebviewUrl::App("rest.html".into()))
                        .title("EyeFoo Rest")
                        .inner_size(size.width as f64, size.height as f64)
                        .position(position.x as f64, position.y as f64)
                        .visible(false)
                        .resizable(false)
                        .decorations(false)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .background_color(tauri::webview::Color(0, 0, 0, 255))
                        .build()?;
                let _ = win.set_fullscreen(true);
            }
            labels.push(label);
        }

        // Close only windows whose monitors disappeared. Existing windows are
        // reused so a close-and-immediately-recreate race cannot hide rest.
        for label in old_labels {
            if !labels.contains(&label) {
                if let Some(window) = app.get_webview_window(&label) {
                    let _ = window.close();
                }
            }
        }
        *app.state::<AppState>().rest_window_labels.lock().unwrap() = labels;
    }
    Ok(())
}

pub fn ensure_reminder_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("reminder").is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "reminder", WebviewUrl::App("index.html".into()))
        .title("EyeFoo")
        .inner_size(360.0, 300.0)
        .visible(false)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .center()
        .skip_taskbar(true)
        .background_color(tauri::webview::Color(45, 45, 45, 255))
        .build()?;
    Ok(())
}

pub fn show_settings_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("EyeFoo 设置")
        .inner_size(460.0, 620.0)
        .visible(true)
        .resizable(false)
        .decorations(true)
        .always_on_top(false)
        .skip_taskbar(false)
        .center()
        .build()?;
    Ok(())
}

// ─── 窗口唤醒（用 emit 替换 eval，类型安全、不易出错）──────────────────────────

/// 通知提醒窗口激活倒计时，携带当前推迟次数
fn wake_reminder_window(
    window: &tauri::WebviewWindow,
    postponed: u64,
    timeout: u64,
    postpone_5: u64,
    postpone_8: u64,
    max_postpones: u64,
) {
    let _ = window.emit(
        "activate-reminder",
        serde_json::json!({
            "postponed": postponed,
            "timeout": timeout,
            "postpone_5": postpone_5,
            "postpone_8": postpone_8,
            "max_postpones": max_postpones
        }),
    );
}

/// 通知休息窗口重新同步剩余时间（取代原来的 eval 调用）
fn wake_rest_window(window: &tauri::WebviewWindow) {
    let _ = window.emit("restart-polling", ());
}

// ─── 显示 / 隐藏 ──────────────────────────────────────────────────────────────

pub fn show_reminder_window(app: &AppHandle) -> tauri::Result<()> {
    ensure_reminder_window(app)?;
    if let Some(win) = app.get_webview_window("reminder") {
        let postponed = app.state::<AppState>().manager.get_postponed();
        let settings = app.state::<AppState>().settings.lock().unwrap().clone();
        let _ = win.show();
        let _ = win.set_focus();
        wake_reminder_window(
            &win,
            postponed,
            settings.reminder_timeout_seconds,
            settings.postpone_5_minutes,
            settings.postpone_8_minutes,
            settings.max_postpones,
        );
    }
    Ok(())
}

pub fn show_forced_rest_window(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();

    if state.is_showing_rest.load(Ordering::SeqCst) {
        return Ok(());
    }
    if state.manager.is_exiting_rest.load(Ordering::SeqCst) {
        return Ok(());
    }

    // 递增 generation，令所有旧倒计时线程自动退出
    let my_gen = state.rest_generation.fetch_add(1, Ordering::SeqCst) + 1;

    state.is_showing_rest.store(true, Ordering::SeqCst);
    state.rest_countdown_active.store(true, Ordering::SeqCst);
    let rest_duration = state.settings.lock().unwrap().rest_duration_secs();
    let exit_unlock = state.settings.lock().unwrap().exit_unlock_seconds;
    state.rest_remaining.store(rest_duration, Ordering::SeqCst);

    ensure_rest_windows(app)?;

    let labels = state.rest_window_labels.lock().unwrap().clone();
    for label in &labels {
        if let Some(win) = app.get_webview_window(label) {
            let _ = win.show();
            let _ = win.set_fullscreen(true);
            let _ = win.set_focus();
            wake_rest_window(&win);
        }
    }

    let _ = app.emit(
        "rest-started",
        serde_json::json!({ "remaining": rest_duration, "duration": rest_duration, "exit_unlock": exit_unlock }),
    );

    // 启动倒计时线程，携带 my_gen 标记；若 generation 已更新则线程自动退出
    let countdown_active = state.rest_countdown_active.clone();
    let rest_remaining = state.rest_remaining.clone();
    let rest_generation = state.rest_generation.clone();
    let app_emit = app.clone();

    std::thread::spawn(move || {
        let mut remaining = rest_duration;
        while remaining > 0 {
            if rest_generation.load(Ordering::SeqCst) != my_gen {
                return; // 被新一代线程取代，退出
            }
            if !countdown_active.load(Ordering::SeqCst) {
                return; // 被 exit_forced_rest 主动停止
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            // sleep 后再次确认，避免退出后仍写入状态
            if rest_generation.load(Ordering::SeqCst) != my_gen {
                return;
            }
            if !countdown_active.load(Ordering::SeqCst) {
                return;
            }
            remaining -= 1;
            rest_remaining.store(remaining, Ordering::SeqCst);
            let _ = app_emit.emit("rest-tick", serde_json::json!({ "remaining": remaining }));
        }
        // 只有当前代线程才能触发 rest-finished
        if remaining == 0 && rest_generation.load(Ordering::SeqCst) == my_gen {
            let _ = app_emit.emit("rest-finished", ());
        }
    });

    Ok(())
}

pub fn hide_rest_windows(app: &AppHandle) {
    let labels = app
        .state::<AppState>()
        .rest_window_labels
        .lock()
        .unwrap()
        .clone();
    for label in &labels {
        if let Some(win) = app.get_webview_window(label) {
            let _ = win.hide();
        }
    }
}

pub fn hide_window(app: &AppHandle, label: &str) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.hide();
    }
}

// ─── 业务触发 ─────────────────────────────────────────────────────────────────

/// 触发强制休息：重置周期、清除退出标志、隐藏提醒窗口，然后显示休息界面
pub fn trigger_forced_rest(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.manager.reset_cycle();
    // 必须在 show_forced_rest_window 之前显式清除，因为 reset_cycle 不动此标志
    state.manager.is_exiting_rest.store(false, Ordering::SeqCst);
    hide_window(app, "reminder");
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = show_forced_rest_window(&app2) {
            eprintln!("failed to start forced rest: {error}");
        }
    });
}

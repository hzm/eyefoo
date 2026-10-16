use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::constants::SUSPEND_THRESHOLD_SECS;
use crate::state::AppState;

// ─── 窗口创建 / 保活 ──────────────────────────────────────────────────────────

/// 强制激活窗口并带到前台。
///
/// 定时弹出的窗口没有前台权限，直接 SetForegroundWindow 会被 Windows 前台锁
/// 拒绝：窗口显示但不激活，第一次点击只用于激活窗口、被系统吞掉。
/// AttachThreadInput 把当前线程附加到前台线程的输入队列，是绕过前台锁的
/// 标准做法（Chromium、AutoHotkey 等均用此模式）。
#[cfg(windows)]
pub fn force_foreground(window: &tauri::WebviewWindow) {
    use std::ffi::c_void;
    use std::ptr::null_mut;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, pid: *mut u32) -> u32;
        fn AttachThreadInput(id_attach: u32, id_attach_to: u32, f_attach: i32) -> i32;
        fn BringWindowToTop(hwnd: *mut c_void) -> i32;
        fn SetForegroundWindow(hwnd: *mut c_void) -> i32;
        fn SetActiveWindow(hwnd: *mut c_void) -> *mut c_void;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThreadId() -> u32;
    }

    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let hwnd = hwnd.0 as *mut c_void;
    unsafe {
        let foreground = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground, null_mut());
        let current_thread = GetCurrentThreadId();
        let need_attach = foreground_thread != 0 && foreground_thread != current_thread;
        if need_attach {
            AttachThreadInput(current_thread, foreground_thread, 1);
        }
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        SetActiveWindow(hwnd);
        if need_attach {
            AttachThreadInput(current_thread, foreground_thread, 0);
        }
    }
}

#[cfg(not(windows))]
pub fn force_foreground(window: &tauri::WebviewWindow) {
    let _ = window.set_focus();
}

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

            let mut need_create = true;
            if let Some(window) = app.get_webview_window(&label) {
                // 显示器位置/尺寸变化（热插拔、缩放调整）时销毁旧窗口重建；
                // 窗口此时处于隐藏状态，不存在「先关后建」遮挡休息界面的竞态
                let mismatch = match (window.outer_size(), window.outer_position()) {
                    (Ok(s), Ok(p)) => {
                        s.width != size.width
                            || s.height != size.height
                            || p.x != position.x
                            || p.y != position.y
                    }
                    _ => true,
                };
                if mismatch {
                    let _ = window.destroy();
                } else {
                    let _ = window.hide();
                    need_create = false;
                }
            }
            if need_create {
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

        // Close only windows whose monitors disappeared. Geometry-matched
        // windows are reused; rebuilding happens only while hidden, so a
        // close-and-immediately-recreate race cannot hide rest.
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

/// 首次启动引导：告诉用户应用驻留在托盘，避免「双击后没反应」的误解
pub fn show_welcome_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("welcome") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "welcome", WebviewUrl::App("welcome.html".into()))
        .title("欢迎使用 EyeFoo")
        .inner_size(420.0, 300.0)
        .visible(true)
        .resizable(false)
        .decorations(true)
        .always_on_top(false)
        .skip_taskbar(false)
        .center()
        .background_color(tauri::webview::Color(0x20, 0x20, 0x20, 255))
        .build()?;
    Ok(())
}

pub fn show_settings_window(app: &AppHandle) -> tauri::Result<()> {    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("EyeFoo 设置")
        .inner_size(460.0, 670.0)
        .visible(true)
        .resizable(false)
        .decorations(true)
        .always_on_top(false)
        .skip_taskbar(false)
        .center()
        .background_color(tauri::webview::Color(0x20, 0x20, 0x20, 255))
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
        force_foreground(&win);
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
            wake_rest_window(&win);
        }
    }
    // 只激活主屏窗口，避免多屏轮流抢焦点；
    // 用 force_foreground 绕过前台锁，否则「退出休息」的第一次点击会被吞
    if let Some(first) = labels.first() {
        if let Some(win) = app.get_webview_window(first) {
            force_foreground(&win);
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
        let mut last_tick = std::time::Instant::now();
        while remaining > 0 {
            if rest_generation.load(Ordering::SeqCst) != my_gen {
                return; // 被新一代线程取代，退出
            }
            if !countdown_active.load(Ordering::SeqCst) {
                return; // 被 finish_rest 主动停止
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            // sleep 后再次确认，避免退出后仍写入状态
            if rest_generation.load(Ordering::SeqCst) != my_gen {
                return;
            }
            if !countdown_active.load(Ordering::SeqCst) {
                return;
            }
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_tick);
            last_tick = now;
            // 系统休眠或进程被长时间挂起的时间不计入休息计时
            if elapsed <= std::time::Duration::from_secs(SUSPEND_THRESHOLD_SECS) {
                remaining = remaining.saturating_sub(elapsed.as_secs().max(1));
            }
            rest_remaining.store(remaining, Ordering::SeqCst);
            let _ = app_emit.emit("rest-tick", serde_json::json!({ "remaining": remaining }));
        }
        // 只有当前代线程才能结束休息；由 Rust 侧直接收尾，
        // 不再依赖前端收到 rest-finished 事件后回调
        if remaining == 0 && rest_generation.load(Ordering::SeqCst) == my_gen {
            if app_emit
                .state::<AppState>()
                .settings
                .lock()
                .unwrap()
                .sound_enabled
            {
                crate::notify::rest_end_sound();
            }
            finish_rest(&app_emit);
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

/// 结束强制休息：停止倒计时、重置工作周期、隐藏休息窗口。
/// 自然结束（倒计时线程）与手动退出（exit_forced_rest）共用，
/// is_showing_rest 的原子交换保证重复调用幂等。
pub fn finish_rest(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.is_showing_rest.swap(false, Ordering::SeqCst) {
        return;
    }
    // 递增 generation，令当前倒计时线程在下次 sleep 结束后立即退出
    state.rest_generation.fetch_add(1, Ordering::SeqCst);
    state.rest_countdown_active.store(false, Ordering::SeqCst);
    state.manager.exit_rest_mode();
    hide_rest_windows(app);
    // 通知页面清空倒计时显示，避免下次休息时闪出上次的残留数字
    let _ = app.emit("rest-ended", ());
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

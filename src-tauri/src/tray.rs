use std::sync::atomic::Ordering;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::state::AppState;
#[cfg(debug_assertions)]
use crate::windows::show_reminder_window;
use crate::windows::{hide_window, show_forced_rest_window, show_settings_window};

pub fn build_tray(app: &tauri::App) -> tauri::Result<tauri::tray::TrayIcon> {
    let pause_30 = MenuItem::with_id(app, "pause_30", "暂停 30 分钟", true, None::<&str>)?;
    let pause_60 = MenuItem::with_id(app, "pause_60", "暂停 1 小时", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "恢复计时", true, None::<&str>)?;
    let rest_now = MenuItem::with_id(app, "rest_now", "立即休息", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    #[cfg(debug_assertions)]
    let test_reminder =
        MenuItem::with_id(app, "test_reminder", "测试提醒 (调试)", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    #[cfg(debug_assertions)]
    let menu = Menu::with_items(
        app,
        &[
            &pause_30,
            &pause_60,
            &resume,
            &rest_now,
            &test_reminder,
            &settings,
            &quit,
        ],
    )?;
    #[cfg(not(debug_assertions))]
    let menu = Menu::with_items(
        app,
        &[&pause_30, &pause_60, &resume, &rest_now, &settings, &quit],
    )?;

    TrayIconBuilder::with_id("eye-foo-tray")
        .icon(app.default_window_icon().expect("no default icon").clone())
        .menu(&menu)
        .tooltip("EyeFoo")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "pause_30" => {
                let state = app.state::<AppState>();
                if !state.is_showing_rest.load(Ordering::SeqCst) {
                    state.manager.pause_for(30);
                    hide_window(app, "reminder");
                }
            }
            "pause_60" => {
                let state = app.state::<AppState>();
                if !state.is_showing_rest.load(Ordering::SeqCst) {
                    state.manager.pause_for(60);
                    hide_window(app, "reminder");
                }
            }
            "resume" => {
                app.state::<AppState>().manager.resume();
            }
            "rest_now" => {
                let state = app.state::<AppState>();
                state.manager.reset_cycle();
                // 显式清除退出标志，允许立即显示休息界面
                state.manager.is_exiting_rest.store(false, Ordering::SeqCst);
                hide_window(app, "reminder");
                if let Err(error) = show_forced_rest_window(app) {
                    eprintln!("failed to start immediate rest: {error}");
                }
            }
            "settings" => {
                let _ = show_settings_window(app);
            }
            #[cfg(debug_assertions)]
            "test_reminder" => {
                let _ = show_reminder_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)
}

fn on_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        update_tray_tooltip(tray.app_handle());
    }
}

pub fn update_tray_tooltip(app: &AppHandle) {
    let state = app.state::<AppState>();
    let tooltip = if state.manager.is_paused() {
        let pr = state.manager.paused_remaining();
        if pr > 0 {
            format!("EyeFoo — 已暂停 {:02}:{:02}", pr / 60, pr % 60)
        } else {
            "EyeFoo — 已暂停".to_string()
        }
    } else {
        let remaining = state.manager.get_remaining();
        format!("EyeFoo — 剩余 {:02}:{:02}", remaining / 60, remaining % 60)
    };

    if let Some(tray) = state.tray_icon.lock().unwrap().as_ref() {
        let _ = tray.set_tooltip(Some(&tooltip));
    };
}

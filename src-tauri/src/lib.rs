mod autostart;
mod commands;
mod constants;
mod notify;
mod single_instance;
mod state;
mod timer;
mod tray;
mod windows;

use tauri::{Listener, Manager};

use commands::*;
use state::{AppSettings, AppState};
use timer::TimerManager;
use tray::build_tray;
use windows::{
    ensure_reminder_window, ensure_rest_windows, show_forced_rest_window, show_reminder_window,
};

pub fn run() {
    if !single_instance::acquire() {
        return;
    }

    let settings = std::sync::Arc::new(std::sync::Mutex::new(AppSettings::default()));
    let timer = TimerManager::new(settings.clone());

    tauri::Builder::default()
        .manage(AppState::new(timer.clone(), settings.clone()))
        .invoke_handler(tauri::generate_handler![
            postpone_5min,
            postpone_8min,
            exit_forced_rest,
            start_forced_rest,
            pause_30min,
            pause_60min,
            get_rest_remaining,
            show_reminder,
            get_sound_enabled,
            set_sound_enabled,
            get_autostart_enabled,
            set_autostart_enabled,
            get_settings,
            set_settings,
        ])
        .setup(move |app| {
            if let Ok(config_dir) = app.path().app_config_dir() {
                app.state::<AppState>()
                    .load_settings(config_dir.join("settings.json"));
            }
            app.state::<AppState>()
                .settings
                .lock()
                .unwrap()
                .autostart_enabled = autostart::is_enabled();
            timer.reset_cycle();
            timer.start_timer(app.handle().clone());
            let _ = ensure_reminder_window(app.handle());
            let _ = ensure_rest_windows(app.handle());

            let app_handle = app.handle().clone();
            let app_handle2 = app_handle.clone();
            let app_handle3 = app_handle.clone();
            let app_handle4 = app_handle.clone();

            app.listen("reminder-triggered", move |_| {
                if app_handle4
                    .state::<AppState>()
                    .settings
                    .lock()
                    .unwrap()
                    .sound_enabled
                {
                    notify::reminder_sound();
                }
                let _ = show_reminder_window(&app_handle);
            });

            app.listen("forced-rest-triggered", move |_| {
                if let Err(error) = show_forced_rest_window(&app_handle2) {
                    eprintln!("failed to start forced rest: {error}");
                }
            });

            // timer-tick → 更新托盘 tooltip
            app.listen("timer-tick", move |event| {
                let state = app_handle3.state::<AppState>();
                let payload: serde_json::Value =
                    serde_json::from_str(event.payload()).unwrap_or_default();

                let tooltip = if payload
                    .get("paused")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let pr = payload
                        .get("paused_remaining")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if pr > 0 {
                        format!("EyeFoo — 已暂停 {:02}:{:02}", pr / 60, pr % 60)
                    } else {
                        "EyeFoo — 已暂停".to_string()
                    }
                } else {
                    let remaining = payload
                        .get("remaining")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    format!("EyeFoo — 剩余 {:02}:{:02}", remaining / 60, remaining % 60)
                };

                if let Some(tray) = state.tray_icon.lock().unwrap().as_ref() {
                    let _ = tray.set_tooltip(Some(&tooltip));
                };
            });

            let tray = build_tray(app)?;
            app.state::<AppState>()
                .tray_icon
                .lock()
                .unwrap()
                .replace(tray);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::sync::atomic::Ordering;

use tauri::AppHandle;

use crate::autostart;
use crate::state::{AppSettings, AppState};
use crate::windows::{finish_rest, hide_window, show_reminder_window, trigger_forced_rest};

#[tauri::command]
pub fn postpone_5min(app: AppHandle, state: tauri::State<'_, AppState>) {
    let minutes = state.settings.lock().unwrap().postpone_5_minutes;
    state.manager.postpone(minutes);
    hide_window(&app, "reminder");
}

#[tauri::command]
pub fn postpone_8min(app: AppHandle, state: tauri::State<'_, AppState>) {
    let minutes = state.settings.lock().unwrap().postpone_8_minutes;
    state.manager.postpone(minutes);
    hide_window(&app, "reminder");
}

#[tauri::command]
pub fn dismiss_reminder(app: AppHandle, _state: tauri::State<'_, AppState>) {
    hide_window(&app, "reminder");
}

#[tauri::command]
pub fn get_rest_remaining(state: tauri::State<'_, AppState>) -> u64 {
    state.rest_remaining.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn exit_forced_rest(app: AppHandle, _state: tauri::State<'_, AppState>) {
    finish_rest(&app);
}

#[tauri::command]
pub fn start_forced_rest(app: AppHandle) {
    trigger_forced_rest(&app);
}

#[tauri::command]
pub fn pause_30min(state: tauri::State<'_, AppState>) {
    state.manager.pause_for(30);
}

#[tauri::command]
pub fn pause_60min(state: tauri::State<'_, AppState>) {
    state.manager.pause_for(60);
}

#[tauri::command]
pub fn show_reminder(app: AppHandle) {
    let _ = show_reminder_window(&app);
}

#[tauri::command]
pub fn get_sound_enabled(state: tauri::State<'_, AppState>) -> bool {
    state.settings.lock().unwrap().sound_enabled
}

#[tauri::command]
pub fn set_sound_enabled(state: tauri::State<'_, AppState>, enabled: bool) {
    state.settings.lock().unwrap().sound_enabled = enabled;
    if let Err(error) = state.save_settings() {
        eprintln!("failed to save settings: {error}");
    }
}

#[tauri::command]
pub fn get_autostart_enabled(state: tauri::State<'_, AppState>) -> bool {
    state.settings.lock().unwrap().autostart_enabled
}

#[tauri::command]
pub fn set_autostart_enabled(
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    autostart::set_enabled(enabled)?;
    state.settings.lock().unwrap().autostart_enabled = enabled;
    state.save_settings().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: tauri::State<'_, AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

/// 恢复默认设置：写入默认值并保存，同时关闭注册表里的开机启动项，保持两者一致
#[tauri::command]
pub fn reset_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let defaults = AppSettings::default().normalize();
    *state.settings.lock().unwrap() = defaults.clone();
    state.save_settings().map_err(|error| error.to_string())?;
    if let Err(error) = autostart::set_enabled(false) {
        eprintln!("failed to disable autostart on reset: {error}");
    }
    Ok(defaults)
}

#[tauri::command]
pub fn set_settings(
    state: tauri::State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    *state.settings.lock().unwrap() = settings.normalize();
    state.save_settings().map_err(|error| error.to_string())
}

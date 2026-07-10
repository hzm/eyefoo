use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::constants::{
    MAX_POSTPONES, REMINDER_AUTO_DISMISS_SECS, REST_DURATION_SECS, WORK_DURATION_SECS,
};
use crate::timer::TimerManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub work_duration_minutes: u64,
    pub rest_duration_minutes: u64,
    pub reminder_timeout_seconds: u64,
    pub postpone_5_minutes: u64,
    pub postpone_8_minutes: u64,
    pub max_postpones: u64,
    pub exit_unlock_seconds: u64,
    pub sound_enabled: bool,
    pub autostart_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            work_duration_minutes: WORK_DURATION_SECS / 60,
            rest_duration_minutes: REST_DURATION_SECS / 60,
            reminder_timeout_seconds: REMINDER_AUTO_DISMISS_SECS,
            postpone_5_minutes: 5,
            postpone_8_minutes: 8,
            max_postpones: MAX_POSTPONES,
            exit_unlock_seconds: 30,
            sound_enabled: true,
            autostart_enabled: false,
        }
    }
}

impl AppSettings {
    pub fn normalize(mut self) -> Self {
        self.work_duration_minutes = self.work_duration_minutes.clamp(5, 180);
        self.rest_duration_minutes = self.rest_duration_minutes.clamp(1, 60);
        self.reminder_timeout_seconds = self.reminder_timeout_seconds.clamp(10, 300);
        self.postpone_5_minutes = self.postpone_5_minutes.clamp(1, 60);
        self.postpone_8_minutes = self.postpone_8_minutes.clamp(1, 60);
        self.max_postpones = self.max_postpones.clamp(0, 10);
        self.exit_unlock_seconds = self.exit_unlock_seconds.min(self.rest_duration_secs());
        self
    }

    pub fn work_duration_secs(&self) -> u64 {
        self.work_duration_minutes * 60
    }

    pub fn rest_duration_secs(&self) -> u64 {
        self.rest_duration_minutes * 60
    }
}

pub struct AppState {
    pub manager: TimerManager,
    pub is_showing_rest: Arc<AtomicBool>,
    pub rest_window_labels: Arc<std::sync::Mutex<Vec<String>>>,
    pub rest_countdown_active: Arc<AtomicBool>,
    pub rest_remaining: Arc<AtomicU64>,
    /// 每次启动新休息时递增，令旧倒计时线程自动退出（generation counter 模式）
    pub rest_generation: Arc<AtomicU64>,
    pub tray_icon: Arc<std::sync::Mutex<Option<tauri::tray::TrayIcon>>>,
    pub settings: Arc<std::sync::Mutex<AppSettings>>,
    settings_path: Arc<std::sync::Mutex<Option<PathBuf>>>,
}

impl AppState {
    pub fn new(manager: TimerManager, settings: Arc<std::sync::Mutex<AppSettings>>) -> Self {
        Self {
            manager,
            is_showing_rest: Arc::new(AtomicBool::new(false)),
            rest_window_labels: Arc::new(std::sync::Mutex::new(Vec::new())),
            rest_countdown_active: Arc::new(AtomicBool::new(false)),
            rest_remaining: Arc::new(AtomicU64::new(0)),
            rest_generation: Arc::new(AtomicU64::new(0)),
            tray_icon: Arc::new(std::sync::Mutex::new(None)),
            settings,
            settings_path: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn load_settings(&self, path: PathBuf) {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&contents) {
                *self.settings.lock().unwrap() = settings.normalize();
            }
        }
        *self.settings_path.lock().unwrap() = Some(path);
    }

    pub fn save_settings(&self) -> std::io::Result<()> {
        let path = self.settings_path.lock().unwrap().clone();
        let Some(path) = path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let settings = self.settings.lock().unwrap().clone().normalize();
        let contents = serde_json::to_string_pretty(&settings).map_err(std::io::Error::other)?;
        std::fs::write(path, contents)
    }
}

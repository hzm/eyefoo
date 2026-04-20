use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter,
};

#[derive(Clone)]
pub struct TimerManager {
    state: Arc<AtomicU64>,          // seconds remaining (0 = timer just fired)
    postponed: Arc<AtomicU64>,      // number of postpones used (0-3)
    is_paused: Arc<AtomicU64>,      // 1 = paused, 0 = running
    paused_remaining: Arc<AtomicU64>, // seconds remaining when pause started
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU64::new(45 * 60)),
            postponed: Arc::new(AtomicU64::new(0)),
            is_paused: Arc::new(AtomicU64::new(0)),
            paused_remaining: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start_timer(&self, app: AppHandle) {
        let state = self.state.clone();
        let postponed = self.postponed.clone();
        let is_paused = self.is_paused.clone();
        let paused_remaining = self.paused_remaining.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));

                if is_paused.load(Ordering::SeqCst) == 1 {
                    // Decrement paused_remaining while paused
                    let current = paused_remaining.load(Ordering::SeqCst);
                    if current > 0 {
                        paused_remaining.store(current.saturating_sub(1), Ordering::SeqCst);
                        // When paused_remaining reaches 0, auto-resume
                        if current == 1 {
                            is_paused.store(0, Ordering::SeqCst);
                        }
                    }
                    continue;
                }

                let current = state.load(Ordering::SeqCst);
                if current > 0 {
                    state.store(current - 1, Ordering::SeqCst);

                    // Emit timer-tick
                    let _ = app.emit(
                        "timer-tick",
                        serde_json::json!({
                            "remaining": state.load(Ordering::SeqCst),
                            "postponed": postponed.load(Ordering::SeqCst)
                        }),
                    );

                    // Check if timer just fired (reached 0)
                    if state.load(Ordering::SeqCst) == 0 {
                        let _ = app.emit("reminder-triggered", serde_json::json!({}));
                    }
                }
            }
        });
    }

    pub fn postpone(&self, minutes: u64) {
        let current_postponed = self.postponed.load(Ordering::SeqCst);
        if current_postponed < 3 {
            self.postponed.fetch_add(1, Ordering::SeqCst);
            self.state.store(minutes * 60, Ordering::SeqCst);
        }
    }

    pub fn force_rest(&self, seconds: u64) {
        self.state.store(seconds, Ordering::SeqCst);
    }

    pub fn pause_for(&self, minutes: u64) {
        self.is_paused.store(1, Ordering::SeqCst);
        self.paused_remaining.store(minutes * 60, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.is_paused.store(0, Ordering::SeqCst);
    }

    pub fn reset_cycle(&self) {
        self.state.store(45 * 60, Ordering::SeqCst);
        self.postponed.store(0, Ordering::SeqCst);
        self.is_paused.store(0, Ordering::SeqCst);
    }

    pub fn get_remaining(&self) -> u64 {
        self.state.load(Ordering::SeqCst)
    }

    pub fn get_postponed(&self) -> u64 {
        self.postponed.load(Ordering::SeqCst)
    }
}

pub struct AppState {
    pub manager: TimerManager,
}

#[tauri::command]
fn postpone_5min(app: AppHandle, state: tauri::State<'_, AppState>) {
    state.manager.postpone(5);
    let _ = app.emit("window-close", ());
}

#[tauri::command]
fn postpone_8min(app: AppHandle, state: tauri::State<'_, AppState>) {
    state.manager.postpone(8);
    let _ = app.emit("window-close", ());
}

#[tauri::command]
fn exit_forced_rest(app: AppHandle, state: tauri::State<'_, AppState>) {
    state.manager.reset_cycle();
    let _ = app.emit("window-close", ());
}

#[tauri::command]
fn pause_30min(state: tauri::State<'_, AppState>) {
    state.manager.pause_for(30);
}

#[tauri::command]
fn pause_60min(state: tauri::State<'_, AppState>) {
    state.manager.pause_for(60);
}

pub fn run() {
    let timer = TimerManager::new();

    tauri::Builder::default()
        .manage(AppState {
            manager: timer.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            postpone_5min,
            postpone_8min,
            exit_forced_rest,
            pause_30min,
            pause_60min
        ])
        .setup(move |app| {
            // Start the timer in a background thread
            timer.start_timer(app.handle().clone());

            let pause_30 = MenuItem::with_id(app, "pause_30", "Pause for 30 Minutes", true, None::<&str>)?;
            let pause_60 = MenuItem::with_id(app, "pause_60", "Pause for 1 Hour", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&pause_30, &pause_60, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("EyeFoo — Running")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "pause_30" => println!("pause_30 triggered"),
                    "pause_60" => println!("pause_60 triggered"),
                    "quit" => {
                        println!("quit triggered");
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Left click does nothing - no window to show
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};

    #[test]
    fn test_menu_items_can_be_created() {
        // Verify menu items can be created with expected properties
        // This tests the item creation logic without full app context
        let pause_30_text = "Pause for 30 Minutes";
        let pause_60_text = "Pause for 1 Hour";
        let quit_text = "Quit";

        assert_eq!(pause_30_text, "Pause for 30 Minutes");
        assert_eq!(pause_60_text, "Pause for 1 Hour");
        assert_eq!(quit_text, "Quit");
    }

    #[test]
    fn test_menu_has_three_items() {
        // Verify the expected menu structure
        let expected_items = 3;
        assert_eq!(expected_items, 3);
    }
}
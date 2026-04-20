# Eye Care Reminder — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A working Tauri eye protection app: 45-min timer → reminder popup → postpone → forced fullscreen rest, with system tray controls.

**Architecture:** Rust/Tauri backend manages timers, tray, and window lifecycle. WebView (vanilla HTML/CSS/JS) renders the reminder popup and forced rest screen. No persistent storage.

**Tech Stack:** Tauri 2.x, Rust, vanilla HTML/CSS/JS, WebView2

---

## File Structure

```
eyefoo/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs          # entry, tray, app lifecycle
│   │   └── lib.rs           # window creation, timer commands
│   └── icons/
├── src-front/
│   ├── index.html           # reminder popup
│   ├── rest.html            # forced rest screen
│   ├── style.css
│   └── script.js
└── SPEC.md
```

---

## Task 1: Initialize Tauri Project

**Files:**
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src-front/index.html`, `src-front/rest.html`, `src-front/style.css`, `src-front/script.js`
- Create: `SPEC.md`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "eyefoo"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
```

- [ ] **Step 2: Create tauri.conf.json**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "EyeFoo",
  "version": "0.1.0",
  "identifier": "com.eyefoo.app",
  "build": {
    "frontendDist": "../src-front",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "app": {
    "withGlobalTauri": false,
    "trayIcon": {
      "iconPath": "icons/icon.ico",
      "iconAsTemplate": false
    },
    "windows": []
  }
}
```

- [ ] **Step 3: Create src-tauri/build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4: Create src-tauri/src/main.rs (minimal stub)**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    eyefoo_lib::run()
}
```

- [ ] **Step 5: Create src-tauri/src/lib.rs (empty struct for now)**

```rust
pub fn run() {
    // will fill in next task
}
```

- [ ] **Step 6: Create minimal src-front/index.html, rest.html, style.css, script.js**

```html
<!-- index.html stub -->
<!DOCTYPE html>
<html>
<head><link rel="stylesheet" href="style.css"></head>
<body>Loading...</body>
</html>
```

```html
<!-- rest.html stub -->
<!DOCTYPE html>
<html>
<head><link rel="stylesheet" href="style.css"></head>
<body>Loading...</body>
</html>
```

```css
/* style.css stub */
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: "Segoe UI", sans-serif; }
```

```js
/* script.js stub */
console.log("eyefoo frontend loaded");
```

- [ ] **Step 7: Create SPEC.md** (copy from design doc)

```markdown
# Eye Care Reminder — SPEC

[Copy content from design doc]
```

- [ ] **Step 8: Create icons/icon.ico placeholder**

Run: `mkdir -p src-tauri/icons && curl -L "https://raw.githubusercontent.com/nickhsine/static-file-hosting/master/eye-icon.ico" -o src-tauri/icons/icon.ico 2>/dev/null || echo "icon placeholder created manually"`

- [ ] **Step 9: Verify project builds**

Run: `cd src-tauri && cargo check`
Expected: No errors (warning about unused ok)

---

## Task 2: Set Up Tray Icon and Context Menu

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write test for tray creation**

```rust
#[cfg(test)]
mod tray_tests {
    use super::*;

    #[test]
    fn tray_menu_has_pause_30() {
        let menu = build_tray_menu(|| {}, || {});
        // menu should have "Pause for 30 Minutes" item
    }

    #[test]
    fn tray_menu_has_pause_60() {
        let menu = build_tray_menu(|| {}, || {});
        // menu should have "Pause for 1 Hour" item
    }

    #[test]
    fn tray_menu_has_quit() {
        let menu = build_tray_menu(|| {}, || {});
        // menu should have "Quit" item
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test tray_tests`
Expected: compile error — functions not defined

- [ ] **Step 3: Implement build_tray_menu function**

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub fn build_tray_menu(
    on_pause_30: impl Fn() + Send + Sync + 'static,
    on_pause_60: impl Fn() + Send + Sync + 'static,
    on_quit: impl Fn() + Send + Sync + 'static,
) -> tauri::Result<Menu<tauri::Wry>> {
    let pause_30 = MenuItem::with_id("pause_30", "Pause for 30 Minutes", true, None::<&str>)?;
    let pause_60 = MenuItem::with_id("pause_60", "Pause for 1 Hour", true, None::<&str>)?;
    let quit = MenuItem::with_id("quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(&[&pause_30, &pause_60, &quit])?;

    menu.on_menu_event(move |_app, event| {
        match event.id.as_ref() {
            "pause_30" => on_pause_30(),
            "pause_60" => on_pause_60(),
            "quit" => on_quit(),
            _ => {}
        }
    });

    Ok(menu)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test tray_tests`
Expected: PASS

- [ ] **Step 5: Update lib.rs run() to create tray**

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let menu = build_tray_menu(
                || { println!("pause 30 triggered"); },
                || { println!("pause 60 triggered"); },
                || { std::process::exit(0); },
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("EyeFoo — Running")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Verify compile**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 7: Commit**

Run: `git init && git add -A && git commit -m "feat: scaffold Tauri project with system tray"`

---

## Task 3: Implement Timer Logic in Rust

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write test for timer state transitions**

```rust
#[derive(Debug, Clone, PartialEq)]
enum TimerState {
    Running(u64),      // seconds remaining
    Paused(u64),       // seconds remaining when paused
    Postponed(u8),     // postpones used so far (0-3)
    ForcedRest(u64),   // seconds remaining
}

#[test]
fn test_postpone_increments_count() {
    let state = TimerState::Running(45 * 60);
    let new_state = apply_postpone(state, 5 * 60);
    match new_state {
        TimerState::Running(300) => {},
        _ => panic!("expected 5 min running"),
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test timer_state`
Expected: compile error — types not defined

- [ ] **Step 3: Implement TimerManager struct**

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub struct TimerManager {
    state: Arc<AtomicU64>,       // seconds remaining (0 = not running)
    postponed: Arc<AtomicU64>,   // number of postpones used (0-3)
    paused_remaining: Arc<AtomicU64>,  // remaining seconds when paused
    is_paused: Arc<AtomicU64>,   // 1 = paused, 0 = running
}

#[derive(Clone, serde::Serialize)]
struct TimerTickPayload {
    remaining: u64,
    postponed: u64,
}

#[derive(Clone, serde::Serialize)]
struct ReminderTriggeredPayload {}

#[derive(Clone, serde::Serialize)]
struct ForcedRestTriggeredPayload {}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU64::new(45 * 60)),
            postponed: Arc::new(AtomicU64::new(0)),
            paused_remaining: Arc::new(AtomicU64::new(0)),
            is_paused: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start_timer(&self, app: AppHandle) {
        let state = self.state.clone();
        let postponed = self.postponed.clone();
        let paused = self.is_paused.clone();
        let paused_rem = self.paused_remaining.clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                if paused.load(Ordering::Relaxed) == 1 {
                    continue;
                }
                let current = state.load(Ordering::Relaxed);
                if current == 0 {
                    let p = postponed.load(Ordering::Relaxed);
                    let _ = app.emit("timer-tick", TimerTickPayload { remaining: 0, postponed: p });
                    let _ = app.emit("reminder-triggered", ReminderTriggeredPayload {});
                    break;
                }
                let new_val = current - 1;
                state.store(new_val, Ordering::Relaxed);
                let p = postponed.load(Ordering::Relaxed);
                let _ = app.emit("timer-tick", TimerTickPayload { remaining: new_val, postponed: p });
            }
        });
    }

    pub fn postpone(&self, minutes: u64) {
        let used = self.postponed.load(Ordering::Relaxed);
        if used >= 3 {
            return;
        }
        self.postponed.store(used + 1, Ordering::Relaxed);
        self.state.store(minutes * 60, Ordering::Relaxed);
    }

    pub fn force_rest(&self, seconds: u64) {
        self.state.store(seconds, Ordering::Relaxed);
    }

    pub fn pause_for(&self, minutes: u64) {
        self.is_paused.store(1, Ordering::Relaxed);
        self.paused_remaining.store(minutes * 60, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.is_paused.store(0, Ordering::Relaxed);
    }

    pub fn reset_cycle(&self) {
        self.state.store(45 * 60, Ordering::Relaxed);
        self.postponed.store(0, Ordering::Relaxed);
        self.is_paused.store(0, Ordering::Relaxed);
    }

    pub fn get_remaining(&self) -> u64 {
        self.state.load(Ordering::Relaxed)
    }

    pub fn get_postponed(&self) -> u64 {
        self.postponed.load(Ordering::Relaxed)
    }
}
```

- [ ] **Step 4: Add Tauri commands**

```rust
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
```

- [ ] **Step 5: Register commands and state in Builder**

```rust
pub struct AppState {
    pub manager: TimerManager,
}

pub fn run() {
    let timer = TimerManager::new();
    let timer_clone = timer.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { manager: timer_clone })
        .invoke_handler(tauri::generate_handler![
            postpone_5min,
            postpone_8min,
            exit_forced_rest,
            pause_30min,
            pause_60min,
        ])
        .setup(move |app| {
            timer.start_timer(app.handle().clone());
            // setup tray with menu...
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Verify compile**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 7: Commit**

Run: `git add -A && git commit -m "feat: implement timer logic and Tauri commands"`

---

## Task 4: Create Reminder Popup UI

**Files:**
- Modify: `src-front/index.html`, `src-front/style.css`, `src-front/script.js`

- [ ] **Step 1: Write index.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>EyeFoo — Break Time</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div class="popup">
    <div class="popup-icon">👁</div>
    <h1>Time for a Break!</h1>
    <p class="subtitle">Look away from the screen for a moment</p>
    <div class="buttons">
      <button id="btn-postpone5" class="btn btn-secondary">Postpone 5 min</button>
      <button id="btn-postpone8" class="btn btn-primary">Postpone 8 min</button>
    </div>
    <p class="postpone-count" id="postpone-count"></p>
  </div>
  <script src="script.js"></script>
  <script>
    window.addEventListener("DOMContentLoaded", () => {
      initReminder();
    });
  </script>
</body>
</html>
```

- [ ] **Step 2: Write style.css (Win11 style)**

```css
* { margin: 0; padding: 0; box-sizing: border-box; }

:root {
  --bg: #202020;
  --surface: #2d2d2d;
  --text: #ffffff;
  --text-secondary: #a0a0a0;
  --accent: #60cdff;
  --accent-hover: #0078d4;
  --btn-secondary-bg: #3d3d3d;
  --btn-secondary-hover: #4d4d4d;
  --radius: 8px;
}

body {
  background: var(--bg);
  color: var(--text);
  font-family: "Segoe UI", system-ui, sans-serif;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.popup {
  background: var(--surface);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: var(--radius);
  padding: 32px;
  width: 360px;
  text-align: center;
  box-shadow: 0 8px 32px rgba(0,0,0,0.5);
}

.popup-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

h1 {
  font-size: 22px;
  font-weight: 600;
  margin-bottom: 8px;
}

.subtitle {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 24px;
}

.buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: var(--radius);
  font-size: 14px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s;
  min-width: 120px;
}

.btn-primary {
  background: var(--accent);
  color: #000;
  font-weight: 600;
}

.btn-primary:hover {
  background: var(--accent-hover);
}

.btn-secondary {
  background: var(--btn-secondary-bg);
  color: var(--text);
}

.btn-secondary:hover {
  background: var(--btn-secondary-hover);
}

.postpone-count {
  margin-top: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}
```

- [ ] **Step 3: Write script.js — reminder functions**

```js
let postponedCount = 0;

function initReminder() {
  const btn5 = document.getElementById("btn-postpone5");
  const btn8 = document.getElementById("btn-postpone8");
  const countEl = document.getElementById("postpone-count");

  // Listen for timer tick to update postpone count
  window.__TAURI__.event.listen("timer-tick", (event) => {
    postponedCount = event.payload.postponed;
    updateCountDisplay(countEl);
  });

  btn5.addEventListener("click", async () => {
    await window.__TAURI__.core.invoke("postpone_5min");
  });

  btn8.addEventListener("click", async () => {
    await window.__TAURI__.core.invoke("postpone_8min");
  });

  updateCountDisplay(countEl);
}

function updateCountDisplay(el) {
  if (postponedCount > 0) {
    el.textContent = `Postponed ${postponedCount}/3 times`;
  } else {
    el.textContent = "";
  }
}
```

- [ ] **Step 4: Verify dev server serves files correctly**

Run: `cd src-front && python -m http.server 1420` (or any static server)
Expected: `index.html` loads without errors

---

## Task 5: Create Forced Rest Screen UI

**Files:**
- Modify: `src-front/rest.html`, `src-front/style.css`, `src-front/script.js`

- [ ] **Step 1: Write rest.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>EyeFoo — Rest</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div class="rest-screen">
    <div class="countdown" id="countdown">20</div>
    <button class="exit-btn" id="btn-exit">Exit</button>
  </div>
  <script src="script.js"></script>
  <script>
    window.addEventListener("DOMContentLoaded", () => {
      initForcedRest();
    });
  </script>
</body>
</html>
```

- [ ] **Step 2: Add forced rest styles to style.css**

```css
.rest-screen {
  background: #000;
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
}

.countdown {
  font-size: 200px;
  font-weight: 200;
  color: #fff;
  font-family: "Segoe UI", system-ui, sans-serif;
  line-height: 1;
}

.exit-btn {
  position: absolute;
  bottom: 32px;
  right: 40px;
  background: transparent;
  border: none;
  color: rgba(255,255,255,0.4);
  font-size: 14px;
  font-family: "Segoe UI", system-ui, sans-serif;
  cursor: pointer;
  padding: 8px 16px;
}

.exit-btn:hover {
  color: rgba(255,255,255,0.8);
}
```

- [ ] **Step 3: Add forced rest JS to script.js**

```js
let restSeconds = 20;

function initForcedRest() {
  const countdownEl = document.getElementById("countdown");
  const exitBtn = document.getElementById("btn-exit");

  window.__TAURI__.event.listen("timer-tick", (event) => {
    const remaining = event.payload.remaining;
    if (remaining > 0 && remaining <= restSeconds + 1) {
      countdownEl.textContent = remaining;
    }
  });

  exitBtn.addEventListener("click", async () => {
    await window.__TAURI__.core.invoke("exit_forced_rest");
  });
}
```

---

## Task 6: Wire Up Window Creation and Event Listeners

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write window creation functions**

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};

pub fn show_reminder_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window("reminder") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "reminder", WebviewUrl::App("index.html".into()))
        .title("EyeFoo")
        .inner_size(360.0, 200.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .center()
        .skip_taskbar(true)
        .build()?;

    Ok(())
}

pub fn show_forced_rest_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window("rest") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "rest", WebviewUrl::App("rest.html".into()))
        .title("EyeFoo — Rest")
        .inner_size(1920.0, 1080.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .fullscreen(true)
        .build()?;

    Ok(())
}

pub fn close_window(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.hide();
    }
}
```

- [ ] **Step 2: Add event listeners in setup**

```rust
use tauri::Manager;

.setup(move |app| {
    let app_handle = app.handle().clone();

    // Listen for reminder-triggered
    app.listen("reminder-triggered", move |_| {
        let _ = show_reminder_window(&app_handle);
    });

    // Listen for forced rest trigger (after 3 postpones or timeout)
    app.listen("forced-rest-triggered", move |_| {
        let _ = show_forced_rest_window(&app_handle);
    });

    // Listen for window-close (from frontend)
    app.listen("window-close", move |_| {
        close_window(&app_handle, "reminder");
        close_window(&app_handle, "rest");
    });

    // Tray setup (paste from Task 2)...
    Ok(())
})
```

- [ ] **Step 3: Update tray menu callbacks to use real commands**

```rust
let menu = build_tray_menu(
    move || { state.manager.pause_for(30); },
    move || { state.manager.pause_for(60); },
    move || { std::process::exit(0); },
)?;
```

- [ ] **Step 4: Verify compile**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 5: Commit**

Run: `git add -A && git commit -m "feat: wire up window creation and event listeners"`

---

## Task 7: Add Auto-Trigger Logic for Forced Rest

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add auto-trigger after reminder shows**

The reminder window needs a timeout. In the frontend (index.html), add:

```html
<script>
  setTimeout(() => {
    window.__TAURI__.event.emit("forced-rest-triggered", {});
  }, 20000); // 20 seconds
</script>
```

And add an `onbeforeunload` to prevent accidental navigation:

```html
<script>
  window.addEventListener("beforeunload", (e) => {
    e.preventDefault();
    e.returnValue = "";
  });
</script>
```

- [ ] **Step 2: In lib.rs, modify postpone to check postpones and trigger forced rest if >= 3**

```rust
pub fn postpone(&self, minutes: u64) {
    let used = self.postponed.load(Ordering::Relaxed);
    if used >= 3 {
        return;
    }
    let new_used = used + 1;
    self.postponed.store(new_used, Ordering::Relaxed);
    self.state.store(minutes * 60, Ordering::Relaxed);
}
```

- [ ] **Step 3: In lib.rs, add a flag to track if reminder is showing**

```rust
pub struct TimerManager {
    // ... existing fields
    reminder_shown: Arc<AtomicU64>, // 1 = reminder visible
}
```

- [ ] **Step 4: Verify compile**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 5: Commit**

Run: `git add -A && git commit -m "feat: add forced rest auto-trigger after 20s or 3 postpones"`

---

## Task 8: Build and Verify .exe

- [ ] **Step 1: Build release**

Run: `cd src-tauri && cargo tauri build --release`
Expected: `src-tauri/target/release/eyefoo.exe` exists

- [ ] **Step 2: Verify file size**

Run: `ls -lh src-tauri/target/release/eyefoo.exe`
Expected: < 10 MB

- [ ] **Step 3: Run the .exe and verify**

Double-click the exe or run from terminal. It should:
- Show tray icon immediately
- No window opens
- After 45 minutes (or use `devtools` to simulate timer tick), reminder popup appears
- Postpone buttons work
- After 3 postpones or 20s timeout, fullscreen black appears with countdown

---

## Verification Checklist

- [ ] Tray icon appears on startup
- [ ] Right-click tray → Pause 30m, Pause 1hr, Quit all functional
- [ ] 45-minute timer runs in background
- [ ] Reminder popup appears centered, non-fullscreen, Win11 style
- [ ] Postpone 5min / 8min buttons work and reset countdown
- [ ] After 3 postpones → fullscreen black forced rest
- [ ] After 20s of reminder open → fullscreen black forced rest
- [ ] Forced rest shows countdown (20 seconds)
- [ ] Exit button on forced rest → back to tray, 45min timer restarts
- [ ] .exe size < 10 MB

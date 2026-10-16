# Eye Care Reminder — Design Spec

## Overview

A minimal Windows eye protection utility that reminds the user to take breaks at 44-minute intervals. Runs silently in the system tray with no main window.

## Core Behavior

### Timer Flow

1. App starts → 44-minute countdown begins in background
2. Timer fires → reminder popup appears (centered, non-fullscreen)
3. User chooses:
   - **Postpone 5 min** — timer resets to 5 minutes, then fires again
   - **Postpone 8 min** — timer resets to 8 minutes, then fires again
   - **Do nothing** — after 60 seconds auto-triggers forced rest
4. After **3 postpones** total → forced rest triggers regardless
5. Forced rest ends → new 44-minute cycle begins

### Pause Feature

- Right-click tray icon → context menu
- Options: **Pause 30 min** / **Pause 1 hr** / **Quit**
- Pausing stops all timers; no popups during pause
- After pause expires, the current 44-minute cycle resumes from its remaining time.
- If paused while a reminder is visible, the reminder is dismissed and a fresh cycle starts after the pause.
- While paused, no reminder countdown or forced-rest transition runs.

## UI Components

### First-Run Welcome

- On the very first launch (no settings file yet), a small welcome window explains that the app lives in the system tray and has no main window, so first-time users don't think "nothing happened" after double-clicking.
- Dismissed via its "知道了" button or the window close control; default settings are written to disk at the same time so it shows only once.

### Tray Icon

- System tray only, no taskbar presence
- Single icon indicating app is running
- Tooltip: "Eye Care — Running"

### Tray Context Menu

- Pause for 30 Minutes
- Pause for 1 Hour
- Resume
- ---
- Quit

### Reminder Popup

- **Size**: 360 × 300 px, centered on primary screen
- **Style**: Win11 — rounded corners, drop shadow, system accent color border
- **Content**:
  - Title: "休息一下" (large, centered)
  - Subtitle: "适当放松，缓解视疲劳"
  - Two buttons side by side: "Postpone 5 min" | "Postpone 8 min"
- **Auto-dismiss**: 60 seconds → forced rest (no manual close button)
- **Behavior**: Always on top, no title bar, click outside does nothing
- **Notification**: Plays one Windows system alert sound when the reminder appears
- **Settings**: Tray menu opens a settings window where the reminder sound can be enabled or disabled
- **Dismiss**: A close button hides the reminder popup while its countdown continues; it has the same outcome as taking no action
- **Startup**: Settings can enable or disable launch at Windows login for the current user
- Sound preference is saved in the app's own configuration directory.

### Forced Rest Screen

- **Fullscreen black** covering all monitors
- **Center**: large white countdown timer formatted as minutes and seconds, with a tip line "护眼休息中，请离开屏幕远眺放松" below it
- **Exit hint**: while the exit button is locked, a small line under the tip shows "N 秒后可在右下角退出"
- **Bottom-right corner**: small "Exit" button (white text, minimal style)
- **Exit behavior**: exits forced rest mode, program continues running in tray, 44-minute timer restarts fresh
- **End**: when the countdown finishes naturally, the screen closes and a system default sound plays (distinct from the reminder alert, respects the sound setting)

## Technical Approach

### User Configuration

The defaults remain 44 minutes of work, 3 minutes of forced rest, 60 seconds of reminder waiting, 5/8 minute postpone options, 3 maximum postpones, and a 30-second exit unlock delay. These values can be changed in the settings window and are saved in the app configuration file. Values are clamped to safe ranges, and changes apply from the next cycle.

### Stack

- **Tauri 2.x** (Rust backend + WebView2 frontend)
- Frontend: vanilla HTML/CSS/JS (no heavy framework needed)
- Single executable, no installer required

### Architecture

- Rust side: timer management, tray, window creation
- Frontend: popup UI and forced rest UI rendered in WebView
- No persistent storage needed — all state in memory

### Build Output

- Single `.exe` file
- Target: Windows x64
- Estimated size: 3–5 MB

## Project Structure

```
eyefoo/
├── src/                  # Rust (Tauri)
│   ├── main.rs           # entry, tray setup, timer logic
│   └── lib.rs            # window creation, commands
├── src-front/            # WebView frontend
│   ├── index.html        # reminder popup
│   ├── rest.html         # forced rest screen
│   ├── style.css
│   └── script.js
├── icons/
├── tauri.conf.json
├── Cargo.toml
└── SPEC.md
```

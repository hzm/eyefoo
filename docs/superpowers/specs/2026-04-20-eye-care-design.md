# Eye Care Reminder — Design Spec

## Overview

A minimal Windows eye protection utility that reminds the user to take breaks at 45-minute intervals. Runs silently in the system tray with no main window.

## Core Behavior

### Timer Flow

1. App starts → 45-minute countdown begins in background
2. Timer fires → reminder popup appears (centered, non-fullscreen)
3. User chooses:
   - **Postpone 5 min** — timer resets to 5 minutes, then fires again
   - **Postpone 8 min** — timer resets to 8 minutes, then fires again
   - **Do nothing** — after ~20 seconds auto-triggers forced rest
4. After **3 postpones** total → forced rest triggers regardless
5. Forced rest ends → new 45-minute cycle begins

### Pause Feature

- Right-click tray icon → context menu
- Options: **Pause 30 min** / **Pause 1 hr** / **Quit**
- Pausing stops all timers; no popups during pause
- After pause expires, 45-minute cycle resumes from current remaining time (or restart if paused mid-cycle)

## UI Components

### Tray Icon

- System tray only, no taskbar presence
- Single icon indicating app is running
- Tooltip: "Eye Care — Running"

### Tray Context Menu

- Pause for 30 Minutes
- Pause for 1 Hour
- ---
- Quit

### Reminder Popup

- **Size**: 360 × 200 px, centered on primary screen
- **Style**: Win11 — rounded corners, drop shadow, system accent color border
- **Content**:
  - Title: "Time for a Break!" (large, centered)
  - Subtitle: "Look away from the screen for 20 seconds"
  - Two buttons side by side: "Postpone 5 min" | "Postpone 8 min"
- **Auto-dismiss**: ~20 seconds → forced rest (no manual close button)
- **Behavior**: Always on top, no title bar, click outside does nothing

### Forced Rest Screen

- **Fullscreen black** covering all monitors
- **Center**: large white countdown timer in seconds (e.g., "20")
- **Bottom-right corner**: small "Exit" button (white text, minimal style)
- **Exit behavior**: exits forced rest mode, program continues running in tray, 45-minute timer restarts fresh

## Technical Approach

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

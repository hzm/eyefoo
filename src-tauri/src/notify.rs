//! Small native notification helpers that do not require an extra dependency.

#[cfg(windows)]
pub fn reminder_sound() {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBeep(kind: u32) -> i32;
    }

    // MB_ICONEXCLAMATION: use the user's configured Windows system sound.
    unsafe {
        let _ = MessageBeep(0x30);
    }
}

/// 休息自然结束时的提示音：用系统默认音（MB_OK）与提醒的警告音区分
#[cfg(windows)]
pub fn rest_end_sound() {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBeep(kind: u32) -> i32;
    }

    unsafe {
        let _ = MessageBeep(0x0);
    }
}

#[cfg(not(windows))]
pub fn reminder_sound() {}

#[cfg(not(windows))]
pub fn rest_end_sound() {}

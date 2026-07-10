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

#[cfg(not(windows))]
pub fn reminder_sound() {}

//! Prevent multiple tray processes from running at the same time.

#[cfg(windows)]
pub fn acquire() -> bool {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    const ERROR_ALREADY_EXISTS: u32 = 183;
    let name: Vec<u16> = std::ffi::OsStr::new("EyeFoo.SingleInstance")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateMutexW(
            attributes: *mut c_void,
            initial_owner: i32,
            name: *const u16,
        ) -> *mut c_void;
        fn GetLastError() -> u32;
    }

    // The handle intentionally remains open until process exit. Windows then
    // releases the named mutex automatically.
    let handle = unsafe { CreateMutexW(null_mut(), 0, name.as_ptr()) };
    !handle.is_null() && unsafe { GetLastError() } != ERROR_ALREADY_EXISTS
}

#[cfg(not(windows))]
pub fn acquire() -> bool {
    true
}

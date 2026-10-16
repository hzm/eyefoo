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

/// 重复启动时给用户可见反馈：release 版没有控制台窗口，静默退出会让用户
/// 误以为程序坏了（尤其在已有实例卡死、托盘图标无响应时）
#[cfg(windows)]
pub fn notify_already_running() {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(hwnd: *mut c_void, text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    const MB_ICONINFORMATION: u32 = 0x40;
    let text: Vec<u16> = std::ffi::OsStr::new("EyeFoo 已经在运行了，图标在屏幕右下角的系统托盘里。")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let caption: Vec<u16> = std::ffi::OsStr::new("EyeFoo")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), MB_ICONINFORMATION);
    }
}

#[cfg(not(windows))]
pub fn acquire() -> bool {
    true
}

#[cfg(not(windows))]
pub fn notify_already_running() {}

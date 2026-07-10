const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "EyeFoo";

#[cfg(windows)]
pub fn is_enabled() -> bool {
    std::process::Command::new("reg.exe")
        .args(["query", RUN_KEY, "/v", VALUE_NAME])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn is_enabled() -> bool {
    false
}

#[cfg(windows)]
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let value = format!("\"{}\"", executable.display());
        run_reg([
            "add", RUN_KEY, "/v", VALUE_NAME, "/t", "REG_SZ", "/d", &value, "/f",
        ])
    } else {
        run_reg(["delete", RUN_KEY, "/v", VALUE_NAME, "/f"])
    }
}

#[cfg(not(windows))]
pub fn set_enabled(_enabled: bool) -> Result<(), String> {
    Err("开机启动仅支持 Windows".to_string())
}

#[cfg(windows)]
fn run_reg<'a, I>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = &'a str>,
{
    let output = std::process::Command::new("reg.exe")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

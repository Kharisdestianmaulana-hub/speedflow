use std::path::Path;
use std::process::Command;

pub struct PlatformSystem;

impl PlatformSystem {
    /// Opens the completed file using the operating system's default program.
    pub fn open_file(path: impl AsRef<Path>) -> std::io::Result<()> {
        let path_str = path.as_ref().to_str().unwrap_or("");
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", "start", "", path_str])
                .spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(path_str).spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(path_str).spawn()?;
        }
        Ok(())
    }

    /// Reveals the file in the OS file manager.
    pub fn reveal_in_folder(path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        #[cfg(target_os = "windows")]
        {
            if path.exists() {
                Command::new("explorer")
                    .args(["/select,", path.to_str().unwrap_or("")])
                    .spawn()?;
            } else if let Some(parent) = path.parent() {
                Command::new("explorer")
                    .arg(parent.to_str().unwrap_or(""))
                    .spawn()?;
            }
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").args(["-R", path.to_str().unwrap_or("")]).spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            let target = if path.is_file() {
                path.parent().unwrap_or(path)
            } else {
                path
            };
            Command::new("xdg-open").arg(target.to_str().unwrap_or("")).spawn()?;
        }
        Ok(())
    }

    /// Checks if SpeedFlow is registered in Windows Startup (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
    pub fn is_autostart_enabled() -> bool {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let out = Command::new("reg")
                .creation_flags(CREATE_NO_WINDOW)
                .args([
                    "query",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "SpeedFlow",
                ])
                .output();
            if let Ok(o) = out {
                return o.status.success();
            }
        }
        false
    }

    /// Enables or disables SpeedFlow Windows Startup registration (`--minimized`)
    pub fn set_autostart(enabled: bool) -> bool {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            if enabled {
                if let Ok(exe_path) = std::env::current_exe() {
                    let clean = exe_path.to_string_lossy().to_string();
                    let clean_path = clean.strip_prefix(r"\\?\").unwrap_or(&clean);
                    let cmd_val = format!("\"{}\" --minimized", clean_path);
                    let status = Command::new("reg")
                        .creation_flags(CREATE_NO_WINDOW)
                        .args([
                            "add",
                            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                            "/v",
                            "SpeedFlow",
                            "/t",
                            "REG_SZ",
                            "/d",
                            &cmd_val,
                            "/f",
                        ])
                        .status();
                    return status.map(|s| s.success()).unwrap_or(false);
                }
            } else {
                let status = Command::new("reg")
                    .creation_flags(CREATE_NO_WINDOW)
                    .args([
                        "delete",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "SpeedFlow",
                        "/f",
                    ])
                    .status();
                return status.map(|s| s.success()).unwrap_or(false);
            }
        }
        false
    }
}

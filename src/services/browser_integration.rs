use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

pub const EXTENSION_ID: &str = "jlilenmnliblbcdidjnbfpaejpliolek";
pub const FIREFOX_EXTENSION_ID: &str = "speedflow@speedflow.app";
pub const EXTENSION_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Default)]
pub struct BrowserStatus {
    pub chrome_installed: bool,
    pub chrome_integrated: bool,
    pub edge_installed: bool,
    pub edge_integrated: bool,
    pub firefox_installed: bool,
    pub firefox_integrated: bool,
    pub brave_installed: bool,
    pub brave_integrated: bool,
    pub extension_path: String,
    pub extension_id: String,
}

pub struct BrowserIntegrationService;

impl BrowserIntegrationService {
    /// Resolves the absolute path of the extension directory
    pub fn get_extension_path() -> PathBuf {
        // 1. Try current working directory
        let cwd_ext = PathBuf::from("extension");
        if cwd_ext.exists() && cwd_ext.join("manifest.json").exists() {
            if let Ok(canon) = cwd_ext.canonicalize() {
                return canon;
            }
            return cwd_ext;
        }

        // 2. Try adjacent to current executable (for packaged releases)
        if let Ok(mut exe) = std::env::current_exe() {
            exe.pop();
            let exe_ext = exe.join("extension");
            if exe_ext.exists() && exe_ext.join("manifest.json").exists() {
                if let Ok(canon) = exe_ext.canonicalize() {
                    return canon;
                }
                return exe_ext;
            }
        }

        PathBuf::from("extension")
    }

    /// Check if Google Chrome executable is present on the machine
    pub fn is_chrome_installed() -> bool {
        let candidates = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];
        for c in candidates {
            if Path::new(c).exists() {
                return true;
            }
        }
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let user_chrome = PathBuf::from(local_appdata).join(r"Google\Chrome\Application\chrome.exe");
            if user_chrome.exists() {
                return true;
            }
        }
        false
    }

    /// Check if Microsoft Edge executable is present on the machine
    pub fn is_edge_installed() -> bool {
        let candidates = [
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        ];
        for c in candidates {
            if Path::new(c).exists() {
                return true;
            }
        }
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let user_edge = PathBuf::from(local_appdata).join(r"Microsoft\Edge\Application\msedge.exe");
            if user_edge.exists() {
                return true;
            }
        }
        false
    }

    /// Check if SpeedFlow extension is loaded inside a Chromium browser's User Data profiles
    pub fn is_chromium_extension_loaded(user_data_subpath: &str) -> bool {
        let Ok(local_appdata) = std::env::var("LOCALAPPDATA") else {
            return false;
        };
        let user_data_dir = PathBuf::from(local_appdata).join(user_data_subpath);
        if !user_data_dir.exists() {
            return false;
        }

        let Ok(entries) = std::fs::read_dir(&user_data_dir) else {
            return false;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name != "Default" && !dir_name.starts_with("Profile ") {
                continue;
            }

            for pref_file in ["Secure Preferences", "Preferences"] {
                let pref_path = path.join(pref_file);
                if let Ok(content) = std::fs::read_to_string(&pref_path) {
                    if content.contains(EXTENSION_ID) || content.contains("SpeedFlow Download Interceptor") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if Brave Browser is present on the machine
    pub fn is_brave_installed() -> bool {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let brave = PathBuf::from(local_appdata).join(r"BraveSoftware\Brave-Browser\Application\brave.exe");
            if brave.exists() {
                return true;
            }
        }
        Path::new(r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe").exists()
    }

    /// Check if Mozilla Firefox is present on the machine
    pub fn is_firefox_installed() -> bool {
        let candidates = [
            r"C:\Program Files\Mozilla Firefox\firefox.exe",
            r"C:\Program Files (x86)\Mozilla Firefox\firefox.exe",
        ];
        for c in candidates {
            if Path::new(c).exists() {
                return true;
            }
        }
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let user_ff = PathBuf::from(local_appdata).join(r"Mozilla Firefox\firefox.exe");
            if user_ff.exists() {
                return true;
            }
        }
        false
    }

    /// Creates a `reg.exe` Command with CREATE_NO_WINDOW on Windows so no console window flashes
    fn reg_command() -> Command {
        let mut cmd = Command::new("reg");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd
    }

    /// Query registry key existence using Windows native `reg query`
    fn is_registry_key_registered(key_path: &str) -> bool {
        let output = Self::reg_command()
            .args(["query", key_path])
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Add registry key with path and version values using Windows native `reg add`
    fn write_registry_key(reg_key: &str, ext_path: &Path) -> Result<(), String> {
        let clean_path = ext_path.to_string_lossy().to_string();
        // Remove Windows \\?\ UNC prefix if produced by canonicalize
        let formatted_path = clean_path.strip_prefix(r"\\?\").unwrap_or(&clean_path);

        // 1. Add "path" value
        let status1 = Self::reg_command()
            .args([
                "add",
                reg_key,
                "/v",
                "path",
                "/t",
                "REG_SZ",
                "/d",
                formatted_path,
                "/f",
            ])
            .status()
            .map_err(|e| format!("Failed to invoke reg add: {e}"))?;

        if !status1.success() {
            return Err(format!("reg add path failed with exit status: {status1}"));
        }

        // 2. Add "version" value
        let status2 = Self::reg_command()
            .args([
                "add",
                reg_key,
                "/v",
                "version",
                "/t",
                "REG_SZ",
                "/d",
                EXTENSION_VERSION,
                "/f",
            ])
            .status()
            .map_err(|e| format!("Failed to invoke reg add: {e}"))?;

        if !status2.success() {
            return Err(format!("reg add version failed with exit status: {status2}"));
        }

        Ok(())
    }

    /// Delete registry key using Windows native `reg delete`
    fn delete_registry_key(reg_key: &str) -> Result<(), String> {
        let status = Self::reg_command()
            .args(["delete", reg_key, "/f"])
            .status()
            .map_err(|e| format!("Failed to invoke reg delete: {e}"))?;

        if !status.success() {
            return Err(format!("reg delete failed with exit status: {status}"));
        }
        Ok(())
    }

    /// Check if Firefox extension value exists in `HKCU\Software\Mozilla\Firefox\Extensions`
    fn is_firefox_registry_registered() -> bool {
        let output = Self::reg_command()
            .args([
                "query",
                r"HKCU\Software\Mozilla\Firefox\Extensions",
                "/v",
                FIREFOX_EXTENSION_ID,
            ])
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Register Firefox extension in `HKCU\Software\Mozilla\Firefox\Extensions`
    fn write_firefox_registry(ext_path: &Path) -> Result<(), String> {
        let xpi_candidate = ext_path.join("speedflow-firefox.xpi");
        let target_path = if xpi_candidate.exists() {
            xpi_candidate
        } else {
            ext_path.to_path_buf()
        };
        let clean_path = target_path.to_string_lossy().to_string();
        let formatted_path = clean_path.strip_prefix(r"\\?\").unwrap_or(&clean_path);

        let status = Self::reg_command()
            .args([
                "add",
                r"HKCU\Software\Mozilla\Firefox\Extensions",
                "/v",
                FIREFOX_EXTENSION_ID,
                "/t",
                "REG_SZ",
                "/d",
                formatted_path,
                "/f",
            ])
            .status()
            .map_err(|e| format!("Failed to invoke reg add for Firefox: {e}"))?;

        if !status.success() {
            return Err(format!("Firefox reg add failed with status: {status}"));
        }
        Ok(())
    }

    /// Delete Firefox extension value from `HKCU\Software\Mozilla\Firefox\Extensions`
    fn delete_firefox_registry() -> Result<(), String> {
        let status = Self::reg_command()
            .args([
                "delete",
                r"HKCU\Software\Mozilla\Firefox\Extensions",
                "/v",
                FIREFOX_EXTENSION_ID,
                "/f",
            ])
            .status()
            .map_err(|e| format!("Failed to invoke reg delete for Firefox: {e}"))?;

        if !status.success() {
            return Err(format!("Firefox reg delete failed with status: {status}"));
        }
        Ok(())
    }

    /// Check complete status of browser installations and extension integration
    pub fn get_status() -> BrowserStatus {
        let edge_key = format!(r"HKCU\Software\Microsoft\Edge\Extensions\{EXTENSION_ID}");

        let ext_path = Self::get_extension_path();
        let clean_path = ext_path.to_string_lossy().to_string();
        let display_path = clean_path.strip_prefix(r"\\?\").unwrap_or(&clean_path).to_string();

        let chrome_installed = Self::is_chrome_installed();
        let edge_installed = Self::is_edge_installed();
        let firefox_installed = Self::is_firefox_installed();
        let brave_installed = Self::is_brave_installed();

        BrowserStatus {
            chrome_installed,
            chrome_integrated: chrome_installed
                && Self::is_chromium_extension_loaded(r"Google\Chrome\User Data"),
            edge_installed,
            edge_integrated: edge_installed
                && (Self::is_registry_key_registered(&edge_key)
                    || Self::is_chromium_extension_loaded(r"Microsoft\Edge\User Data")),
            firefox_installed,
            firefox_integrated: firefox_installed && Self::is_firefox_registry_registered(),
            brave_installed,
            brave_integrated: brave_installed
                && Self::is_chromium_extension_loaded(r"BraveSoftware\Brave-Browser\User Data"),
            extension_path: display_path,
            extension_id: EXTENSION_ID.to_string(),
        }
    }

    /// Automatically registers extension into installed Microsoft Edge and Mozilla Firefox browsers
    pub fn install_integration() -> Result<BrowserStatus, String> {
        let ext_path = Self::get_extension_path();
        if !ext_path.exists() {
            return Err(format!("Extension directory not found at: {}", ext_path.display()));
        }

        let edge_installed = Self::is_edge_installed();
        let firefox_installed = Self::is_firefox_installed();

        if !edge_installed && !firefox_installed {
            return Err("Neither Microsoft Edge nor Mozilla Firefox is installed on this PC.".to_string());
        }

        info!("Registering SpeedFlow browser extension into Windows registry from: {}", ext_path.display());

        let mut errors = Vec::new();

        // 1. Microsoft Edge (only if installed)
        if edge_installed {
            let edge_key = format!(r"HKCU\Software\Microsoft\Edge\Extensions\{EXTENSION_ID}");
            if let Err(e) = Self::write_registry_key(&edge_key, &ext_path) {
                warn!("Failed to register Edge extension: {e}");
                errors.push(format!("Edge: {e}"));
            }
        }

        // 2. Mozilla Firefox (only if installed)
        if firefox_installed {
            if let Err(e) = Self::write_firefox_registry(&ext_path) {
                warn!("Failed to register Firefox extension: {e}");
                errors.push(format!("Firefox: {e}"));
            }
        }

        if !errors.is_empty() && errors.len() == (edge_installed as usize + firefox_installed as usize) {
            return Err(format!("Browser registrations failed: {}", errors.join("; ")));
        }

        Ok(Self::get_status())
    }

    /// Unregisters extension from Windows registry
    pub fn remove_integration() -> Result<BrowserStatus, String> {
        let chrome_key = format!(r"HKCU\Software\Google\Chrome\Extensions\{EXTENSION_ID}");
        let edge_key = format!(r"HKCU\Software\Microsoft\Edge\Extensions\{EXTENSION_ID}");
        let brave_key = format!(r"HKCU\Software\BraveSoftware\Brave-Browser\Extensions\{EXTENSION_ID}");

        let _ = Self::delete_registry_key(&chrome_key);
        let _ = Self::delete_registry_key(&edge_key);
        let _ = Self::delete_registry_key(&brave_key);
        let _ = Self::delete_firefox_registry();

        Ok(Self::get_status())
    }

    /// Opens the extension folder in Windows Explorer
    pub fn open_extension_folder() -> Result<(), String> {
        let ext_path = Self::get_extension_path();
        Command::new("explorer")
            .arg(ext_path)
            .spawn()
            .map_err(|e| format!("Failed to open Explorer: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_path_resolution() {
        let p = BrowserIntegrationService::get_extension_path();
        assert!(p.exists(), "Extension directory must exist");
        assert!(p.join("manifest.json").exists(), "manifest.json must exist in extension directory");
    }

    #[test]
    fn test_browser_detection() {
        // At least one browser should be installed on test environment
        let chrome = BrowserIntegrationService::is_chrome_installed();
        let edge = BrowserIntegrationService::is_edge_installed();
        assert!(chrome || edge, "At least Chrome or Edge should be installed on Windows");
    }

    #[test]
    fn test_integration_lifecycle() {
        // 1. Install integration
        let status_after_install = BrowserIntegrationService::install_integration().expect("Install should succeed");
        assert!(status_after_install.chrome_integrated, "Chrome integration should be registered in HKCU");
        assert!(status_after_install.edge_integrated, "Edge integration should be registered in HKCU");

        // 2. Query status
        let query_status = BrowserIntegrationService::get_status();
        assert_eq!(query_status.extension_id, EXTENSION_ID);
        assert!(query_status.chrome_integrated);

        // 3. Remove integration
        let status_after_remove = BrowserIntegrationService::remove_integration().expect("Remove should succeed");
        assert!(!status_after_remove.chrome_integrated, "Chrome integration should be unregistered");
        assert!(!status_after_remove.edge_integrated, "Edge integration should be unregistered");

        // 4. Re-install to keep it active
        let _ = BrowserIntegrationService::install_integration();
    }
}

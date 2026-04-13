use std::path::PathBuf;

const APP_IDENTIFIER: &str = "com.magical-merchant.app";

/// Returns the app data directory, matching the Tauri app's data location.
///
/// On macOS: `~/Library/Application Support/com.magical-merchant.app/`
/// On Linux: `~/.local/share/com.magical-merchant.app/`
/// On Windows: `C:\Users\<user>\AppData\Roaming\com.magical-merchant.app\`
pub fn base_dir() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine data directory")
        .join(APP_IDENTIFIER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_dir_is_not_empty() {
        let dir = base_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn base_dir_ends_with_app_identifier() {
        let dir = base_dir();
        assert!(dir.ends_with(APP_IDENTIFIER));
    }
}

pub mod audit;
pub mod preflight;
pub mod protected_paths;

use std::fs;
use std::path::PathBuf;

pub fn ensure_log_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".storage-cleanup");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
    }
    dir
}

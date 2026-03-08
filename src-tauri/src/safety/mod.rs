pub mod audit;
pub mod preflight;
pub mod protected_paths;

use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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

/// Read the last `limit` entries from a JSONL file, deserializing each line.
pub fn read_jsonl_tail<T: DeserializeOwned>(path: &Path, limit: usize) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries: VecDeque<T> = VecDeque::with_capacity(limit);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<T>(&line) {
            if entries.len() == limit {
                entries.pop_front();
            }
            entries.push_back(entry);
        }
    }

    Ok(entries.into())
}

/// Open a file for appending with secure permissions (0o600 on Unix).
pub fn open_append_secure(path: &Path) -> anyhow::Result<File> {
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    Ok(opts.open(path)?)
}

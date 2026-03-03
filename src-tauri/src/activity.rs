use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityLevel {
    Info,
    Command,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Utc>,
    pub level: ActivityLevel,
    pub category: Option<String>,
    pub message: String,
    pub detail: Option<String>,
    pub command: Option<String>,
    pub duration_ms: Option<u64>,
}

pub struct ActivityLogger {
    app_handle: AppHandle,
    file: Mutex<File>,
}

fn activity_dir() -> PathBuf {
    crate::safety::ensure_log_dir()
}

fn activity_path() -> PathBuf {
    activity_dir().join("activity.log")
}

impl ActivityLogger {
    pub fn new(app_handle: AppHandle) -> anyhow::Result<Self> {
        // Rotate if file > 10 MB
        let path = activity_path();
        if path.exists()
            && let Ok(meta) = fs::metadata(&path)
            && meta.len() > 10 * 1024 * 1024
        {
            Self::rotate(&path);
        }

        let mut opts = OpenOptions::new();
        opts.create(true).append(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }

        let file = opts.open(&path)?;

        Ok(Self {
            app_handle,
            file: Mutex::new(file),
        })
    }

    fn rotate(path: &PathBuf) {
        // Keep last 5000 lines, using buffered line-by-line reading
        let Ok(file) = fs::File::open(path) else {
            return;
        };
        let lines: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();
        if lines.len() > 5000 {
            let kept = &lines[lines.len() - 5000..];
            let _ = fs::write(path, kept.join("\n") + "\n");
        }
    }

    pub fn log(&self, entry: &ActivityEntry) {
        if let Ok(json) = serde_json::to_string(entry) {
            if let Ok(mut file) = self.file.lock() {
                let _ = writeln!(file, "{}", json);
                let _ = file.flush();
            }
            let _ = self.app_handle.emit("activity-log", entry);
        }
    }

    pub fn info(&self, category: Option<&str>, message: &str) {
        self.log(&ActivityEntry {
            timestamp: Utc::now(),
            level: ActivityLevel::Info,
            category: category.map(|s| s.to_string()),
            message: message.to_string(),
            detail: None,
            command: None,
            duration_ms: None,
        });
    }

    pub fn command(
        &self,
        category: Option<&str>,
        message: &str,
        cmd: &str,
        duration_ms: Option<u64>,
    ) {
        self.log(&ActivityEntry {
            timestamp: Utc::now(),
            level: ActivityLevel::Command,
            category: category.map(|s| s.to_string()),
            message: message.to_string(),
            detail: None,
            command: Some(cmd.to_string()),
            duration_ms,
        });
    }

    pub fn success(
        &self,
        category: Option<&str>,
        message: &str,
        detail: Option<&str>,
        duration_ms: Option<u64>,
    ) {
        self.log(&ActivityEntry {
            timestamp: Utc::now(),
            level: ActivityLevel::Success,
            category: category.map(|s| s.to_string()),
            message: message.to_string(),
            detail: detail.map(|s| s.to_string()),
            command: None,
            duration_ms,
        });
    }

    pub fn warning(&self, category: Option<&str>, message: &str, detail: Option<&str>) {
        self.log(&ActivityEntry {
            timestamp: Utc::now(),
            level: ActivityLevel::Warning,
            category: category.map(|s| s.to_string()),
            message: message.to_string(),
            detail: detail.map(|s| s.to_string()),
            command: None,
            duration_ms: None,
        });
    }

    pub fn error(&self, category: Option<&str>, message: &str, detail: Option<&str>) {
        self.log(&ActivityEntry {
            timestamp: Utc::now(),
            level: ActivityLevel::Error,
            category: category.map(|s| s.to_string()),
            message: message.to_string(),
            detail: detail.map(|s| s.to_string()),
            command: None,
            duration_ms: None,
        });
    }
}

pub fn read_log(limit: usize) -> anyhow::Result<Vec<ActivityEntry>> {
    let path = activity_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries: VecDeque<ActivityEntry> = VecDeque::with_capacity(limit);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<ActivityEntry>(&line) {
            if entries.len() == limit {
                entries.pop_front();
            }
            entries.push_back(entry);
        }
    }

    Ok(entries.into())
}

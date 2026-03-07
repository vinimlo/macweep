use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::models::{CleanResult, ScanResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub category: String,
    pub label: String,
    pub path: String,
    pub size_bytes: u64,
    pub freed_bytes: u64,
    pub success: bool,
    pub error: Option<String>,
}

fn audit_path() -> PathBuf {
    super::ensure_log_dir().join("audit.jsonl")
}

fn open_audit_file() -> anyhow::Result<File> {
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    Ok(opts.open(audit_path())?)
}

fn build_entry(item: &ScanResult, result: &CleanResult) -> AuditEntry {
    AuditEntry {
        timestamp: Utc::now(),
        action: "clean".to_string(),
        category: item.category.clone(),
        label: item.label.clone(),
        path: item.path.clone(),
        size_bytes: item.size_bytes,
        freed_bytes: result.freed_bytes,
        success: result.success,
        error: result.error.clone(),
    }
}

pub fn log_cleanup_batch(pairs: &[(ScanResult, CleanResult)]) -> anyhow::Result<()> {
    if pairs.is_empty() {
        return Ok(());
    }
    let mut file = open_audit_file()?;
    for (item, result) in pairs {
        let entry = build_entry(item, result);
        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;
    }
    file.sync_all()?;
    Ok(())
}

pub fn read_log(limit: usize) -> anyhow::Result<Vec<AuditEntry>> {
    let path = audit_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries: std::collections::VecDeque<AuditEntry> =
        std::collections::VecDeque::with_capacity(limit);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
            if entries.len() == limit {
                entries.pop_front();
            }
            entries.push_back(entry);
        }
    }

    Ok(entries.into())
}

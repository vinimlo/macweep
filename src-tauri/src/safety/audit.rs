use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;
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

    // Rotate if over 10,000 entries (cap unbounded growth)
    let path = audit_path();
    rotate_if_needed(&path);

    let mut file = super::open_append_secure(&path)?;
    for (item, result) in pairs {
        let entry = build_entry(item, result);
        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;
    }
    file.sync_all()?;
    Ok(())
}

fn rotate_if_needed(path: &std::path::Path) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 10_000 {
        let kept = &lines[lines.len() - 5_000..];
        let _ = std::fs::write(path, kept.join("\n") + "\n");
    }
}

pub fn read_log(limit: usize) -> anyhow::Result<Vec<AuditEntry>> {
    super::read_jsonl_tail(&audit_path(), limit)
}

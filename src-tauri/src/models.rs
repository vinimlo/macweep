use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Zero,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: String,
    pub category: String,
    pub label: String,
    pub risk_level: RiskLevel,
    pub path: String,
    pub size_bytes: u64,
    pub detail: String,
    pub regeneration_hint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub id: String,
    pub freed_bytes: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScanProgress {
    Started {
        total_scanners: usize,
    },
    ScannerStarted {
        category: String,
    },
    ScannerCompleted {
        category: String,
        items_found: usize,
        bytes: u64,
    },
    ScannerFailed {
        category: String,
        error: String,
    },
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CleanProgress {
    Started {
        total_items: usize,
    },
    ItemCompleted {
        id: String,
        success: bool,
        freed_bytes: u64,
    },
    Completed {
        total_freed: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub items: Vec<ScanResult>,
    pub total_bytes: u64,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanReport {
    pub results: Vec<CleanResult>,
    /// Sum of per-item freed bytes. Filesystem items are measured before and after removal;
    /// tool-driven items (Docker, Ollama) report the scanner's estimate.
    pub freed_bytes: u64,
    /// Growth of free space on the startup disk during the cleanup. Can be lower than
    /// `freed_bytes`: Docker's disk image and APFS snapshots release space later.
    pub disk_freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub free_percent: f64,
}

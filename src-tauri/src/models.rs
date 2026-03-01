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
    Cancelled {
        completed_scanners: usize,
        total_scanners: usize,
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
    Failed {
        id: String,
        error: String,
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
    pub available_tools: Vec<String>,
    pub disk_free_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub free_percent: f64,
}

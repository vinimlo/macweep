use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

pub struct PipScanner;

#[async_trait]
impl Scanner for PipScanner {
    fn category(&self) -> &str {
        "pip-cache"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Zero
    }

    async fn is_available(&self) -> bool {
        scanner::tool_installed("pip3").await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let cache_path = home.join("Library/Caches/pip");
        let path_str = cache_path.to_string_lossy().to_string();
        let size = scanner::dir_size_bytes(&path_str).await;

        if size == 0 {
            return Ok(vec![]);
        }

        Ok(vec![ScanResult {
            id: uuid::Uuid::new_v4().to_string(),
            category: self.category().to_string(),
            label: "pip cache".to_string(),
            risk_level: self.risk_level(),
            path: path_str,
            size_bytes: size,
            detail: "pip download cache — safe to remove".to_string(),
            regeneration_hint: "pip install will re-download as needed".to_string(),
            warning: None,
        }])
    }

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        let mut results = Vec::new();
        for item in items {
            let mut cmd = Command::new("pip3");
            cmd.args(["cache", "purge"]);
            results.push(scanner::clean_with_command(item, &mut cmd, 120).await);
        }
        results
    }
}

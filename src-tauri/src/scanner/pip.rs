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

        if !cache_path.exists() {
            return Ok(vec![]);
        }

        let path_str = cache_path.to_string_lossy().to_string();
        let du_output = Command::new("du").args(["-sk", &path_str]).output().await?;
        let du_str = String::from_utf8_lossy(&du_output.stdout);
        let size_kb: u64 = du_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if size_kb == 0 {
            return Ok(vec![]);
        }

        Ok(vec![ScanResult {
            id: uuid::Uuid::new_v4().to_string(),
            category: self.category().to_string(),
            label: "pip cache".to_string(),
            risk_level: self.risk_level(),
            path: path_str,
            size_bytes: size_kb * 1024,
            detail: "pip download cache — safe to remove".to_string(),
            regeneration_hint: "pip install will re-download as needed".to_string(),
            warning: None,
        }])
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        let mut results = Vec::new();
        for item in items {
            let output = Command::new("pip3")
                .args(["cache", "purge"])
                .output()
                .await?;

            results.push(scanner::command_to_clean_result(item, &output));
        }
        Ok(results)
    }
}

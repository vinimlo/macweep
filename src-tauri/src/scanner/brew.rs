use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

pub struct BrewScanner;

#[async_trait]
impl Scanner for BrewScanner {
    fn category(&self) -> &str {
        "brew-cache"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Zero
    }

    async fn is_available(&self) -> bool {
        scanner::tool_installed("brew").await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let cache_output = Command::new("brew").arg("--cache").output().await?;
        let cache_path = String::from_utf8_lossy(&cache_output.stdout)
            .trim()
            .to_string();

        if cache_path.is_empty() || !std::path::Path::new(&cache_path).exists() {
            return Ok(vec![]);
        }

        // Validate brew cache path is under expected locations
        let valid_prefixes = ["/opt/homebrew", "/usr/local"];
        let home = dirs::home_dir().unwrap_or_default();
        let home_caches = home.join("Library/Caches/Homebrew");
        let is_valid = valid_prefixes.iter().any(|p| cache_path.starts_with(p))
            || cache_path.starts_with(&home_caches.to_string_lossy().to_string());
        if !is_valid {
            return Ok(vec![]);
        }

        let du_output = Command::new("du")
            .args(["-sk", &cache_path])
            .output()
            .await?;
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
            label: "Homebrew cache".to_string(),
            risk_level: self.risk_level(),
            path: cache_path,
            size_bytes: size_kb * 1024,
            detail: "Homebrew download cache — safe to remove".to_string(),
            regeneration_hint: "brew install will re-download as needed".to_string(),
        }])
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        let mut results = Vec::new();
        for item in items {
            let output = Command::new("brew")
                .args(["cleanup", "--prune=all"])
                .output()
                .await?;

            results.push(scanner::command_to_clean_result(item, &output));
        }
        Ok(results)
    }
}

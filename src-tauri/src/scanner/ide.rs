use anyhow::Result;
use async_trait::async_trait;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

pub struct IdeScanner;

fn app_installed(app_name: &str) -> bool {
    let path = format!("/Applications/{}.app", app_name);
    std::path::Path::new(&path).exists()
}

#[async_trait]
impl Scanner for IdeScanner {
    fn category(&self) -> &str {
        "ide-unused"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let mut items = Vec::new();

        let ide_dirs = vec![
            (".trae", "Trae", "Trae IDE"),
            (".antigravity", "Antigravity", "Antigravity IDE"),
            (".cursor", "Cursor", "Cursor IDE"),
        ];

        for (dir_name, app_name, label) in ide_dirs {
            let dir = home.join(dir_name);
            if dir.exists() {
                let installed = app_installed(app_name);
                let path = dir.to_string_lossy().to_string();
                let size = scanner::dir_size_bytes(&path).await;
                if size > 0 {
                    items.push(ScanResult {
                        id: uuid::Uuid::new_v4().to_string(),
                        category: self.category().to_string(),
                        label: format!("{} config", label),
                        risk_level: self.risk_level(),
                        path,
                        size_bytes: size,
                        detail: if installed {
                            format!("{} is installed — config may be in use", label)
                        } else {
                            format!("{} is NOT installed — config is likely stale", label)
                        },
                        regeneration_hint: format!("{} will recreate config on first launch", label),
                    });
                }
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        scanner::clean_filesystem_items(items).await
    }
}

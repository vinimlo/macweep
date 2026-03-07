use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;
use walkdir::WalkDir;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

pub struct NodeCacheScanner;
pub struct NodeModulesScanner;

#[async_trait]
impl Scanner for NodeCacheScanner {
    fn category(&self) -> &str {
        "node-cache"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Zero
    }

    fn handles_category(&self, cat: &str) -> bool {
        matches!(cat, "node-cache" | "npm-cache" | "yarn-cache" | "bun-cache")
    }

    async fn is_available(&self) -> bool {
        scanner::tool_installed("npm").await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let mut items = Vec::new();

        let caches = [
            (".npm", "npm-cache", "npm cache", "npm download cache — safe to remove", "npm install will re-download as needed"),
            ("Library/Caches/yarn", "yarn-cache", "Yarn cache", "Yarn download cache — safe to remove", "yarn install will re-download as needed"),
            (".bun/install/cache", "bun-cache", "Bun cache", "Bun download cache — safe to remove", "bun install will re-download as needed"),
        ];

        for (dir, category, label, detail, hint) in caches {
            let cache_dir = home.join(dir);
            if cache_dir.exists() {
                let path = cache_dir.to_string_lossy().to_string();
                let size = scanner::dir_size_bytes(&path).await;
                if size > 0 {
                    items.push(ScanResult {
                        id: uuid::Uuid::new_v4().to_string(),
                        category: category.to_string(),
                        label: label.to_string(),
                        risk_level: RiskLevel::Zero,
                        path,
                        size_bytes: size,
                        detail: detail.to_string(),
                        regeneration_hint: hint.to_string(),
                    });
                }
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        let mut results = Vec::new();
        for item in items {
            let output = match item.category.as_str() {
                "npm-cache" => {
                    Command::new("npm")
                        .args(["cache", "clean", "--force"])
                        .output()
                        .await?
                }
                "yarn-cache" => {
                    Command::new("yarn")
                        .args(["cache", "clean"])
                        .output()
                        .await?
                }
                "bun-cache" => {
                    Command::new("bun")
                        .args(["pm", "cache", "rm"])
                        .output()
                        .await?
                }
                _ => {
                    results.push(CleanResult {
                        id: item.id.clone(),
                        freed_bytes: 0,
                        success: false,
                        error: Some(format!("Unknown node cache category: {}", item.category)),
                    });
                    continue;
                }
            };

            results.push(scanner::command_to_clean_result(item, &output));
        }
        Ok(results)
    }
}

// --- NodeModulesScanner (Risk Low) ---

const SEARCH_DIRS: &[&str] = &[
    "Projects", "projects", "dev", "Developer", "Code", "code", "testeProjetos",
];

#[async_trait]
impl Scanner for NodeModulesScanner {
    fn category(&self) -> &str {
        "node-modules"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn handles_category(&self, cat: &str) -> bool {
        matches!(cat, "node-modules" | "next-cache")
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let mut items = Vec::new();

        // Single walk finds both node_modules and .next/cache
        for dir_name in SEARCH_DIRS {
            let search_dir = home.join(dir_name);
            if !search_dir.exists() {
                continue;
            }
            for entry in WalkDir::new(&search_dir)
                .max_depth(5)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    name != ".git" && name != "target" && name != "node_modules"
                })
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_dir() {
                    continue;
                }

                let name = entry.file_name().to_string_lossy();

                // Check for node_modules
                if name == "node_modules" {
                    let path = entry.path().to_string_lossy().to_string();
                    let size = scanner::dir_size_bytes(&path).await;
                    if size > 1_048_576 {
                        let parent = entry
                            .path()
                            .parent()
                            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                            .unwrap_or_default();

                        items.push(ScanResult {
                            id: uuid::Uuid::new_v4().to_string(),
                            category: self.category().to_string(),
                            label: format!("node_modules ({})", parent),
                            risk_level: self.risk_level(),
                            path,
                            size_bytes: size,
                            detail: format!("node_modules in project \"{}\"", parent),
                            regeneration_hint: "npm install / yarn install / bun install".to_string(),
                        });
                    }
                }

                // Check for .next/cache
                if name == "cache"
                    && entry.path().parent().map(|p| p.file_name().unwrap_or_default() == ".next").unwrap_or(false)
                {
                    let path = entry.path().to_string_lossy().to_string();
                    let size = scanner::dir_size_bytes(&path).await;
                    if size > 1_048_576 {
                        let project = entry
                            .path()
                            .ancestors()
                            .nth(2)
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        items.push(ScanResult {
                            id: uuid::Uuid::new_v4().to_string(),
                            category: "next-cache".to_string(),
                            label: format!(".next/cache ({})", project),
                            risk_level: RiskLevel::Low,
                            path,
                            size_bytes: size,
                            detail: format!("Next.js build cache in \"{}\"", project),
                            regeneration_hint: "Next.js will rebuild on next dev/build".to_string(),
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        scanner::clean_filesystem_items(items).await
    }
}

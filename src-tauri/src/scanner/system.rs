use anyhow::Result;
use async_trait::async_trait;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

pub struct SystemCacheScanner;
pub struct AppSupportScanner;
pub struct LogsScanner;

#[async_trait]
impl Scanner for SystemCacheScanner {
    fn category(&self) -> &str {
        "system-cache"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Zero
    }

    fn handles_category(&self, cat: &str) -> bool {
        matches!(cat, "system-cache" | "ts-cache" | "cursor-updates")
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let mut items = Vec::new();

        let caches = [
            (
                "Library/Caches/typescript",
                "ts-cache",
                "TypeScript cache",
                "TypeScript compilation cache — safe to remove",
                "TypeScript will recreate cache on next compilation",
            ),
            (
                "Library/Caches/com.todesktop.230313mzl4w4u92.ShipIt",
                "cursor-updates",
                "Cursor update cache",
                "Cursor editor update downloads — safe to remove",
                "Cursor will re-download updates when needed",
            ),
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
        scanner::clean_filesystem_items(items).await
    }
}

// --- AppSupportScanner (Risk High) ---

#[async_trait]
impl Scanner for AppSupportScanner {
    fn category(&self) -> &str {
        "app-support"
    }
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }
    async fn is_available(&self) -> bool {
        true
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        use crate::safety::protected_paths;

        let home = dirs::home_dir().unwrap_or_default();
        let app_support = home.join("Library/Application Support");
        if !app_support.exists() {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        let entries: Vec<_> = std::fs::read_dir(&app_support)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        let mut sized: Vec<(String, String, u64)> = Vec::new();
        for entry in &entries {
            let path = entry.path().to_string_lossy().to_string();
            if protected_paths::is_protected(&path) {
                continue;
            }
            let size = scanner::dir_size_bytes(&path).await;
            if size > 104_857_600 {
                let name = entry.file_name().to_string_lossy().to_string();
                sized.push((name, path, size));
            }
        }
        sized.sort_by(|a, b| b.2.cmp(&a.2));

        for (name, path, size) in sized.into_iter().take(10) {
            items.push(ScanResult {
                id: uuid::Uuid::new_v4().to_string(),
                category: self.category().to_string(),
                label: format!("App Support: {}", name),
                risk_level: self.risk_level(),
                path,
                size_bytes: size,
                detail: format!(
                    "Application data for \"{}\" — may contain important state",
                    name
                ),
                regeneration_hint: "Application may need to be reconfigured after removal"
                    .to_string(),
            });
        }
        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        scanner::clean_filesystem_items(items).await
    }
}

// --- LogsScanner (Risk High) ---

#[async_trait]
impl Scanner for LogsScanner {
    fn category(&self) -> &str {
        "logs"
    }
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }
    async fn is_available(&self) -> bool {
        true
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let logs_dir = home.join("Library/Logs");
        if !logs_dir.exists() {
            return Ok(vec![]);
        }

        let path = logs_dir.to_string_lossy().to_string();
        let size = scanner::dir_size_bytes(&path).await;

        if size > 10_485_760 {
            Ok(vec![ScanResult {
                id: uuid::Uuid::new_v4().to_string(),
                category: self.category().to_string(),
                label: "User logs".to_string(),
                risk_level: self.risk_level(),
                path,
                size_bytes: size,
                detail: "Application logs — may be needed for debugging".to_string(),
                regeneration_hint: "Logs are recreated by applications as they run".to_string(),
            }])
        } else {
            Ok(vec![])
        }
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        // LogsScanner removes contents but keeps the directory
        let mut results = Vec::new();
        for item in items {
            let path = std::path::Path::new(&item.path);
            if !path.exists() {
                results.push(CleanResult {
                    id: item.id.clone(),
                    freed_bytes: item.size_bytes,
                    success: true,
                    error: None,
                });
                continue;
            }
            if let Err(e) = crate::safety::protected_paths::validate_before_delete(path) {
                results.push(CleanResult {
                    id: item.id.clone(),
                    freed_bytes: 0,
                    success: false,
                    error: Some(e.to_string()),
                });
                continue;
            }
            let mut ok = true;
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let ep = entry.path();
                    if ep.is_dir() {
                        if tokio::fs::remove_dir_all(&ep).await.is_err() {
                            ok = false;
                        }
                    } else {
                        if tokio::fs::remove_file(&ep).await.is_err() {
                            ok = false;
                        }
                    }
                }
            }
            results.push(CleanResult {
                id: item.id.clone(),
                freed_bytes: if ok { item.size_bytes } else { 0 },
                success: ok,
                error: if ok {
                    None
                } else {
                    Some("Some log files could not be removed".to_string())
                },
            });
        }
        Ok(results)
    }
}

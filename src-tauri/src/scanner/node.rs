use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;
use walkdir::WalkDir;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

async fn has_uncommitted_changes(project_dir: &Path) -> bool {
    let Ok(output) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Command::new("git")
            .args([
                "-C",
                &project_dir.to_string_lossy(),
                "status",
                "--porcelain",
            ])
            .output(),
    )
    .await
    else {
        return false;
    };
    let Ok(output) = output else {
        return false;
    };
    output.status.success() && !output.stdout.is_empty()
}

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

    /// The caches are plain download directories, so they are measured and removed
    /// directly. This does not depend on npm/yarn/bun being on the app's PATH (GUI apps
    /// get a minimal PATH, and bun lives in ~/.bun/bin).
    async fn is_available(&self) -> bool {
        true
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let mut items = Vec::new();

        let caches = [
            (
                // What `npm cache clean --force` removes; the rest of ~/.npm is left alone.
                ".npm/_cacache",
                "npm-cache",
                "npm cache",
                "npm download cache — safe to remove",
                "npm install will re-download as needed",
            ),
            (
                "Library/Caches/Yarn",
                "yarn-cache",
                "Yarn cache",
                "Yarn download cache — safe to remove",
                "yarn install will re-download as needed",
            ),
            (
                ".bun/install/cache",
                "bun-cache",
                "Bun cache",
                "Bun download cache — safe to remove",
                "bun install will re-download as needed",
            ),
        ];

        for (dir, category, label, detail, hint) in caches {
            let cache_dir = home.join(dir);
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
                    warning: None,
                });
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        scanner::clean_filesystem_items(items).await
    }
}

// --- NodeModulesScanner (Risk Low) ---

/// Directories under $HOME that physically cannot contain user projects.
/// Skipping these keeps the walk fast and avoids scanning huge system trees.
const SKIP_DIRS: &[&str] = &[
    // macOS system / user data
    "Library",
    ".Trash",
    "Applications",
    "Public",
    // Media (never contain code projects)
    "Movies",
    "Music",
    "Pictures",
    "Photos",
    // Protected user dirs (safety — don't even scan)
    "Documents",
    "Desktop",
    "Downloads",
    // Heavy toolchain/runtime dirs
    ".rustup",
    ".cargo",
    ".ollama",
    ".cache",
    ".docker",
    ".local",
    ".npm",
    ".yarn",
    ".bun",
    ".pnpm",
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

        // Walk $HOME directly — finds node_modules and .next/cache anywhere
        let walker = WalkDir::new(&home).max_depth(6).follow_links(false);

        let mut it = walker.into_iter();
        while let Some(entry) = it.next() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let name = entry.file_name().to_string_lossy();

            // Skip directories we never want to descend into
            if entry.file_type().is_dir()
                && (name == ".git"
                    || name == "target"
                    || SKIP_DIRS.iter().any(|&s| s.eq_ignore_ascii_case(&name)))
            {
                it.skip_current_dir();
                continue;
            }

            if !entry.file_type().is_dir() {
                continue;
            }

            // Check for node_modules
            if name == "node_modules" {
                it.skip_current_dir(); // don't walk inside node_modules

                let path = entry.path().to_string_lossy().to_string();
                let size = scanner::dir_size_bytes(&path).await;
                if size > 1_048_576 {
                    let project_dir = entry.path().parent();
                    let parent = project_dir
                        .map(|p| {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        })
                        .unwrap_or_default();

                    let warning = match project_dir {
                        Some(dir) if has_uncommitted_changes(dir).await => {
                            Some("Project has uncommitted Git changes".to_string())
                        }
                        _ => None,
                    };

                    items.push(ScanResult {
                        id: uuid::Uuid::new_v4().to_string(),
                        category: self.category().to_string(),
                        label: format!("node_modules ({})", parent),
                        risk_level: self.risk_level(),
                        path,
                        size_bytes: size,
                        detail: format!("node_modules in project \"{}\"", parent),
                        regeneration_hint: "npm install / yarn install / bun install".to_string(),
                        warning,
                    });
                }
                continue;
            }

            // Check for .next/cache
            if name == "cache"
                && entry
                    .path()
                    .parent()
                    .map(|p| p.file_name().unwrap_or_default() == ".next")
                    .unwrap_or(false)
            {
                it.skip_current_dir();

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
                        warning: None,
                    });
                }
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        scanner::clean_filesystem_items(items).await
    }
}

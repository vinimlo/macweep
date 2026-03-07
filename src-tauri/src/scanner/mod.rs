pub mod ai_tools;
pub mod brew;
pub mod docker;
pub mod ide;
pub mod node;
pub mod pip;
pub mod system;

use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::models::{CleanResult, RiskLevel, ScanResult};

#[async_trait]
pub trait Scanner: Send + Sync {
    fn category(&self) -> &str;
    fn risk_level(&self) -> RiskLevel;
    /// Whether this scanner handles items with the given category.
    /// Override for scanners that produce sub-categories different from `category()`.
    fn handles_category(&self, cat: &str) -> bool {
        cat == self.category()
    }
    async fn is_available(&self) -> bool;
    async fn scan(&self) -> Result<Vec<ScanResult>>;
    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>>;
}

pub fn build_registry() -> Vec<Box<dyn Scanner>> {
    vec![
        // Risk Zero
        Box::new(brew::BrewScanner),
        Box::new(pip::PipScanner),
        Box::new(node::NodeCacheScanner),
        Box::new(docker::DockerBuildCacheScanner),
        Box::new(system::SystemCacheScanner),
        // Risk Low
        Box::new(node::NodeModulesScanner),
        Box::new(docker::DockerImagesScanner),
        Box::new(docker::DockerOrphanVolumesScanner),
        // Risk Medium
        Box::new(ai_tools::AiToolsScanner),
        Box::new(ide::IdeScanner),
        // Risk High
        Box::new(docker::DockerNamedVolumesScanner),
        Box::new(system::AppSupportScanner),
        Box::new(system::LogsScanner),
    ]
}

/// Run an external command with a timeout (default 30s).
/// Returns an error if the command exceeds the timeout.
pub async fn run_with_timeout(cmd: &mut Command, secs: u64) -> Result<std::process::Output> {
    timeout(Duration::from_secs(secs), cmd.output())
        .await
        .map_err(|_| anyhow::anyhow!("Command timed out after {}s", secs))?
        .map_err(Into::into)
}

/// Get the size of a directory in bytes using `du -sk`.
pub async fn dir_size_bytes(path: &str) -> u64 {
    Command::new("du")
        .args(["-sk", path])
        .output()
        .await
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
        })
        .unwrap_or(0)
        * 1024
}

/// Build a `CleanResult` from a command's output.
/// Used by scanners that clean via an external CLI command.
pub fn command_to_clean_result(item: &ScanResult, output: &std::process::Output) -> CleanResult {
    CleanResult {
        id: item.id.clone(),
        freed_bytes: if output.status.success() {
            item.size_bytes
        } else {
            0
        },
        success: output.status.success(),
        error: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    }
}

/// Parse a size string with separate number and unit parts (e.g. "5.2", "GB").
pub fn parse_size_with_units(num_str: &str, unit: &str) -> u64 {
    let num: f64 = num_str.trim().parse().unwrap_or(0.0);
    let multiplier = match unit.trim_start() {
        "GB" => 1_073_741_824.0,
        "MB" => 1_048_576.0,
        "KB" | "kB" => 1024.0,
        _ => 1.0,
    };
    let result = num * multiplier;
    if result >= u64::MAX as f64 {
        u64::MAX
    } else {
        result as u64
    }
}

/// Check if a command-line tool is available via `which`.
pub async fn tool_installed(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Clean filesystem items with the standard validate-and-remove pattern.
/// Checks existence, validates against protected paths, then removes.
pub async fn clean_filesystem_items(
    items: &[crate::models::ScanResult],
) -> Result<Vec<crate::models::CleanResult>> {
    use crate::safety::protected_paths;
    let mut results = Vec::new();
    for item in items {
        let path = std::path::Path::new(&item.path);
        if !path.exists() {
            results.push(crate::models::CleanResult {
                id: item.id.clone(),
                freed_bytes: item.size_bytes,
                success: true,
                error: None,
            });
            continue;
        }
        if let Err(e) = protected_paths::validate_before_delete(path) {
            results.push(crate::models::CleanResult {
                id: item.id.clone(),
                freed_bytes: 0,
                success: false,
                error: Some(e.to_string()),
            });
            continue;
        }
        match tokio::fs::remove_dir_all(path).await {
            Ok(()) => results.push(crate::models::CleanResult {
                id: item.id.clone(),
                freed_bytes: item.size_bytes,
                success: true,
                error: None,
            }),
            Err(e) => results.push(crate::models::CleanResult {
                id: item.id.clone(),
                freed_bytes: 0,
                success: false,
                error: Some(format!("Failed to remove: {}", e)),
            }),
        }
    }
    Ok(results)
}

/// All known item-level categories for input validation.
pub const KNOWN_CATEGORIES: &[&str] = &[
    // Scanner-level categories
    "brew-cache",
    "pip-cache",
    "node-cache",
    "docker-build",
    "system-cache",
    "node-modules",
    "docker-images",
    "docker-volumes-orphan",
    "ai-tools",
    "ide-unused",
    "docker-volumes-named",
    "app-support",
    "logs",
    // Item-level categories (sub-categories)
    "npm-cache",
    "yarn-cache",
    "bun-cache",
    "ts-cache",
    "cursor-updates",
    "next-cache",
    "ollama-models",
    "langflow",
    "gemini-cache",
    "coderabbit",
    "opencode",
];

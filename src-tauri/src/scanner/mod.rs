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
use tokio::time::{Duration, timeout};

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
    /// Clean the given items, returning exactly one result per item.
    /// A failure on one item must never abort the others.
    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult>;
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

/// Run an external command with a timeout. The child is killed if the timeout expires,
/// so a slow command never keeps running unobserved in the background.
pub async fn run_with_timeout(cmd: &mut Command, secs: u64) -> Result<std::process::Output> {
    timeout(Duration::from_secs(secs), cmd.kill_on_drop(true).output())
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

/// Clean one item with an external CLI command. Spawn errors (tool not on PATH),
/// timeouts and non-zero exits all become a failed result for this item only.
pub async fn clean_with_command(item: &ScanResult, cmd: &mut Command, secs: u64) -> CleanResult {
    match run_with_timeout(cmd, secs).await {
        Ok(output) if output.status.success() => success_result(item),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = stderr.trim();
            error_result(
                item,
                if message.is_empty() {
                    format!("Command failed ({})", output.status)
                } else {
                    message.to_string()
                },
            )
        }
        Err(e) => error_result(item, e.to_string()),
    }
}

/// Build a `CleanResult` for a validation-rejected or failed item.
pub fn error_result(item: &ScanResult, msg: String) -> CleanResult {
    CleanResult {
        id: item.id.clone(),
        freed_bytes: 0,
        success: false,
        error: Some(msg),
    }
}

/// Build a `CleanResult` for a successfully cleaned item.
pub fn success_result(item: &ScanResult) -> CleanResult {
    CleanResult {
        id: item.id.clone(),
        freed_bytes: item.size_bytes,
        success: true,
        error: None,
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

/// Check if a command-line tool exists in any PATH directory.
/// Probes the filesystem directly instead of shelling out to `which`,
/// which may not resolve correctly in GUI app contexts.
pub async fn tool_installed(cmd: &str) -> bool {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let full = format!("{}/{}", dir, cmd);
        if tokio::fs::metadata(&full).await.is_ok() {
            return true;
        }
    }
    false
}

/// Clean filesystem items with the standard validate-and-remove pattern.
/// Checks existence, validates against protected paths, then removes.
pub async fn clean_filesystem_items(items: &[ScanResult]) -> Vec<CleanResult> {
    use crate::safety::protected_paths;
    let mut results = Vec::new();
    for item in items {
        let path = std::path::Path::new(&item.path);
        if !path.exists() {
            // Removed by something else since the scan: nothing was freed by us.
            results.push(CleanResult {
                freed_bytes: 0,
                ..success_result(item)
            });
            continue;
        }
        if let Err(e) = protected_paths::validate_before_delete(path) {
            results.push(error_result(item, e.to_string()));
            continue;
        }
        match tokio::fs::remove_dir_all(path).await {
            Ok(()) => results.push(success_result(item)),
            Err(e) => results.push(error_result(item, format!("Failed to remove: {}", e))),
        }
    }
    results
}

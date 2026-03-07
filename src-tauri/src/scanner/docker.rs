use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

/// Validate Docker image ID or name contains only safe characters.
fn is_valid_docker_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 512
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:/-".contains(c))
}

/// Validate Docker volume name contains only safe characters.
fn is_valid_volume_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

pub struct DockerBuildCacheScanner;
pub struct DockerImagesScanner;
pub struct DockerOrphanVolumesScanner;
pub struct DockerNamedVolumesScanner;

async fn docker_available() -> bool {
    scanner::run_with_timeout(Command::new("docker").arg("info"), 10)
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Shared clean implementation for Docker volume scanners.
async fn clean_docker_volumes(items: &[ScanResult]) -> Result<Vec<CleanResult>> {
    let mut results = Vec::new();
    for item in items {
        if !is_valid_volume_name(&item.path) {
            results.push(scanner::error_result(
                item,
                "Invalid Docker volume name".to_string(),
            ));
            continue;
        }
        let output = scanner::run_with_timeout(
            Command::new("docker").args(["volume", "rm", &item.path]),
            30,
        )
        .await?;
        results.push(scanner::command_to_clean_result(item, &output));
    }
    Ok(results)
}

#[async_trait]
impl Scanner for DockerBuildCacheScanner {
    fn category(&self) -> &str {
        "docker-build"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Zero
    }

    async fn is_available(&self) -> bool {
        docker_available().await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let output = scanner::run_with_timeout(
            Command::new("docker").args([
                "system",
                "df",
                "--format",
                "{{.Type}}\t{{.Size}}\t{{.Reclaimable}}",
            ]),
            30,
        )
        .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 && parts[0] == "Build Cache" {
                let size_bytes = parse_docker_size(parts[1]);
                if size_bytes > 0 {
                    items.push(ScanResult {
                        id: uuid::Uuid::new_v4().to_string(),
                        category: self.category().to_string(),
                        label: "Docker build cache".to_string(),
                        risk_level: self.risk_level(),
                        path: "docker build cache".to_string(),
                        size_bytes,
                        detail: format!("Docker build cache — {}", parts[2].trim()),
                        regeneration_hint: "docker build will recreate cache layers as needed"
                            .to_string(),
                        warning: None,
                    });
                }
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        let mut results = Vec::new();
        for item in items {
            let output = scanner::run_with_timeout(
                Command::new("docker").args(["builder", "prune", "-a", "-f"]),
                30,
            )
            .await?;

            results.push(scanner::command_to_clean_result(item, &output));
        }
        Ok(results)
    }
}

// --- DockerImagesScanner (Risk Low) ---

#[async_trait]
impl Scanner for DockerImagesScanner {
    fn category(&self) -> &str {
        "docker-images"
    }
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }
    async fn is_available(&self) -> bool {
        docker_available().await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let output = scanner::run_with_timeout(
            Command::new("docker").args([
                "images",
                "--format",
                "{{.Repository}}:{{.Tag}}\t{{.ID}}\t{{.Size}}",
            ]),
            30,
        )
        .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let name = parts[0];
                let id = parts[1];
                let size_bytes = parse_docker_size(parts[2]);
                if size_bytes > 0 {
                    items.push(ScanResult {
                        id: uuid::Uuid::new_v4().to_string(),
                        category: self.category().to_string(),
                        label: format!("Image: {}", name),
                        risk_level: self.risk_level(),
                        path: id.to_string(),
                        size_bytes,
                        detail: format!("Docker image {} ({})", name, id),
                        regeneration_hint: format!("docker pull {}", name),
                        warning: None,
                    });
                }
            }
        }
        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        let mut results = Vec::new();
        for item in items {
            if !is_valid_docker_id(&item.path) {
                results.push(scanner::error_result(
                    item,
                    "Invalid Docker image ID".to_string(),
                ));
                continue;
            }
            let output = scanner::run_with_timeout(
                Command::new("docker").args(["rmi", "-f", &item.path]),
                30,
            )
            .await?;
            results.push(scanner::command_to_clean_result(item, &output));
        }
        Ok(results)
    }
}

// --- DockerOrphanVolumesScanner (Risk Low) ---

#[async_trait]
impl Scanner for DockerOrphanVolumesScanner {
    fn category(&self) -> &str {
        "docker-volumes-orphan"
    }
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }
    async fn is_available(&self) -> bool {
        docker_available().await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let output = scanner::run_with_timeout(
            Command::new("docker").args([
                "volume",
                "ls",
                "-f",
                "dangling=true",
                "--format",
                "{{.Name}}",
            ]),
            30,
        )
        .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for name in stdout.lines() {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            items.push(ScanResult {
                id: uuid::Uuid::new_v4().to_string(),
                category: self.category().to_string(),
                label: format!("Orphan volume: {}", name),
                risk_level: self.risk_level(),
                path: name.to_string(),
                size_bytes: 1_048_576, // 1MB estimate
                detail: "Dangling Docker volume — not attached to any container".to_string(),
                regeneration_hint: "docker compose up will recreate as needed".to_string(),
                warning: None,
            });
        }
        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        clean_docker_volumes(items).await
    }
}

// --- DockerNamedVolumesScanner (Risk High) ---

#[async_trait]
impl Scanner for DockerNamedVolumesScanner {
    fn category(&self) -> &str {
        "docker-volumes-named"
    }
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }
    async fn is_available(&self) -> bool {
        docker_available().await
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let output = scanner::run_with_timeout(
            Command::new("docker").args(["volume", "ls", "--format", "{{.Name}}"]),
            30,
        )
        .await?;

        let dangling = scanner::run_with_timeout(
            Command::new("docker").args([
                "volume",
                "ls",
                "-f",
                "dangling=true",
                "--format",
                "{{.Name}}",
            ]),
            30,
        )
        .await?;
        let dangling_names: HashSet<String> = String::from_utf8_lossy(&dangling.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .collect();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for name in stdout.lines() {
            let name = name.trim();
            if name.is_empty() || dangling_names.contains(name) {
                continue;
            }

            items.push(ScanResult {
                id: uuid::Uuid::new_v4().to_string(),
                category: self.category().to_string(),
                label: format!("Volume: {}", name),
                risk_level: self.risk_level(),
                path: name.to_string(),
                size_bytes: 1_048_576,
                detail: "Named Docker volume — may contain database or application data"
                    .to_string(),
                regeneration_hint: "Data will be LOST. Only remove if you're sure.".to_string(),
                warning: None,
            });
        }
        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>> {
        clean_docker_volumes(items).await
    }
}

/// Parse Docker size strings like "5.2GB", "100MB", "512kB", "1024B".
/// Splits the concatenated number+unit and delegates to the shared parser.
pub fn parse_docker_size(s: &str) -> u64 {
    let s = s.trim();
    let (num_str, unit) = if let Some(n) = s.strip_suffix("GB") {
        (n, "GB")
    } else if let Some(n) = s.strip_suffix("MB") {
        (n, "MB")
    } else if let Some(n) = s.strip_suffix("kB") {
        (n, "kB")
    } else if let Some(n) = s.strip_suffix('B') {
        (n, "B")
    } else {
        return 0;
    };
    scanner::parse_size_with_units(num_str, unit)
}

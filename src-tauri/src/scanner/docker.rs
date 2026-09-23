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
        && !s.starts_with('-')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:/-".contains(c))
}

/// Validate Docker volume name contains only safe characters.
fn is_valid_volume_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && !s.starts_with('-')
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

struct Container {
    id: String,
    name: String,
    running: bool,
}

/// Containers (running or stopped) that mount the given volume.
async fn containers_using(volume: &str) -> Result<Vec<Container>> {
    let output = scanner::run_with_timeout(
        Command::new("docker").args([
            "ps",
            "-a",
            "--filter",
            &format!("volume={volume}"),
            "--format",
            "{{.ID}}\t{{.Names}}\t{{.State}}",
        ]),
        15,
    )
    .await?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let id = parts.next()?.trim();
            let name = parts.next()?.trim();
            let state = parts.next().unwrap_or("").trim();
            is_valid_docker_id(id).then(|| Container {
                id: id.to_string(),
                name: name.to_string(),
                running: state == "running",
            })
        })
        .collect())
}

fn container_names(containers: &[Container]) -> String {
    containers
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Remove a volume. Docker refuses to remove a volume that any container still references,
/// even a stopped one, so the stopped containers using it are removed first (the scan
/// discloses them in the item's warning). Running containers are never touched.
async fn remove_volume(item: &ScanResult) -> CleanResult {
    if !is_valid_volume_name(&item.path) {
        return scanner::error_result(item, "Invalid Docker volume name".to_string());
    }
    let containers = match containers_using(&item.path).await {
        Ok(c) => c,
        Err(e) => return scanner::error_result(item, format!("Could not inspect volume: {e}")),
    };
    if containers.iter().any(|c| c.running) {
        return scanner::error_result(
            item,
            format!(
                "Volume is in use by a running container ({}) — stop it first",
                container_names(&containers)
            ),
        );
    }
    if !containers.is_empty() {
        let mut rm = Command::new("docker");
        rm.arg("rm").args(containers.iter().map(|c| c.id.as_str()));
        let removed = scanner::clean_with_command(item, &mut rm, 60).await;
        if !removed.success {
            return removed;
        }
    }
    let mut cmd = Command::new("docker");
    cmd.args(["volume", "rm", &item.path]);
    scanner::clean_with_command(item, &mut cmd, 60).await
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

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        let mut results = Vec::new();
        for item in items {
            let mut cmd = Command::new("docker");
            cmd.args(["builder", "prune", "-a", "-f"]);
            // Large caches take minutes to prune; 30s used to report a failure while
            // Docker went on to free the space anyway.
            let mut result = scanner::clean_with_command(item, &mut cmd, 600).await;
            if result
                .error
                .as_deref()
                .is_some_and(|e| e.starts_with("Command timed out"))
            {
                result.error = Some(
                    "Timed out after 10 minutes. Docker may still be pruning — run a new scan to check."
                        .to_string(),
                );
            }
            results.push(result);
        }
        results
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
        Ok(group_images(&stdout)
            .into_iter()
            .map(|(id, names, size_bytes)| {
                let name = names.join(", ");
                ScanResult {
                    id: uuid::Uuid::new_v4().to_string(),
                    category: self.category().to_string(),
                    label: format!("Image: {}", name),
                    risk_level: self.risk_level(),
                    detail: format!("Docker image {} ({})", name, id),
                    regeneration_hint: format!("docker pull {}", names[0]),
                    path: id,
                    size_bytes,
                    warning: None,
                }
            })
            .collect())
    }

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        let mut results = Vec::new();
        for item in items {
            if !is_valid_docker_id(&item.path) {
                results.push(scanner::error_result(
                    item,
                    "Invalid Docker image ID".to_string(),
                ));
                continue;
            }
            let mut cmd = Command::new("docker");
            cmd.args(["rmi", "-f", &item.path]);
            results.push(scanner::clean_with_command(item, &mut cmd, 60).await);
        }
        results
    }
}

/// Group `docker images` lines by image ID. An image with several tags is listed once
/// per tag; offering each line separately double-counted its size and made every
/// removal after the first fail with "No such image".
fn group_images(stdout: &str) -> Vec<(String, Vec<String>, u64)> {
    let mut images: Vec<(String, Vec<String>, u64)> = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let (name, id, size) = (parts[0], parts[1], parse_docker_size(parts[2]));
        if size == 0 {
            continue;
        }
        match images.iter_mut().find(|(existing, _, _)| existing == id) {
            Some((_, names, _)) => names.push(name.to_string()),
            None => images.push((id.to_string(), vec![name.to_string()], size)),
        }
    }
    images
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

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        let mut results = Vec::new();
        for item in items {
            results.push(remove_volume(item).await);
        }
        results
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

            let warning = match containers_using(name).await {
                Ok(c) if c.iter().any(|c| c.running) => Some(format!(
                    "In use by a running container ({}) — stop it before cleaning",
                    container_names(&c)
                )),
                Ok(c) if !c.is_empty() => Some(format!(
                    "Used by stopped container {} — removed together with the volume",
                    container_names(&c)
                )),
                _ => None,
            };

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
                warning,
            });
        }
        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        let mut results = Vec::new();
        for item in items {
            results.push(remove_volume(item).await);
        }
        results
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_with_several_tags_are_offered_once() {
        let out = "app:latest\tabc123\t1.5GB\napp:v2\tabc123\t1.5GB\nredis:7\tdef456\t100MB\n";
        let images = group_images(out);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].1, vec!["app:latest", "app:v2"]);
        assert_eq!(images[0].2, parse_docker_size("1.5GB"));
    }

    #[test]
    fn ids_and_names_cannot_look_like_flags() {
        assert!(!is_valid_docker_id("--force"));
        assert!(!is_valid_volume_name("-v"));
        assert!(is_valid_docker_id("sha256:abc-1"));
        assert!(is_valid_volume_name("shop_pgdata"));
    }
}

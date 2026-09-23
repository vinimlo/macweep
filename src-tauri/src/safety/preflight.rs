use tokio::process::Command;

use crate::scanner;

/// Whether any Docker container is running. Docker images and volumes are only
/// removed when this is `Ok(false)`; an error (e.g. the daemon is down) also blocks them.
pub async fn docker_containers_running() -> anyhow::Result<bool> {
    let output = scanner::run_with_timeout(Command::new("docker").args(["ps", "-q"]), 15).await?;
    if !output.status.success() {
        anyhow::bail!(
            "Docker is not reachable: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

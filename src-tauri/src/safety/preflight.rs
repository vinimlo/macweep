use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::scanner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub docker_running: bool,
    pub docker_containers_active: bool,
    pub disk_free_percent: f64,
    pub available_tools: Vec<String>,
}

pub async fn run_preflight() -> anyhow::Result<PreflightResult> {
    // Run all checks in parallel
    let (docker_result, disk_free, available_tools) = tokio::join!(
        check_docker(),
        check_disk_free(),
        detect_tools()
    );

    let (docker_running, docker_containers_active) = docker_result;

    Ok(PreflightResult {
        docker_running,
        docker_containers_active,
        disk_free_percent: disk_free.unwrap_or(0.0),
        available_tools,
    })
}

async fn check_docker() -> (bool, bool) {
    let info = Command::new("docker").arg("info").output().await;
    let running = info.map(|o| o.status.success()).unwrap_or(false);

    if !running {
        return (false, false);
    }

    let ps = Command::new("docker")
        .args(["ps", "-q"])
        .output()
        .await;
    let containers_active = ps
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);

    (true, containers_active)
}

async fn check_disk_free() -> anyhow::Result<f64> {
    let output = Command::new("df").args(["-k", "/"]).output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().nth(1).ok_or_else(|| anyhow::anyhow!("df parse error"))?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        anyhow::bail!("unexpected df format");
    }
    let total: f64 = parts[1].parse()?;
    let free: f64 = parts[3].parse()?;
    Ok(if total > 0.0 { (free / total) * 100.0 } else { 0.0 })
}

async fn detect_tools() -> Vec<String> {
    let tools = [
        ("docker", "docker"),
        ("npm", "npm"),
        ("yarn", "yarn"),
        ("bun", "bun"),
        ("brew", "brew"),
        ("pip", "pip3"),
        ("ollama", "ollama"),
    ];

    let checks: Vec<_> = tools.iter().map(|(_, cmd)| scanner::tool_installed(cmd)).collect();
    let results = futures::future::join_all(checks).await;

    tools.iter().zip(results)
        .filter(|(_, available)| *available)
        .map(|((name, _), _)| name.to_string())
        .collect()
}

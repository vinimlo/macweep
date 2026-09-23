use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

use crate::models::{CleanResult, RiskLevel, ScanResult};
use crate::scanner::{self, Scanner};

/// Validate that an Ollama model name contains only safe characters.
fn is_valid_ollama_model(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c))
}

pub struct AiToolsScanner;

#[async_trait]
impl Scanner for AiToolsScanner {
    fn category(&self) -> &str {
        "ai-tools"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn handles_category(&self, cat: &str) -> bool {
        matches!(
            cat,
            "ai-tools" | "ollama-models" | "langflow" | "gemini-cache" | "coderabbit" | "opencode"
        )
    }

    async fn is_available(&self) -> bool {
        true // Always check for leftover dirs
    }

    async fn scan(&self) -> Result<Vec<ScanResult>> {
        let home = dirs::home_dir().unwrap_or_default();
        let mut items = Vec::new();

        // Ollama models
        if scanner::tool_installed("ollama").await {
            let output = scanner::run_with_timeout(Command::new("ollama").arg("list"), 15).await;
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[0];
                        let size_str = if parts.len() >= 4 { parts[2] } else { "0" };
                        let unit = if parts.len() >= 5 { parts[3] } else { "B" };
                        let size_bytes = scanner::parse_size_with_units(size_str, unit);
                        items.push(ScanResult {
                            id: uuid::Uuid::new_v4().to_string(),
                            category: "ollama-models".to_string(),
                            label: format!("Ollama model: {}", name),
                            risk_level: RiskLevel::Medium,
                            path: format!("ollama:{}", name),
                            size_bytes,
                            detail: format!(
                                "AI model — can be re-downloaded with `ollama pull {}`",
                                name
                            ),
                            regeneration_hint: format!("ollama pull {}", name),
                            warning: None,
                        });
                    }
                }
            }
        }

        // Directory-based AI tools
        let dir_tools: &[(&[&str], &str, &str, Option<&str>)] = &[
            (
                &[".langflow"],
                "langflow",
                "Langflow data",
                Some("langflow"),
            ),
            (
                &[".gemini"],
                "gemini-cache",
                "Gemini CLI cache",
                Some("gemini"),
            ),
            (&[".coderabbit"], "coderabbit", "CodeRabbit data", None),
            (
                &[".opencode", ".config/opencode", ".cache/opencode"],
                "opencode",
                "OpenCode",
                None,
            ),
        ];

        for (dirs, category, base_label, check_cmd) in dir_tools {
            let installed = match check_cmd {
                Some(cmd) => scanner::tool_installed(cmd).await,
                None => false,
            };

            for subpath in *dirs {
                let dir = home.join(subpath);
                let path = dir.to_string_lossy().to_string();
                let size = scanner::dir_size_bytes(&path).await;
                if size == 0 {
                    continue;
                }

                let label = if dirs.len() > 1 {
                    format!("{} ({})", base_label, subpath)
                } else {
                    base_label.to_string()
                };

                let detail = if check_cmd.is_some() {
                    if installed {
                        format!("{} is installed — data may be in use", base_label)
                    } else {
                        format!("{} is NOT installed — data is likely stale", base_label)
                    }
                } else {
                    format!("{} data — check if still in use", base_label)
                };

                items.push(ScanResult {
                    id: uuid::Uuid::new_v4().to_string(),
                    category: category.to_string(),
                    label,
                    risk_level: RiskLevel::Medium,
                    path,
                    size_bytes: size,
                    detail,
                    regeneration_hint: format!("{} will recreate on next use", base_label),
                    warning: None,
                });
            }
        }

        Ok(items)
    }

    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult> {
        let (ollama_items, fs_items): (Vec<ScanResult>, Vec<ScanResult>) = items
            .iter()
            .cloned()
            .partition(|i| i.category == "ollama-models");

        let mut results = Vec::new();

        // Handle ollama models via CLI
        for item in &ollama_items {
            let model = item.path.strip_prefix("ollama:").unwrap_or(&item.path);
            if !is_valid_ollama_model(model) {
                results.push(scanner::error_result(
                    item,
                    "Invalid model name".to_string(),
                ));
                continue;
            }
            let mut cmd = Command::new("ollama");
            cmd.args(["rm", model]);
            results.push(scanner::clean_with_command(item, &mut cmd, 60).await);
        }

        // Handle filesystem items via shared helper
        results.extend(scanner::clean_filesystem_items(&fs_items).await);
        results
    }
}

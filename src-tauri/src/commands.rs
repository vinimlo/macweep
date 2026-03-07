use std::sync::atomic::Ordering;
use std::time::Instant;

use tauri::State;
use tauri::ipc::Channel;
use tokio::process::Command;

use crate::ScanState;
use crate::activity::ActivityLogger;
use crate::models::*;
use crate::safety::{audit, preflight, protected_paths};
use crate::scanner;

#[tauri::command]
pub async fn cancel_scan(state: State<'_, ScanState>) -> Result<(), String> {
    state.cancelled.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn scan_all(
    channel: Channel<ScanProgress>,
    state: State<'_, ScanState>,
    logger: State<'_, ActivityLogger>,
) -> Result<ScanReport, String> {
    state.cancelled.store(false, Ordering::SeqCst);

    let start = Instant::now();
    let registry = scanner::build_registry();

    // Check availability in parallel
    let availability_futures: Vec<_> = registry.iter().map(|s| s.is_available()).collect();
    let availability_results = futures::future::join_all(availability_futures).await;

    let available_scanners: Vec<_> = registry
        .iter()
        .zip(availability_results)
        .filter(|(_, available)| *available)
        .map(|(s, _)| s)
        .collect();

    let total_scanners = available_scanners.len();
    let available_tools: Vec<String> = available_scanners
        .iter()
        .map(|s| s.category().to_string())
        .collect();

    logger.info(
        None,
        &format!("Scan started ({} scanners available)", total_scanners),
    );

    channel
        .send(ScanProgress::Started { total_scanners })
        .map_err(|e| e.to_string())?;

    let mut all_items = Vec::new();
    let mut completed_count: usize = 0;

    for scanner in &available_scanners {
        // Check cancellation before starting next scanner
        if state.cancelled.load(Ordering::SeqCst) {
            logger.warning(None, "Scan cancelled by user", None);
            channel
                .send(ScanProgress::Cancelled {
                    completed_scanners: completed_count,
                    total_scanners,
                })
                .map_err(|e| e.to_string())?;

            let total_bytes = all_items.iter().map(|i: &ScanResult| i.size_bytes).sum();
            let disk_info = disk_info_or_default().await;

            return Ok(ScanReport {
                items: all_items,
                total_bytes,
                scan_duration_ms: start.elapsed().as_millis() as u64,
                available_tools,
                disk_free_percent: disk_info.free_percent,
            });
        }

        let category = scanner.category().to_string();
        logger.info(Some(&category), &format!("Scanner started: {}", category));
        channel
            .send(ScanProgress::ScannerStarted {
                category: category.clone(),
            })
            .map_err(|e| e.to_string())?;

        let scanner_start = Instant::now();
        match scanner.scan().await {
            Ok(items) => {
                let bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
                let count = items.len();
                let elapsed = scanner_start.elapsed().as_millis() as u64;
                logger.success(
                    Some(&category),
                    &format!("Scanner completed: {} items, {} bytes", count, bytes),
                    None,
                    Some(elapsed),
                );
                all_items.extend(items);
                completed_count += 1;
                channel
                    .send(ScanProgress::ScannerCompleted {
                        category,
                        items_found: count,
                        bytes,
                    })
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                let err_msg = e.to_string();
                logger.error(
                    Some(&category),
                    &format!("Scanner failed: {}", err_msg),
                    Some(&err_msg),
                );
                completed_count += 1;
                channel
                    .send(ScanProgress::ScannerFailed {
                        category,
                        error: e.to_string(),
                    })
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let total_elapsed = start.elapsed().as_millis() as u64;
    let total_bytes = all_items.iter().map(|i| i.size_bytes).sum();
    logger.success(
        None,
        &format!(
            "Scan completed: {} items found, {} scanners in {}ms",
            all_items.len(),
            completed_count,
            total_elapsed
        ),
        None,
        Some(total_elapsed),
    );

    channel
        .send(ScanProgress::Completed)
        .map_err(|e| e.to_string())?;

    let disk_info = disk_info_or_default().await;

    Ok(ScanReport {
        items: all_items,
        total_bytes,
        scan_duration_ms: total_elapsed,
        available_tools,
        disk_free_percent: disk_info.free_percent,
    })
}

const DOCKER_CATEGORIES: &[&str] = &[
    "docker-images",
    "docker-volumes-orphan",
    "docker-volumes-named",
];

#[tauri::command]
pub async fn clean_items(
    items: Vec<ScanResult>,
    channel: Channel<CleanProgress>,
    logger: State<'_, ActivityLogger>,
) -> Result<Vec<CleanResult>, String> {
    // Input bounds validation
    if items.len() > 10_000 {
        return Err("Too many items: maximum 10,000 per request".to_string());
    }
    for item in &items {
        if item.path.len() > 4096 {
            return Err(format!(
                "Path too long: {}...",
                &item.path[..item.path.len().min(64)]
            ));
        }
        if !scanner::KNOWN_CATEGORIES.contains(&item.category.as_str()) {
            return Err(format!("Unknown category: {}", item.category));
        }
    }

    // Validate no protected paths
    for item in &items {
        if protected_paths::is_protected(&item.path) {
            return Err(format!("Refusing to clean protected path: {}", item.path));
        }
    }

    // Enforce Docker preflight: refuse Docker cleanup if containers are running
    let has_docker_items = items
        .iter()
        .any(|i| DOCKER_CATEGORIES.contains(&i.category.as_str()));
    if has_docker_items {
        logger.info(None, "Running Docker preflight check");
        let preflight_result = preflight::run_preflight()
            .await
            .map_err(|e| e.to_string())?;
        if preflight_result.docker_containers_active {
            logger.error(None, "Docker containers are running, cleanup refused", None);
            return Err(
                "Cannot clean Docker volumes/images while containers are running. \
                 Stop all containers first with `docker stop $(docker ps -q)`."
                    .to_string(),
            );
        }
    }

    logger.info(None, &format!("Cleanup started ({} items)", items.len()));

    channel
        .send(CleanProgress::Started {
            total_items: items.len(),
        })
        .map_err(|e| e.to_string())?;

    let registry = scanner::build_registry();
    let mut results = Vec::new();
    let mut total_freed: u64 = 0;
    let mut audit_pairs: Vec<(ScanResult, CleanResult)> = Vec::new();

    // Group items by category and clean via the appropriate scanner
    for scanner in &registry {
        let scanner_items: Vec<ScanResult> = items
            .iter()
            .filter(|i| scanner.handles_category(&i.category))
            .cloned()
            .collect();

        if scanner_items.is_empty() {
            continue;
        }

        match scanner.clean(&scanner_items).await {
            Ok(clean_results) => {
                for cr in &clean_results {
                    total_freed += cr.freed_bytes;
                    if cr.success {
                        logger.success(
                            Some(scanner.category()),
                            &format!("Cleaned: {} ({} bytes freed)", cr.id, cr.freed_bytes),
                            None,
                            None,
                        );
                    } else {
                        logger.error(
                            Some(scanner.category()),
                            &format!("Failed to clean: {}", cr.id),
                            cr.error.as_deref(),
                        );
                    }
                    channel
                        .send(CleanProgress::ItemCompleted {
                            id: cr.id.clone(),
                            success: cr.success,
                            freed_bytes: cr.freed_bytes,
                        })
                        .map_err(|e| e.to_string())?;

                    // Collect for batch audit write
                    if let Some(item) = items.iter().find(|i| i.id == cr.id) {
                        audit_pairs.push((item.clone(), cr.clone()));
                    }
                }
                results.extend(clean_results);
            }
            Err(e) => {
                logger.error(
                    Some(scanner.category()),
                    &format!("Cleanup batch failed for {}", scanner.category()),
                    Some(&e.to_string()),
                );
                for item in &scanner_items {
                    channel
                        .send(CleanProgress::Failed {
                            id: item.id.clone(),
                            error: e.to_string(),
                        })
                        .map_err(|e| e.to_string())?;
                    results.push(CleanResult {
                        id: item.id.clone(),
                        freed_bytes: 0,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    // Batch write all audit entries at once
    let _ = audit::log_cleanup_batch(&audit_pairs);

    logger.success(
        None,
        &format!("Cleanup completed, {} bytes freed", total_freed),
        None,
        None,
    );

    channel
        .send(CleanProgress::Completed { total_freed })
        .map_err(|e| e.to_string())?;

    Ok(results)
}

#[tauri::command]
pub async fn get_disk_info(logger: State<'_, ActivityLogger>) -> Result<DiskInfo, String> {
    logger.info(None, "Querying disk info");
    let result = get_disk_info_internal().await.map_err(|e| e.to_string());
    if let Ok(ref info) = result {
        logger.success(
            None,
            &format!(
                "Disk info: {:.1}% free ({} bytes free)",
                info.free_percent, info.free_bytes
            ),
            None,
            None,
        );
    }
    result
}

async fn disk_info_or_default() -> DiskInfo {
    get_disk_info_internal().await.unwrap_or(DiskInfo {
        total_bytes: 0,
        used_bytes: 0,
        free_bytes: 0,
        free_percent: 0.0,
    })
}

async fn get_disk_info_internal() -> anyhow::Result<DiskInfo> {
    let output = Command::new("df").args(["-k", "/"]).output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("df output parse error"))?;
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 4 {
        anyhow::bail!("unexpected df output format");
    }

    let total_kb: u64 = parts[1].parse()?;
    let free_kb: u64 = parts[3].parse()?;
    let used_kb: u64 = total_kb - free_kb;
    let total_bytes = total_kb * 1024;
    let used_bytes = used_kb * 1024;
    let free_bytes = free_kb * 1024;
    let free_percent = if total_bytes > 0 {
        (free_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(DiskInfo {
        total_bytes,
        used_bytes,
        free_bytes,
        free_percent,
    })
}

#[tauri::command]
pub async fn get_audit_log(limit: Option<usize>) -> Result<Vec<audit::AuditEntry>, String> {
    audit::read_log(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_preflight(
    logger: State<'_, ActivityLogger>,
) -> Result<preflight::PreflightResult, String> {
    logger.info(None, "Running preflight checks");
    let result = preflight::run_preflight().await.map_err(|e| e.to_string());
    if let Ok(ref pf) = result {
        logger.success(
            None,
            &format!(
                "Preflight: docker={}, containers_active={}, {:.1}% free",
                pf.docker_running, pf.docker_containers_active, pf.disk_free_percent
            ),
            None,
            None,
        );
    }
    result
}

#[tauri::command]
pub async fn get_activity_log(
    limit: Option<usize>,
) -> Result<Vec<crate::activity::ActivityEntry>, String> {
    crate::activity::read_log(limit.unwrap_or(200)).map_err(|e| e.to_string())
}

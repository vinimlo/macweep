use std::time::Instant;

use tauri::State;
use tauri::ipc::Channel;
use tokio::process::Command;

use crate::activity::ActivityLogger;
use crate::cleanup;
use crate::models::*;
use crate::safety::{audit, preflight};
use crate::scanner;
use crate::state::ScanState;

/// Upper bound for list-style requests coming from the webview.
const MAX_REQUEST_ITEMS: usize = 10_000;
const MAX_LOG_ENTRIES: usize = 1_000;

#[tauri::command]
pub async fn cancel_scan(state: State<'_, ScanState>) -> Result<(), String> {
    state.cancel();
    Ok(())
}

#[tauri::command]
pub async fn scan_all(
    channel: Channel<ScanProgress>,
    state: State<'_, ScanState>,
    logger: State<'_, ActivityLogger>,
) -> Result<ScanReport, String> {
    let generation = state.begin_scan();
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
        // A newer scan or a cancel supersedes this one: stop here instead of
        // running on in parallel.
        if !state.is_current(generation) {
            logger.warning(None, "Scan cancelled", None);
            return Err("Scan cancelled".to_string());
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
                        error: err_msg,
                    })
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    if !state.finish_scan(generation, &all_items) {
        logger.warning(None, "Scan cancelled", None);
        return Err("Scan cancelled".to_string());
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

    Ok(ScanReport {
        items: all_items,
        total_bytes,
        scan_duration_ms: total_elapsed,
    })
}

const DOCKER_CATEGORIES: &[&str] = &[
    "docker-images",
    "docker-volumes-orphan",
    "docker-volumes-named",
];

/// Clean items found by the last scan. The webview only sends IDs: what gets deleted is
/// always what the backend itself found, never a path supplied by the UI.
#[tauri::command]
pub async fn clean_items(
    ids: Vec<String>,
    channel: Channel<CleanProgress>,
    state: State<'_, ScanState>,
    logger: State<'_, ActivityLogger>,
) -> Result<CleanReport, String> {
    if ids.len() > MAX_REQUEST_ITEMS {
        return Err(format!(
            "Too many items: maximum {} per request",
            MAX_REQUEST_ITEMS
        ));
    }

    channel
        .send(CleanProgress::Started {
            total_items: ids.len(),
        })
        .map_err(|e| e.to_string())?;
    logger.info(None, &format!("Cleanup started ({} items)", ids.len()));

    let (mut items, missing) = state.take(&ids);
    let mut results: Vec<CleanResult> = Vec::with_capacity(ids.len());
    let mut report = |result: CleanResult| {
        let _ = channel.send(CleanProgress::ItemCompleted {
            id: result.id.clone(),
            success: result.success,
            freed_bytes: result.freed_bytes,
        });
        results.push(result);
    };

    for id in missing {
        report(CleanResult {
            id,
            freed_bytes: 0,
            success: false,
            error: Some("No longer available — it was already cleaned or the scan is outdated. Run a new scan.".to_string()),
        });
    }

    // Docker images/volumes are never removed while containers run. Skip just those items.
    let mut audit_pairs: Vec<(ScanResult, CleanResult)> = Vec::new();
    let mut blocked = Vec::new();
    if items
        .iter()
        .any(|i| DOCKER_CATEGORIES.contains(&i.category.as_str()))
    {
        let reason = match preflight::docker_containers_running().await {
            Ok(false) => None,
            Ok(true) => Some(
                "Skipped: Docker containers are running. Stop them first (docker stop $(docker ps -q))."
                    .to_string(),
            ),
            Err(e) => Some(format!("Skipped: {e}")),
        };
        if let Some(reason) = reason {
            logger.warning(None, "Docker items skipped", Some(&reason));
            let (docker, rest): (Vec<_>, Vec<_>) = items
                .into_iter()
                .partition(|i| DOCKER_CATEGORIES.contains(&i.category.as_str()));
            items = rest;
            for item in docker {
                let result = scanner::error_result(&item, reason.clone());
                audit_pairs.push((item.clone(), result.clone()));
                report(result);
                blocked.push(item);
            }
        }
    }

    let disk_before = get_disk_info_internal().await.ok();
    let registry = scanner::build_registry();
    let cleaned = cleanup::clean_items(&registry, &items, |item, result| {
        if result.success {
            logger.success(
                Some(&item.category),
                &format!(
                    "Cleaned: {} ({} bytes freed)",
                    item.label, result.freed_bytes
                ),
                None,
                None,
            );
        } else {
            logger.error(
                Some(&item.category),
                &format!("Failed to clean: {}", item.label),
                result.error.as_deref(),
            );
        }
        let _ = channel.send(CleanProgress::ItemCompleted {
            id: result.id.clone(),
            success: result.success,
            freed_bytes: result.freed_bytes,
        });
    })
    .await;
    let disk_after = get_disk_info_internal().await.ok();

    let mut failed = blocked;
    for (item, result) in cleaned {
        if !result.success {
            failed.push(item.clone());
        }
        audit_pairs.push((item, result.clone()));
        results.push(result);
    }
    // Failed items stay available for a retry; cleaned ones are gone for good.
    state.restore(failed);

    if let Err(e) = audit::log_cleanup_batch(&audit_pairs) {
        logger.error(None, "Failed to write audit log", Some(&e.to_string()));
    }

    let freed_bytes: u64 = results.iter().map(|r| r.freed_bytes).sum();
    let disk_freed_bytes = match (disk_before, disk_after) {
        (Some(before), Some(after)) => after.free_bytes.saturating_sub(before.free_bytes),
        _ => 0,
    };
    logger.success(
        None,
        &format!(
            "Cleanup completed: {} bytes removed, disk free space +{} bytes",
            freed_bytes, disk_freed_bytes
        ),
        None,
        None,
    );

    channel
        .send(CleanProgress::Completed {
            total_freed: freed_bytes,
        })
        .map_err(|e| e.to_string())?;

    Ok(CleanReport {
        results,
        freed_bytes,
        disk_freed_bytes,
    })
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

pub(crate) async fn get_disk_info_internal() -> anyhow::Result<DiskInfo> {
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
    let used_kb: u64 = total_kb.saturating_sub(free_kb);
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
    audit::read_log(limit.unwrap_or(50).min(MAX_LOG_ENTRIES)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity_log(
    limit: Option<usize>,
) -> Result<Vec<crate::activity::ActivityEntry>, String> {
    crate::activity::read_log(limit.unwrap_or(200).min(MAX_LOG_ENTRIES)).map_err(|e| e.to_string())
}

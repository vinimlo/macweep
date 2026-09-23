use std::collections::HashMap;
use std::path::Path;

use crate::models::{CleanResult, ScanResult};
use crate::scanner::{self, Scanner};

/// Clean `items` through the scanner that owns each category. Returns one result per item.
///
/// Filesystem items are measured right before and right after removal, so the bytes
/// reported are what actually left the disk now rather than the scan-time estimate:
/// an item removed by something else in the meantime counts as 0, and a partial removal
/// counts what was really deleted. Tool-driven items (Docker, Ollama) keep the estimate.
pub async fn clean_items(
    registry: &[Box<dyn Scanner>],
    items: &[ScanResult],
    mut on_result: impl FnMut(&ScanResult, &CleanResult),
) -> Vec<(ScanResult, CleanResult)> {
    let mut done: Vec<(ScanResult, CleanResult)> = Vec::with_capacity(items.len());

    for scanner in registry {
        let batch: Vec<ScanResult> = items
            .iter()
            .filter(|i| scanner.handles_category(&i.category))
            .cloned()
            .collect();
        if batch.is_empty() {
            continue;
        }

        let mut size_before = HashMap::new();
        for item in batch.iter().filter(|i| is_filesystem_item(i)) {
            size_before.insert(item.id.clone(), scanner::dir_size_bytes(&item.path).await);
        }

        for mut result in scanner.clean(&batch).await {
            let Some(item) = batch.iter().find(|i| i.id == result.id) else {
                continue;
            };
            if let Some(before) = size_before.get(&item.id) {
                let remaining = if Path::new(&item.path).exists() {
                    scanner::dir_size_bytes(&item.path).await
                } else {
                    0
                };
                result.freed_bytes = before.saturating_sub(remaining);
            }
            on_result(item, &result);
            done.push((item.clone(), result));
        }
    }

    // Every item gets a result, even if no scanner claimed it or a scanner skipped it.
    for item in items {
        if !done.iter().any(|(i, _)| i.id == item.id) {
            let result = scanner::error_result(item, "No cleaner handled this item".to_string());
            on_result(item, &result);
            done.push((item.clone(), result));
        }
    }

    done
}

fn is_filesystem_item(item: &ScanResult) -> bool {
    Path::new(&item.path).is_absolute()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RiskLevel;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("macweep-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn node_modules_item(path: &Path, size_bytes: u64) -> ScanResult {
        ScanResult {
            id: uuid::Uuid::new_v4().to_string(),
            category: "node-modules".to_string(),
            label: "node_modules (test)".to_string(),
            risk_level: RiskLevel::Low,
            path: path.to_string_lossy().to_string(),
            size_bytes,
            detail: String::new(),
            regeneration_hint: String::new(),
            warning: None,
        }
    }

    fn write_file(path: &Path, kb: usize) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![7u8; kb * 1024]).unwrap();
    }

    #[tokio::test]
    async fn removed_directory_reports_measured_bytes() {
        let root = temp_dir();
        let target = root.join("node_modules");
        write_file(&target.join("pkg/index.js"), 256);
        let item = node_modules_item(&target, 999_999_999);

        let done = clean_items(&scanner::build_registry(), &[item], |_, _| {}).await;

        let (_, result) = &done[0];
        assert!(result.success);
        assert!(!target.exists());
        // Measured (~256 KiB), not the inflated scan-time estimate.
        assert!(result.freed_bytes >= 256 * 1024 && result.freed_bytes < 1024 * 1024);
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn already_missing_directory_frees_nothing() {
        let root = temp_dir();
        let item = node_modules_item(&root.join("node_modules"), 5_000_000);

        let done = clean_items(&scanner::build_registry(), &[item], |_, _| {}).await;

        assert!(done[0].1.success);
        assert_eq!(done[0].1.freed_bytes, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn partial_removal_reports_what_was_actually_freed() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir();
        let target = root.join("node_modules");
        write_file(&target.join("a/big.bin"), 512);
        let locked = target.join("locked");
        write_file(&locked.join("kept.bin"), 64);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o500)).unwrap();
        let item = node_modules_item(&target, 999_999_999);

        let mut seen = 0;
        let done = clean_items(&scanner::build_registry(), &[item], |_, _| seen += 1).await;

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
        let (_, result) = &done[0];
        assert_eq!(seen, 1);
        assert!(!result.success);
        assert!(locked.join("kept.bin").exists());
        // Whatever the traversal order removed, never more than what existed on disk.
        assert!(
            result.freed_bytes <= 600 * 1024,
            "freed {}",
            result.freed_bytes
        );
        fs::remove_dir_all(root).unwrap();
    }
}

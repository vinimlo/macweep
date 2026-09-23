use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::models::ScanResult;

/// Scan bookkeeping shared by the IPC commands.
///
/// Every scan takes a new generation number. Starting another scan or cancelling bumps it,
/// so a superseded scan notices at its next checkpoint and stops instead of running on in
/// parallel and feeding progress into the same UI.
///
/// The items of the last finished scan live here so cleanup only ever acts on what the
/// backend itself found: the webview sends item IDs, never paths.
#[derive(Default)]
pub struct ScanState {
    generation: AtomicU64,
    items: Mutex<HashMap<String, ScanResult>>,
}

impl ScanState {
    /// Start a new scan: forget previous results and supersede any scan still running.
    pub fn begin_scan(&self) -> u64 {
        let mut items = self.items();
        items.clear();
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }

    /// Keep the results of a scan, unless it was superseded or cancelled meanwhile.
    pub fn finish_scan(&self, generation: u64, results: &[ScanResult]) -> bool {
        let mut items = self.items();
        if !self.is_current(generation) {
            return false;
        }
        *items = results.iter().map(|i| (i.id.clone(), i.clone())).collect();
        true
    }

    /// Remove the requested items so no other cleanup can pick them up.
    /// Returns the known items and the IDs that are unknown (stale or already cleaned).
    pub fn take(&self, ids: &[String]) -> (Vec<ScanResult>, Vec<String>) {
        let mut items = self.items();
        let mut seen = HashSet::new();
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            if !seen.insert(id) {
                continue;
            }
            match items.remove(id) {
                Some(item) => found.push(item),
                None => missing.push(id.clone()),
            }
        }
        (found, missing)
    }

    /// Put items back, e.g. after a failed cleanup, so they can be retried.
    pub fn restore(&self, restored: impl IntoIterator<Item = ScanResult>) {
        let mut items = self.items();
        for item in restored {
            items.insert(item.id.clone(), item);
        }
    }

    fn items(&self) -> MutexGuard<'_, HashMap<String, ScanResult>> {
        self.items.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RiskLevel;

    fn item(id: &str) -> ScanResult {
        ScanResult {
            id: id.to_string(),
            category: "logs".to_string(),
            label: id.to_string(),
            risk_level: RiskLevel::High,
            path: format!("/tmp/{id}"),
            size_bytes: 1,
            detail: String::new(),
            regeneration_hint: String::new(),
            warning: None,
        }
    }

    #[test]
    fn new_scan_supersedes_running_one() {
        let state = ScanState::default();
        let first = state.begin_scan();
        let second = state.begin_scan();
        assert!(!state.is_current(first));
        assert!(state.is_current(second));
        assert!(!state.finish_scan(first, &[item("stale")]));
        assert!(state.finish_scan(second, &[item("a")]));
        assert_eq!(state.take(&["stale".into()]).1, vec!["stale".to_string()]);
    }

    #[test]
    fn cancel_discards_results() {
        let state = ScanState::default();
        let generation = state.begin_scan();
        state.cancel();
        assert!(!state.finish_scan(generation, &[item("a")]));
        assert!(state.take(&["a".into()]).0.is_empty());
    }

    #[test]
    fn items_can_only_be_taken_once() {
        let state = ScanState::default();
        let generation = state.begin_scan();
        state.finish_scan(generation, &[item("a"), item("b")]);

        let (found, missing) = state.take(&["a".into(), "a".into(), "zzz".into()]);
        assert_eq!(found.len(), 1);
        assert_eq!(missing, vec!["zzz".to_string()]);

        let (found, missing) = state.take(&["a".into()]);
        assert!(found.is_empty());
        assert_eq!(missing, vec!["a".to_string()]);
    }

    #[test]
    fn restored_items_can_be_retried() {
        let state = ScanState::default();
        let generation = state.begin_scan();
        state.finish_scan(generation, &[item("a")]);
        let (found, _) = state.take(&["a".into()]);
        state.restore(found);
        assert_eq!(state.take(&["a".into()]).0.len(), 1);
    }
}

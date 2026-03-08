pub mod activity;
mod commands;
mod models;
mod safety;
mod scanner;
mod tray;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri::Manager;

use commands::{
    cancel_scan, clean_items, get_activity_log, get_audit_log, get_disk_info, run_preflight,
    scan_all,
};

pub struct ScanState {
    pub cancelled: Arc<AtomicBool>,
}

/// Extend PATH so child processes (docker, brew, npm, etc.) can be found.
///
/// GUI apps launched from Finder inherit a minimal PATH (/usr/bin:/bin:/usr/sbin:/sbin).
/// We replicate what `/usr/libexec/path_helper` does: read `/etc/paths` and `/etc/paths.d/*`,
/// then merge with the current PATH and add hardcoded fallbacks.
fn extend_path() {
    use std::collections::HashSet;

    let mut system_paths: Vec<String> = Vec::new();

    // 1. Read /etc/paths (one directory per line)
    if let Ok(contents) = std::fs::read_to_string("/etc/paths") {
        for line in contents.lines() {
            let p = line.trim();
            if !p.is_empty() {
                system_paths.push(p.to_string());
            }
        }
    }

    // 2. Read /etc/paths.d/* (one directory per line per file, sorted by filename)
    if let Ok(entries) = std::fs::read_dir("/etc/paths.d") {
        let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                for line in contents.lines() {
                    let p = line.trim();
                    if !p.is_empty() {
                        system_paths.push(p.to_string());
                    }
                }
            }
        }
    }

    // 3. Hardcoded fallbacks (in case /etc/paths.d is incomplete)
    for p in ["/usr/local/bin", "/opt/homebrew/bin", "/opt/homebrew/sbin"] {
        system_paths.push(p.to_string());
    }

    // 4. Merge: current PATH first, then system paths — deduplicated
    let current = std::env::var("PATH").unwrap_or_default();
    let mut seen = HashSet::new();
    let mut final_paths: Vec<String> = Vec::new();

    for p in current
        .split(':')
        .chain(system_paths.iter().map(|s| s.as_str()))
    {
        if !p.is_empty() && seen.insert(p.to_string()) {
            final_paths.push(p.to_string());
        }
    }

    let joined = final_paths.join(":");
    unsafe {
        std::env::set_var("PATH", &joined);
    }
    eprintln!("[macweep] PATH set to: {}", joined);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Must run before Builder — GUI apps inherit a minimal PATH from Finder
    extend_path();

    tauri::Builder::default()
        .manage(ScanState {
            cancelled: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            scan_all,
            cancel_scan,
            clean_items,
            get_disk_info,
            get_audit_log,
            get_activity_log,
            run_preflight,
        ])
        .setup(|app| {
            let logger = activity::ActivityLogger::new(app.handle().clone())
                .map_err(|e| format!("Failed to initialize activity logger: {e}"))?;
            logger.info(None, "macweep started");
            app.manage(logger);
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("macweep failed to start: {e}");
            std::process::exit(1);
        });
}

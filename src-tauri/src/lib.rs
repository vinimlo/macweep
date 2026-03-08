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

/// Extend PATH to include common macOS developer tool locations.
/// GUI apps launched from Finder inherit a minimal PATH (/usr/bin:/bin:/usr/sbin:/sbin)
/// which misses /usr/local/bin (Docker CLI) and /opt/homebrew/bin (Homebrew).
fn extend_path() {
    let extra_paths = ["/usr/local/bin", "/opt/homebrew/bin", "/opt/homebrew/sbin"];
    let current = std::env::var("PATH").unwrap_or_default();
    let mut paths: Vec<&str> = current.split(':').collect();
    for p in &extra_paths {
        if !paths.contains(p) {
            paths.push(p);
        }
    }
    unsafe {
        std::env::set_var("PATH", paths.join(":"));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            extend_path();

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

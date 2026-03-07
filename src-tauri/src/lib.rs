pub mod activity;
mod commands;
mod models;
mod scanner;
mod safety;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::Manager;

use commands::{
    cancel_scan, clean_items, get_activity_log, get_audit_log, get_disk_info, run_preflight,
    scan_all,
};

pub struct ScanState {
    pub cancelled: Arc<AtomicBool>,
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
            let logger = activity::ActivityLogger::new(app.handle().clone())
                .expect("Failed to initialize activity logger");
            logger.info(None, "macweep started");
            app.manage(logger);
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running macweep");
}

import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  ScanReport,
  ScanProgress,
  CleanResult,
  CleanProgress,
  ScanResult,
  DiskInfo,
  AuditEntry,
  ActivityEntry,
  PreflightResult,
} from "./types";

export async function scanAll(
  onProgress: (progress: ScanProgress) => void,
): Promise<ScanReport> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke<ScanReport>("scan_all", { channel });
}

export async function cancelScan(): Promise<void> {
  return invoke<void>("cancel_scan");
}

export async function cleanItems(
  items: ScanResult[],
  onProgress: (progress: CleanProgress) => void,
): Promise<CleanResult[]> {
  const channel = new Channel<CleanProgress>();
  channel.onmessage = onProgress;
  return invoke<CleanResult[]>("clean_items", { items, channel });
}

export async function getDiskInfo(): Promise<DiskInfo> {
  return invoke<DiskInfo>("get_disk_info");
}

export async function getAuditLog(limit: number = 50): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>("get_audit_log", { limit });
}

export async function runPreflight(): Promise<PreflightResult> {
  return invoke<PreflightResult>("run_preflight");
}

export async function getActivityLog(
  limit: number = 200,
): Promise<ActivityEntry[]> {
  return invoke<ActivityEntry[]>("get_activity_log", { limit });
}

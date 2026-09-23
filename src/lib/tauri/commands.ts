import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  ScanReport,
  ScanProgress,
  CleanReport,
  CleanProgress,
  DiskInfo,
  AuditEntry,
  ActivityEntry,
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

/** Clean items from the last scan. Only IDs cross the IPC boundary: the backend
 * decides what to delete from its own scan results. */
export async function cleanItems(
  ids: string[],
  onProgress: (progress: CleanProgress) => void,
): Promise<CleanReport> {
  const channel = new Channel<CleanProgress>();
  channel.onmessage = onProgress;
  return invoke<CleanReport>("clean_items", { ids, channel });
}

export async function getDiskInfo(): Promise<DiskInfo> {
  return invoke<DiskInfo>("get_disk_info");
}

export async function getAuditLog(limit: number = 50): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>("get_audit_log", { limit });
}

export async function getActivityLog(
  limit: number = 200,
): Promise<ActivityEntry[]> {
  return invoke<ActivityEntry[]>("get_activity_log", { limit });
}

export type RiskLevel = "Zero" | "Low" | "Medium" | "High";

export interface ScanResult {
  id: string;
  category: string;
  label: string;
  risk_level: RiskLevel;
  path: string;
  size_bytes: number;
  detail: string;
  regeneration_hint: string;
  warning?: string;
}

export interface CleanResult {
  id: string;
  freed_bytes: number;
  success: boolean;
  error: string | null;
}

export type ScanProgress =
  | { type: "Started"; total_scanners: number }
  | { type: "ScannerStarted"; category: string }
  | {
      type: "ScannerCompleted";
      category: string;
      items_found: number;
      bytes: number;
    }
  | { type: "ScannerFailed"; category: string; error: string }
  | { type: "Cancelled"; completed_scanners: number; total_scanners: number }
  | { type: "Completed" };

export type CleanProgress =
  | { type: "Started"; total_items: number }
  | { type: "ItemCompleted"; id: string; success: boolean; freed_bytes: number }
  | { type: "Failed"; id: string; error: string }
  | { type: "Completed"; total_freed: number };

export interface ScanReport {
  items: ScanResult[];
  total_bytes: number;
  scan_duration_ms: number;
  available_tools: string[];
  disk_free_percent: number;
}

export interface DiskInfo {
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  free_percent: number;
}

export interface PreflightResult {
  docker_running: boolean;
  docker_containers_active: boolean;
  disk_free_percent: number;
  available_tools: string[];
}

export type ActivityLevel =
  | "Info"
  | "Command"
  | "Success"
  | "Warning"
  | "Error";

export interface ActivityEntry {
  timestamp: string;
  level: ActivityLevel;
  category: string | null;
  message: string;
  detail: string | null;
  command: string | null;
  duration_ms: number | null;
}

export interface AuditEntry {
  timestamp: string;
  action: string;
  category: string;
  label: string;
  path: string;
  size_bytes: number;
  freed_bytes: number;
  success: boolean;
  error: string | null;
}

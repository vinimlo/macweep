import type { ScanResult, ScanProgress, ScanReport } from "$lib/tauri/types";
import { riskOrder } from "$lib/utils/risk";

export type ScanStatus = "idle" | "scanning" | "completed" | "error";

interface CategoryGroup {
  category: string;
  items: ScanResult[];
  totalBytes: number;
}

class ScanStore {
  status = $state<ScanStatus>("idle");
  items = $state<ScanResult[]>([]);
  totalBytes = $state(0);
  scanDurationMs = $state(0);
  availableTools = $state<string[]>([]);
  diskFreePercent = $state(0);

  // Progress tracking
  totalScanners = $state(0);
  currentScanner = $state<string | null>(null);
  completedScanners = $state(0);
  scannerStatuses = $state<
    Record<string, "pending" | "scanning" | "completed" | "failed">
  >({});

  categories = $derived.by(() => {
    const groups = new Map<string, CategoryGroup>();
    for (const item of this.items) {
      const existing = groups.get(item.category);
      if (existing) {
        existing.items.push(item);
        existing.totalBytes += item.size_bytes;
      } else {
        groups.set(item.category, {
          category: item.category,
          items: [item],
          totalBytes: item.size_bytes,
        });
      }
    }
    return [...groups.values()].sort((a, b) => {
      const riskA = riskOrder(a.items[0].risk_level);
      const riskB = riskOrder(b.items[0].risk_level);
      if (riskA !== riskB) return riskA - riskB;
      return b.totalBytes - a.totalBytes;
    });
  });

  byRisk = $derived.by(() => {
    const groups: Record<string, ScanResult[]> = {
      Zero: [],
      Low: [],
      Medium: [],
      High: [],
    };
    for (const item of this.items) {
      groups[item.risk_level].push(item);
    }
    return groups;
  });

  handleProgress(progress: ScanProgress) {
    switch (progress.type) {
      case "Started":
        this.totalScanners = progress.total_scanners;
        this.completedScanners = 0;
        this.status = "scanning";
        break;
      case "ScannerStarted":
        this.currentScanner = progress.category;
        this.scannerStatuses = {
          ...this.scannerStatuses,
          [progress.category]: "scanning",
        };
        break;
      case "ScannerCompleted":
        this.completedScanners++;
        this.scannerStatuses = {
          ...this.scannerStatuses,
          [progress.category]: "completed",
        };
        break;
      case "ScannerFailed":
        this.completedScanners++;
        this.scannerStatuses = {
          ...this.scannerStatuses,
          [progress.category]: "failed",
        };
        break;
      case "Cancelled":
        this.currentScanner = null;
        this.status = "idle";
        break;
      case "Completed":
        this.currentScanner = null;
        this.status = "completed";
        break;
    }
  }

  setReport(report: ScanReport) {
    this.items = report.items;
    this.totalBytes = report.total_bytes;
    this.scanDurationMs = report.scan_duration_ms;
    this.availableTools = report.available_tools;
    this.diskFreePercent = report.disk_free_percent;
    this.status = "completed";
  }

  reset() {
    this.status = "idle";
    this.items = [];
    this.totalBytes = 0;
    this.scanDurationMs = 0;
    this.totalScanners = 0;
    this.currentScanner = null;
    this.completedScanners = 0;
    this.scannerStatuses = {};
  }
}

export const scanStore = new ScanStore();

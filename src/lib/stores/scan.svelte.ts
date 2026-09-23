import type { ScanResult, ScanProgress } from "$lib/tauri/types";
import { scanAll, cancelScan, getDiskInfo } from "$lib/tauri/commands";
import { riskOrder } from "$lib/utils/risk";
import { sanitizeError } from "$lib/utils/errors";
import { appStore } from "./app.svelte";
import { toastStore } from "./toasts.svelte";

export type ScanStatus = "idle" | "scanning" | "completed";
export type ScannerStatus = "pending" | "scanning" | "completed" | "failed";

interface CategoryGroup {
  category: string;
  items: ScanResult[];
  totalBytes: number;
}

class ScanStore {
  status = $state<ScanStatus>("idle");
  items = $state<ScanResult[]>([]);
  scanDurationMs = $state(0);

  // Progress tracking
  totalScanners = $state(0);
  completedScanners = $state(0);
  scannerStatuses = $state<Record<string, ScannerStatus>>({});

  /** Bumped by every start/stop. Progress and results from an older run are ignored,
   * so two scans can never feed the same counter. */
  private run = 0;

  totalBytes = $derived(this.items.reduce((sum, i) => sum + i.size_bytes, 0));

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

  async start() {
    if (this.status === "scanning") return;
    const run = ++this.run;
    this.reset();
    // Flip to "scanning" right away: the backend only reports Started after its
    // availability checks, and the Scan button must not stay clickable meanwhile.
    this.status = "scanning";
    try {
      const report = await scanAll((progress) => {
        if (run === this.run) this.handleProgress(progress);
      });
      if (run !== this.run) return;
      this.items = report.items;
      this.scanDurationMs = report.scan_duration_ms;
      this.status = "completed";
      getDiskInfo()
        .then((info) => (appStore.diskInfo = info))
        .catch(() => {});
    } catch (e) {
      if (run !== this.run) return;
      this.status = "idle";
      toastStore.error(`Scan failed: ${sanitizeError(e)}`);
    }
  }

  async stop() {
    if (this.status !== "scanning") return;
    this.run++;
    this.reset();
    try {
      await cancelScan();
    } catch {
      // best-effort
    }
    toastStore.info("Scan stopped");
  }

  /** Drop cleaned items so they are never offered (or counted) again. */
  removeItems(ids: Iterable<string>) {
    const gone = new Set(ids);
    this.items = this.items.filter((i) => !gone.has(i.id));
  }

  private handleProgress(progress: ScanProgress) {
    switch (progress.type) {
      case "Started":
        this.totalScanners = progress.total_scanners;
        this.completedScanners = 0;
        break;
      case "ScannerStarted":
        this.scannerStatuses = {
          ...this.scannerStatuses,
          [progress.category]: "scanning",
        };
        break;
      case "ScannerCompleted":
      case "ScannerFailed":
        this.completedScanners++;
        this.scannerStatuses = {
          ...this.scannerStatuses,
          [progress.category]:
            progress.type === "ScannerCompleted" ? "completed" : "failed",
        };
        break;
    }
  }

  private reset() {
    this.status = "idle";
    this.items = [];
    this.scanDurationMs = 0;
    this.totalScanners = 0;
    this.completedScanners = 0;
    this.scannerStatuses = {};
  }
}

export const scanStore = new ScanStore();

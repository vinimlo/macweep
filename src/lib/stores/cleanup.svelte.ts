import type { ScanResult, CleanReport, CleanProgress } from "$lib/tauri/types";
import { cleanItems, getDiskInfo } from "$lib/tauri/commands";
import { formatSize } from "$lib/utils/format";
import { sanitizeError } from "$lib/utils/errors";
import { appStore } from "./app.svelte";
import { scanStore } from "./scan.svelte";
import { toastStore } from "./toasts.svelte";

export type CleanupStatus = "idle" | "cleaning" | "completed";

export interface CleanupRun {
  items: ScanResult[];
  report: CleanReport;
  error?: string;
}

class CleanupStore {
  status = $state<CleanupStatus>("idle");
  selectedItems = $state<ScanResult[]>([]);
  lastRun = $state<CleanupRun | null>(null);
  totalItems = $state(0);
  completedItems = $state(0);

  private selectedIds = $derived(new Set(this.selectedItems.map((i) => i.id)));

  select(items: ScanResult[]) {
    this.selectedItems = items;
  }

  toggleItem(item: ScanResult) {
    if (this.selectedIds.has(item.id)) {
      this.selectedItems = this.selectedItems.filter((i) => i.id !== item.id);
    } else {
      this.selectedItems = [...this.selectedItems, item];
    }
  }

  isSelected(id: string): boolean {
    return this.selectedIds.has(id);
  }

  /** Run one cleanup. Only ever called from an explicit confirmation. */
  async run(items: ScanResult[]) {
    if (this.status === "cleaning") return;
    this.status = "cleaning";
    this.totalItems = items.length;
    this.completedItems = 0;
    this.lastRun = null;
    try {
      const report = await cleanItems(
        items.map((i) => i.id),
        (progress) => this.handleProgress(progress),
      );
      scanStore.removeItems(
        report.results.filter((r) => r.success).map((r) => r.id),
      );
      this.lastRun = { items, report };
      // The report says it already; the toast is for when the user navigated away.
      if (location.pathname !== "/cleanup") {
        toastStore.success(`Removed ${formatSize(report.freed_bytes)}`);
      }
    } catch (e) {
      const error = sanitizeError(e);
      this.lastRun = {
        items,
        report: { results: [], freed_bytes: 0, disk_freed_bytes: 0 },
        error,
      };
      toastStore.error(`Cleanup failed: ${error}`);
    } finally {
      this.selectedItems = [];
      this.status = "completed";
      getDiskInfo()
        .then((info) => (appStore.diskInfo = info))
        .catch(() => {});
    }
  }

  private handleProgress(progress: CleanProgress) {
    switch (progress.type) {
      case "Started":
        this.totalItems = progress.total_items;
        this.completedItems = 0;
        break;
      case "ItemCompleted":
        this.completedItems++;
        break;
    }
  }

  reset() {
    this.status = "idle";
    this.selectedItems = [];
    this.lastRun = null;
    this.totalItems = 0;
    this.completedItems = 0;
  }
}

export const cleanupStore = new CleanupStore();

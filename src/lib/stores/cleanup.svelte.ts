import type { ScanResult, CleanResult, CleanProgress } from "$lib/tauri/types";

export type CleanupStatus = "idle" | "confirming" | "cleaning" | "completed";

class CleanupStore {
  status = $state<CleanupStatus>("idle");
  selectedItems = $state<ScanResult[]>([]);
  results = $state<CleanResult[]>([]);
  totalFreed = $state(0);
  totalItems = $state(0);
  completedItems = $state(0);
  error = $state<string | null>(null);

  select(items: ScanResult[]) {
    this.selectedItems = items;
    this.status = "confirming";
  }

  toggleItem(item: ScanResult) {
    const idx = this.selectedItems.findIndex((i) => i.id === item.id);
    if (idx >= 0) {
      this.selectedItems = this.selectedItems.filter((_, i) => i !== idx);
    } else {
      this.selectedItems = [...this.selectedItems, item];
    }
  }

  private selectedIds = $derived(new Set(this.selectedItems.map((i) => i.id)));

  isSelected(id: string): boolean {
    return this.selectedIds.has(id);
  }

  handleProgress(progress: CleanProgress) {
    switch (progress.type) {
      case "Started":
        this.totalItems = progress.total_items;
        this.completedItems = 0;
        this.status = "cleaning";
        break;
      case "ItemCompleted":
        this.completedItems++;
        this.totalFreed += progress.freed_bytes;
        break;
      case "Failed":
        this.completedItems++;
        break;
      case "Completed":
        this.totalFreed = progress.total_freed;
        this.status = "completed";
        break;
    }
  }

  setResults(results: CleanResult[]) {
    this.results = results;
    this.status = "completed";
  }

  reset() {
    this.status = "idle";
    this.selectedItems = [];
    this.results = [];
    this.totalFreed = 0;
    this.totalItems = 0;
    this.completedItems = 0;
    this.error = null;
  }
}

export const cleanupStore = new CleanupStore();

import type { ActivityEntry, ActivityLevel } from "$lib/tauri/types";

export type ActivityFilter = "all" | "commands" | "errors" | "success";

const DRAWER_HEIGHT_KEY = "macweep:activity-drawer-height";
const MAX_ENTRIES = 500;

class ActivityStore {
  entries = $state<ActivityEntry[]>([]);
  isExpanded = $state(false);
  drawerHeight = $state(200);
  filter = $state<ActivityFilter>("all");
  searchQuery = $state("");

  filteredEntries = $derived.by(() => {
    let result = this.entries;

    if (this.filter === "commands")
      result = result.filter((e) => e.level === "Command");
    else if (this.filter === "errors")
      result = result.filter(
        (e) => e.level === "Error" || e.level === "Warning",
      );
    else if (this.filter === "success")
      result = result.filter((e) => e.level === "Success");

    if (this.searchQuery.trim()) {
      const q = this.searchQuery.toLowerCase();
      result = result.filter(
        (e) =>
          e.message.toLowerCase().includes(q) ||
          (e.category && e.category.toLowerCase().includes(q)) ||
          (e.command && e.command.toLowerCase().includes(q)) ||
          (e.detail && e.detail.toLowerCase().includes(q)),
      );
    }

    return result;
  });

  latestEntry = $derived(
    this.entries.length > 0 ? this.entries[this.entries.length - 1] : null,
  );

  errorCount = $derived(
    this.entries.filter((e) => e.level === "Error" || e.level === "Warning")
      .length,
  );

  constructor() {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem(DRAWER_HEIGHT_KEY);
      if (saved) {
        const parsed = parseInt(saved, 10);
        if (parsed >= 120 && parsed <= 600) {
          this.drawerHeight = parsed;
        }
      }
    }
  }

  addEntry(entry: ActivityEntry) {
    this.entries = [...this.entries, entry].slice(-MAX_ENTRIES);
  }

  loadHistory(entries: ActivityEntry[]) {
    this.entries = entries.slice(-MAX_ENTRIES);
  }

  clear() {
    this.entries = [];
  }

  toggleExpanded() {
    this.isExpanded = !this.isExpanded;
  }

  setHeight(px: number) {
    const clamped = Math.max(120, Math.min(px, window.innerHeight * 0.5));
    this.drawerHeight = clamped;
    localStorage.setItem(DRAWER_HEIGHT_KEY, String(clamped));
  }
}

export const activityStore = new ActivityStore();

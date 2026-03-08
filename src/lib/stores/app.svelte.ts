import type { DiskInfo } from "$lib/tauri/types";

class AppStore {
  diskInfo = $state<DiskInfo | null>(null);
}

export const appStore = new AppStore();

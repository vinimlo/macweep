import type { DiskInfo } from '$lib/tauri/types';

export type ExecutionMode = 'interactive' | 'scan-only' | 'safe' | 'dry-run';

class AppStore {
	executionMode = $state<ExecutionMode>('interactive');
	diskInfo = $state<DiskInfo | null>(null);
}

export const appStore = new AppStore();

<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import { appStore } from '$lib/stores/app.svelte';
	import { toastStore } from '$lib/stores/toasts.svelte';
	import { scanAll, cancelScan, getDiskInfo } from '$lib/tauri/commands';
	import { sanitizeError } from '$lib/utils/errors';
	import DiskUsageBar from '$lib/components/dashboard/DiskUsageBar.svelte';
	import SpaceSummary from '$lib/components/dashboard/SpaceSummary.svelte';
	import CategoryGrid from '$lib/components/dashboard/CategoryGrid.svelte';
	import QuickActions from '$lib/components/dashboard/QuickActions.svelte';
	import ScanProgress from '$lib/components/scan/ScanProgress.svelte';

	let wasCancelled = false;

	async function handleScan() {
		scanStore.reset();
		wasCancelled = false;
		try {
			const report = await scanAll((progress) => {
				if (!wasCancelled) {
					scanStore.handleProgress(progress);
				}
			});
			if (!wasCancelled && report.items.length > 0) {
				scanStore.setReport(report);
				appStore.diskInfo = await getDiskInfo();
			}
		} catch (e) {
			if (!wasCancelled) {
				scanStore.error = sanitizeError(e);
				toastStore.error(`Scan failed: ${sanitizeError(e)}`);
			}
		}
	}

	async function handleStop() {
		wasCancelled = true;
		scanStore.status = 'idle';
		scanStore.currentScanner = null;
		try {
			await cancelScan();
		} catch {
			// best-effort
		}
		toastStore.info('Scan stopped');
	}
</script>

<div class="dashboard">
	{#if appStore.diskInfo}
		<DiskUsageBar info={appStore.diskInfo} />
	{/if}

	{#if scanStore.status === 'completed'}
		<SpaceSummary />
	{/if}

	<QuickActions onscan={handleScan} onstop={handleStop} />

	{#if scanStore.status === 'scanning'}
		<div class="scan-section">
			<ScanProgress />
		</div>
	{/if}

	<CategoryGrid />
</div>

<style>
	.dashboard {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		max-width: 640px;
		margin: 0 auto;
		width: 100%;
		min-height: 100%;
	}

	.scan-section {
		padding: var(--space-base);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
	}
</style>

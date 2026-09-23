<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import { appStore } from '$lib/stores/app.svelte';
	import DiskUsageBar from '$lib/components/dashboard/DiskUsageBar.svelte';
	import SpaceSummary from '$lib/components/dashboard/SpaceSummary.svelte';
	import CategoryGrid from '$lib/components/dashboard/CategoryGrid.svelte';
	import QuickActions from '$lib/components/dashboard/QuickActions.svelte';
	import ScanProgress from '$lib/components/scan/ScanProgress.svelte';
</script>

<div class="dashboard">
	{#if appStore.diskInfo}
		<DiskUsageBar info={appStore.diskInfo} />
	{/if}

	{#if scanStore.status === 'completed'}
		<SpaceSummary />
	{/if}

	<QuickActions />

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
		/* Keeps the last category reachable above the floating selection bar. */
		padding-bottom: 64px;
	}

	.scan-section {
		padding: var(--space-base);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		box-shadow: var(--highlight);
	}
</style>

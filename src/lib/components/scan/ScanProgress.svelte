<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import ProgressBar from '$lib/components/shared/ProgressBar.svelte';
	import CategoryScanRow from './CategoryScanRow.svelte';

	const progress = $derived(
		scanStore.totalScanners > 0
			? (scanStore.completedScanners / scanStore.totalScanners) * 100
			: 0
	);
</script>

<div class="scan-progress">
	{#if scanStore.status === 'scanning'}
		<div class="header">
			<div class="radar">
				<span class="radar-center"></span>
				<span class="radar-ring"></span>
				<span class="radar-ring delay"></span>
			</div>
			<div class="header-text">
				<span class="title">Scanning system</span>
				<span class="counter">
				{#if scanStore.totalScanners > 0}
					{scanStore.completedScanners} of {scanStore.totalScanners} scanners
				{:else}
					Checking installed tools…
				{/if}
			</span>
			</div>
		</div>
		<ProgressBar value={progress} color="var(--accent)" />
	{/if}

	<div class="scanners">
		{#each Object.entries(scanStore.scannerStatuses) as [category, status] (category)}
			<CategoryScanRow {category} {status} />
		{/each}
	</div>
</div>

<style>
	.scan-progress {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.radar {
		position: relative;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.radar-center {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent);
	}

	.radar-ring {
		position: absolute;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		border: 1.5px solid var(--accent);
		animation: radar-ring 1.8s ease-out infinite;
	}

	.radar-ring.delay {
		animation-delay: 0.6s;
	}

	.header-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.title {
		font-weight: 600;
		font-size: var(--text-base);
	}

	.counter {
		font-size: var(--text-xs);
		color: var(--text-muted);
	}

	.scanners {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
</style>

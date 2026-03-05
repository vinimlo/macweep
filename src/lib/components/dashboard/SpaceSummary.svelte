<script lang="ts">
	import { formatSize, formatDuration } from '$lib/utils/format';
	import { scanStore } from '$lib/stores/scan.svelte';
</script>

<div class="summary">
	<div class="hero">
		<span class="hero-value">{formatSize(scanStore.totalBytes)}</span>
		<span class="hero-label">recoverable</span>
	</div>

	<div class="metrics">
		<div class="metric">
			<span class="metric-value">{scanStore.items.length}</span>
			<span class="metric-label">items</span>
		</div>
		<span class="metric-sep"></span>
		<div class="metric">
			<span class="metric-value">{scanStore.categories.length}</span>
			<span class="metric-label">categories</span>
		</div>
		{#if scanStore.scanDurationMs > 0}
			<span class="metric-sep"></span>
			<div class="metric">
				<span class="metric-value">{formatDuration(scanStore.scanDurationMs)}</span>
				<span class="metric-label">scan time</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.summary {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-base) 0;
		animation: slideUp var(--duration-slow) var(--ease-out);
	}

	.hero {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
	}

	.hero-value {
		font-size: 36px;
		font-weight: 200;
		letter-spacing: -0.03em;
		line-height: 1;
		color: var(--accent);
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
	}

	.hero-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		letter-spacing: 0.02em;
	}

	.metrics {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.metric {
		display: flex;
		align-items: baseline;
		gap: 4px;
	}

	.metric-value {
		font-family: var(--font-mono);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.metric-label {
		font-size: 10px;
		color: var(--text-muted);
		letter-spacing: 0.02em;
	}

	.metric-sep {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--bg-active);
	}
</style>

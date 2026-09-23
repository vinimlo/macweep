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
		gap: var(--space-sm);
		padding: var(--space-sm) 0;
		animation: fadeIn var(--duration-slow) var(--ease-out);
	}

	.hero {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
	}

	/* The one bold moment of the dashboard: how much can be swept. */
	.hero-value {
		font-size: var(--text-display);
		font-weight: 300;
		letter-spacing: -0.03em;
		line-height: 1;
		color: var(--accent);
	}

	.hero-label {
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-secondary);
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
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--text-primary);
	}

	.metric-label {
		font-size: var(--text-sm);
		color: var(--text-muted);
	}

	.metric-sep {
		width: 1px;
		height: 10px;
		background: var(--border-default);
	}
</style>

<script lang="ts">
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import ProgressBar from '$lib/components/shared/ProgressBar.svelte';

	const progress = $derived(
		cleanupStore.totalItems > 0
			? (cleanupStore.completedItems / cleanupStore.totalItems) * 100
			: 0
	);
</script>

<div class="cleanup-progress">
	<div class="sweep-icon">
		<svg width="32" height="32" viewBox="0 0 32 32" fill="none">
			<circle cx="16" cy="16" r="12" stroke="var(--risk-zero)" stroke-width="1.5" opacity="0.3"/>
			<circle cx="16" cy="16" r="12" stroke="var(--risk-zero)" stroke-width="1.5"
				stroke-dasharray={2 * Math.PI * 12}
				stroke-dashoffset={2 * Math.PI * 12 * (1 - progress / 100)}
				stroke-linecap="round"
				transform="rotate(-90 16 16)"
				style="transition: stroke-dashoffset 0.3s var(--ease-out)"
			/>
		</svg>
	</div>

	<div class="info">
		<span class="title">Sweeping</span>
		<span class="counter">{cleanupStore.completedItems} of {cleanupStore.totalItems}</span>
	</div>

	<div class="bar-wrap">
		<ProgressBar value={progress} color="var(--risk-zero)" />
	</div>
</div>

<style>
	.cleanup-progress {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-lg);
		padding: var(--space-2xl);
		animation: fadeIn var(--duration-slow) var(--ease-out);
	}

	.sweep-icon {
		animation: spin 3s linear infinite;
	}

	.info {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}

	.title {
		font-weight: 600;
		font-size: 15px;
	}

	.counter {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}

	.bar-wrap {
		width: 200px;
	}
</style>

<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import { goto } from '$app/navigation';

	const zeroItems = $derived(scanStore.byRisk.Zero);
	const zeroBytes = $derived(zeroItems.reduce((sum, i) => sum + i.size_bytes, 0));
	function cleanSafe() {
		cleanupStore.select(zeroItems);
		goto('/cleanup');
	}
</script>

<div class="actions">
	{#if scanStore.status === 'scanning'}
		<button class="btn btn-lg btn-tinted stop" onclick={() => scanStore.stop()}>
			<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
				<rect x="2" y="2" width="8" height="8" rx="1.5" fill="currentColor"/>
			</svg>
			Stop Scan
		</button>
	{:else}
		<button class="btn btn-lg btn-primary" onclick={() => scanStore.start()}>
			<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
				<circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1.5"/>
				<path d="M9.5 9.5L13 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
			Scan System
		</button>
	{/if}

	{#if zeroItems.length > 0}
		<button class="btn btn-lg btn-tinted safe" onclick={cleanSafe}>
			<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
				<path d="M2 6l3 3 5-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
			Auto-clean Safe
			<span class="btn-meta">{formatSize(zeroBytes)}</span>
		</button>
	{/if}

</div>

<style>
	.actions {
		display: flex;
		gap: var(--space-sm);
		flex-wrap: wrap;
		justify-content: center;
	}

	.stop {
		--tint: var(--risk-high);
	}

	.safe {
		--tint: var(--risk-zero);
	}

	.btn-meta {
		font-size: var(--text-xs);
		font-weight: 500;
		opacity: 0.8;
	}
</style>

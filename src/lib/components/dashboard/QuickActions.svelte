<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import { goto } from '$app/navigation';


	let { onscan, onstop }: { onscan: () => void; onstop: () => void } = $props();

	const zeroItems = $derived(scanStore.byRisk.Zero);
	const zeroBytes = $derived(zeroItems.reduce((sum, i) => sum + i.size_bytes, 0));
	function cleanSafe() {
		cleanupStore.select(zeroItems);
		goto('/cleanup');
	}
</script>

<div class="actions">
	{#if scanStore.status === 'scanning'}
		<button class="btn btn-stop" onclick={onstop}>
			<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
				<rect x="2" y="2" width="8" height="8" rx="1.5" fill="currentColor"/>
			</svg>
			Stop Scan
		</button>
	{:else}
		<button class="btn btn-scan" onclick={onscan}>
			<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
				<circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1.5"/>
				<path d="M9.5 9.5L13 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
			Scan System
		</button>
	{/if}

	{#if zeroItems.length > 0}
		<button class="btn btn-safe" onclick={cleanSafe}>
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

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: var(--radius-md);
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.01em;
		transition: all var(--duration-fast) var(--ease-out);
	}

	.btn-scan {
		background: var(--accent);
		color: var(--text-inverse);
	}

	.btn-scan:hover {
		background: var(--accent-hover);
		box-shadow: 0 0 16px var(--accent-glow);
	}

	.btn-stop {
		background: var(--risk-high-dim);
		color: var(--risk-high);
		border: 1px solid color-mix(in srgb, var(--risk-high) 25%, transparent);
	}

	.btn-stop:hover {
		background: color-mix(in srgb, var(--risk-high) 16%, transparent);
		border-color: color-mix(in srgb, var(--risk-high) 40%, transparent);
	}

	.btn-safe {
		background: var(--risk-zero-dim);
		color: var(--risk-zero);
		border: 1px solid color-mix(in srgb, var(--risk-zero) 25%, transparent);
	}

	.btn-safe:hover {
		background: color-mix(in srgb, var(--risk-zero) 16%, transparent);
		border-color: color-mix(in srgb, var(--risk-zero) 40%, transparent);
	}

	.btn-meta {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 500;
		opacity: 0.7;
		font-variant-numeric: tabular-nums;
	}
</style>

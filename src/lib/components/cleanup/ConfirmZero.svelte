<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import { formatSize } from '$lib/utils/format';

	let { items, onconfirm, onskip }: {
		items: ScanResult[];
		onconfirm: () => void;
		onskip: () => void;
	} = $props();

	const totalBytes = $derived(items.reduce((s, i) => s + i.size_bytes, 0));
</script>

<div class="confirm">
	<div class="header">
		<div class="icon-wrap safe">
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
				<path d="M3 8l4 4 6-7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</div>
		<div>
			<h3>Safe to Clean</h3>
			<p class="subtitle">100% regenerable caches &mdash; <strong>{formatSize(totalBytes)}</strong></p>
		</div>
	</div>

	<div class="items">
		{#each items as item, i (item.id)}
			<div class="item" style="animation-delay: {i * 20}ms">
				<span class="item-label">{item.label}</span>
				<span class="item-size">{formatSize(item.size_bytes)}</span>
			</div>
		{/each}
	</div>

	<div class="actions">
		<button class="btn btn-skip" onclick={onskip}>Skip</button>
		<button class="btn btn-confirm" onclick={onconfirm}>
			Clean All
			<span class="btn-size">{formatSize(totalBytes)}</span>
		</button>
	</div>
</div>

<style>
	@import './confirm-shared.css';

	.confirm { gap: var(--space-lg); }

	.icon-wrap.safe {
		background: var(--risk-zero-dim);
		color: var(--risk-zero);
	}

	.subtitle strong {
		font-family: var(--font-mono);
		color: var(--text-primary);
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 1px;
		background: var(--bg-base);
		border-radius: var(--radius-md);
		overflow: hidden;
		max-height: 220px;
		overflow-y: auto;
	}

	.item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-raised);
		font-size: 12px;
		animation: fadeIn var(--duration-base) var(--ease-out) both;
	}

	.item-label {
		color: var(--text-primary);
	}

	.btn-confirm {
		background: var(--risk-zero);
		color: var(--text-inverse);
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.btn-confirm:hover {
		opacity: 0.9;
	}

	.btn-size {
		font-family: var(--font-mono);
		font-size: 10px;
		opacity: 0.7;
	}
</style>

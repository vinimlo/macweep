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

<div class="confirm-step">
	<div class="confirm-header">
		<div class="confirm-icon safe">
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
				<path d="M3 8l4 4 6-7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</div>
		<div>
			<h3 class="confirm-title">Safe to Clean</h3>
			<p class="confirm-subtitle">Caches that tools recreate on their own when needed.</p>
		</div>
	</div>

	<div class="items">
		{#each items as item (item.id)}
			<div class="item">
				<span class="item-label">{item.label}</span>
				<span class="confirm-size">{formatSize(item.size_bytes)}</span>
			</div>
		{/each}
	</div>

	<div class="confirm-actions">
		<button class="btn btn-secondary" onclick={onskip}>Skip</button>
		<button class="btn btn-solid confirm" onclick={onconfirm}>
			Clean All
			<span class="btn-size">{formatSize(totalBytes)}</span>
		</button>
	</div>
</div>

<style>
	.safe {
		--tone: var(--risk-zero);
	}

	.items {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-raised);
		overflow-y: auto;
		max-height: 240px;
	}

	.item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		font-size: var(--text-sm);
	}

	.item + .item {
		border-top: 1px solid var(--border-subtle);
	}

	.item-label {
		color: var(--text-primary);
	}

	.confirm {
		--tint: var(--risk-zero);
	}

	.btn-size {
		font-weight: 500;
		opacity: 0.75;
	}
</style>

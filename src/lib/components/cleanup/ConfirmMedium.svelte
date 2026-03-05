<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import { formatSize } from '$lib/utils/format';

	let { items, onconfirm, onskip }: {
		items: ScanResult[];
		onconfirm: (selected: ScanResult[]) => void;
		onskip: () => void;
	} = $props();

	let selected = $state<Set<string>>(new Set());

	function toggle(id: string) {
		const next = new Set(selected);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selected = next;
	}

	const selectedItems = $derived(items.filter((i) => selected.has(i.id)));
	const selectedBytes = $derived(selectedItems.reduce((s, i) => s + i.size_bytes, 0));
</script>

<div class="confirm">
	<div class="header">
		<div class="icon-wrap medium">
			<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
				<path d="M7 1.5L1 12h12L7 1.5z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
				<path d="M7 6v2.5M7 10.5v0" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
		</div>
		<div>
			<h3>Possibly Stale Items</h3>
			<p class="subtitle">May belong to tools no longer installed. Review each item.</p>
		</div>
	</div>

	<div class="items">
		{#each items as item, i (item.id)}
			<label class="item" style="animation-delay: {i * 25}ms">
				<input type="checkbox" checked={selected.has(item.id)} onchange={() => toggle(item.id)} />
				<div class="item-info">
					<div class="item-top">
						<span class="item-label">{item.label}</span>
						<span class="item-size">{formatSize(item.size_bytes)}</span>
					</div>
					<span class="item-detail">{item.detail}</span>
					<span class="item-path">{item.path}</span>
				</div>
			</label>
		{/each}
	</div>

	<div class="actions">
		<span class="actions-info">{selected.size} selected &middot; {formatSize(selectedBytes)}</span>
		<button class="btn btn-skip" onclick={onskip}>Skip</button>
		<button class="btn btn-confirm" disabled={selected.size === 0} onclick={() => onconfirm(selectedItems)}>
			Clean {selected.size} Item{selected.size !== 1 ? 's' : ''}
		</button>
	</div>
</div>

<style>
	@import './confirm-shared.css';

	.icon-wrap.medium {
		background: var(--risk-medium-dim);
		color: var(--risk-medium);
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		max-height: 320px;
		overflow-y: auto;
	}

	.item {
		display: flex;
		gap: var(--space-md);
		padding: var(--space-md);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: all var(--duration-fast);
		animation: fadeIn var(--duration-base) var(--ease-out) both;
	}

	.item:hover {
		background: var(--bg-surface);
		border-color: var(--border-default);
	}

	.item input {
		accent-color: var(--risk-medium);
		margin-top: 2px;
		flex-shrink: 0;
	}

	.item-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}

	.item-top {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.item-label {
		font-weight: 600;
		font-size: 12px;
		color: var(--text-primary);
	}

	.item-detail {
		font-size: 11px;
		color: var(--text-secondary);
	}

	.item-path {
		font-size: 10px;
		color: var(--text-muted);
		font-family: var(--font-mono);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.actions { align-items: center; }

	.actions-info {
		margin-right: auto;
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}

	.btn-confirm {
		background: var(--risk-medium);
		color: var(--text-inverse);
	}

	.btn-confirm:hover:not(:disabled) { opacity: 0.9; }
</style>

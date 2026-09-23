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

<div class="confirm-step">
	<div class="confirm-header">
		<div class="confirm-icon medium">
			<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
				<path d="M7 1.5L1 12h12L7 1.5z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
				<path d="M7 6v2.5M7 10.5v0" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
		</div>
		<div>
			<h3 class="confirm-title">Possibly Stale Items</h3>
			<p class="confirm-subtitle">Data from tools that may no longer be in use. Pick each one you want to remove.</p>
		</div>
	</div>

	<div class="items">
		{#each items as item (item.id)}
			<label class="item" class:checked={selected.has(item.id)}>
				<input
					type="checkbox"
					checked={selected.has(item.id)}
					onchange={() => toggle(item.id)}
				/>
				<div class="item-info">
					<div class="item-top">
						<span class="item-label">{item.label}</span>
						<span class="confirm-size">{formatSize(item.size_bytes)}</span>
					</div>
					<span class="item-detail">{item.detail}</span>
					<span class="confirm-path path selectable" title={item.path}>{item.path}</span>
				</div>
			</label>
		{/each}
	</div>

	<div class="confirm-actions">
		<span class="actions-info">{selected.size} selected, {formatSize(selectedBytes)}</span>
		<button class="btn btn-secondary" onclick={onskip}>Skip</button>
		<button
			class="btn btn-solid confirm"
			disabled={selected.size === 0}
			onclick={() => onconfirm(selectedItems)}
		>
			Clean {selected.size} Item{selected.size !== 1 ? 's' : ''}
		</button>
	</div>
</div>

<style>
	.medium {
		--tone: var(--risk-medium);
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
		align-items: flex-start;
		gap: var(--space-md);
		padding: var(--space-md);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		box-shadow: var(--highlight);
		cursor: pointer;
		transition:
			background-color var(--duration-fast) var(--ease-out),
			border-color var(--duration-fast) var(--ease-out);
	}

	.item:hover {
		background: var(--bg-surface);
		border-color: var(--border-default);
	}

	.item.checked {
		border-color: color-mix(in srgb, var(--risk-medium) 35%, transparent);
	}

	.item input {
		--check-color: var(--risk-medium);
		margin-top: 1px;
	}

	.item-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.item-top {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-md);
	}

	.item-label {
		font-weight: 600;
		font-size: var(--text-base);
		color: var(--text-primary);
	}

	.item-detail {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.actions-info {
		margin-right: auto;
		font-size: var(--text-sm);
		color: var(--text-muted);
	}

	.confirm {
		--tint: var(--risk-medium);
	}
</style>

<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import { formatSize } from '$lib/utils/format';

	let { items, onconfirm, onskip }: {
		items: ScanResult[];
		onconfirm: (selected: ScanResult[]) => void;
		onskip: () => void;
	} = $props();

	function getAllIds() { return new Set(items.map((i) => i.id)); }
	let selected = $state<Set<string>>(getAllIds());

	function toggle(id: string) {
		const next = new Set(selected);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selected = next;
	}

	function selectAll() { selected = getAllIds(); }
	function selectNone() { selected = new Set(); }

	const selectedItems = $derived(items.filter((i) => selected.has(i.id)));
	const selectedBytes = $derived(selectedItems.reduce((s, i) => s + i.size_bytes, 0));
</script>

<div class="confirm">
	<div class="header">
		<div class="icon-wrap low">
			<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
				<circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/>
				<path d="M7 4.5v3M7 9.5v0" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
		</div>
		<div>
			<h3>Regenerable Items</h3>
			<p class="subtitle">Can be restored with a command (e.g. <code>npm install</code>)</p>
		</div>
	</div>

	<div class="toolbar">
		<button class="link" onclick={selectAll}>All</button>
		<span class="sep">/</span>
		<button class="link" onclick={selectNone}>None</button>
		<span class="selected-info">
			{selected.size} selected &middot; {formatSize(selectedBytes)}
		</span>
	</div>

	<div class="items">
		{#each items as item, i (item.id)}
			<label class="item" class:checked={selected.has(item.id)} style="animation-delay: {i * 15}ms">
				<input type="checkbox" checked={selected.has(item.id)} onchange={() => toggle(item.id)} />
				<span class="item-label">{item.label}</span>
				{#if item.warning}
					<span class="item-warning">{item.warning}</span>
				{/if}
				<span class="item-path">{item.path}</span>
				<span class="item-size">{formatSize(item.size_bytes)}</span>
			</label>
		{/each}
	</div>

	{#if items[0]?.regeneration_hint}
		<p class="hint">{items[0].regeneration_hint}</p>
	{/if}

	<div class="actions">
		<button class="btn btn-skip" onclick={onskip}>Skip</button>
		<button class="btn btn-confirm" disabled={selected.size === 0} onclick={() => onconfirm(selectedItems)}>
			Clean {selected.size} Items
		</button>
	</div>
</div>

<style>
	@import './confirm-shared.css';

	.icon-wrap.low {
		background: var(--risk-low-dim);
		color: var(--risk-low);
	}

	code {
		font-family: var(--font-mono);
		font-size: 11px;
		background: var(--bg-overlay);
		padding: 1px 4px;
		border-radius: 3px;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
	}

	.link {
		color: var(--accent);
		font-size: 11px;
		font-weight: 500;
	}

	.link:hover { text-decoration: underline; }

	.sep { color: var(--text-muted); }

	.selected-info {
		color: var(--text-muted);
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 10px;
		font-variant-numeric: tabular-nums;
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 1px;
		background: var(--bg-base);
		border-radius: var(--radius-md);
		overflow: hidden;
		max-height: 260px;
		overflow-y: auto;
	}

	.item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		font-size: 12px;
		cursor: pointer;
		background: var(--bg-raised);
		transition: background var(--duration-fast);
		animation: fadeIn var(--duration-base) var(--ease-out) both;
	}

	.item:hover { background: var(--bg-surface); }

	.item input {
		accent-color: var(--accent);
		flex-shrink: 0;
	}

	.item-label {
		font-weight: 500;
		white-space: nowrap;
		color: var(--text-primary);
	}

	.item-path {
		flex: 1;
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-size: 10px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		direction: rtl;
		text-align: left;
	}

	.item-warning {
		font-size: 9px;
		color: var(--risk-medium, #e6a700);
		font-weight: 500;
		white-space: nowrap;
	}

	.item-size { white-space: nowrap; }

	.hint {
		font-size: 11px;
		color: var(--text-muted);
		font-style: italic;
		padding-left: var(--space-sm);
		border-left: 2px solid var(--border-default);
	}

	.btn-confirm {
		background: var(--risk-low);
		color: var(--text-inverse);
	}

	.btn-confirm:hover:not(:disabled) { opacity: 0.9; }
</style>

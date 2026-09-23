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

<div class="confirm-step">
	<div class="confirm-header">
		<div class="confirm-icon low">
			<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
				<path d="M11.5 5A4.5 4.5 0 102.9 9.2M11.5 5V2M11.5 5H8.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</div>
		<div>
			<h3 class="confirm-title">Regenerable Items</h3>
			<p class="confirm-subtitle">Restored with a command such as <code>npm install</code> or <code>docker pull</code>.</p>
		</div>
	</div>

	<div class="toolbar">
		<button class="link" onclick={selectAll}>Select All</button>
		<button class="link" onclick={selectNone}>Select None</button>
		<span class="selected-info">{selected.size} selected, {formatSize(selectedBytes)}</span>
	</div>

	<div class="items">
		{#each items as item (item.id)}
			<label class="item">
				<input
					type="checkbox"
					checked={selected.has(item.id)}
					onchange={() => toggle(item.id)}
				/>
				<span class="item-main">
					<span class="item-label">{item.label}</span>
					<span class="confirm-path path selectable" title={item.path}>{item.path}</span>
				</span>
				{#if item.warning}
					<span class="confirm-warning">{item.warning}</span>
				{/if}
				<span class="confirm-size">{formatSize(item.size_bytes)}</span>
			</label>
		{/each}
	</div>

	<div class="confirm-actions">
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
	.low {
		--tone: var(--risk-low);
	}

	code {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		background: var(--bg-overlay);
		padding: 1px 5px;
		border-radius: 4px;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		font-size: var(--text-sm);
	}

	.link {
		color: var(--accent);
		font-weight: 500;
		border-radius: var(--radius-sm);
	}

	.link:hover {
		color: var(--accent-hover);
	}

	.selected-info {
		color: var(--text-muted);
		margin-left: auto;
	}

	.items {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-raised);
		max-height: 260px;
		overflow-y: auto;
	}

	.item {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		font-size: var(--text-sm);
		cursor: pointer;
		transition: background-color var(--duration-fast) var(--ease-out);
	}

	.item + .item {
		border-top: 1px solid var(--border-subtle);
	}

	.item:hover {
		background: var(--bg-surface);
	}

	.item input {
		--check-color: var(--risk-low);
	}

	.item-main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.item-label {
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.confirm {
		--tint: var(--risk-low);
	}
</style>

<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import RiskBadge from '$lib/components/shared/RiskBadge.svelte';
	import { formatSize } from '$lib/utils/format';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { riskColor } from '$lib/utils/risk';

	let { category, items, totalBytes, maxBytes, onclick }: {
		category: string;
		items: ScanResult[];
		totalBytes: number;
		maxBytes: number;
		onclick: () => void;
	} = $props();

	const selectedCount = $derived(items.filter((i) => cleanupStore.isSelected(i.id)).length);
	const allSelected = $derived(selectedCount === items.length);
	const barPercent = $derived(maxBytes > 0 ? (totalBytes / maxBytes) * 100 : 0);
	const color = $derived(riskColor(items[0].risk_level));

	function toggleAll() {
		const select = !allSelected;
		for (const item of items) {
			if (cleanupStore.isSelected(item.id) !== select) cleanupStore.toggleItem(item);
		}
	}
</script>

<div class="row" style="--risk-color: {color}">
	<input
		type="checkbox"
		class="row-check"
		checked={allSelected}
		indeterminate={selectedCount > 0 && !allSelected}
		onchange={toggleAll}
		aria-label="Select all {category} items"
	/>

	<button class="row-content" onclick={onclick}>
		<div class="row-top">
			<span class="row-name">{category}</span>
			<RiskBadge level={items[0].risk_level} />
			<span class="row-count" title="{items.length} item{items.length !== 1 ? 's' : ''}">{items.length}</span>
			<span class="row-size">{formatSize(totalBytes)}</span>
		</div>
		<div class="row-bar-track">
			<div class="row-bar-fill" style="width: {barPercent}%"></div>
		</div>
	</button>
</div>

<style>
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: 10px var(--space-base) 10px var(--space-md);
		border-radius: var(--radius-md);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		box-shadow: var(--highlight);
		transition:
			background-color var(--duration-fast) var(--ease-out),
			border-color var(--duration-fast) var(--ease-out);
	}

	.row:hover {
		background: var(--bg-surface);
		border-color: var(--border-default);
	}

	.row-check {
		--check-color: var(--risk-color);
	}

	.row-content {
		flex: 1;
		min-width: 0;
		text-align: left;
		display: flex;
		flex-direction: column;
		gap: 7px;
		border-radius: var(--radius-sm);
	}

	.row-top {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.row-name {
		font-weight: 600;
		font-size: var(--text-base);
		color: var(--text-primary);
	}

	.row-count {
		font-size: var(--text-xs);
		font-weight: 500;
		color: var(--text-muted);
		margin-left: auto;
	}

	.row-size {
		font-size: var(--text-base);
		font-weight: 600;
		color: var(--text-primary);
		min-width: 64px;
		text-align: right;
	}

	.row-bar-track {
		height: 3px;
		background: var(--bg-active);
		border-radius: var(--radius-round);
		overflow: hidden;
	}

	.row-bar-fill {
		height: 100%;
		border-radius: var(--radius-round);
		background: var(--risk-color);
		opacity: 0.55;
		transition:
			width 0.6s var(--ease-out),
			opacity var(--duration-fast);
	}

	.row:hover .row-bar-fill {
		opacity: 0.9;
	}
</style>

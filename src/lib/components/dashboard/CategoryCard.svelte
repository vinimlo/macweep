<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import RiskBadge from '$lib/components/shared/RiskBadge.svelte';
	import { formatSize } from '$lib/utils/format';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { riskColor } from '$lib/utils/risk';

	let { category, items, totalBytes, maxBytes, onclick, index = 0 }: {
		category: string;
		items: ScanResult[];
		totalBytes: number;
		maxBytes: number;
		onclick: () => void;
		index?: number;
	} = $props();

	const allSelected = $derived(items.every((i) => cleanupStore.isSelected(i.id)));
	const barPercent = $derived(maxBytes > 0 ? (totalBytes / maxBytes) * 100 : 0);
	const color = $derived(riskColor(items[0].risk_level));

	function toggleAll() {
		if (allSelected) {
			for (const item of items) {
				if (cleanupStore.isSelected(item.id)) cleanupStore.toggleItem(item);
			}
		} else {
			for (const item of items) {
				if (!cleanupStore.isSelected(item.id)) cleanupStore.toggleItem(item);
			}
		}
	}
</script>

<div
	class="row"
	style="animation-delay: {index * 40}ms"
>
	<label class="row-check">
		<input type="checkbox" checked={allSelected} onchange={toggleAll} />
		<span class="checkmark" style="--check-color: {color}"></span>
	</label>

	<button class="row-content" onclick={onclick}>
		<div class="row-top">
			<span class="row-name">{category}</span>
			<RiskBadge level={items[0].risk_level} />
			<span class="row-count">{items.length}</span>
			<span class="row-size">{formatSize(totalBytes)}</span>
		</div>
		<div class="row-bar-track">
			<div
				class="row-bar-fill"
				style="width: {barPercent}%; --bar-color: {color}"
			></div>
		</div>
	</button>
</div>

<style>
	.row {
		display: flex;
		align-items: stretch;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-base);
		border-radius: var(--radius-md);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		transition: all var(--duration-base) var(--ease-out);
		animation: slideUp var(--duration-slow) var(--ease-out) both;
	}

	.row:hover {
		background: var(--bg-surface);
		border-color: var(--border-default);
	}

	.row-check {
		display: flex;
		align-items: center;
		cursor: pointer;
		position: relative;
	}

	.row-check input {
		position: absolute;
		opacity: 0;
		width: 0;
		height: 0;
	}

	.checkmark {
		width: 16px;
		height: 16px;
		border-radius: 4px;
		border: 1.5px solid var(--border-hover);
		background: var(--bg-base);
		transition: all var(--duration-fast);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.checkmark::after {
		content: '';
		width: 8px;
		height: 8px;
		border-radius: 2px;
		background: var(--check-color);
		opacity: 0;
		transform: scale(0);
		transition: all var(--duration-fast) var(--ease-spring);
	}

	.row-check input:checked + .checkmark {
		border-color: var(--check-color);
	}

	.row-check input:checked + .checkmark::after {
		opacity: 1;
		transform: scale(1);
	}

	.row-content {
		flex: 1;
		text-align: left;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.row-top {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.row-name {
		font-weight: 600;
		font-size: 12px;
		color: var(--text-primary);
	}

	.row-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		background: var(--bg-overlay);
		padding: 1px 5px;
		border-radius: var(--radius-sm);
		font-variant-numeric: tabular-nums;
		margin-left: auto;
	}

	.row-size {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
		min-width: 60px;
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
		background: var(--bar-color);
		transition: width 0.6s var(--ease-out);
		opacity: 0.6;
	}

	.row:hover .row-bar-fill {
		opacity: 1;
	}
</style>

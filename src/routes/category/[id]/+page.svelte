<script lang="ts">
	import { page } from '$app/state';
	import { scanStore } from '$lib/stores/scan.svelte';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import RiskBadge from '$lib/components/shared/RiskBadge.svelte';
	import { goto } from '$app/navigation';

	const categoryId = $derived(decodeURIComponent(page.params.id ?? ''));
	const group = $derived(scanStore.categories.find((c) => c.category === categoryId));

	let sortBy = $state<'size-desc' | 'size-asc' | 'name'>('size-desc');

	const sortedItems = $derived.by(() => {
		if (!group) return [];
		const items = [...group.items];
		switch (sortBy) {
			case 'size-desc': return items.sort((a, b) => b.size_bytes - a.size_bytes);
			case 'size-asc': return items.sort((a, b) => a.size_bytes - b.size_bytes);
			case 'name': return items.sort((a, b) => a.label.localeCompare(b.label));
		}
	});

	function cycleSortMode() {
		const modes: typeof sortBy[] = ['size-desc', 'size-asc', 'name'];
		sortBy = modes[(modes.indexOf(sortBy) + 1) % modes.length];
	}

	const sortConfig: Record<typeof sortBy, { label: string; title: string }> = {
		'size-desc': { label: '↓', title: 'Largest first' },
		'size-asc': { label: '↑', title: 'Smallest first' },
		'name': { label: 'AZ', title: 'Name A–Z' }
	};

	const sortLabel = $derived(sortConfig[sortBy].label);
	const sortTitle = $derived(sortConfig[sortBy].title);
</script>

{#if group}
	<div class="category-page">
		<div class="page-header">
			<button class="back" onclick={() => goto('/')} aria-label="Back to dashboard">
				<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
					<path d="M8 2L3 6l5 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</button>
			<h2>{group.category}</h2>
			<RiskBadge level={group.items[0].risk_level} />
			<button class="sort-btn" onclick={cycleSortMode} title={sortTitle} aria-label="Sort: {sortTitle}">
				{sortLabel}
			</button>
			<span class="total-size">{formatSize(group.totalBytes)}</span>
		</div>

		<div class="items">
			{#each sortedItems as item (item.id)}
				<label class="item">
					<input
						type="checkbox"
						checked={cleanupStore.isSelected(item.id)}
						onchange={() => cleanupStore.toggleItem(item)}
					/>
					<div class="item-content">
						<div class="item-top">
							<span class="item-label">{item.label}</span>
							<span class="item-size">{formatSize(item.size_bytes)}</span>
						</div>
						<span class="item-path selectable">{item.path}</span>
						{#if item.detail}
							<span class="item-detail">{item.detail}</span>
						{/if}
						{#if item.warning}
							<span class="item-warning">{item.warning}</span>
						{/if}
						{#if item.regeneration_hint}
							<span class="item-hint">To restore: {item.regeneration_hint}</span>
						{/if}
					</div>
				</label>
			{/each}
		</div>

	</div>
{:else}
	<div class="empty">
		<p>Category not found.</p>
		<a href="/">Back to Dashboard</a>
	</div>
{/if}

<style>
	.category-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-base);
		max-width: 640px;
		margin: 0 auto;
		width: 100%;
	}

	.page-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.back {
		display: inline-flex;
		color: var(--text-secondary);
		padding: 4px;
		border-radius: var(--radius-sm);
		transition: color var(--duration-fast) var(--ease-out);
	}

	.back:hover {
		color: var(--accent);
	}

	.sort-btn {
		min-width: 26px;
		height: 22px;
		padding: 0 6px;
		color: var(--text-secondary);
		font-size: var(--text-xs);
		font-weight: 600;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-default);
		background: var(--bg-raised);
		transition:
			color var(--duration-fast) var(--ease-out),
			border-color var(--duration-fast) var(--ease-out);
	}

	.sort-btn:hover {
		color: var(--accent);
		border-color: var(--border-hover);
	}

	h2 {
		font-size: var(--text-xl);
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.total-size {
		margin-left: auto;
		font-size: var(--text-xl);
		font-weight: 300;
		letter-spacing: -0.01em;
		color: var(--accent);
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.item {
		display: flex;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-base) var(--space-md) var(--space-md);
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

	.item input {
		margin-top: 1px;
	}

	.item-content {
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
		gap: var(--space-md);
	}

	.item-label {
		font-weight: 600;
		font-size: var(--text-base);
	}

	.item-size {
		font-size: var(--text-base);
		font-weight: 600;
		color: var(--text-primary);
		white-space: nowrap;
	}

	.item-path {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--text-muted);
		word-break: break-all;
	}

	.item-detail {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.item-warning {
		font-size: var(--text-xs);
		color: var(--risk-medium);
		font-weight: 500;
		padding: 1px 7px;
		background: var(--risk-medium-dim);
		border-radius: var(--radius-sm);
		margin-top: 2px;
		width: fit-content;
	}

	.item-hint {
		font-size: var(--text-xs);
		color: var(--text-muted);
	}

	.empty {
		text-align: center;
		padding: var(--space-3xl);
		color: var(--text-secondary);
		font-size: var(--text-base);
	}

	.empty a {
		display: inline-block;
		margin-top: var(--space-sm);
	}
</style>

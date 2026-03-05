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

	const sortLabel = $derived(
		sortBy === 'size-desc' ? '↓' : sortBy === 'size-asc' ? '↑' : 'AZ'
	);

	const sortTitle = $derived(
		sortBy === 'size-desc' ? 'Largest first' : sortBy === 'size-asc' ? 'Smallest first' : 'Name A–Z'
	);
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
			<button class="sort-btn" onclick={cycleSortMode} title={sortTitle}>
				{sortLabel}
			</button>
			<span class="total-size">{formatSize(group.totalBytes)}</span>
		</div>

		<div class="items">
			{#each sortedItems as item, i (item.id)}
				<label class="item" style="animation-delay: {i * 25}ms">
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
						<span class="item-path">{item.path}</span>
						{#if item.detail}
							<span class="item-detail">{item.detail}</span>
						{/if}
						{#if item.warning}
							<span class="item-warning">{item.warning}</span>
						{/if}
						{#if item.regeneration_hint}
							<span class="item-hint">{item.regeneration_hint}</span>
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
		color: var(--text-secondary);
		padding: 4px;
		border-radius: var(--radius-sm);
		transition: color var(--duration-fast);
	}

	.back:hover { color: var(--accent); }

	.sort-btn {
		color: var(--text-secondary);
		font-size: 11px;
		font-weight: 600;
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		background: var(--bg-raised);
		cursor: pointer;
		transition: all var(--duration-fast);
		line-height: 1;
		font-family: var(--font-mono);
	}

	.sort-btn:hover {
		color: var(--accent);
		border-color: var(--border-default);
	}

	h2 {
		font-size: 16px;
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.total-size {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 16px;
		font-weight: 200;
		letter-spacing: -0.02em;
		color: var(--accent);
		font-variant-numeric: tabular-nums;
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.item {
		display: flex;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-base);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: all var(--duration-fast);
		animation: slideUp var(--duration-slow) var(--ease-out) both;
	}

	.item:hover {
		background: var(--bg-surface);
		border-color: var(--border-default);
	}

	.item input {
		accent-color: var(--accent);
		margin-top: 3px;
		flex-shrink: 0;
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
	}

	.item-label {
		font-weight: 600;
		font-size: 12px;
	}

	.item-size {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.item-path {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		word-break: break-all;
	}

	.item-detail {
		font-size: 11px;
		color: var(--text-secondary);
	}

	.item-warning {
		font-size: 10px;
		color: var(--risk-medium, #e6a700);
		font-weight: 500;
		padding: 2px 6px;
		background: color-mix(in srgb, var(--risk-medium, #e6a700) 10%, transparent);
		border-radius: var(--radius-sm);
		margin-top: 2px;
		width: fit-content;
	}

	.item-hint {
		font-size: 10px;
		color: var(--text-muted);
		font-style: italic;
		padding-left: var(--space-sm);
		border-left: 2px solid var(--border-default);
		margin-top: 2px;
	}

	.empty {
		text-align: center;
		padding: var(--space-3xl);
		color: var(--text-secondary);
		font-size: 13px;
	}

	.empty a {
		display: inline-block;
		margin-top: var(--space-sm);
	}
</style>

<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import CategoryCard from './CategoryCard.svelte';
	import { goto } from '$app/navigation';

	const maxBytes = $derived(
		scanStore.categories.length > 0
			? Math.max(...scanStore.categories.map((c) => c.totalBytes))
			: 0
	);
</script>

{#if scanStore.categories.length === 0 && scanStore.status !== 'scanning'}
	<div class="empty">
		{#if scanStore.status === 'completed'}
			<div class="empty-icon">
				<svg width="32" height="32" viewBox="0 0 32 32" fill="none">
					<circle cx="16" cy="16" r="12" stroke="var(--risk-zero)" stroke-width="1.5" opacity="0.4"/>
					<path d="M11 16l3 3 7-8" stroke="var(--risk-zero)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</div>
			<p class="empty-title">System is clean</p>
			<p class="empty-detail">No recoverable space found.</p>
		{:else}
			<div class="empty-icon">
				<svg width="32" height="32" viewBox="0 0 32 32" fill="none">
					<circle cx="16" cy="16" r="12" stroke="var(--text-muted)" stroke-width="1.5" stroke-dasharray="3 3"/>
					<circle cx="16" cy="16" r="2" fill="var(--text-muted)"/>
				</svg>
			</div>
			<p class="empty-title">No scan data</p>
			<p class="empty-detail">Run a scan to discover recoverable space.</p>
		{/if}
	</div>
{:else}
	<div class="list-header">
		<span class="list-title">Categories</span>
		<span class="list-count">{scanStore.categories.length}</span>
	</div>
	<div class="list">
		{#each scanStore.categories as group, i (group.category)}
			<CategoryCard
				category={group.category}
				items={group.items}
				totalBytes={group.totalBytes}
				{maxBytes}
				index={i}
				onclick={() => goto(`/category/${encodeURIComponent(group.category)}`)}
			/>
		{/each}
	</div>
{/if}

<style>
	.list-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-md);
	}

	.list-title {
		font-weight: 600;
		font-size: 10px;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.list-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		background: var(--bg-overlay);
		padding: 1px 5px;
		border-radius: var(--radius-sm);
		font-variant-numeric: tabular-nums;
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		padding: var(--space-lg);
		text-align: center;
		animation: fadeIn var(--duration-slow) var(--ease-out);
	}

	.empty-icon {
		margin-bottom: var(--space-base);
		opacity: 0.6;
	}

	.empty-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-secondary);
		margin-bottom: var(--space-xs);
	}

	.empty-detail {
		font-size: 12px;
		color: var(--text-muted);
	}
</style>

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
		<h2 class="list-title">Categories</h2>
		<span class="list-count">{scanStore.categories.length}</span>
	</div>
	<div class="list">
		{#each scanStore.categories as group (group.category)}
			<CategoryCard
				category={group.category}
				items={group.items}
				totalBytes={group.totalBytes}
				{maxBytes}
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
		margin-bottom: var(--space-sm);
		padding: 0 var(--space-2xs);
	}

	.list-title {
		font-weight: 600;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.list-count {
		font-size: var(--text-xs);
		font-weight: 500;
		color: var(--text-muted);
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 6px;
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
		margin-bottom: var(--space-md);
		opacity: 0.7;
	}

	.empty-title {
		font-size: var(--text-lg);
		font-weight: 600;
		color: var(--text-primary);
		margin-bottom: var(--space-2xs);
	}

	.empty-detail {
		font-size: var(--text-sm);
		color: var(--text-muted);
	}
</style>

<script lang="ts">
	import Spinner from '$lib/components/shared/Spinner.svelte';

	let { category, status, index = 0 }: {
		category: string;
		status: 'pending' | 'scanning' | 'completed' | 'failed';
		index?: number;
	} = $props();
</script>

<div
	class="row"
	class:pending={status === 'pending'}
	class:scanning={status === 'scanning'}
	class:completed={status === 'completed'}
	class:failed={status === 'failed'}
	style="animation-delay: {index * 25}ms"
>
	<span class="status-icon">
		{#if status === 'scanning'}
			<Spinner size={12} color="var(--accent)" />
		{:else if status === 'completed'}
			<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
				<path d="M2.5 6l2.5 2.5 4.5-5" stroke="var(--risk-zero)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		{:else if status === 'failed'}
			<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
				<path d="M3 3l6 6M9 3L3 9" stroke="var(--risk-high)" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
		{:else}
			<span class="pending-dot"></span>
		{/if}
	</span>
	<span class="category">{category}</span>
</div>

<style>
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 5px var(--space-sm);
		border-radius: var(--radius-sm);
		font-size: 12px;
		transition: all var(--duration-base) var(--ease-out);
		animation: fadeIn var(--duration-base) var(--ease-out) both;
	}

	.row.pending {
		color: var(--text-muted);
	}

	.row.scanning {
		color: var(--text-primary);
		background: var(--accent-glow);
	}

	.row.completed {
		color: var(--text-secondary);
	}

	.row.failed {
		color: var(--risk-high);
	}

	.status-icon {
		width: 14px;
		height: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.pending-dot {
		width: 4px;
		height: 4px;
		border-radius: 50%;
		background: var(--text-muted);
	}

	.category {
		font-family: var(--font-mono);
		font-size: 11px;
		letter-spacing: 0.01em;
	}
</style>

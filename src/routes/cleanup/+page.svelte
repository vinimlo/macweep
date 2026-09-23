<script lang="ts">
	import CleanupFlow from '$lib/components/cleanup/CleanupFlow.svelte';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { goto } from '$app/navigation';

	if (cleanupStore.selectedItems.length === 0 || cleanupStore.status === 'cleaning') {
		goto('/');
	}
</script>

<div class="cleanup-page">
	<div class="page-header">
		<button class="back" onclick={() => goto('/')} aria-label="Back to dashboard">
			<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
				<path d="M8 2L3 6l5 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
			Dashboard
		</button>
		<h2>Cleanup</h2>
	</div>
	<CleanupFlow />
</div>

<style>
	.cleanup-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
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
		align-items: center;
		gap: 4px;
		padding: 2px 4px;
		border-radius: var(--radius-sm);
		color: var(--text-secondary);
		font-size: var(--text-sm);
		font-weight: 500;
		transition: color var(--duration-fast) var(--ease-out);
	}

	.back:hover {
		color: var(--accent);
	}

	h2 {
		font-size: var(--text-xl);
		font-weight: 600;
		letter-spacing: -0.01em;
	}
</style>

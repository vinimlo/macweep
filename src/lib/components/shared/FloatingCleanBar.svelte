<script lang="ts">
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	const count = $derived(cleanupStore.selectedItems.length);
	const totalBytes = $derived(
		cleanupStore.selectedItems.reduce((sum, i) => sum + i.size_bytes, 0)
	);
	// Hidden on the cleanup page itself (it used to float over the confirmation steps).
	const visible = $derived(
		count > 0 && cleanupStore.status !== 'cleaning' && page.url.pathname !== '/cleanup'
	);
</script>

{#if visible}
	<div class="floating-bar">
		<div class="bar-inner">
			<div class="bar-info">
				<span class="bar-count">{count}</span>
				<span class="bar-label">item{count !== 1 ? 's' : ''} selected</span>
				<span class="bar-sep"></span>
				<span class="bar-size">{formatSize(totalBytes)}</span>
			</div>

			<div class="bar-actions">
				<button class="btn btn-ghost btn-pill" onclick={() => cleanupStore.reset()}>
					Clear
				</button>
				<button class="btn btn-primary btn-pill" onclick={() => goto('/cleanup')}>
					Clean
					<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
						<path d="M4 2l5 4-5 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
					</svg>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.floating-bar {
		position: fixed;
		bottom: 44px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 900;
		animation: bar-enter var(--duration-slow) var(--ease-spring) both;
	}

	.bar-inner {
		display: flex;
		align-items: center;
		gap: var(--space-base);
		padding: 6px 6px 6px var(--space-lg);
		background: color-mix(in srgb, var(--bg-overlay) 88%, transparent);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-round);
		box-shadow: var(--highlight), var(--shadow-lg);
		backdrop-filter: blur(20px) saturate(1.4);
		-webkit-backdrop-filter: blur(20px) saturate(1.4);
	}

	.bar-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		white-space: nowrap;
	}

	.bar-count {
		font-size: var(--text-base);
		font-weight: 700;
		color: var(--accent);
		min-width: 1.4ch;
		text-align: right;
	}

	.bar-label {
		font-size: var(--text-sm);
		color: var(--text-secondary);
		font-weight: 500;
	}

	.bar-sep {
		width: 1px;
		height: 14px;
		background: var(--border-default);
	}

	.bar-size {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--text-primary);
	}

	.bar-actions {
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
	}

	@keyframes bar-enter {
		from {
			opacity: 0;
			transform: translateX(-50%) translateY(12px) scale(0.97);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) translateY(0) scale(1);
		}
	}
</style>

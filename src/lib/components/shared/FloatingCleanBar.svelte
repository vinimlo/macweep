<script lang="ts">
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import { goto } from '$app/navigation';

	const count = $derived(cleanupStore.selectedItems.length);
	const totalBytes = $derived(
		cleanupStore.selectedItems.reduce((sum, i) => sum + i.size_bytes, 0)
	);
	const visible = $derived(count > 0 && cleanupStore.status === 'idle');
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
				<button class="btn-clear" onclick={() => cleanupStore.reset()}>
					Clear
				</button>
				<button class="btn-clean" onclick={() => goto('/cleanup')}>
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
		bottom: var(--space-lg);
		left: 50%;
		transform: translateX(-50%);
		z-index: 900;
		animation: bar-enter var(--duration-slow) var(--ease-spring) both;
	}

	.bar-inner {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
		padding: 10px 10px 10px var(--space-lg);
		background: var(--bg-surface);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-round);
		box-shadow:
			0 8px 32px rgba(0, 0, 0, 0.5),
			0 0 0 1px rgba(255, 255, 255, 0.03),
			0 0 48px var(--accent-glow);
		backdrop-filter: blur(16px);
		-webkit-backdrop-filter: blur(16px);
	}

	.bar-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		white-space: nowrap;
	}

	.bar-count {
		font-family: var(--font-mono);
		font-size: 14px;
		font-weight: 700;
		color: var(--accent);
		font-variant-numeric: tabular-nums;
		min-width: 1.4ch;
		text-align: right;
	}

	.bar-label {
		font-size: 12px;
		color: var(--text-secondary);
		font-weight: 500;
	}

	.bar-sep {
		width: 1px;
		height: 14px;
		background: var(--border-default);
	}

	.bar-size {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.02em;
	}

	.bar-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.btn-clear {
		padding: 6px 14px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		border-radius: var(--radius-round);
		transition: all var(--duration-fast);
	}

	.btn-clear:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.btn-clean {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		background: var(--accent);
		color: var(--text-inverse);
		border-radius: var(--radius-round);
		font-weight: 700;
		font-size: 12px;
		letter-spacing: 0.01em;
		transition: all var(--duration-fast);
	}

	.btn-clean:hover {
		background: var(--accent-hover);
		box-shadow: 0 0 20px var(--accent-glow-strong);
	}

	@keyframes bar-enter {
		from {
			opacity: 0;
			transform: translateX(-50%) translateY(20px) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) translateY(0) scale(1);
		}
	}
</style>

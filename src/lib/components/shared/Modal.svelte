<script lang="ts">
	import type { Snippet } from 'svelte';

	let { title, onclose, children }: { title: string; onclose: () => void; children: Snippet } = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onclose();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay" onclick={onclose} role="presentation">
	<div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-label={title}>
		<div class="header">
			<h2>{title}</h2>
			<button class="close" onclick={onclose} aria-label="Close">
				<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
					<path d="M1 1l12 12M13 1L1 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
				</svg>
			</button>
		</div>
		<div class="body">
			{@render children()}
		</div>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(5, 6, 10, 0.75);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
		backdrop-filter: blur(8px);
		animation: fadeIn var(--duration-base) var(--ease-out);
	}

	.modal {
		background: var(--bg-raised);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-lg);
		max-width: 520px;
		width: 92%;
		max-height: 80vh;
		overflow-y: auto;
		animation: scaleIn var(--duration-slow) var(--ease-out);
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-base) var(--space-lg);
		border-bottom: 1px solid var(--border-subtle);
	}

	.header h2 {
		font-size: 14px;
		font-weight: 600;
	}

	.close {
		color: var(--text-muted);
		padding: 4px;
		border-radius: var(--radius-sm);
		transition: color var(--duration-fast);
	}

	.close:hover {
		color: var(--text-primary);
	}

	.body {
		padding: var(--space-lg);
	}
</style>

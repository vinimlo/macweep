<script lang="ts">
	import type { ToastType } from '$lib/stores/toasts.svelte';

	let { type, message, onclose }: { type: ToastType; message: string; onclose: () => void } = $props();

	const colors: Record<ToastType, string> = {
		success: 'var(--success)',
		error: 'var(--error)',
		info: 'var(--info)',
		warning: 'var(--warning)'
	};

	const icons: Record<ToastType, string> = {
		success: '\u2713',
		error: '\u2717',
		info: '\u2139',
		warning: '\u26A0'
	};
</script>

<div class="toast" style="--toast-color: {colors[type]}" role={type === 'error' ? 'alert' : 'status'}>
	<span class="icon">{icons[type]}</span>
	<span class="message selectable">{message}</span>
	<button class="close" onclick={onclose} aria-label="Dismiss">
		<svg width="10" height="10" viewBox="0 0 10 10" fill="none">
			<path d="M1 1l8 8M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
		</svg>
	</button>
</div>

<style>
	.toast {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 10px var(--space-md);
		background: color-mix(in srgb, var(--bg-overlay) 92%, transparent);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		box-shadow: var(--highlight), var(--shadow-md);
		backdrop-filter: blur(16px);
		-webkit-backdrop-filter: blur(16px);
		animation: toast-in 0.25s var(--ease-out);
	}

	.icon {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--toast-color) 16%, transparent);
		color: var(--toast-color);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: var(--text-2xs);
		font-weight: 700;
		flex-shrink: 0;
	}

	.message {
		flex: 1;
		font-size: var(--text-sm);
		line-height: 1.4;
		color: var(--text-primary);
	}

	.close {
		display: flex;
		color: var(--text-muted);
		padding: 4px;
		border-radius: var(--radius-sm);
		flex-shrink: 0;
		transition: color var(--duration-fast) var(--ease-out);
	}

	.close:hover {
		color: var(--text-primary);
	}
</style>

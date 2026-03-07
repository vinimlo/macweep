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

<div class="toast" style="--toast-color: {colors[type]}">
	<span class="icon">{icons[type]}</span>
	<span class="message">{message}</span>
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
		padding: var(--space-md) var(--space-base);
		background: var(--bg-raised);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-md);
		animation: toast-in 0.25s var(--ease-out);
	}

	.icon {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--toast-color) 15%, transparent);
		color: var(--toast-color);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 10px;
		font-weight: 700;
		flex-shrink: 0;
	}

	.message {
		flex: 1;
		font-size: 12px;
		line-height: 1.4;
		color: var(--text-primary);
	}

	.close {
		color: var(--text-muted);
		padding: 4px;
		flex-shrink: 0;
		transition: color var(--duration-fast);
	}

	.close:hover {
		color: var(--text-primary);
	}
</style>

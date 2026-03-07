<script lang="ts">
	import ScanProgress from '$lib/components/scan/ScanProgress.svelte';
	import { scanStore } from '$lib/stores/scan.svelte';
	import { scanAll } from '$lib/tauri/commands';
	import { toastStore } from '$lib/stores/toasts.svelte';
	import { sanitizeError } from '$lib/utils/errors';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(async () => {
		if (scanStore.status !== 'scanning') {
			scanStore.reset();
			try {
				const report = await scanAll((progress) => {
					scanStore.handleProgress(progress);
				});
				scanStore.setReport(report);
			} catch (e) {
				toastStore.error(`Scan failed: ${sanitizeError(e)}`);
			}
		}
	});
</script>

<div class="scan-page">
	<div class="page-header">
		<h2>System Scan</h2>
		{#if scanStore.status === 'completed'}
			<button class="btn-view" onclick={() => goto('/')}>
				View Dashboard
				<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
					<path d="M4 2l5 4-5 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</button>
		{/if}
	</div>

	<div class="scan-area">
		<ScanProgress />
	</div>
</div>

<style>
	.scan-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		max-width: 560px;
		margin: 0 auto;
		width: 100%;
	}

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	h2 {
		font-size: 16px;
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.btn-view {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 6px 14px;
		background: var(--accent);
		color: var(--text-inverse);
		border-radius: var(--radius-md);
		font-weight: 600;
		font-size: 11px;
		transition: all var(--duration-fast);
	}

	.btn-view:hover {
		background: var(--accent-hover);
		box-shadow: 0 0 12px var(--accent-glow);
	}

	.scan-area {
		padding: var(--space-base);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
	}
</style>

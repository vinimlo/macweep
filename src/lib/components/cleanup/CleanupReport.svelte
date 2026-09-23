<script lang="ts">
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import { goto } from '$app/navigation';

	const MB = 1024 * 1024;

	const run = $derived(cleanupStore.lastRun);
	const labels = $derived(new Map(run?.items.map((i) => [i.id, i.label]) ?? []));
	const succeeded = $derived(run?.report.results.filter((r) => r.success) ?? []);
	const failed = $derived(run?.report.results.filter((r) => !r.success) ?? []);
	const freed = $derived(run?.report.freed_bytes ?? 0);
	const diskFreed = $derived(run?.report.disk_freed_bytes ?? 0);
	const touchedDocker = $derived(
		run?.items.some((i) => i.category.startsWith('docker')) ?? false
	);
	// Explain the gap only when it is large enough to be noticed.
	const diskLags = $derived(freed - diskFreed > Math.max(100 * MB, freed * 0.1));

	const title = $derived(
		run?.error
			? 'Cleanup Failed'
			: failed.length > 0
				? 'Cleanup Finished with Errors'
				: 'Cleanup Complete'
	);

	function done() {
		cleanupStore.reset();
		goto('/');
	}
</script>

{#if run}
	<div class="report">
		<div class="status-icon" class:warn={failed.length > 0 || run.error}>
			{#if failed.length > 0 || run.error}
				<svg width="24" height="24" viewBox="0 0 24 24" fill="none">
					<path d="M12 7v6M12 16.5v0" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"/>
				</svg>
			{:else}
				<svg width="26" height="26" viewBox="0 0 28 28" fill="none">
					<path d="M7 14l5 5 9-10" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			{/if}
		</div>

		<h2>{title}</h2>

		{#if run.error}
			<p class="run-error selectable">{run.error}</p>
		{:else}
			<div class="stats">
				<div class="stat">
					<span class="stat-value">{formatSize(freed)}</span>
					<span class="stat-label">removed</span>
				</div>
				<span class="stat-sep"></span>
				<div class="stat">
					<span class="stat-value">{succeeded.length}</span>
					<span class="stat-label">cleaned</span>
				</div>
				{#if failed.length > 0}
					<span class="stat-sep"></span>
					<div class="stat error">
						<span class="stat-value">{failed.length}</span>
						<span class="stat-label">failed</span>
					</div>
				{/if}
			</div>

			<p class="disk">
				Free space on disk <strong>+{formatSize(diskFreed)}</strong>
			</p>
			{#if diskLags}
				<p class="note">
					{#if touchedDocker}
						Docker keeps freed space inside its disk image until Docker Desktop restarts.
					{:else}
						macOS can hold deleted files in local snapshots for a while before the space shows up as free.
					{/if}
				</p>
			{/if}
		{/if}

		{#if failed.length > 0}
			<div class="errors">
				<span class="errors-title">Not cleaned</span>
				{#each failed as result (result.id)}
					<div class="error-row">
						<span class="error-label">{labels.get(result.id) ?? 'Unknown item'}</span>
						<span class="error-msg selectable">{result.error}</span>
					</div>
				{/each}
			</div>
		{/if}

		<button class="btn btn-lg btn-primary" onclick={done}>Back to Dashboard</button>
	</div>
{/if}

<style>
	.report {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-base);
		padding: var(--space-xl) var(--space-lg);
		text-align: center;
		animation: scaleIn var(--duration-slow) var(--ease-out);
	}

	.status-icon {
		width: 52px;
		height: 52px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--risk-zero);
		background: var(--risk-zero-dim);
		animation: check-pop 0.5s var(--ease-spring) 0.1s both;
	}

	.status-icon.warn {
		color: var(--risk-medium);
		background: var(--risk-medium-dim);
	}

	h2 {
		font-size: var(--text-xl);
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.stats {
		display: flex;
		align-items: center;
		gap: var(--space-base);
	}

	.stat {
		display: flex;
		align-items: baseline;
		gap: 5px;
	}

	.stat-value {
		font-size: var(--text-2xl);
		font-weight: 300;
		letter-spacing: -0.02em;
		color: var(--text-primary);
	}

	.stat-label {
		font-size: var(--text-sm);
		color: var(--text-muted);
	}

	.stat.error .stat-value {
		color: var(--risk-high);
	}

	.stat-sep {
		width: 1px;
		height: 14px;
		background: var(--border-default);
	}

	.disk {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.disk strong {
		font-weight: 600;
		color: var(--text-primary);
	}

	.note {
		max-width: 400px;
		font-size: var(--text-sm);
		color: var(--text-muted);
		margin-top: calc(var(--space-sm) * -1);
	}

	.run-error {
		max-width: 440px;
		font-size: var(--text-sm);
		color: var(--risk-high);
	}

	.errors {
		width: 100%;
		text-align: left;
		padding: var(--space-md) var(--space-base);
		background: var(--risk-high-dim);
		border: 1px solid color-mix(in srgb, var(--risk-high) 22%, transparent);
		border-radius: var(--radius-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.errors-title {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--risk-high);
	}

	.error-row {
		display: flex;
		flex-direction: column;
		font-size: var(--text-sm);
	}

	.error-label {
		font-weight: 600;
		color: var(--text-primary);
	}

	.error-msg {
		color: var(--text-secondary);
		word-break: break-word;
	}
</style>

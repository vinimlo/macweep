<script lang="ts">
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { formatSize } from '$lib/utils/format';
	import { goto } from '$app/navigation';

	const successCount = $derived(cleanupStore.results.filter((r) => r.success).length);
	const errorCount = $derived(cleanupStore.results.filter((r) => !r.success).length);
</script>

<div class="report">
	<div class="check-circle">
		<svg width="28" height="28" viewBox="0 0 28 28" fill="none">
			<path d="M7 14l5 5 9-10" stroke="var(--risk-zero)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
		</svg>
	</div>

	<h2>Sweep Complete</h2>

	<div class="stats">
		<div class="stat">
			<span class="stat-value">{formatSize(cleanupStore.totalFreed)}</span>
			<span class="stat-label">freed</span>
		</div>
		<span class="stat-sep"></span>
		<div class="stat">
			<span class="stat-value">{successCount}</span>
			<span class="stat-label">cleaned</span>
		</div>
		{#if errorCount > 0}
			<span class="stat-sep"></span>
			<div class="stat error">
				<span class="stat-value">{errorCount}</span>
				<span class="stat-label">failed</span>
			</div>
		{/if}
	</div>

	{#if errorCount > 0}
		<div class="errors">
			<span class="errors-title">Errors</span>
			{#each cleanupStore.results.filter((r) => !r.success) as result (result.id)}
				<div class="error-row">
					<span class="error-id">{result.id.slice(0, 8)}</span>
					<span class="error-msg">{result.error}</span>
				</div>
			{/each}
		</div>
	{/if}

	<button class="btn-done" onclick={() => { cleanupStore.reset(); goto('/'); }}>
		Back to Dashboard
	</button>
</div>

<style>
	.report {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-lg);
		padding: var(--space-2xl);
		text-align: center;
		animation: scaleIn var(--duration-slow) var(--ease-out);
	}

	.check-circle {
		width: 56px;
		height: 56px;
		border-radius: 50%;
		background: var(--risk-zero-dim);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: check-pop 0.5s var(--ease-spring) 0.15s both;
	}

	h2 {
		font-size: 18px;
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.stats {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
	}

	.stat {
		display: flex;
		align-items: baseline;
		gap: 5px;
	}

	.stat-value {
		font-family: var(--font-mono);
		font-size: 22px;
		font-weight: 200;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.stat-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.stat.error .stat-value { color: var(--risk-high); }

	.stat-sep {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--bg-active);
	}

	.errors {
		width: 100%;
		text-align: left;
		padding: var(--space-md);
		background: var(--risk-high-dim);
		border: 1px solid color-mix(in srgb, var(--risk-high) 20%, transparent);
		border-radius: var(--radius-md);
	}

	.errors-title {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--risk-high);
		display: block;
		margin-bottom: var(--space-sm);
	}

	.error-row {
		font-size: 11px;
		color: var(--text-secondary);
		padding: 3px 0;
		display: flex;
		gap: var(--space-sm);
	}

	.error-id {
		font-family: var(--font-mono);
		color: var(--text-muted);
		font-size: 10px;
	}

	.error-msg { color: var(--risk-high); }

	.btn-done {
		padding: 10px 28px;
		background: var(--accent);
		color: var(--text-inverse);
		border-radius: var(--radius-md);
		font-weight: 600;
		font-size: 12px;
		transition: all var(--duration-fast);
	}

	.btn-done:hover {
		background: var(--accent-hover);
		box-shadow: 0 0 16px var(--accent-glow);
	}
</style>

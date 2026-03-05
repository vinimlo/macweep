<script lang="ts">
	import { scanStore } from '$lib/stores/scan.svelte';
	import { formatSize, formatDuration } from '$lib/utils/format';
</script>

<div class="summary">
	<div class="check-icon">
		<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
			<path d="M3 8l4 4 6-7" stroke="var(--risk-zero)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
		</svg>
	</div>
	<div class="summary-text">
		<span>
			Found <strong>{formatSize(scanStore.totalBytes)}</strong> across
			<strong>{scanStore.items.length}</strong> items
		</span>
		<span class="duration">{formatDuration(scanStore.scanDurationMs)}</span>
	</div>
</div>

<style>
	.summary {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-md);
		background: var(--risk-zero-dim);
		border: 1px solid color-mix(in srgb, var(--risk-zero) 20%, transparent);
		border-radius: var(--radius-md);
		font-size: 12px;
		color: var(--text-primary);
		animation: scaleIn var(--duration-slow) var(--ease-out);
	}

	.check-icon {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--risk-zero) 15%, transparent);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: check-pop 0.4s var(--ease-spring) 0.1s both;
	}

	strong {
		font-family: var(--font-mono);
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}

	.summary-text {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		flex: 1;
	}

	.duration {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}
</style>

<script lang="ts">
	import type { DiskInfo } from '$lib/tauri/types';
	import { scanStore } from '$lib/stores/scan.svelte';
	import { formatSize, formatPercent, diskColor } from '$lib/utils/format';

	let { info }: { info: DiskInfo } = $props();

	const size = 180;
	const strokeWidth = 10;
	const radius = (size - strokeWidth) / 2;
	const circumference = 2 * Math.PI * radius;
	const cx = size / 2;
	const cy = size / 2;

	const usedPercent = $derived(info.total_bytes > 0 ? (info.used_bytes / info.total_bytes) * 100 : 0);
	const recoverablePercent = $derived(
		info.total_bytes > 0 ? (scanStore.totalBytes / info.total_bytes) * 100 : 0
	);

	const usedOffset = $derived(circumference * (1 - usedPercent / 100));
	const recoverableOffset = $derived(circumference * (1 - recoverablePercent / 100));

	const ringColor = $derived(diskColor(usedPercent, 'var(--text-muted)'));
</script>

<div class="gauge">
	<div class="gauge-ring">
		<svg width={size} height={size} viewBox="0 0 {size} {size}">
			<!-- Track -->
			<circle
				cx={cx} cy={cy} r={radius}
				fill="none"
				stroke="var(--bg-active)"
				stroke-width={strokeWidth}
			/>
			<!-- Used space -->
			<circle
				cx={cx} cy={cy} r={radius}
				fill="none"
				stroke={ringColor}
				stroke-width={strokeWidth}
				stroke-dasharray={circumference}
				stroke-dashoffset={usedOffset}
				stroke-linecap="round"
				transform="rotate(-90 {cx} {cy})"
				class="used-arc"
			/>
			<!-- Recoverable overlay -->
			{#if recoverablePercent > 0}
				<circle
					cx={cx} cy={cy} r={radius}
					fill="none"
					stroke="var(--accent)"
					stroke-width={strokeWidth}
					stroke-dasharray={circumference}
					stroke-dashoffset={recoverableOffset}
					stroke-linecap="round"
					transform="rotate({-90 + (usedPercent - recoverablePercent) * 3.6} {cx} {cy})"
					class="recoverable-arc"
					opacity="0.9"
				/>
			{/if}
		</svg>

		<div class="gauge-center">
			<span class="gauge-value">{formatPercent(usedPercent)}</span>
			<span class="gauge-label">used</span>
		</div>
	</div>

	<div class="gauge-stats">
		<div class="stat-row">
			<span class="stat-dot" style="background: var(--text-muted)"></span>
			<span class="stat-label">Used</span>
			<span class="stat-value">{formatSize(info.used_bytes)}</span>
		</div>
		<div class="stat-row">
			<span class="stat-dot" style="background: var(--bg-active)"></span>
			<span class="stat-label">Free</span>
			<span class="stat-value">{formatSize(info.free_bytes)}</span>
		</div>
		{#if scanStore.totalBytes > 0}
			<div class="stat-row recoverable">
				<span class="stat-dot" style="background: var(--accent)"></span>
				<span class="stat-label">Sweepable</span>
				<span class="stat-value">{formatSize(scanStore.totalBytes)}</span>
			</div>
		{/if}
		<div class="stat-total">
			<span class="stat-label">Total</span>
			<span class="stat-value">{formatSize(info.total_bytes)}</span>
		</div>
	</div>
</div>

<style>
	.gauge {
		display: flex;
		align-items: center;
		gap: var(--space-xl);
		justify-content: center;
		padding: var(--space-lg) 0;
		animation: fadeIn var(--duration-slow) var(--ease-out);
	}

	.gauge-ring {
		position: relative;
		flex-shrink: 0;
	}

	.used-arc {
		transition: stroke-dashoffset 0.8s var(--ease-out), stroke 0.4s;
	}

	.recoverable-arc {
		transition: stroke-dashoffset 0.8s var(--ease-out);
		filter: drop-shadow(0 0 6px var(--accent-glow-strong));
	}

	.gauge-center {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
	}

	.gauge-value {
		font-size: 32px;
		font-weight: 200;
		letter-spacing: -0.03em;
		line-height: 1;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.gauge-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-muted);
		margin-top: 2px;
	}

	.gauge-stats {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		min-width: 150px;
	}

	.stat-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.stat-row.recoverable {
		padding: var(--space-xs) 0;
		margin: var(--space-2xs) 0;
		border-top: 1px solid var(--border-subtle);
		border-bottom: 1px solid var(--border-subtle);
	}

	.stat-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.stat-label {
		flex: 1;
		font-size: 11px;
		color: var(--text-secondary);
	}

	.stat-value {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.stat-total {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding-left: 14px;
	}

	.stat-total .stat-label {
		flex: 1;
		color: var(--text-muted);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.stat-total .stat-value {
		font-size: 10px;
		color: var(--text-secondary);
	}
</style>

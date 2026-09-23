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

	const ringColor = $derived(diskColor(usedPercent, 'var(--text-secondary)'));
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
			<span class="stat-dot used"></span>
			<span class="stat-label">Used</span>
			<span class="stat-value">{formatSize(info.used_bytes)}</span>
		</div>
		<div class="stat-row">
			<span class="stat-dot free"></span>
			<span class="stat-label">Free</span>
			<span class="stat-value">{formatSize(info.free_bytes)}</span>
		</div>
		{#if scanStore.totalBytes > 0}
			<div class="stat-row recoverable">
				<span class="stat-dot sweepable"></span>
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
		padding: var(--space-base) 0;
		animation: fadeIn var(--duration-slow) var(--ease-out);
	}

	.gauge-ring {
		position: relative;
		flex-shrink: 0;
	}

	.used-arc {
		transition:
			stroke-dashoffset 0.8s var(--ease-out),
			stroke 0.4s;
	}

	.recoverable-arc {
		transition: stroke-dashoffset 0.8s var(--ease-out);
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
		font-size: 30px;
		font-weight: 300;
		letter-spacing: -0.02em;
		line-height: 1;
		color: var(--text-primary);
	}

	.gauge-label {
		font-size: var(--text-xs);
		font-weight: 500;
		color: var(--text-muted);
		margin-top: 4px;
	}

	.gauge-stats {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		min-width: 160px;
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
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.stat-dot.used {
		background: var(--text-secondary);
	}

	.stat-dot.free {
		background: var(--bg-active);
	}

	.stat-dot.sweepable {
		background: var(--accent);
	}

	.stat-label {
		flex: 1;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.stat-value {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--text-primary);
	}

	.stat-total {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding-left: 15px;
	}

	.stat-total .stat-label {
		flex: 1;
		color: var(--text-muted);
		font-size: var(--text-xs);
	}

	.stat-total .stat-value {
		font-size: var(--text-xs);
		font-weight: 500;
		color: var(--text-secondary);
	}
</style>

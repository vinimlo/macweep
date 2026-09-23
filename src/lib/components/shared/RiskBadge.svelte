<script lang="ts">
	import type { RiskLevel } from '$lib/tauri/types';
	import { riskColor, riskLabel } from '$lib/utils/risk';

	let { level }: { level: RiskLevel } = $props();
</script>

<span class="badge" style="--badge-color: {riskColor(level)}">
	<span class="dot"></span>
	{riskLabel(level)}
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 1px 8px 1px 7px;
		border-radius: var(--radius-round);
		font-size: var(--text-xs);
		font-weight: 600;
		white-space: nowrap;
		color: var(--badge-color);
		background: color-mix(in srgb, var(--badge-color) 12%, transparent);
	}

	.dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--badge-color);
		flex-shrink: 0;
	}
</style>

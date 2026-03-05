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
		padding: 2px 8px 2px 6px;
		border-radius: var(--radius-round);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.03em;
		text-transform: uppercase;
		color: var(--badge-color);
		background: color-mix(in srgb, var(--badge-color) 10%, transparent);
	}

	.dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--badge-color);
		flex-shrink: 0;
	}
</style>

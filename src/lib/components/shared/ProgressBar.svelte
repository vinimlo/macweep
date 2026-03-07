<script lang="ts">
	let { value, max = 100, color = 'var(--accent)' }: { value: number; max?: number; color?: string } = $props();

	const percent = $derived(max > 0 ? Math.min((value / max) * 100, 100) : 0);
</script>

<div class="track">
	<div
		class="fill"
		style="width: {percent}%; --bar-color: {color}"
		class:shimmer={percent > 0 && percent < 100}
	></div>
</div>

<style>
	.track {
		width: 100%;
		height: 3px;
		background: var(--bg-active);
		border-radius: var(--radius-round);
		overflow: hidden;
	}

	.fill {
		height: 100%;
		border-radius: var(--radius-round);
		background: var(--bar-color);
		transition: width 0.4s var(--ease-out);
		position: relative;
	}

	.fill.shimmer::after {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(
			90deg,
			transparent 0%,
			rgba(255, 255, 255, 0.15) 50%,
			transparent 100%
		);
		background-size: 200% 100%;
		animation: shimmer 1.8s linear infinite;
	}
</style>

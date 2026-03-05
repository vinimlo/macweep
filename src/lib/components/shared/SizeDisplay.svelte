<script lang="ts">
	import { formatSize } from '$lib/utils/format';

	let { bytes, size = 'md' }: { bytes: number; size?: 'sm' | 'md' | 'lg' } = $props();
</script>

<span class="size size-{size}">{formatSize(bytes)}</span>

<style>
	.size {
		font-family: var(--font-mono);
		font-weight: 600;
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}

	.size-sm {
		font-size: 10px;
	}

	.size-md {
		font-size: 12px;
	}

	.size-lg {
		font-size: 22px;
		font-weight: 200;
		letter-spacing: -0.02em;
	}
</style>

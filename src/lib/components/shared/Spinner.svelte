<script lang="ts">
	let { size = 20, color = 'var(--accent)' }: { size?: number; color?: string } = $props();
</script>

<div
	class="spinner"
	style="width: {size}px; height: {size}px; --spinner-color: {color}"
	role="status"
	aria-label="Loading"
></div>

<style>
	.spinner {
		border: 2px solid var(--border-default);
		border-top-color: var(--spinner-color);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
</style>

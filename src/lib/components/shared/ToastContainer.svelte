<script lang="ts">
	import { toastStore } from '$lib/stores/toasts.svelte';
	import Toast from './Toast.svelte';
</script>

{#if toastStore.toasts.length > 0}
	<div class="container">
		{#each toastStore.toasts as toast (toast.id)}
			<Toast type={toast.type} message={toast.message} onclose={() => toastStore.remove(toast.id)} />
		{/each}
	</div>
{/if}

<style>
	.container {
		position: fixed;
		bottom: 44px;
		right: var(--space-base);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		max-width: 340px;
		z-index: 950;
	}
</style>

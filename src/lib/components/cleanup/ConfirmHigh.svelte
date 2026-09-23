<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import { formatSize } from '$lib/utils/format';
	import TypeToConfirm from '$lib/components/shared/TypeToConfirm.svelte';

	let { items, onconfirm, onskip }: {
		items: ScanResult[];
		onconfirm: (selected: ScanResult[]) => void;
		onskip: () => void;
	} = $props();

	let confirmedIds = $state<Set<string>>(new Set());

	function markConfirmed(id: string) {
		confirmedIds = new Set([...confirmedIds, id]);
	}

	const confirmedItems = $derived(items.filter((i) => confirmedIds.has(i.id)));
</script>

<div class="confirm-step">
	<div class="warning-bar" role="note">
		<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
			<path d="M8 2L1.5 13.5h13L8 2z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
			<path d="M8 7v3M8 12v0" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
		</svg>
		<span>These may hold data you can't get back. Type each name to confirm it.</span>
	</div>

	<div class="items">
		{#each items as item (item.id)}
			<div class="item" class:confirmed={confirmedIds.has(item.id)}>
				<div class="item-header">
					<div class="item-title">
						<span class="item-label">{item.label}</span>
						<span class="confirm-size">{formatSize(item.size_bytes)}</span>
					</div>
					<span class="item-detail">{item.detail}</span>
					{#if item.warning}
						<span class="confirm-warning wrap">{item.warning}</span>
					{/if}
					<span class="confirm-path path selectable">{item.path}</span>
				</div>

				{#if !confirmedIds.has(item.id)}
					<div class="confirm-zone">
						<TypeToConfirm target={item.label} onconfirm={() => markConfirmed(item.id)} />
					</div>
				{:else}
					<div class="confirmed-badge">
						<svg width="12" height="12" viewBox="0 0 12 12" fill="none">
							<path d="M2 6l3 3 5-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
						</svg>
						Confirmed for deletion
					</div>
				{/if}
			</div>
		{/each}
	</div>

	<div class="confirm-actions">
		<button class="btn btn-secondary" onclick={onskip}>Skip</button>
		<button
			class="btn btn-danger"
			disabled={confirmedItems.length === 0}
			onclick={() => onconfirm(confirmedItems)}
		>
			Delete {confirmedItems.length} Item{confirmedItems.length !== 1 ? 's' : ''}
		</button>
	</div>
</div>

<style>
	.warning-bar {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-base);
		background: var(--risk-high-dim);
		border: 1px solid color-mix(in srgb, var(--risk-high) 25%, transparent);
		border-radius: var(--radius-md);
		font-size: var(--text-sm);
		color: var(--risk-high);
		line-height: 1.4;
	}

	.warning-bar svg {
		flex-shrink: 0;
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.item {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		padding: var(--space-base);
		background: var(--bg-raised);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		box-shadow: var(--highlight);
		transition: border-color var(--duration-base) var(--ease-out);
	}

	.item.confirmed {
		border-color: color-mix(in srgb, var(--risk-high) 40%, transparent);
	}

	.item-header {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.item-title {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-md);
	}

	.item-label {
		font-weight: 600;
		font-size: var(--text-base);
	}

	.item-detail {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.wrap {
		white-space: normal;
	}

	.path {
		word-break: break-all;
	}

	.confirm-zone {
		padding-top: var(--space-md);
		border-top: 1px solid var(--border-subtle);
	}

	.confirmed-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--risk-high);
		font-size: var(--text-sm);
		font-weight: 600;
		padding-top: var(--space-md);
		border-top: 1px solid var(--border-subtle);
		animation: fadeIn var(--duration-base) var(--ease-out);
	}
</style>

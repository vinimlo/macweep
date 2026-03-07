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

<div class="confirm">
	<div class="warning-bar">
		<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
			<path d="M8 2L1.5 13.5h13L8 2z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
			<path d="M8 7v3M8 12v0" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
		</svg>
		<span>May contain important state or data. Type the name to confirm each item.</span>
	</div>

	<div class="items">
		{#each items as item, i (item.id)}
			<div class="item" class:confirmed={confirmedIds.has(item.id)} style="animation-delay: {i * 40}ms">
				<div class="item-header">
					<div class="item-title">
						<span class="item-label">{item.label}</span>
						<span class="item-size">{formatSize(item.size_bytes)}</span>
					</div>
					<span class="item-detail">{item.detail}</span>
					<span class="item-path">{item.path}</span>
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

	<div class="actions">
		<button class="btn btn-skip" onclick={onskip}>Skip</button>
		<button
			class="btn btn-delete"
			disabled={confirmedItems.length === 0}
			onclick={() => onconfirm(confirmedItems)}
		>
			Delete {confirmedItems.length} Item{confirmedItems.length !== 1 ? 's' : ''}
		</button>
	</div>
</div>

<style>
	@import './confirm-shared.css';

	.warning-bar {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-md) var(--space-base);
		background: var(--risk-high-dim);
		border: 1px solid color-mix(in srgb, var(--risk-high) 25%, transparent);
		border-radius: var(--radius-md);
		font-size: 12px;
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
		transition: border-color var(--duration-base);
		animation: fadeIn var(--duration-base) var(--ease-out) both;
	}

	.item.confirmed {
		border-color: color-mix(in srgb, var(--risk-zero) 30%, transparent);
	}

	.item-header {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.item-title {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.item-label {
		font-weight: 600;
		font-size: 13px;
	}

	.item-size { font-size: 12px; }

	.item-detail {
		font-size: 11px;
		color: var(--text-secondary);
	}

	.item-path {
		font-size: 10px;
		color: var(--text-muted);
		font-family: var(--font-mono);
		word-break: break-all;
	}

	.confirm-zone {
		padding-top: var(--space-sm);
		border-top: 1px solid var(--border-subtle);
	}

	.confirmed-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--risk-zero);
		font-size: 11px;
		font-weight: 600;
		padding-top: var(--space-sm);
		border-top: 1px solid var(--border-subtle);
		animation: check-pop 0.3s var(--ease-spring);
	}

	.btn-delete {
		background: var(--risk-high);
		color: white;
	}

	.btn-delete:hover:not(:disabled) { opacity: 0.9; }
</style>

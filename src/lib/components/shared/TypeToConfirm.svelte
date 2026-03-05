<script lang="ts">
	let { target, onconfirm }: { target: string; onconfirm: () => void } = $props();
	let input = $state('');

	const matches = $derived(input === target);
</script>

<div class="type-confirm">
	<p class="instruction">
		Type <code>{target}</code> to confirm deletion
	</p>
	<div class="input-row">
		<input
			type="text"
			bind:value={input}
			placeholder={target}
			class="input"
			class:valid={matches}
			spellcheck="false"
			autocomplete="off"
		/>
		<button class="btn-confirm" disabled={!matches} onclick={onconfirm}>
			Confirm
		</button>
	</div>
</div>

<style>
	.type-confirm {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.instruction {
		font-size: 11px;
		color: var(--text-secondary);
	}

	code {
		font-family: var(--font-mono);
		font-size: 11px;
		background: var(--bg-overlay);
		padding: 1px 5px;
		border-radius: 3px;
		color: var(--risk-high);
		font-weight: 600;
	}

	.input-row {
		display: flex;
		gap: var(--space-sm);
	}

	.input {
		flex: 1;
		padding: 6px var(--space-md);
		background: var(--bg-base);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: 12px;
		outline: none;
		transition: border-color var(--duration-fast);
	}

	.input:focus {
		border-color: var(--border-hover);
	}

	.input.valid {
		border-color: var(--risk-zero);
	}

	.btn-confirm {
		padding: 6px var(--space-base);
		background: var(--risk-high);
		color: white;
		border-radius: var(--radius-sm);
		font-weight: 600;
		font-size: 11px;
		letter-spacing: 0.02em;
		transition: opacity var(--duration-fast);
		white-space: nowrap;
	}

	.btn-confirm:hover:not(:disabled) {
		opacity: 0.9;
	}
</style>

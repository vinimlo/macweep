<script lang="ts">
	let { target, onconfirm }: { target: string; onconfirm: () => void } = $props();
	let input = $state('');

	const matches = $derived(input === target);
</script>

<div class="type-confirm">
	<p class="instruction">
		Type <code class="target">{target}</code> to confirm
	</p>
	<div class="input-row">
		<input
			type="text"
			bind:value={input}
			placeholder={target}
			class="input"
			class:valid={matches}
			aria-label="Type {target} to confirm"
			spellcheck="false"
			autocomplete="off"
		/>
		<button class="btn btn-danger" disabled={!matches} onclick={onconfirm}>
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
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.target {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		background: var(--bg-overlay);
		padding: 1px 5px;
		border-radius: 4px;
		color: var(--text-primary);
		font-weight: 600;
	}

	.input-row {
		display: flex;
		gap: var(--space-sm);
	}

	.input {
		flex: 1;
		height: 30px;
		padding: 0 var(--space-md);
		background: var(--bg-base);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: var(--text-sm);
		outline: none;
		transition:
			border-color var(--duration-fast) var(--ease-out),
			box-shadow var(--duration-fast) var(--ease-out);
	}

	.input::placeholder {
		color: var(--text-muted);
		opacity: 0.6;
	}

	.input:focus {
		border-color: color-mix(in srgb, var(--accent) 60%, transparent);
		box-shadow: 0 0 0 3px var(--accent-glow);
	}

	.input.valid {
		border-color: color-mix(in srgb, var(--risk-high) 60%, transparent);
	}
</style>

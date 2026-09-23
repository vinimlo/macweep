<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { toastStore } from '$lib/stores/toasts.svelte';
	import { goto } from '$app/navigation';
	import ConfirmZero from './ConfirmZero.svelte';
	import ConfirmLow from './ConfirmLow.svelte';
	import ConfirmMedium from './ConfirmMedium.svelte';
	import ConfirmHigh from './ConfirmHigh.svelte';
	import CleanupProgress from './CleanupProgress.svelte';
	import CleanupReport from './CleanupReport.svelte';

	type RiskStep = 'zero' | 'low' | 'medium' | 'high';
	type Step = RiskStep | 'executing' | 'done';

	const stepLabels: Record<RiskStep, string> = {
		zero: 'Safe',
		low: 'Low',
		medium: 'Medium',
		high: 'High'
	};

	// The flow works on a snapshot of the selection taken when it opens. Nothing here
	// is reactive to later selection changes, so leaving the report (which resets the
	// selection) can never re-trigger a cleanup.
	const selection = [...cleanupStore.selectedItems];
	const itemsByStep: Record<RiskStep, ScanResult[]> = {
		zero: selection.filter((i) => i.risk_level === 'Zero'),
		low: selection.filter((i) => i.risk_level === 'Low'),
		medium: selection.filter((i) => i.risk_level === 'Medium'),
		high: selection.filter((i) => i.risk_level === 'High')
	};
	const steps = (['zero', 'low', 'medium', 'high'] as const).filter(
		(s) => itemsByStep[s].length > 0
	);

	let step = $state<Step>(steps[0] ?? 'done');
	const confirmed: ScanResult[] = [];
	let started = false;

	const currentStepIndex = $derived(steps.indexOf(step as RiskStep));

	function advance(from: RiskStep, items: ScanResult[]) {
		confirmed.push(...items);
		const next = steps[steps.indexOf(from) + 1];
		if (next) step = next;
		else execute();
	}

	async function execute() {
		if (started) return;
		started = true;
		if (confirmed.length === 0) {
			toastStore.info('Nothing was confirmed for cleanup');
			cleanupStore.reset();
			goto('/');
			return;
		}
		step = 'executing';
		await cleanupStore.run(confirmed);
		step = 'done';
	}
</script>

<div class="flow">
	{#if step !== 'executing' && step !== 'done' && steps.length > 1}
		<div class="stepper">
			{#each steps as s, i (s)}
				<div class="step" class:active={step === s} class:done={currentStepIndex > i}>
					<span class="step-dot"></span>
					<span class="step-label">{stepLabels[s]}</span>
				</div>
			{/each}
		</div>
	{/if}

	<div class="step-content">
		{#if step === 'zero'}
			<ConfirmZero
				items={itemsByStep.zero}
				onconfirm={() => advance('zero', itemsByStep.zero)}
				onskip={() => advance('zero', [])}
			/>
		{:else if step === 'low'}
			<ConfirmLow
				items={itemsByStep.low}
				onconfirm={(selected) => advance('low', selected)}
				onskip={() => advance('low', [])}
			/>
		{:else if step === 'medium'}
			<ConfirmMedium
				items={itemsByStep.medium}
				onconfirm={(selected) => advance('medium', selected)}
				onskip={() => advance('medium', [])}
			/>
		{:else if step === 'high'}
			<ConfirmHigh
				items={itemsByStep.high}
				onconfirm={(selected) => advance('high', selected)}
				onskip={() => advance('high', [])}
			/>
		{:else if step === 'executing'}
			<CleanupProgress />
		{:else if step === 'done'}
			<CleanupReport />
		{/if}
	</div>
</div>

<style>
	.flow {
		max-width: 560px;
		margin: 0 auto;
		width: 100%;
	}

	.stepper {
		display: flex;
		align-items: center;
		gap: var(--space-lg);
		justify-content: center;
		margin-bottom: var(--space-lg);
	}

	.step {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--text-muted);
		transition: color var(--duration-base) var(--ease-out);
	}

	.step.active {
		color: var(--text-primary);
	}

	.step.done {
		color: var(--text-secondary);
	}

	.step-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--bg-active);
		transition: background-color var(--duration-base) var(--ease-out);
	}

	.step.active .step-dot {
		background: var(--accent);
	}

	.step.done .step-dot {
		background: var(--risk-zero);
	}

	.step-label {
		font-size: var(--text-sm);
		font-weight: 500;
	}
</style>

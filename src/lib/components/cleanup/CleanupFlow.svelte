<script lang="ts">
	import type { ScanResult } from '$lib/tauri/types';
	import { cleanupStore } from '$lib/stores/cleanup.svelte';
	import { scanStore } from '$lib/stores/scan.svelte';
	import { toastStore } from '$lib/stores/toasts.svelte';
	import { cleanItems } from '$lib/tauri/commands';
	import { formatSize } from '$lib/utils/format';
	import { sanitizeError } from '$lib/utils/errors';
	import ConfirmZero from './ConfirmZero.svelte';
	import ConfirmLow from './ConfirmLow.svelte';
	import ConfirmMedium from './ConfirmMedium.svelte';
	import ConfirmHigh from './ConfirmHigh.svelte';
	import CleanupProgress from './CleanupProgress.svelte';
	import CleanupReport from './CleanupReport.svelte';

	type Step = 'zero' | 'low' | 'medium' | 'high' | 'executing' | 'done';
	let step = $state<Step>('zero');
	let itemsToClean = $state<ScanResult[]>([]);

	const steps: Step[] = ['zero', 'low', 'medium', 'high'];
	const zeroItems = $derived(cleanupStore.selectedItems.filter((i) => i.risk_level === 'Zero'));
	const lowItems = $derived(cleanupStore.selectedItems.filter((i) => i.risk_level === 'Low'));
	const mediumItems = $derived(cleanupStore.selectedItems.filter((i) => i.risk_level === 'Medium'));
	const highItems = $derived(cleanupStore.selectedItems.filter((i) => i.risk_level === 'High'));

	const currentStepIndex = $derived(steps.indexOf(step as typeof steps[number]));

	function addAndAdvance(items: ScanResult[], nextStep: Step) {
		itemsToClean = [...itemsToClean, ...items];
		advanceTo(nextStep);
	}

	function advanceTo(target: Step) {
		if (target === 'low' && lowItems.length === 0) target = 'medium';
		if (target === 'medium' && mediumItems.length === 0) target = 'high';
		if (target === 'high' && highItems.length === 0) target = 'executing';
		if (target === 'executing') {
			executeCleanup();
			return;
		}
		step = target;
	}

	async function executeCleanup() {
		if (itemsToClean.length === 0) {
			step = 'done';
			return;
		}
		step = 'executing';
		try {
			const results = await cleanItems(itemsToClean, (progress) => {
				cleanupStore.handleProgress(progress);
			});
			cleanupStore.setResults(results);
			toastStore.success(`Freed ${formatSize(cleanupStore.totalFreed)}`);
		} catch (e) {
			toastStore.error(`Cleanup failed: ${sanitizeError(e)}`);
		}
		step = 'done';
	}

	$effect(() => {
		if (zeroItems.length > 0) step = 'zero';
		else if (lowItems.length > 0) step = 'low';
		else if (mediumItems.length > 0) step = 'medium';
		else if (highItems.length > 0) step = 'high';
		else executeCleanup();
	});
</script>

<div class="flow">
	{#if step !== 'executing' && step !== 'done'}
		<div class="stepper">
			{#each steps as s, i}
				{@const hasItems = (s === 'zero' && zeroItems.length > 0) || (s === 'low' && lowItems.length > 0) || (s === 'medium' && mediumItems.length > 0) || (s === 'high' && highItems.length > 0)}
				{#if hasItems}
					<div class="step" class:active={step === s} class:done={currentStepIndex > i}>
						<span class="step-dot"></span>
						<span class="step-label">{s === 'zero' ? 'Safe' : s === 'low' ? 'Low' : s === 'medium' ? 'Medium' : 'High'}</span>
					</div>
				{/if}
			{/each}
		</div>
	{/if}

	<div class="step-content">
		{#if step === 'zero'}
			<ConfirmZero
				items={zeroItems}
				onconfirm={() => addAndAdvance(zeroItems, 'low')}
				onskip={() => advanceTo('low')}
			/>
		{:else if step === 'low'}
			<ConfirmLow
				items={lowItems}
				onconfirm={(selected) => addAndAdvance(selected, 'medium')}
				onskip={() => advanceTo('medium')}
			/>
		{:else if step === 'medium'}
			<ConfirmMedium
				items={mediumItems}
				onconfirm={(selected) => addAndAdvance(selected, 'high')}
				onskip={() => advanceTo('high')}
			/>
		{:else if step === 'high'}
			<ConfirmHigh
				items={highItems}
				onconfirm={(selected) => addAndAdvance(selected, 'executing')}
				onskip={() => advanceTo('executing')}
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
		gap: var(--space-xl);
		justify-content: center;
		margin-bottom: var(--space-xl);
	}

	.step {
		display: flex;
		align-items: center;
		gap: 6px;
		opacity: 0.35;
		transition: opacity var(--duration-base);
	}

	.step.active {
		opacity: 1;
	}

	.step.done {
		opacity: 0.6;
	}

	.step-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--text-muted);
		transition: all var(--duration-base);
	}

	.step.active .step-dot {
		background: var(--accent);
		box-shadow: 0 0 8px var(--accent-glow);
	}

	.step.done .step-dot {
		background: var(--risk-zero);
	}

	.step-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.step.active .step-label {
		color: var(--text-primary);
	}

	.step-content {
		animation: fadeIn var(--duration-base) var(--ease-out);
	}
</style>

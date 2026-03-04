<script lang="ts">
	import '../app.css';
	import { appStore } from '$lib/stores/app.svelte';
	import { scanStore } from '$lib/stores/scan.svelte';
	import { activityStore } from '$lib/stores/activity.svelte';
	import { getDiskInfo, getActivityLog } from '$lib/tauri/commands';
	import { formatSize, formatPercent } from '$lib/utils/format';
	import ToastContainer from '$lib/components/shared/ToastContainer.svelte';
	import ActivityDrawer from '$lib/components/shared/ActivityDrawer.svelte';
	import FloatingCleanBar from '$lib/components/shared/FloatingCleanBar.svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/state';

	let { children } = $props();

	let unlistenActivity: (() => void) | undefined;
	let historyLoaded = false;

	onMount(() => {
		getDiskInfo()
			.then((info) => { appStore.diskInfo = info; })
			.catch(() => {});

		getActivityLog(200)
			.then((entries) => {
				activityStore.loadHistory(entries);
				historyLoaded = true;
			})
			.catch(() => { historyLoaded = true; });

		import('@tauri-apps/api/event')
			.then(({ listen }) =>
				listen('activity-log', (event: any) => {
					if (!historyLoaded) return;
					activityStore.addEntry(event.payload);
				})
			)
			.then((fn) => { unlistenActivity = fn; })
			.catch(() => {});

		return () => {
			unlistenActivity?.();
		};
	});

	const navItems = [
		{ href: '/', label: 'Dashboard', icon: 'grid' },
		{ href: '/history', label: 'History', icon: 'clock' },
		{ href: '/settings', label: 'Settings', icon: 'sliders' }
	];

	const diskPercent = $derived(
		appStore.diskInfo && appStore.diskInfo.total_bytes > 0
			? (appStore.diskInfo.used_bytes / appStore.diskInfo.total_bytes) * 100
			: 0
	);
</script>

<div class="app">
	<header class="chrome" data-tauri-drag-region>
		<div class="chrome-left" data-tauri-drag-region>
			<svg class="logo-mark" width="20" height="20" viewBox="0 0 512 512" fill="none">
				<defs>
					<linearGradient id="lm-g" x1="0.05" y1="0.68" x2="0.92" y2="0.32">
						<stop offset="0%" stop-color="#0e7490"/>
						<stop offset="50%" stop-color="#22d3ee"/>
						<stop offset="100%" stop-color="#a5f3fc"/>
					</linearGradient>
				</defs>
				<path d="M 66,348 C 106,336 170,292 240,250 C 310,208 378,186 448,178 C 378,200 310,252 240,308 C 170,364 106,374 66,362 Z" fill="url(#lm-g)"/>
				<path d="M 48,310 C 100,288 175,250 260,218 C 345,186 408,170 452,164" stroke="url(#lm-g)" stroke-width="5" stroke-linecap="round" opacity="0.4"/>
				<path d="M 54,378 C 118,366 194,342 276,318 C 358,294 416,280 458,274" stroke="url(#lm-g)" stroke-width="4" stroke-linecap="round" opacity="0.3"/>
			</svg>
			<span class="wordmark">macweep</span>
		</div>

		<nav class="chrome-nav">
			{#each navItems as item (item.href)}
				<a
					href={item.href}
					class="nav-item"
					class:active={page.url.pathname === item.href || (item.href !== '/' && page.url.pathname.startsWith(item.href))}
				>
					{item.label}
				</a>
			{/each}
		</nav>

		<div class="chrome-right">
			{#if appStore.diskInfo}
				<div class="disk-pill">
					<svg class="disk-ring" width="18" height="18" viewBox="0 0 18 18">
						<circle cx="9" cy="9" r="7" fill="none" stroke="var(--bg-active)" stroke-width="2" />
						<circle
							cx="9" cy="9" r="7"
							fill="none"
							stroke={diskPercent > 85 ? 'var(--risk-high)' : diskPercent > 70 ? 'var(--risk-medium)' : 'var(--accent)'}
							stroke-width="2"
							stroke-dasharray={2 * Math.PI * 7}
							stroke-dashoffset={2 * Math.PI * 7 * (1 - diskPercent / 100)}
							stroke-linecap="round"
							transform="rotate(-90 9 9)"
							style="transition: stroke-dashoffset 0.6s var(--ease-out)"
						/>
					</svg>
					<span class="disk-text">{formatPercent(diskPercent)}</span>
				</div>
			{/if}

			{#if scanStore.status === 'scanning'}
				<div class="status-beacon scanning">
					<span class="beacon-dot"></span>
				</div>
			{:else if scanStore.status === 'completed'}
				<div class="status-beacon completed">
					<span class="beacon-dot"></span>
				</div>
			{/if}
		</div>
	</header>

	<main class="viewport">
		{@render children()}
	</main>
	<ActivityDrawer />
	<FloatingCleanBar />
</div>

<ToastContainer />

<style>
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--bg-base);
	}

	.chrome {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 var(--space-base);
		height: 44px;
		background: var(--bg-raised);
		border-bottom: 1px solid var(--border-subtle);
		-webkit-app-region: drag;
		flex-shrink: 0;
	}

	.chrome-left {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 140px;
	}

	.logo-mark {
		flex-shrink: 0;
		-webkit-app-region: no-drag;
	}

	.wordmark {
		font-weight: 700;
		font-size: 14px;
		letter-spacing: -0.03em;
		color: var(--accent);
		-webkit-app-region: no-drag;
	}

	.chrome-nav {
		display: flex;
		align-items: center;
		gap: 2px;
		-webkit-app-region: no-drag;
		background: var(--bg-base);
		border-radius: var(--radius-md);
		padding: 3px;
	}

	.nav-item {
		padding: 4px 14px;
		border-radius: 6px;
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0.01em;
		color: var(--text-secondary);
		transition: all var(--duration-fast) var(--ease-out);
		-webkit-app-region: no-drag;
	}

	.nav-item:hover {
		color: var(--text-primary);
		background: var(--bg-surface);
	}

	.nav-item.active {
		color: var(--text-primary);
		background: var(--bg-surface);
		box-shadow: var(--shadow-sm);
	}

	.chrome-right {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		min-width: 140px;
		justify-content: flex-end;
		-webkit-app-region: no-drag;
	}

	.disk-pill {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.disk-ring {
		flex-shrink: 0;
	}

	.disk-text {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		color: var(--text-secondary);
		font-variant-numeric: tabular-nums;
	}

	.status-beacon {
		position: relative;
		width: 8px;
		height: 8px;
	}

	.beacon-dot {
		display: block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}

	.status-beacon.scanning .beacon-dot {
		background: var(--accent);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.status-beacon.completed .beacon-dot {
		background: var(--risk-zero);
	}

	.viewport {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		overflow-x: hidden;
		padding: var(--space-lg) var(--space-xl);
		-webkit-app-region: no-drag;
	}
</style>

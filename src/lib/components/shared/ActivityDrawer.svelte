<script lang="ts">
	import { activityStore, type ActivityFilter } from '$lib/stores/activity.svelte';
	import { formatRelativeTime, formatTimeShort } from '$lib/utils/format';
	import { tick } from 'svelte';

	let logContainer: HTMLDivElement | undefined = $state();
	let isAtBottom = $state(true);
	let isDragging = $state(false);
	let dragStartY = 0;
	let dragStartHeight = 0;

	const filters: { label: string; value: ActivityFilter; dot?: string }[] = [
		{ label: 'All', value: 'all' },
		{ label: 'Commands', value: 'commands', dot: 'var(--accent)' },
		{ label: 'Errors', value: 'errors', dot: 'var(--risk-high)' },
		{ label: 'Success', value: 'success', dot: 'var(--risk-zero)' }
	];

	function levelColor(level: string): string {
		switch (level) {
			case 'Command': return 'var(--accent)';
			case 'Error': return 'var(--risk-high)';
			case 'Warning': return 'var(--risk-medium)';
			case 'Success': return 'var(--risk-zero)';
			default: return 'var(--bg-active)';
		}
	}

	function levelIcon(level: string): string {
		switch (level) {
			case 'Command': return '>';
			case 'Error': return '!';
			case 'Warning': return '~';
			case 'Success': return '\u2713';
			default: return '\u00b7';
		}
	}

	function handleScroll() {
		if (!logContainer) return;
		const { scrollTop, scrollHeight, clientHeight } = logContainer;
		isAtBottom = scrollHeight - scrollTop - clientHeight < 30;
	}

	async function scrollToBottom() {
		await tick();
		if (logContainer) {
			logContainer.scrollTop = logContainer.scrollHeight;
			isAtBottom = true;
		}
	}

	function startDrag(e: MouseEvent) {
		isDragging = true;
		dragStartY = e.clientY;
		dragStartHeight = activityStore.drawerHeight;
		e.preventDefault();
	}

	function onMouseMove(e: MouseEvent) {
		if (!isDragging) return;
		const delta = dragStartY - e.clientY;
		activityStore.setHeight(dragStartHeight + delta);
	}

	function onMouseUp() {
		isDragging = false;
	}

	$effect(() => {
		const _len = activityStore.filteredEntries.length;
		if (isAtBottom && activityStore.isExpanded && logContainer) {
			scrollToBottom();
		}
	});

</script>

{#if isDragging}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="drag-overlay" onmousemove={onMouseMove} onmouseup={onMouseUp}></div>
{/if}

<div class="activity-drawer" class:expanded={activityStore.isExpanded} class:dragging={isDragging}>
	{#if activityStore.isExpanded}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="drag-handle" onmousedown={startDrag}>
			<div class="drag-notch"></div>
		</div>
	{/if}

	<button
		class="drawer-bar"
		onclick={() => activityStore.toggleExpanded()}
	>
		<div class="bar-left">
			<div class="bar-indicator" class:has-errors={activityStore.errorCount > 0}>
				<svg width="13" height="13" viewBox="0 0 16 16" fill="none">
					<path d="M2 3l5 5-5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
					<path d="M9 13h5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
				</svg>
			</div>
			<span class="bar-label">Activity</span>
			{#if activityStore.errorCount > 0 && !activityStore.isExpanded}
				<span class="error-pip">{activityStore.errorCount}</span>
			{/if}
		</div>

		{#if activityStore.latestEntry && !activityStore.isExpanded}
			<div class="bar-preview">
				<span class="bar-preview-dot" style="background: {levelColor(activityStore.latestEntry.level)}"></span>
				<span class="bar-preview-msg">{activityStore.latestEntry.message}</span>
				<span class="bar-preview-time">{formatRelativeTime(activityStore.latestEntry.timestamp)}</span>
			</div>
		{/if}

		<div class="bar-right">
			{#if activityStore.entries.length > 0}
				<span class="entry-badge">{activityStore.filteredEntries.length}</span>
			{/if}
			<svg
				class="chevron"
				class:flipped={activityStore.isExpanded}
				width="12" height="12" viewBox="0 0 12 12" fill="none"
			>
				<path d="M3 5l3-3 3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</div>
	</button>

	{#if activityStore.isExpanded}
		<div class="toolbar">
			{#each filters as f (f.value)}
				<button
					class="filter-pill"
					class:active={activityStore.filter === f.value}
					onclick={() => { activityStore.filter = f.value; }}
				>
					{#if f.dot}
						<span class="pill-dot" style="background: {f.dot}"></span>
					{/if}
					{f.label}
				</button>
			{/each}

			<div class="toolbar-divider"></div>

			<button class="toolbar-action" onclick={() => activityStore.clear()} title="Clear log">
				<svg width="11" height="11" viewBox="0 0 16 16" fill="none">
					<path d="M2 4h12M5 4V3a1 1 0 011-1h4a1 1 0 011 1v1M6 7v5M10 7v5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
					<path d="M3 4l1 9a2 2 0 002 2h4a2 2 0 002-2l1-9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</button>
		</div>
	{/if}

	{#if activityStore.isExpanded}
		<div
			class="log-area"
			style="height: {activityStore.drawerHeight - 32}px"
			bind:this={logContainer}
			onscroll={handleScroll}
		>
			{#each activityStore.filteredEntries as entry, i (i + ':' + entry.timestamp)}
				<div class="log-entry" style="--entry-color: {levelColor(entry.level)}">
					<div class="entry-gutter">
						<span class="entry-icon">{levelIcon(entry.level)}</span>
					</div>

					<div class="entry-body">
						<div class="entry-main">
							{#if entry.level === 'Command' && entry.command}
								<span class="entry-command selectable">$ {entry.command}</span>
							{:else}
								<span class="entry-message selectable">{entry.message}</span>
							{/if}

							{#if entry.detail}
								<span class="entry-detail selectable" title={entry.detail}>{entry.detail}</span>
							{/if}
						</div>

						<div class="entry-meta">
							{#if entry.category}
								<span class="entry-category">{entry.category}</span>
							{/if}
							{#if entry.duration_ms != null}
								<span class="entry-duration">{entry.duration_ms}ms</span>
							{/if}
							<span class="entry-time" title={new Date(entry.timestamp).toLocaleString()}>{formatTimeShort(entry.timestamp)}</span>
						</div>
					</div>
				</div>
			{/each}

			{#if activityStore.filteredEntries.length === 0}
				<div class="log-empty">
					{#if activityStore.filter !== 'all' || activityStore.searchQuery}
						<span class="empty-icon">
							<svg width="18" height="18" viewBox="0 0 18 18" fill="none">
								<circle cx="8" cy="8" r="5.5" stroke="var(--text-muted)" stroke-width="1.3"/>
								<path d="M12 12l3.5 3.5" stroke="var(--text-muted)" stroke-width="1.3" stroke-linecap="round"/>
							</svg>
						</span>
						<span>No matching entries</span>
					{:else}
						<span class="empty-icon">
							<svg width="18" height="18" viewBox="0 0 18 18" fill="none">
								<circle cx="9" cy="9" r="6.5" stroke="var(--text-muted)" stroke-width="1.3" stroke-dasharray="2.5 2.5"/>
								<path d="M9 6v3l2 1" stroke="var(--text-muted)" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
							</svg>
						</span>
						<span>Activity will appear here as you scan and clean</span>
					{/if}
				</div>
			{/if}
		</div>

		{#if !isAtBottom}
			<button class="jump-latest" onclick={scrollToBottom}>
				<svg width="10" height="10" viewBox="0 0 12 12" fill="none">
					<path d="M3 5l3 3 3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
				Latest
			</button>
		{/if}
	{/if}
</div>

<style>
	.drag-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999;
		cursor: ns-resize;
	}

	.activity-drawer {
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		background: var(--bg-raised);
		border-top: 1px solid var(--border-subtle);
		position: relative;
		-webkit-app-region: no-drag;
	}

	.activity-drawer.dragging {
		user-select: none;
	}

	/* ── Drag handle ── */

	.drag-handle {
		height: 6px;
		cursor: ns-resize;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.drag-notch {
		width: 32px;
		height: 3px;
		border-radius: var(--radius-round);
		background: var(--border-subtle);
		transition: background var(--duration-fast) var(--ease-out);
	}

	.drag-handle:hover .drag-notch {
		background: var(--border-hover);
	}

	/* ── Top bar ── */

	.drawer-bar {
		display: flex;
		align-items: center;
		height: 28px;
		padding: 0 var(--space-sm);
		gap: var(--space-sm);
		cursor: pointer;
		border: none;
		background: none;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		font-size: var(--text-xs);
		width: 100%;
		text-align: left;
		flex-shrink: 0;
	}

	.drawer-bar:hover {
		background: var(--bg-hover);
	}

	.bar-left {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.bar-indicator {
		color: var(--text-muted);
		display: flex;
		align-items: center;
	}

	.bar-indicator.has-errors {
		color: var(--risk-high);
	}

	.bar-label {
		font-weight: 600;
		color: var(--text-secondary);
		letter-spacing: -0.01em;
	}

	.error-pip {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		font-weight: 700;
		color: var(--risk-high);
		background: var(--risk-high-dim);
		padding: 0 5px;
		border-radius: var(--radius-round);
		line-height: 16px;
	}

	/* ── Collapsed preview ── */

	.bar-preview {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 6px;
		overflow: hidden;
	}

	.bar-preview-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.bar-preview-msg {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.bar-preview-time {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		color: var(--text-muted);
		flex-shrink: 0;
		margin-left: auto;
	}

	/* ── Toolbar ── */

	.toolbar {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 0 var(--space-sm);
		flex-shrink: 0;
	}

	.filter-pill {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border: none;
		background: none;
		color: var(--text-muted);
		font-size: var(--text-2xs);
		font-family: var(--font-sans);
		border-radius: var(--radius-round);
		cursor: pointer;
		transition:
			color var(--duration-fast) var(--ease-out),
			background-color var(--duration-fast) var(--ease-out),
			border-color var(--duration-fast) var(--ease-out);
		white-space: nowrap;
	}

	.filter-pill:hover {
		color: var(--text-secondary);
		background: var(--bg-surface);
	}

	.filter-pill.active {
		color: var(--text-primary);
		background: var(--bg-active);
	}

	.pill-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.toolbar-divider {
		width: 1px;
		height: 12px;
		background: var(--border-subtle);
		margin: 0 4px;
	}

	.toolbar-action {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border-radius: var(--radius-sm);
		color: var(--text-muted);
		cursor: pointer;
		transition:
			color var(--duration-fast) var(--ease-out),
			background-color var(--duration-fast) var(--ease-out),
			border-color var(--duration-fast) var(--ease-out);
	}

	.toolbar-action:hover {
		color: var(--text-secondary);
		background: var(--bg-surface);
	}

	/* ── Right side ── */

	.bar-right {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		flex-shrink: 0;
	}

	.entry-badge {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		font-weight: 600;
		color: var(--text-muted);
		background: var(--bg-active);
		padding: 1px 5px;
		border-radius: var(--radius-round);
		font-variant-numeric: tabular-nums;
	}

	.chevron {
		color: var(--text-muted);
		transition: transform var(--duration-fast) var(--ease-out);
	}

	.chevron.flipped {
		transform: rotate(180deg);
	}

	/* ── Log area ── */

	.log-area {
		overflow-y: auto;
		overflow-x: hidden;
		padding: var(--space-2xs) 0;
	}

	/* ── Log entries ── */

	.log-entry {
		display: flex;
		gap: 0;
		padding: 3px var(--space-sm);
		border-left: 2px solid var(--entry-color);
		transition: background-color var(--duration-fast);
		animation: entrySlide var(--duration-base) var(--ease-out);
	}

	.log-entry:hover {
		background: var(--bg-surface);
	}

	@keyframes entrySlide {
		from {
			opacity: 0;
			transform: translateX(-4px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	.entry-gutter {
		width: 20px;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		flex-shrink: 0;
		padding-top: 1px;
	}

	.entry-icon {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		font-weight: 700;
		color: var(--entry-color);
		line-height: 1;
	}

	.entry-body {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
	}

	.entry-main {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: baseline;
		gap: var(--space-xs);
	}

	.entry-message {
		font-family: var(--font-sans);
		font-size: var(--text-xs);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		line-height: 1.5;
	}

	.entry-command {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		color: var(--accent-dim);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		line-height: 1.5;
	}

	.entry-detail {
		font-family: var(--font-sans);
		font-size: var(--text-2xs);
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.entry-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		margin-left: auto;
	}

	.entry-category {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		color: var(--text-muted);
		background: var(--bg-overlay);
		padding: 1px 5px;
		border-radius: 3px;
		white-space: nowrap;
	}

	.entry-duration {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.entry-time {
		font-size: var(--text-2xs);
		color: var(--text-muted);
		white-space: nowrap;
	}

	/* ── Empty state ── */

	.log-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		padding: var(--space-xl);
		color: var(--text-muted);
		font-size: var(--text-xs);
		font-family: var(--font-sans);
	}

	.empty-icon {
		display: flex;
		align-items: center;
		opacity: 0.6;
	}

	/* ── Jump to latest ── */

	.jump-latest {
		position: absolute;
		bottom: 36px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 12px;
		border: 1px solid var(--border-default);
		background: var(--bg-overlay);
		color: var(--text-secondary);
		font-size: var(--text-2xs);
		font-family: var(--font-sans);
		font-weight: 500;
		border-radius: var(--radius-round);
		cursor: pointer;
		z-index: 1;
		box-shadow: var(--shadow-md);
		transition:
			color var(--duration-fast) var(--ease-out),
			background-color var(--duration-fast) var(--ease-out),
			border-color var(--duration-fast) var(--ease-out);
		animation: fadeIn var(--duration-fast) var(--ease-out);
	}

	.jump-latest:hover {
		background: var(--bg-active);
		color: var(--text-primary);
		border-color: var(--border-hover);
	}
</style>

<script lang="ts">
	import { onMount } from 'svelte';
	import { getAuditLog } from '$lib/tauri/commands';
	import type { AuditEntry } from '$lib/tauri/types';
	import { formatSize, formatRelativeTime, formatTimeShort, groupByDate } from '$lib/utils/format';
	import Spinner from '$lib/components/shared/Spinner.svelte';

	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let expandedEntry = $state<string | null>(null);
	let categoryFilter = $state<string | null>(null);

	onMount(async () => {
		try {
			entries = await getAuditLog(100);
		} catch {
			// Outside Tauri
		}
		loading = false;
	});

	const totalFreed = $derived(
		entries.filter((e) => e.success).reduce((sum, e) => sum + e.freed_bytes, 0)
	);

	const successCount = $derived(entries.filter((e) => e.success).length);
	const failedCount = $derived(entries.filter((e) => !e.success).length);

	const categories = $derived(
		[...new Set(entries.map((e) => e.category))].sort()
	);

	const filteredEntries = $derived(
		categoryFilter ? entries.filter((e) => e.category === categoryFilter) : entries
	);

	const groupedEntries = $derived(
		groupByDate(filteredEntries.toReversed())
	);

	function toggleEntry(id: string) {
		expandedEntry = expandedEntry === id ? null : id;
	}

	function entryId(entry: AuditEntry): string {
		return entry.timestamp + entry.path;
	}
</script>

<div class="history-page">
	<!-- Summary strip -->
	{#if entries.length > 0}
		<div class="summary-strip">
			<div class="stat-card stat-freed">
				<div class="stat-value">{formatSize(totalFreed)}</div>
				<div class="stat-label">Total freed</div>
			</div>
			<div class="stat-card stat-ops">
				<div class="stat-value">{successCount}</div>
				<div class="stat-label">{successCount === 1 ? 'cleanup' : 'cleanups'}</div>
			</div>
			{#if failedCount > 0}
				<div class="stat-card stat-failed">
					<div class="stat-value">{failedCount}</div>
					<div class="stat-label">{failedCount === 1 ? 'failure' : 'failures'}</div>
				</div>
			{/if}
			<div class="stat-card stat-rate">
				<div class="stat-value">{entries.length > 0 ? Math.round((successCount / entries.length) * 100) : 0}%</div>
				<div class="stat-label">Success rate</div>
			</div>
		</div>
	{/if}

	<!-- Header with filters -->
	<div class="page-header">
		<h2>Cleanup History</h2>
		{#if categories.length > 1}
			<div class="category-filters">
				<button
					class="cat-chip"
					class:active={categoryFilter === null}
					onclick={() => categoryFilter = null}
				>All</button>
				{#each categories as cat (cat)}
					<button
						class="cat-chip"
						class:active={categoryFilter === cat}
						onclick={() => categoryFilter = categoryFilter === cat ? null : cat}
					>{cat}</button>
				{/each}
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="loading">
			<Spinner size={20} />
		</div>
	{:else if entries.length === 0}
		<div class="empty">
			<div class="empty-visual">
				<svg width="48" height="48" viewBox="0 0 48 48" fill="none">
					<circle cx="24" cy="24" r="18" stroke="var(--border-default)" stroke-width="1.5" stroke-dasharray="4 4"/>
					<path d="M24 14v10l6 3" stroke="var(--text-muted)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
				</svg>
			</div>
			<p class="empty-title">No cleanup history yet</p>
			<p class="empty-detail">Run a scan and clean up some items to see your history here.</p>
		</div>
	{:else}
		<div class="timeline">
			{#each groupedEntries as group (group.label)}
				<div class="date-group">
					<div class="date-header">
						<span class="date-line"></span>
						<span class="date-label">{group.label}</span>
						<span class="date-line"></span>
					</div>

					<div class="entries">
						{#each group.items as entry, i (entryId(entry))}
							{@const isExpanded = expandedEntry === entryId(entry)}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="entry"
								class:failed={!entry.success}
								class:expanded={isExpanded}
								style="animation-delay: {i * 25}ms"
								onclick={() => toggleEntry(entryId(entry))}
								onkeydown={(e) => { if (e.key === 'Enter') toggleEntry(entryId(entry)); }}
								role="button"
								tabindex="0"
							>
								<div class="entry-indicator">
									{#if entry.success}
										<div class="indicator-dot success"></div>
									{:else}
										<div class="indicator-dot failed"></div>
									{/if}
									{#if i < group.items.length - 1}
										<div class="indicator-line"></div>
									{/if}
								</div>

								<div class="entry-content">
									<div class="entry-top">
										<div class="entry-info">
											<span class="entry-label">{entry.label}</span>
											<span class="entry-cat">{entry.category}</span>
										</div>
										<div class="entry-value">
											{#if entry.success}
												<span class="freed-badge">-{formatSize(entry.freed_bytes)}</span>
											{:else}
												<span class="error-badge">failed</span>
											{/if}
										</div>
									</div>

									<div class="entry-bottom">
										<span class="entry-time" title={new Date(entry.timestamp).toLocaleString()}>
											{formatRelativeTime(entry.timestamp)}
										</span>
										{#if entry.action}
											<span class="entry-action">{entry.action}</span>
										{/if}
									</div>

									{#if isExpanded}
										<div class="entry-details">
											<div class="detail-row">
												<span class="detail-key">Path</span>
												<span class="detail-val mono">{entry.path}</span>
											</div>
											{#if entry.success}
												<div class="detail-row">
													<span class="detail-key">Freed</span>
													<span class="detail-val mono">{formatSize(entry.freed_bytes)} ({entry.freed_bytes.toLocaleString()} bytes)</span>
												</div>
												<div class="detail-row">
													<span class="detail-key">Original</span>
													<span class="detail-val mono">{formatSize(entry.size_bytes)}</span>
												</div>
											{:else if entry.error}
												<div class="detail-row">
													<span class="detail-key">Error</span>
													<span class="detail-val error">{entry.error}</span>
												</div>
											{/if}
											<div class="detail-row">
												<span class="detail-key">When</span>
												<span class="detail-val">{new Date(entry.timestamp).toLocaleString()}</span>
											</div>
										</div>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.history-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		max-width: 640px;
		margin: 0 auto;
		width: 100%;
		padding-bottom: var(--space-2xl);
	}

	/* ── Summary strip ── */

	.summary-strip {
		display: flex;
		gap: 1px;
		background: var(--border-subtle);
		border-radius: var(--radius-lg);
		overflow: hidden;
		animation: slideUp var(--duration-base) var(--ease-out);
	}

	.stat-card {
		flex: 1;
		padding: var(--space-base) var(--space-md);
		background: var(--bg-raised);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}

	.stat-value {
		font-family: var(--font-mono);
		font-size: 16px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.02em;
	}

	.stat-freed .stat-value { color: var(--risk-zero); }
	.stat-ops .stat-value { color: var(--text-primary); }
	.stat-failed .stat-value { color: var(--risk-high); }
	.stat-rate .stat-value { color: var(--accent); }

	.stat-label {
		font-size: 10px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		font-weight: 500;
	}

	/* ── Page header ── */

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
	}

	h2 {
		font-size: 16px;
		font-weight: 600;
		letter-spacing: -0.01em;
		flex-shrink: 0;
	}

	.category-filters {
		display: flex;
		gap: 3px;
		flex-wrap: wrap;
		justify-content: flex-end;
	}

	.cat-chip {
		padding: 3px 10px;
		font-size: 10px;
		font-weight: 500;
		color: var(--text-muted);
		background: none;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-round);
		cursor: pointer;
		transition: all var(--duration-fast) var(--ease-out);
		white-space: nowrap;
	}

	.cat-chip:hover {
		color: var(--text-secondary);
		border-color: var(--border-default);
	}

	.cat-chip.active {
		color: var(--text-primary);
		background: var(--bg-active);
		border-color: var(--border-hover);
	}

	/* ── Loading / Empty ── */

	.loading {
		display: flex;
		justify-content: center;
		padding: var(--space-2xl);
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-3xl) var(--space-lg);
		text-align: center;
		animation: fadeIn var(--duration-slow) var(--ease-out);
	}

	.empty-visual {
		margin-bottom: var(--space-lg);
		opacity: 0.4;
	}

	.empty-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-secondary);
		margin-bottom: var(--space-xs);
	}

	.empty-detail {
		font-size: 12px;
		color: var(--text-muted);
		max-width: 280px;
		line-height: 1.5;
	}

	/* ── Timeline ── */

	.timeline {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
	}

	.date-group {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.date-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: 0 var(--space-xs);
		margin-bottom: var(--space-sm);
	}

	.date-line {
		flex: 1;
		height: 1px;
		background: var(--border-subtle);
	}

	.date-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-muted);
		white-space: nowrap;
	}

	.entries {
		display: flex;
		flex-direction: column;
	}

	/* ── Entry ── */

	.entry {
		display: flex;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-sm);
		cursor: pointer;
		border-radius: var(--radius-md);
		transition: background var(--duration-fast);
		animation: fadeIn var(--duration-base) var(--ease-out) both;
	}

	.entry:hover {
		background: var(--bg-surface);
	}

	.entry.expanded {
		background: var(--bg-raised);
	}

	/* ── Indicator (timeline dot + line) ── */

	.entry-indicator {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 12px;
		flex-shrink: 0;
		padding-top: 5px;
	}

	.indicator-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
		position: relative;
	}

	.indicator-dot.success {
		background: var(--risk-zero);
		box-shadow: 0 0 6px var(--risk-zero-dim);
	}

	.indicator-dot.failed {
		background: var(--risk-high);
		box-shadow: 0 0 6px var(--risk-high-dim);
	}

	.indicator-line {
		width: 1px;
		flex: 1;
		min-height: 12px;
		background: var(--border-subtle);
		margin-top: 4px;
	}

	/* ── Entry content ── */

	.entry-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.entry-top {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		justify-content: space-between;
	}

	.entry-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		min-width: 0;
	}

	.entry-label {
		font-weight: 600;
		font-size: 12px;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.entry-cat {
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--text-muted);
		background: var(--bg-overlay);
		padding: 1px 6px;
		border-radius: 3px;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.entry-value {
		flex-shrink: 0;
	}

	.freed-badge {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		color: var(--risk-zero);
		font-variant-numeric: tabular-nums;
	}

	.error-badge {
		font-size: 10px;
		font-weight: 600;
		color: var(--risk-high);
		background: var(--risk-high-dim);
		padding: 1px 6px;
		border-radius: var(--radius-sm);
	}

	.entry-bottom {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.entry-time {
		font-size: 10px;
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
	}

	.entry-action {
		font-size: 10px;
		color: var(--text-muted);
	}

	/* ── Expanded details ── */

	.entry-details {
		margin-top: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-base);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-subtle);
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		animation: scaleIn var(--duration-fast) var(--ease-out);
	}

	.detail-row {
		display: flex;
		gap: var(--space-md);
		align-items: baseline;
	}

	.detail-key {
		font-size: 10px;
		font-weight: 600;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		width: 56px;
		flex-shrink: 0;
	}

	.detail-val {
		font-size: 11px;
		color: var(--text-secondary);
		word-break: break-all;
	}

	.detail-val.mono {
		font-family: var(--font-mono);
		font-size: 10px;
	}

	.detail-val.error {
		color: var(--risk-high);
	}
</style>

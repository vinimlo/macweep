<script lang="ts">
	const version = __APP_VERSION__;
	const protectedPaths = [
		'~/Documents/', '~/Desktop/', '~/Downloads/', '~/Pictures/', '~/Photos/',
		'~/Library/Keychains/', '~/Library/Application Support/MobileSync/',
		'~/.ssh/', '~/.gnupg/', '~/.gitconfig', '~/.zshrc', '~/.bashrc'
	];
</script>

<div class="settings-page">
	<h2>Settings</h2>

	<section>
		<h3 class="section-label">Protected Paths</h3>
		<p class="section-desc">These paths are never offered for cleanup.</p>
		<div class="protected-list">
			{#each protectedPaths as path (path)}
				<div class="protected-item">
					<svg width="10" height="10" viewBox="0 0 10 10" fill="none">
						<rect x="2" y="4" width="6" height="5" rx="1" stroke="var(--text-muted)" stroke-width="1"/>
						<path d="M3.5 4V3a1.5 1.5 0 013 0v1" stroke="var(--text-muted)" stroke-width="1"/>
					</svg>
					<code>{path}</code>
				</div>
			{/each}
		</div>
	</section>

	<section>
		<h3 class="section-label">About</h3>
		<div class="about">
			<div class="about-row">
				<span class="about-key">Version</span>
				<span class="about-val">{version}</span>
			</div>
			<div class="about-row">
				<span class="about-key">Audit Log</span>
				<code class="about-val">~/.storage-cleanup/audit.jsonl</code>
			</div>
			<p class="about-desc">macOS storage cleanup tool for developers</p>
		</div>
	</section>
</div>

<style>
	.settings-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
		max-width: 520px;
		margin: 0 auto;
		width: 100%;
	}

	h2 {
		font-size: var(--text-xl);
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	section {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.section-label {
		font-weight: 600;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.section-desc {
		font-size: var(--text-sm);
		color: var(--text-muted);
	}

	.protected-list {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-raised);
		overflow: hidden;
	}

	.protected-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 6px var(--space-md);
	}

	.protected-item + .protected-item {
		border-top: 1px solid var(--border-subtle);
	}

	code {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}

	.about {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-md);
		background: var(--bg-raised);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
	}

	.about-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.about-key {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.about-val {
		font-size: var(--text-sm);
		color: var(--text-primary);
	}

	code.about-val {
		font-size: var(--text-xs);
	}

	.about-desc {
		font-size: var(--text-sm);
		color: var(--text-muted);
		margin-top: var(--space-xs);
	}
</style>

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**macweep** is a macOS-native storage cleanup tool built with Tauri v2. It scans the system for regenerable artifacts (Docker images, `node_modules`, caches, AI models, IDE data), categorizes them by risk level, and provides an interactive dashboard for selective cleanup with safety-first confirmation flows.

Target: MacBook developers with limited SSDs (245 GB) who accumulate 20-60 GB of recoverable space. Replaces ad-hoc manual terminal commands with a unified, risk-categorized, auditable cleanup experience.

**Status**: Design phase — full spec in `docs/README.md`, no source code yet.

## Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (Tauri v2), async via tokio |
| Frontend | SvelteKit 2 + Svelte 5 (runes), CSS puro (scoped styles, no Tailwind) |
| Distribution | macOS `.dmg` native app |

## Expected Project Structure (when implemented)

```
macweep/
├── src-tauri/                 # Rust backend (Tauri v2)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Tauri command registration
│       ├── scanner/           # Scanner trait + per-category implementations
│       └── safety/            # Protected paths, pre-flight checks, audit log
├── src/                       # SvelteKit frontend
│   ├── routes/                # Dashboard pages
│   ├── components/            # UI components (confirmation flows per risk level)
│   └── lib/                   # Shared utilities
├── docs/README.md             # Full design spec (source of truth)
├── Makefile                   # Standard entrypoint (make dev, make build, make test)
├── package.json
├── vite.config.ts
└── tauri.conf.json
```

## Architecture

### Core: Scanner Trait

Every cleanup category implements a `Scanner` trait in Rust:

```rust
#[async_trait]
trait Scanner: Send + Sync {
    fn category(&self) -> &str;
    fn risk_level(&self) -> RiskLevel;
    async fn scan(&self) -> Result<Vec<ScanResult>>;
    async fn clean(&self, items: &[ScanResult]) -> Result<CleanResult>;
}
```

**Scanners**: `DockerScanner`, `NodeScanner`, `BrewScanner`, `PipScanner`, `IdeScanner`, `AiToolsScanner`, `SystemScanner`.

### Risk Classification Model

This is the core design pattern — it determines the UX confirmation flow:

| Risk | Meaning | Confirmation UX |
|------|---------|----------------|
| Zero | 100% regenerable caches (npm cache, brew cache, Docker build cache) | Simple Y/n |
| Low | Regenerable with a command (node_modules, idle Docker images) | Itemized list + confirm |
| Medium | Tool possibly no longer installed (Ollama models, Langflow) | Install status + last use + confirm |
| High | May contain state (named Docker volumes, Application Support) | Double confirmation — type name to confirm |

`RiskLevel` is a Rust enum enforced at the type level.

### Data Flow

```
SCAN (all categories in parallel) → REPORT (dashboard) → SELECT (user picks) → EXECUTE (clean with risk-appropriate confirmation) → LOG (audit trail)
```

- Scan progress streams via **Tauri Channel API** (ordered delivery, not events)
- Cleanup runs via `tokio::process::Command` (not Tauri Shell Plugin) for full control
- Audit trail: JSONL append-only log at `~/.storage-cleanup/audit.jsonl`

### Safety Rules (critical — must be enforced)

**Protected paths — NEVER offer for cleanup:**
- `~/Documents/`, `~/Desktop/`, `~/Downloads/`, `~/Pictures/`, `~/Photos/`
- `~/Library/Keychains/`, `~/Library/Application Support/MobileSync/`
- `~/.ssh/`, `~/.gnupg/`, `~/.gitconfig`, `~/.zshrc`, `~/.bashrc`
- Any project directory with uncommitted Git changes

**Pre-flight checks:**
- Docker containers must not be running before volume/image removal
- Skip cleanup offer if disk >30% free (unless `--force`)
- Detect available tools at startup; only enable relevant categories

### Execution Modes

```bash
./macweep                     # Interactive dashboard (default)
./macweep --scan              # Report only, no cleanup
./macweep --safe              # Auto-clean risk-zero items only
./macweep --only docker       # Specific category group
./macweep --dry-run           # Show what would be done
./macweep --scan --json       # Machine-readable output
```

### System Tray

Menu bar icon with: auto-clean safe, full scan, open dashboard, last cleanup summary.

## Scan Categories Reference

- **Risk Zero** (8): `npm-cache`, `yarn-cache`, `bun-cache`, `pip-cache`, `brew-cache`, `docker-build`, `ts-cache`, `cursor-updates`
- **Risk Low** (4): `node-modules`, `docker-images`, `docker-volumes-orphan`, `next-cache`
- **Risk Medium** (6): `langflow`, `ollama-models`, `ide-unused`, `gemini-cache`, `coderabbit`, `opencode`
- **Risk High** (3): `docker-volumes-named`, `app-support`, `logs`

Full details (scan commands, clean commands, detection criteria) in `docs/README.md`.

## Tooling & Automation

- **Atena** (`.atena/`): Strategic task tracking via GitHub Issues. Config in `.atena/atena.yaml`, templates in `.atena/templates/`.
- **Hefesto** (`.hefesto/`): CI/CD automation with CodeRabbit code review, stacked branches enabled. Config in `.hefesto/hefesto.yaml`.
- **Repository**: Private GitHub `vinimlo/macweep`.

## Anti-Patterns

- Do not offer cleanup of protected paths under any circumstance
- Do not remove Docker volumes/images while containers are running
- Do not use Tailwind — this project uses CSS puro (scoped styles) for a native macOS feel
- Do not use Tauri Shell Plugin for cleanup commands — use `tokio::process::Command` directly
- Do not use Tauri events for scan progress — use Channel API (ordered delivery)

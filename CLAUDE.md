# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**macweep** is a macOS-native storage cleanup tool built with Tauri v2. It scans the system for regenerable artifacts (Docker images, `node_modules`, caches, AI models, IDE data), categorizes them by risk level, and provides an interactive dashboard for selective cleanup with safety-first confirmation flows.

Target: MacBook developers with limited SSDs who accumulate 20-60 GB of recoverable space.

## Commands

```bash
make dev            # Frontend (Docker) + Tauri dev mode
make dev-frontend   # Frontend only (Docker, port 5173)
make dev-tauri      # Tauri dev mode only (no Docker)
make build          # Production build (frontend + Tauri)
make build-dmg      # macOS .dmg bundle via dmgbuild
make lint           # cargo clippy + svelte-check
make test           # cargo test + svelte-check
make clean          # Stop Docker, clean all build artifacts
```

**Environment gotcha**: Rust is installed via rustup — `source ~/.cargo/env` is needed before cargo commands (already handled in Makefile targets).

## Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (Tauri v2), async via tokio |
| Frontend | SvelteKit 2 + Svelte 5 (runes), pure CSS (scoped styles, no Tailwind) |
| Dev env | Docker Compose for frontend (hot reload on port 5173), Tauri on host (needs native webview) |
| Distribution | macOS `.dmg` native app |

## Project Structure

```
macweep/
├── src-tauri/src/
│   ├── main.rs                # Entry point
│   ├── lib.rs                 # Tauri app builder, command registration, managed state
│   ├── commands.rs            # Tauri IPC commands (scan_all, clean_items, get_disk_info, get_audit_log, etc.)
│   ├── models.rs              # Shared types (RiskLevel, ScanResult, CleanResult, ScanProgress, etc.)
│   ├── activity.rs            # ActivityLogger — JSONL file + Tauri event emission (managed state)
│   ├── tray.rs                # System tray menu (uses events, not window.eval)
│   ├── scanner/
│   │   ├── mod.rs             # Scanner trait + build_registry() with all 13 scanners
│   │   ├── docker.rs          # DockerBuildCacheScanner, DockerImagesScanner, DockerOrphanVolumesScanner, DockerNamedVolumesScanner
│   │   ├── node.rs            # NodeCacheScanner (npm/yarn/bun), NodeModulesScanner (+ next-cache)
│   │   ├── brew.rs            # BrewScanner
│   │   ├── pip.rs             # PipScanner
│   │   ├── ai_tools.rs        # AiToolsScanner (ollama/langflow/gemini/coderabbit/opencode)
│   │   ├── ide.rs             # IdeScanner
│   │   └── system.rs          # SystemCacheScanner (ts/cursor), AppSupportScanner, LogsScanner
│   └── safety/
│       ├── mod.rs
│       ├── protected_paths.rs # Protected paths enforcement
│       ├── preflight.rs       # Pre-flight checks (Docker running, disk space, etc.)
│       └── audit.rs           # JSONL audit trail at ~/.storage-cleanup/audit.jsonl
├── src/
│   ├── routes/                # Pages: dashboard (+page), scan, cleanup, category/[id], history, settings
│   ├── lib/
│   │   ├── components/
│   │   │   ├── shared/        # Modal, Toast, RiskBadge, ProgressBar, TypeToConfirm, ActivityDrawer, FloatingCleanBar
│   │   │   ├── dashboard/     # DiskUsageBar, SpaceSummary, CategoryCard, CategoryGrid, QuickActions
│   │   │   ├── scan/          # ScanProgress, CategoryScanRow, ScanSummary
│   │   │   └── cleanup/       # CleanupFlow (orchestrator), ConfirmZero/Low/Medium/High, CleanupProgress, CleanupReport
│   │   ├── stores/            # Svelte 5 rune stores: scan, cleanup, app, toasts, activity (.svelte.ts)
│   │   ├── tauri/             # Typed IPC wrappers (commands.ts) + shared types (types.ts)
│   │   └── utils/             # format.ts, risk.ts, errors.ts
│   └── app.d.ts
├── docs/README.md             # Full design spec (source of truth for planned features)
├── Makefile                   # Standard entrypoint
├── docker-compose.yml         # Frontend dev container
├── Dockerfile.dev
└── tauri.conf.json
```

## Architecture

### Scanner Trait

Every cleanup category implements a `Scanner` trait in Rust (`src-tauri/src/scanner/mod.rs`):

```rust
#[async_trait]
pub trait Scanner: Send + Sync {
    fn category(&self) -> &str;
    fn risk_level(&self) -> RiskLevel;
    fn handles_category(&self, cat: &str) -> bool;
    async fn is_available(&self) -> bool;
    async fn scan(&self) -> Result<Vec<ScanResult>>;
    async fn clean(&self, items: &[ScanResult]) -> Result<Vec<CleanResult>>;
}
```

**13 scanners** registered in `build_registry()`:
- **Risk Zero (5)**: BrewScanner, PipScanner, NodeCacheScanner, DockerBuildCacheScanner, SystemCacheScanner
- **Risk Low (3)**: NodeModulesScanner, DockerImagesScanner, DockerOrphanVolumesScanner
- **Risk Medium (2)**: AiToolsScanner, IdeScanner
- **Risk High (3)**: DockerNamedVolumesScanner, AppSupportScanner, LogsScanner

### Risk Classification Model

| Risk | Meaning | Confirmation UX |
|------|---------|----------------|
| Zero | 100% regenerable caches | Simple Y/n |
| Low | Regenerable with a command | Itemized list + confirm |
| Medium | Tool possibly no longer installed | Install status + last use + confirm |
| High | May contain state | Double confirmation — type name to confirm |

### Data Flow

```
SCAN (parallel) → REPORT (dashboard) → SELECT (user picks) → EXECUTE (risk-appropriate confirmation) → LOG (audit trail)
```

- Scan progress streams via **Tauri Channel API** (ordered delivery, not events)
- Cleanup runs via `tokio::process::Command` (not Tauri Shell Plugin)
- Audit trail: JSONL at `~/.storage-cleanup/audit.jsonl`
- Activity log: `ActivityLogger` managed state — JSONL file + Tauri events for UI drawer

### Safety Rules (critical)

**Protected paths — NEVER offer for cleanup:**
- `~/Documents/`, `~/Desktop/`, `~/Downloads/`, `~/Pictures/`, `~/Photos/`
- `~/Library/Keychains/`, `~/Library/Application Support/MobileSync/`
- `~/.ssh/`, `~/.gnupg/`, `~/.gitconfig`, `~/.zshrc`, `~/.bashrc`
- Any project directory with uncommitted Git changes

**Pre-flight checks:**
- Docker containers must not be running before volume/image removal
- Skip cleanup offer if disk >30% free (unless forced)
- Detect available tools at startup; only enable relevant categories

## Tooling & Automation

- **Atena** (`.atena/`): Strategic task tracking via GitHub Issues
- **Hefesto** (`.hefesto/`): CI/CD automation with CodeRabbit code review, stacked branches
- **Repository**: Private GitHub `vinimlo/macweep`

## Anti-Patterns

- Do not offer cleanup of protected paths under any circumstance
- Do not remove Docker volumes/images while containers are running
- Do not use Tailwind — pure CSS (scoped styles) for native macOS feel
- Do not use Tauri Shell Plugin — use `tokio::process::Command` directly
- Do not use Tauri events for scan progress — use Channel API (ordered delivery)
- System tray uses events, not direct JS evaluation for window communication

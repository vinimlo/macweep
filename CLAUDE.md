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
make format         # cargo fmt + prettier (run before finishing work)
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
│   ├── state.rs               # ScanState: scan generation counter + items of the last finished scan
│   ├── cleanup.rs             # Cleanup orchestration: routes items to scanners, measures freed bytes
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
│       ├── preflight.rs       # Docker "containers running?" check before image/volume removal
│       └── audit.rs           # JSONL audit trail at ~/.storage-cleanup/audit.jsonl
├── src/
│   ├── routes/                # Pages: dashboard (+page), cleanup, category/[id], history, settings
│   ├── lib/
│   │   ├── components/
│   │   │   ├── shared/        # Toast, RiskBadge, ProgressBar, Spinner, TypeToConfirm, ActivityDrawer, FloatingCleanBar
│   │   │   ├── dashboard/     # DiskUsageBar, SpaceSummary, CategoryCard, CategoryGrid, QuickActions
│   │   │   ├── scan/          # ScanProgress, CategoryScanRow
│   │   │   └── cleanup/       # CleanupFlow (orchestrator), ConfirmZero/Low/Medium/High, CleanupProgress, CleanupReport
│   │   ├── stores/            # Svelte 5 rune stores: scan, cleanup, app, toasts, activity (.svelte.ts).
│   │   │                      # scanStore.start/stop and cleanupStore.run own the IPC calls
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
    /// Exactly one result per item; a failure on one item never aborts the others.
    async fn clean(&self, items: &[ScanResult]) -> Vec<CleanResult>;
}
```

Cleaning helpers in `scanner/mod.rs`: `clean_filesystem_items` (validate + `remove_dir_all`) and `clean_with_command` (CLI with timeout + `kill_on_drop`; spawn errors, timeouts and non-zero exits become a per-item failure). Prefer removing a cache directory directly over shelling out to a tool that may not be on the GUI app's PATH.

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
SCAN → REPORT (dashboard) → SELECT (user picks) → EXECUTE (risk-appropriate confirmation) → LOG (audit trail)
```

- Scan progress streams via **Tauri Channel API** (ordered delivery, not events)
- Every scan takes a new generation in `ScanState`; a newer scan or `cancel_scan` supersedes it, and the frontend ignores events from older runs
- `clean_items` takes **IDs only** and cleans items from the last finished scan (removed from state while cleaning, restored if they fail) — the webview can never name a path
- Freed bytes for filesystem items are measured with `du` right before and after removal; the report also carries the disk's free-space delta (`df`)
- Cleanup runs via `tokio::process::Command` (not Tauri Shell Plugin)
- Audit trail: JSONL at `~/.storage-cleanup/audit.jsonl`
- Activity log: `ActivityLogger` managed state — JSONL file + Tauri events for UI drawer

### Safety Rules (critical)

**Protected paths — NEVER offer for cleanup:**
- `~/Documents/`, `~/Desktop/`, `~/Downloads/`, `~/Pictures/`, `~/Photos/`
- `~/Library/Keychains/`, `~/Library/Application Support/MobileSync/`
- `~/.ssh/`, `~/.gnupg/`, `~/.gitconfig`, `~/.zshrc`, `~/.bashrc`
- Symlinks and paths resolving outside their parent (`validate_before_delete`)

`node_modules` in a repo with uncommitted Git changes is still offered, with a warning on the item.

**Pre-flight checks:**
- Docker images/volumes are skipped (per item, with a reason) while any container is running
- Each scanner's `is_available()` decides whether it runs

## Workflow Rules

- **Always run `make format` before finishing any task** — ensures `cargo fmt` and `prettier` pass in CI
- Never trigger a cleanup from a `$effect`: effects re-run when any state they read changes (this once re-ran every cleanup when the report reset the selection)

## Design System

Tokens and primitives live in `src/app.css`: one brand accent (the logo's cyan), risk colors reserved for risk, a fixed type scale (`--text-2xs` … `--text-display`), SF Pro with tabular figures for numbers and SF Mono only for paths/commands. Use the global `.btn` variants (`btn-primary`, `btn-secondary`, `btn-ghost`, `btn-tinted`/`btn-solid` with `--tint`, `btn-danger`), the styled native checkbox (`--check-color`), `.selectable` for copyable text, and the `confirm-*` classes for cleanup steps. Buttons and titles use Title Case (macOS HIG); labels use sentence case.

The CSP (`style-src 'self'`) blocks **static** `style="..."` attributes in the packaged app; dynamic `style="--x: {value}"` is fine (Svelte applies it through CSSOM). Put static styles in CSS.

## Anti-Patterns

- Do not offer cleanup of protected paths under any circumstance
- Do not remove Docker volumes/images while containers are running
- Do not use Tailwind — pure CSS (scoped styles) for native macOS feel
- Do not use Tauri Shell Plugin — use `tokio::process::Command` directly
- Do not use Tauri events for scan progress — use Channel API (ordered delivery)
- System tray uses events, not direct JS evaluation for window communication

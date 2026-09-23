<div align="center">

<img src="static/logo.svg" alt="macweep logo" width="140" />

# macweep

### Sweep your Mac clean.

A native macOS app that finds and removes regenerable junk — Docker leftovers, `node_modules`, package caches, AI models, IDE bloat — so you get your SSD space back without the risk.

[![macOS](https://img.shields.io/badge/macOS-14%2B-000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://v2.tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue?style=flat-square)](LICENSE)
<br/>

**20–60 GB recovered** on a typical developer MacBook.

Zero caches. Stale containers. Forgotten models. Gone.

<br/>

</div>

---

## The Problem

You're a developer on a 256 GB MacBook. You get the "disk almost full" warning. You open the terminal and start the ritual:

```bash
docker system prune -a -f
npm cache clean --force
brew cleanup --prune=all
rm -rf ~/project-a/node_modules ~/project-b/node_modules
# What else? Where else? How much did that free?
```

You do this every few weeks. You never remember all the commands. You're never sure what's safe to delete.

**macweep replaces that ritual with a single scan.**

---

## How It Works

```
SCAN ──→ CATEGORIZE ──→ SELECT ──→ CONFIRM ──→ CLEAN ──→ LOG
  │           │             │          │          │        │
  │     risk-sorted    interactive  flow matches  async   audit
  │      dashboard      checkboxes  risk level    tokio   trail
  │
  13 scanners
```

macweep scans your system across **13 categories**, groups findings by risk level, and lets you pick exactly what to remove — with confirmation flows that match the stakes.

---

## Risk Levels

The core design principle: **not all deletions are equal.**

| | Risk | What it means | Confirmation |
|---|---|---|---|
| 🟢 | **Zero** | Pure caches — recreated automatically | One-click |
| 🟡 | **Low** | Regenerable with a command (`npm install`, `docker pull`) | Itemized review |
| 🟠 | **Medium** | Tool may no longer be installed | Install status check + confirm |
| 🔴 | **High** | May contain state or user data | Type the name to confirm |

This means you can confidently clean 🟢 items every day, and make informed decisions on the rest.

---

## What It Finds

<table>
<tr>
<td width="50%" valign="top">

**🟢 Zero Risk** — always safe
- npm / yarn / bun caches
- pip cache
- Homebrew cache
- Docker build cache
- TypeScript build info
- Cursor / IDE update caches

</td>
<td width="50%" valign="top">

**🟡 Low Risk** — quick to restore
- `node_modules` directories
- Idle Docker images
- Orphaned Docker volumes
- `.next/cache` build artifacts

</td>
</tr>
<tr>
<td width="50%" valign="top">

**🟠 Medium Risk** — verify first
- Ollama models
- Langflow / Gemini / CodeRabbit data
- Unused IDE configs & extensions

</td>
<td width="50%" valign="top">

**🔴 High Risk** — proceed with care
- Named Docker volumes (db-data, etc.)
- Application Support folders
- System & app logs

</td>
</tr>
</table>

---

## Safety First

macweep is paranoid by design.

**Protected paths — never touched, never offered:**
- `~/Documents`, `~/Desktop`, `~/Downloads`, `~/Pictures`
- `~/.ssh`, `~/.gnupg`, `~/.gitconfig`, `~/.zshrc`
- `~/Library/Keychains`, iCloud backups
- Symlinks, and anything that resolves outside its parent directory

**Checks before deleting:**
- Docker images and volumes are skipped while any container is running
- `node_modules` in a Git repo with uncommitted changes is flagged with a warning
- Only what the last scan found can be cleaned — the UI sends item IDs, never paths

**Honest numbers:**
Folders are measured right before and right after removal, so the report shows what actually left the disk, next to how much free space the disk gained. Items that fail are listed with the reason and stay available for a retry.

**Audit trail:**
Every action is logged to `~/.storage-cleanup/audit.jsonl` — what was removed, when, how much space was freed. Viewable in the History tab.

---

## Install

Download the `.dmg` from [Releases](https://github.com/vinimlo/macweep/releases) (Apple Silicon). The app is not notarized yet, so the first launch is blocked by Gatekeeper: open **System Settings → Privacy & Security** and click **Open Anyway** next to the macweep message.

## Build from Source

### Prerequisites

- macOS 14+
- [Rust](https://rustup.rs) (stable)
- Node.js 22+

### Development

```bash
# Clone
git clone https://github.com/vinimlo/macweep.git
cd macweep

# Install dependencies
npm install

# Run in development mode
make dev
```

### Build

```bash
make build-dmg    # .dmg in src-tauri/target/release/bundle/dmg/
```

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  SvelteKit Frontend               │
│  ┌────────────┐ ┌──────────┐ ┌────────────────┐  │
│  │  Dashboard  │ │   Scan   │ │ Cleanup Flows  │  │
│  │  + Disk bar │ │ Progress │ │ (per risk lvl) │  │
│  └─────┬──────┘ └────┬─────┘ └───────┬────────┘  │
│        └──────────────┼───────────────┘           │
│                       │ Channel API (ordered)     │
├───────────────────────┼──────────────────────────┤
│                  Tauri v2 / Rust                  │
│  ┌────────────────────┼────────────────────────┐  │
│  │            Scanner Registry                 │  │
│  │  ┌───────┐ ┌──────┐ ┌──────┐ ┌──────────┐  │  │
│  │  │Docker │ │ Node │ │ Brew │ │ AI Tools │  │  │
│  │  │Scanner│ │Scannr│ │Scannr│ │ Scanner  │  │  │
│  │  └───────┘ └──────┘ └──────┘ └──────────┘  │  │
│  │         + 9 more scanners                   │  │
│  ├─────────────────────────────────────────────┤  │
│  │  Safety Layer │ Activity Log │ Audit Trail  │  │
│  └─────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

**Key decisions:**
- **Channel API** for scan progress (ordered delivery, not Tauri events)
- **`tokio::process::Command`** for cleanup execution (not Shell Plugin — full control)
- **pure CSS** — scoped styles, no Tailwind, for a native macOS feel
- **Scanner trait** — each category is a pluggable Rust struct implementing `scan()` + `clean()`

---

## System Tray

macweep lives in your menu bar for quick access:

- **Auto-clean Safe** — opens the zero-risk confirmation (scans first if needed)
- **Full Scan** — opens the dashboard and starts a scan
- **Open Dashboard** / **Quit macweep**

---

## Project Structure

```
macweep/
├── src-tauri/                 # Rust backend
│   └── src/
│       ├── lib.rs             # App builder, command registration
│       ├── commands.rs        # IPC command handlers
│       ├── state.rs           # Scan generations + last scan's items
│       ├── cleanup.rs         # Cleanup orchestration + measurement
│       ├── scanner/           # Scanner trait + 13 implementations
│       ├── safety/            # Protected paths, pre-flight, audit trail
│       └── activity.rs        # Activity log (JSONL + UI events)
├── src/                       # SvelteKit frontend
│   ├── routes/                # Dashboard pages
│   └── lib/
│       ├── components/        # UI (dashboard, scan, cleanup, shared)
│       ├── stores/            # Svelte 5 rune stores
│       └── tauri/             # Typed IPC wrappers
├── docs/                      # Design spec
├── Makefile                   # make dev | build | test | lint
└── CLAUDE.md                  # AI assistant guidance
```

---

## Contributing

macweep is a personal project, but issues and ideas are welcome. If you're a Mac developer drowning in Docker images and `node_modules`, you know why this exists.

---

<div align="center">
<br/>
<sub>Built with Tauri, Rust, and the frustration of running out of disk space.</sub>
<br/><br/>
</div>

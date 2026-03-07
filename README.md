<div align="center">

<img src="static/logo.svg" alt="macweep logo" width="140" />

# macweep

### Sweep your Mac clean.

A native macOS app that finds and removes regenerable junk — Docker leftovers, `node_modules`, package caches, AI models, IDE bloat — so you get your SSD space back without the risk.

[![macOS](https://img.shields.io/badge/macOS-14%2B-000?style=flat-square&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://v2.tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

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
  13 scanners in parallel
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
- Any Git repo with uncommitted changes

**Pre-flight checks:**
- Won't remove Docker resources while containers are running
- Detects which tools are actually installed before scanning
- Skips cleanup prompt if disk has >30% free (unless `--force`)

**Audit trail:**
Every action is logged to `~/.storage-cleanup/audit.jsonl` — what was removed, when, how much space was freed. Viewable in the activity drawer.

---

## Quick Start

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
make build    # Produces a .dmg in src-tauri/target/release/bundle/
```

---

## CLI Modes

macweep works as a full dashboard by default, but also supports targeted workflows:

```bash
macweep                       # Interactive dashboard
macweep --scan                # Report only — no cleanup
macweep --safe                # Auto-clean zero-risk items
macweep --only docker         # Scan a specific category group
macweep --dry-run             # Preview what would be cleaned
macweep --scan --json         # Machine-readable output
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

- **Auto-clean safe** — one-click zero-risk cleanup
- **Full scan** — open the dashboard
- **Last cleanup** — summary of the most recent session

---

## Project Structure

```
macweep/
├── src-tauri/                 # Rust backend
│   └── src/
│       ├── main.rs            # Tauri command registration
│       ├── commands.rs        # IPC command handlers
│       ├── scanner/           # Scanner trait + 13 implementations
│       ├── safety/            # Protected paths, pre-flight checks
│       └── activity.rs        # JSONL audit logger
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

## License

MIT

---

<div align="center">
<br/>
<sub>Built with Tauri, Rust, and the frustration of running out of disk space.</sub>
<br/><br/>
</div>

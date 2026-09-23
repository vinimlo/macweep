<div align="center">

<img src="static/logo.svg" alt="" width="112" />

# macweep

Get back the disk space your dev tools quietly eat,<br />without gambling with anything you care about.

[Download for Mac](https://github.com/vinimlo/macweep/releases/latest) &nbsp;·&nbsp; [Build from source](#build-from-source) &nbsp;·&nbsp; [Leia em português](README.pt-BR.md)

[![Latest release](https://img.shields.io/github/v/release/vinimlo/macweep?style=flat-square&color=22d3ee&label=release)](https://github.com/vinimlo/macweep/releases/latest)
[![macOS 14+](https://img.shields.io/badge/macOS-14%2B-1c1d21?style=flat-square&logo=apple&logoColor=white)](#install)
[![Apple Silicon](https://img.shields.io/badge/Apple_Silicon-arm64-1c1d21?style=flat-square)](#install)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache_2.0-blue?style=flat-square)](LICENSE)

<br />

<img src="docs/images/dashboard.png" alt="The macweep dashboard after a scan: a 228 GB disk at 96% used, with 22.7 GB recoverable across 14 categories" width="860" />

</div>

## Why

Every few weeks my 256 GB MacBook ran out of space, and every time I went through the same ritual:

```bash
docker system prune -a
npm cache clean --force
brew cleanup --prune=all
rm -rf ~/code/*/node_modules
# ...and whatever else I could remember
```

I never remembered all of it, I never knew how much each command gave back, and every `rm -rf` came with a small worry. macweep turns that ritual into one scan. It finds what your tools can rebuild on their own, tells you how risky each thing is to delete, and asks for more care the more there is to lose.

## What it finds

Thirteen scanners look through the usual suspects and sort everything into four levels. The level decides how you confirm.

| Level | What lands here | How you confirm |
|---|---|---|
| Safe | npm, Yarn and Bun caches; pip and Homebrew caches; Docker build cache; TypeScript cache; Cursor update downloads | One click for the whole group |
| Low | `node_modules` and `.next/cache` in your projects; Docker images; dangling Docker volumes | An itemized list, all checked, where you uncheck what you want to keep |
| Medium | Ollama models; Langflow, Gemini CLI, CodeRabbit and OpenCode data; leftover config from Cursor, Trae and Antigravity | Nothing is preselected, and each item says whether the tool is still installed |
| High | Named Docker volumes; the largest folders in `~/Library/Application Support`; `~/Library/Logs` | You type each item's name |

Projects are found by walking your home folder up to six levels deep. `Library`, `Documents`, `Desktop`, `Downloads`, media folders and toolchain directories such as `~/.cargo` are skipped, so a project living inside `~/Documents` won't show up.

<p align="center">
  <img src="docs/images/risk-levels.png" alt="Categories grouped by risk, from Low to High, with two of them selected and a bar showing 8 items and 13.1 GB ready to clean" width="760" />
</p>

## Built not to break things

macweep deletes for real. Nothing goes to the Trash, so the guardrails live in the Rust backend, not only in the interface.

Some paths are never offered and never deleted: `~/Documents`, `~/Desktop`, `~/Downloads`, `~/Pictures`, `~/Photos`, `~/.ssh`, `~/.gnupg`, `~/Library/Keychains`, your iPhone backups, `~/.gitconfig`, `~/.zshrc` and `~/.bashrc`. Right before anything is removed, its path is checked again, and symlinks or paths that resolve outside their parent folder are refused.

Docker images and volumes are left alone while any container is running, or when Docker doesn't answer. A named volume that still belongs to a stopped container tells you so, and that container goes away with it. A `node_modules` whose project has uncommitted Git changes gets a warning.

The interface never sends a path to the backend. It sends the IDs of items the last scan found, and each one can be cleaned only once. Commands run as a program plus arguments, never through a shell. There's no telemetry, no account, and the app makes no network requests.

<p align="center">
  <img src="docs/images/confirm-high-risk.png" alt="High-risk confirmation for a named Docker volume: a warning that it may hold data you can't get back, and a field where you type the volume's name to confirm" width="760" />
</p>

## Numbers you can trust

Every folder is measured right before and right after it's removed, so the report shows what actually left the disk. Right below it you see how much free space the disk really gained.

Those two numbers can differ, and the report says why when it happens. Docker Desktop keeps freed space inside its disk image until it restarts, and macOS can hold deleted files in local snapshots for a while before the space shows up as free.

If an item fails, it's listed with the reason and stays available for another try. Every action is written to `~/.storage-cleanup/audit.jsonl`, which you can browse in the History tab.

<p align="center">
  <img src="docs/images/cleanup-report.png" alt="Cleanup report: 13.1 GB removed across 8 items, free space on disk up by 7.2 GB, and a note explaining that Docker keeps freed space until Docker Desktop restarts" width="760" />
</p>

## Install

Download the latest `.dmg` from [Releases](https://github.com/vinimlo/macweep/releases/latest) and drag macweep into Applications. It runs on Apple Silicon with macOS 14 or later.

The app isn't notarized yet, so Gatekeeper blocks the first launch. Open System Settings › Privacy & Security, scroll down and click Open Anyway next to the macweep message. You only need to do this once.

macweep also lives in the menu bar. Full Scan opens the dashboard and starts scanning, and Auto-clean Safe takes you straight to the Safe group's confirmation, scanning first if needed.

To uninstall, drag macweep from Applications to the Trash. Its logs live in `~/.storage-cleanup`, so delete that folder too if you want nothing left behind.

## Build from source

You'll need macOS 14 or later, the Xcode Command Line Tools, [Rust](https://rustup.rs) stable, Node.js 22 or later, and Docker for the development frontend.

```bash
git clone https://github.com/vinimlo/macweep.git
cd macweep
npm ci
cargo install tauri-cli --version "^2" --locked   # only once

make dev        # hot reload: frontend in Docker, app on your Mac
make install    # release build, copied to /Applications
```

`make test` runs the Rust tests and svelte-check, and `make lint` runs clippy and svelte-check. The Docker cleanup tests need a running daemon, so they're opt-in: `cd src-tauri && cargo test -- --ignored docker`. `make build-dmg` builds the installer and needs `pip install -r requirements-dev.txt` first.

## Under the hood

A Rust backend on [Tauri v2](https://v2.tauri.app) and a SvelteKit frontend with Svelte 5. Each category is a Rust type implementing one `Scanner` trait with `scan` and `clean`, so teaching macweep about a new kind of junk means adding one file to `src-tauri/src/scanner/`. Scan progress streams to the window over Tauri channels, and cleanup runs through `tokio::process::Command` or plain filesystem calls.

## Contributing

Issues and pull requests are welcome, new scanners for tools that hoard disk space most of all. [CONTRIBUTING.md](CONTRIBUTING.md) covers the setup and the safety rules every change has to keep. If you find a security problem, please follow [SECURITY.md](SECURITY.md) instead of opening a public issue.

## License

[Apache 2.0](LICENSE)

<br />

<div align="center">
<sub>Built with Tauri, Rust, and the frustration of running out of disk space.</sub>
</div>

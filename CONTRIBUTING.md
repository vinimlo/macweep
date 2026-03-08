# Contributing to macweep

Thanks for your interest in contributing to macweep! This guide covers the basics.

## Development Setup

### Prerequisites

- macOS 14+
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 22+
- [Docker](https://www.docker.com/) (for frontend dev container)
- Tauri v2 prerequisites: Xcode Command Line Tools

### Getting Started

```bash
git clone https://github.com/vinimlo/macweep.git
cd macweep
make dev          # Frontend (Docker) + Tauri dev mode
```

See the [Makefile](Makefile) for all available targets.

## Code Style

- **Rust**: `cargo fmt` (default style) + `cargo clippy`
- **Frontend**: Prettier (via `npx prettier --write`)
- **CSS**: Pure scoped CSS — no Tailwind
- **Always run `make format` before submitting a PR**

## Architecture

- **Backend**: Tauri v2 + Rust with async scanners (`src-tauri/src/scanner/`)
- **Frontend**: SvelteKit 2 + Svelte 5 runes (`src/`)
- **IPC**: Tauri Channel API for ordered progress streaming
- See [CLAUDE.md](CLAUDE.md) for detailed architecture docs

## Making Changes

1. Fork the repo and create a feature branch from `main`
2. Make your changes
3. Run checks:
   ```bash
   make format    # Auto-format code
   make lint      # cargo clippy + svelte-check
   make test      # cargo test + svelte-check
   ```
4. Commit with a clear, concise message
5. Open a PR against `main`

## Safety Rules

macweep deletes files, so safety is critical:

- **Never** add cleanup of protected paths (Documents, Desktop, SSH keys, keychains, etc.)
- **Never** use shell string concatenation for commands — always use `.arg()`
- **Never** use `unsafe` Rust
- **Always** validate paths with `canonicalize()` before deletion
- **Always** check for running Docker containers before removing volumes/images

See `src-tauri/src/safety/` for the full safety implementation.

## Reporting Bugs

Open a [GitHub issue](https://github.com/vinimlo/macweep/issues) with:
- macOS version
- Steps to reproduce
- Expected vs actual behavior

For security vulnerabilities, see [SECURITY.md](SECURITY.md).

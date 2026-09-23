# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in macweep, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email: **vinimelo@riseup.net**

Include:
- A description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

You should receive an acknowledgment within 48 hours. We will work with you to understand the issue and coordinate a fix before any public disclosure.

## Scope

macweep runs locally on macOS with user-level permissions. Security concerns include:

- **Path traversal** — bypassing protected path checks to delete unintended files
- **Command injection** — injecting shell commands via crafted file paths or scanner inputs
- **Privilege escalation** — gaining elevated permissions beyond the logged-in user
- **Data loss** — deleting files that should be protected (uncommitted Git changes, keychains, SSH keys)

## Security Design

- All shell commands use `tokio::process::Command` with `.arg()` — no string interpolation
- Protected paths are enforced with `canonicalize()` + component-level matching + symlink detection
- The webview never sends paths: cleanup requests carry item IDs, and the backend only deletes items its own last scan found (each at most once)
- One `unsafe` block: `std::env::set_var` extends `PATH` at startup, before any thread is spawned (required by Rust 2024)
- No network access — the app is fully offline
- Audit trail logged to `~/.storage-cleanup/audit.jsonl` with mode `0o600`
- CSP: `default-src 'self'` — no inline scripts, no eval, no external resources

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

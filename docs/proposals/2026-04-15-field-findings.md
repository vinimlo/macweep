# Field Findings — 2026-04-15 Cleanup Session

> Status: **📥 Proposal** — new scanner gaps discovered during a real-world rescue on a 228 GB MBA with only 3.9 GB free.

## Session Summary

Started at 3.9 GB free, ended at 33 GB free (**~29 GB reclaimed**) without touching user data. Cleanup order:

1. `~/Library/Caches/*` (manual `rm -rf`) → +11 GB
2. `docker system prune -a --volumes` → +11 GB (logical 15.21 GB, but sparse file didn't fully shrink)
3. `~/Library/Application Support/Claude/vm_bundles/claudevm.bundle` (manual `rm -rf`) → +7 GB

All steps fall *outside* the coverage of the current 13 scanners. This doc captures the gaps.

## Gap 1 — Docker.raw Sparse File Not Compacted 🔴 Critical UX bug

**Current behavior**: `DockerImagesScanner` + `DockerBuildCacheScanner` run `docker ... prune` and report bytes freed from `docker system df`.

**Problem observed**: After `prune -a --volumes` reclaimed 15.21 GB logically, `Docker.raw` on disk only shrunk from 48 GB → 34 GB. The missing 1 GB is real user confusion — they'll check `df -h` and think the tool lied. Even running `docker/desktop-reclaim-space` container changed nothing.

**Root cause**: Docker Desktop's `Docker.raw` is a sparse disk image. `prune` frees blocks *inside* the image but macOS only reclaims them after a TRIM pass, which only runs on Docker Desktop restart (or via private API).

**Proposed fix**:
- After any Docker cleanup scanner runs, check `Docker.raw` actual size (`du -h`, not `ls -lh`) vs logical size and report the delta
- Offer a **"Compact Docker disk"** post-cleanup step that quits & restarts Docker Desktop:
  ```bash
  osascript -e 'quit app "Docker"'
  sleep 5
  open -a Docker
  ```
- Surface a warning in the cleanup report: *"Docker reported 15 GB freed, but the disk image still holds 34 GB. Restart Docker Desktop to reclaim the full amount."*

**Risk level**: 🟢 Zero (pure TRIM) but requires stopping Docker — pre-flight check should verify no containers are running.

---

## Gap 2 — Claude Desktop VM Bundles 🟢 New Scanner

**Path**: `~/Library/Application Support/Claude/vm_bundles/claudevm.bundle/`

**Contents**:
- `rootfs.img` — 10 GB sparse (~7 GB actual)
- `sessiondata.img` — 2.4 GB
- Small metadata files (efivars, machineIdentifier, macAddress, vmIP)

**What it is**: The macOS Virtualization.framework VM disk used by Claude Desktop's "claude-code-vm" sandbox feature. Fully regeneratable — Claude Desktop re-provisions on next sandbox launch (~5-10 min).

**Not covered by**: `AiToolsScanner` (which handles ollama/langflow/gemini/coderabbit/opencode). Claude Desktop data lives in `Application Support/Claude/`, not in any existing scanner path.

**Proposed scanner**: `ClaudeDesktopScanner` or extend `AiToolsScanner`.

| Risk | Target | Scan | Clean |
|------|--------|------|-------|
| 🟡 Low | `claudevm.bundle` | `du -sh ~/Library/Application\ Support/Claude/vm_bundles/claudevm.bundle` | `rm -rf ~/Library/Application\ Support/Claude/vm_bundles/claudevm.bundle` |
| 🟢 Zero | `Claude/Cache` | `du -sh ~/Library/Application\ Support/Claude/Cache` | `rm -rf ~/Library/Application\ Support/Claude/Cache/*` |
| 🟡 Low | `claude-code-vm` | `du -sh ~/Library/Application\ Support/Claude/claude-code-vm` | Confirm + remove |
| 🟡 Low | `local-agent-mode-sessions` (older than N days) | size + mtime | selective removal |

**Detection logic**: Check if Claude Desktop is installed (`/Applications/Claude.app` exists). Only enable if true.

**Safety note**: Must warn that the sandbox feature will re-provision on next use. `sessiondata.img` may contain state from ongoing sessions — classify as Low, not Zero.

**Expected savings**: 7-12 GB on active Claude Desktop users.

---

## Gap 3 — WhatsApp Media 🟠 External App Category

**Path**: `~/Library/Group Containers/group.net.whatsapp.WhatsApp.shared/Message/` (16 GB observed)

**What it is**: WhatsApp for Mac's media cache — photos, videos, voice messages downloaded from chats. Technically regenerable from phone backup, but macweep should NOT touch it directly (WhatsApp has its own "Manage Storage" UI and the on-disk format is undocumented).

**Proposed treatment**: Not a delete scanner. Instead, a new category: **"External App Guidance"** — surfaces large third-party caches that macweep can *detect* but not safely clean, and points the user to the app's own cleanup UI.

| Detection | Guidance |
|-----------|----------|
| WhatsApp Messages folder > 2 GB | "Open WhatsApp → Settings → Storage Usage to review media" |
| Steam library > 10 GB | "Open Steam → Settings → Downloads → Clear Download Cache, or uninstall unused games" |
| Chrome Application Support > 5 GB | "Chrome → Settings → Privacy → Clear browsing data (profiles intact)" |

**Value**: Users get a complete picture of disk usage, not just what macweep can clean. Avoids the "I cleaned 10 GB but I still see 50 GB of unknown stuff" gap.

**Risk level**: N/A (macweep never writes to these paths).

---

## Gap 4 — Sparse vs Actual Size Display

Observed confusion: `ls -lh Docker.raw` shows 228 GB (max allocation), but `du -sh` shows 34 GB (real). The current scanners presumably use `du` already, but the cleanup report should:

1. Always display *actual* size via `du`, never *apparent* size via `ls`/stat
2. When a file is sparse (apparent >> actual), show both with a tooltip: *"This file is sparse — it reserves up to 228 GB but only uses 34 GB on disk right now."*

Applies especially to: `Docker.raw`, `rootfs.img` (Claude VM), any `.img`/`.dmg` backing store.

---

## Gap 5 — `~/Library/Caches/*` Bulk Sweep

Existing `SystemCacheScanner` covers `ts` and `cursor`. The session manually wiped `~/Library/Caches/*` and reclaimed 7.7 GB across:

- `Caches/Google/` — 3.2 GB (Chrome HTTP cache, **not profile**)
- `Caches/ms-playwright/` — 1.2 GB
- `Caches/vscode-cpptools/` — 1.0 GB
- `Caches/Trae/` — 542 MB
- `Caches/com.apple.python/` — 456 MB
- `Caches/SiriTTS/` — 261 MB
- `Caches/ollama/` — 156 MB
- `Caches/ms-playwright-go/` — 127 MB
- `Caches/Comet/` — 123 MB
- `Caches/claude-cli-nodejs/` — 89 MB

**Proposed**: Extend `SystemCacheScanner` to iterate all of `~/Library/Caches/*` with a **domain allow-list/deny-list** model:

- **🟢 Zero (bulk safe)**: Everything except the deny-list below
- **❌ Deny-list (never touch)**: `com.apple.*` protected caches (Safari, HomeKit, FindMy, Siri speech data user might not want to redownload)
- **🟡 Low**: Caches for apps currently running (could destabilize session) — skip with warning

**Safety note**: macOS SIP already blocks deletion of `com.apple.findmy.imagecache`, `com.apple.HomeKit`, `com.apple.Safari`, etc. — scanner should catch `Operation not permitted` gracefully and report them as skipped, not failed.

**Expected savings**: 5-10 GB on typical dev machines.

---

## Summary of Proposed Work

| Gap | Type | Effort | Expected Impact |
|-----|------|--------|-----------------|
| 1. Docker.raw compaction | UX bug fix in existing scanner | S | Fixes misleading reports; reclaims real bytes |
| 2. Claude Desktop VM | New scanner | M | +7-12 GB per affected user |
| 3. External app guidance | New category type | M | Visibility over 20-40 GB of user-owned caches |
| 4. Sparse vs actual display | UI polish | S | Prevents user confusion on `.img` files |
| 5. `~/Library/Caches/*` sweep | Extend existing scanner | M | +5-10 GB per run |

**Combined potential**: an additional ~20-35 GB of reclaimable space surfaced per session, on top of current scanner coverage.

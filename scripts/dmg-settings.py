"""
dmgbuild settings for macweep.

Bypasses Tauri's AppleScript-based DMG background (broken on macOS 26 Tahoe)
by writing .DS_Store programmatically with correct backgroundType=2.

Usage:
    dmgbuild -s scripts/dmg-settings.py "macweep" output.dmg
"""

import os

# Use cwd since dmgbuild is invoked from the project root
PROJECT_ROOT = os.getcwd()
APP_PATH = os.path.join(
    PROJECT_ROOT,
    "src-tauri", "target", "release", "bundle", "macos", "macweep.app",
)
BACKGROUND_PATH = os.path.join(PROJECT_ROOT, "src-tauri", "dmg-background.png")
ICON_PATH = os.path.join(PROJECT_ROOT, "src-tauri", "icons", "icon.icns")

# --- dmgbuild settings ---

format = "UDBZ"  # bzip2 compressed
size = None  # auto-calculate
files = [APP_PATH]
symlinks = {"Applications": "/Applications"}
icon = ICON_PATH
icon_size = 128
background = BACKGROUND_PATH

# Window geometry
window_rect = ((200, 120), (660, 400))

# Icon positions: (name, (x, y))
icon_locations = {
    "macweep.app": (185, 190),
    "Applications": (475, 190),
}

# Hide file extensions and use icon view
text_size = 14

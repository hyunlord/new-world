#!/usr/bin/env bash
set -euo pipefail

echo "[gate] repo: $(pwd)"
echo "[gate] branch: $(git rev-parse --abbrev-ref HEAD)"
git status --porcelain || true

test -f project.godot || { echo "[gate] ERROR: project.godot not found (run in project root)"; exit 1; }

# Resolve Godot executable
# Recommended: export GODOT="/path/to/Godot.app/Contents/MacOS/Godot"
GODOT_BIN="${GODOT:-}"

if [ -z "$GODOT_BIN" ]; then
  # Common path for Godot installed as .app in /Applications
  if [ -x "/Applications/Godot.app/Contents/MacOS/Godot" ]; then
    GODOT_BIN="/Applications/Godot.app/Contents/MacOS/Godot"
  fi
fi

if [ -z "$GODOT_BIN" ]; then
  # Try PATH
  if command -v godot >/dev/null 2>&1; then
    GODOT_BIN="$(command -v godot)"
  fi
fi

if [ -z "$GODOT_BIN" ] || [ ! -x "$GODOT_BIN" ]; then
  echo "[gate] ERROR: Godot executable not found."
  echo "[gate] Set env: export GODOT='/Applications/Godot.app/Contents/MacOS/Godot'"
  exit 1
fi

echo "[gate] GODOT: $GODOT_BIN"
echo "[gate] headless smoke (import + quit)"
"$GODOT_BIN" --headless --path . --quit

echo "[gate] PASS"

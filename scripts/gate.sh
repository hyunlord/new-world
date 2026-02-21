#!/usr/bin/env bash
set -euo pipefail

echo "[gate] repo: $(pwd)"
echo "[gate] branch: $(git rev-parse --abbrev-ref HEAD)"
git status --porcelain || true

test -f project.godot || { echo "[gate] ERROR: project.godot not found"; exit 1; }

# --- Localization JSON validation ---
echo "[gate] checking localization JSON..."
for f in localization/en/*.json localization/ko/*.json; do
  python3 -m json.tool "$f" > /dev/null || { echo "[gate] ERROR: invalid JSON: $f"; exit 1; }
done
echo "[gate] localization JSON: OK"

# --- Hardcoded string scan ---
echo "[gate] scanning for hardcoded strings..."
VIOLATIONS=$(grep -rn '\.text\s*=\s*"' scripts/ui/ 2>/dev/null | grep -v 'Locale\.' || true)
if [ -n "$VIOLATIONS" ]; then
  echo "[gate] ERROR: hardcoded strings found:"
  echo "$VIOLATIONS"
  exit 1
fi
echo "[gate] hardcoded string scan: OK"

# --- Godot resolution ---
GODOT_BIN="${GODOT:-}"
if [ -z "$GODOT_BIN" ]; then
  for candidate in \
    "/Applications/Godot.app/Contents/MacOS/Godot" \
    "$HOME/Downloads/Godot.app/Contents/MacOS/Godot" \
    "$HOME/Applications/Godot.app/Contents/MacOS/Godot"; do
    if [ -x "$candidate" ]; then
      GODOT_BIN="$candidate"
      break
    fi
  done
fi
if [ -z "$GODOT_BIN" ] && command -v godot >/dev/null 2>&1; then
  GODOT_BIN="$(command -v godot)"
fi
if [ -z "$GODOT_BIN" ] || [ ! -x "$GODOT_BIN" ]; then
  echo "[gate] ERROR: Godot not found. export GODOT='/Applications/Godot.app/Contents/MacOS/Godot'"
  exit 1
fi

# --- Godot headless import ---
echo "[gate] GODOT: $GODOT_BIN"
echo "[gate] headless smoke (import + quit)..."
"$GODOT_BIN" --headless --path . --quit
echo "[gate] Godot: OK"

echo "[gate] PASS"
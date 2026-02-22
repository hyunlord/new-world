#!/usr/bin/env bash
set -euo pipefail

echo "[gate] repo: $(pwd)"
echo "[gate] branch: $(git rev-parse --abbrev-ref HEAD)"
git status --porcelain || true

test -f project.godot || { echo "[gate] ERROR: project.godot not found"; exit 1; }

# --- Notion update verification ---
echo "[gate] checking Notion update documentation..."
if [ ! -f PROGRESS.md ]; then
  echo "[gate] ERROR: PROGRESS.md not found. Write it before running gate."
  exit 1
fi
if ! grep -q "Notion Update" PROGRESS.md; then
  echo "[gate] ERROR: No 'Notion Update' section found in PROGRESS.md."
  echo "[gate] Complete Notion update (SKILL.md Part 2) and document it in PROGRESS.md first."
  exit 1
fi
# Check that it's not just the template — must have actual page entries
NOTION_LINES=$(grep -A 20 "Notion Update" PROGRESS.md | grep -c "^|" || true)
if [ "$NOTION_LINES" -lt 1 ]; then
  echo "[gate] ERROR: Notion Update section exists but has no page entries."
  echo "[gate] Fill in the Notion Update table in PROGRESS.md before running gate."
  exit 1
fi
echo "[gate] Notion update: OK"

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
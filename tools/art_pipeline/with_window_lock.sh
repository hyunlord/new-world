#!/usr/bin/env bash
set -euo pipefail
LOCK=/Users/rexxa/work/.worldsim-godot-window.lock
limit="$1"; shift
until mkdir "$LOCK" 2>/dev/null; do
  # Pass 4 may hold the shared lock: age alone does not prove it is stale.
  sleep 30
done
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
bash tools/art_pipeline/run_bounded.sh "$limit" "$@"

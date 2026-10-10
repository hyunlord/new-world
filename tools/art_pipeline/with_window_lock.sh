#!/usr/bin/env bash
set -euo pipefail
LOCK=/Users/rexxa/work/.worldsim-godot-window.lock
limit="$1"; shift
until mkdir "$LOCK" 2>/dev/null; do
  age=$(( $(date +%s) - $(stat -f %m "$LOCK") ))
  if (( age > 1800 )); then
    rmdir "$LOCK" 2>/dev/null || true
    echo "stale window lock removed (age ${age}s)" >&2
    printf '\n- %s stale window lock removed (age %ss).\n' "$(date -u +%FT%TZ)" "$age" >> docs/art/production/DECISIONS.md
  else sleep 60; fi
done
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
bash tools/art_pipeline/run_bounded.sh "$limit" "$@"

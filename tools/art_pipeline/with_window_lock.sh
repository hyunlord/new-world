#!/usr/bin/env bash
set -euo pipefail
LOCK=/Users/rexxa/work/.worldsim-godot-window.lock
limit="$1"; shift
p13_wait_start=$(date +%s)
until mkdir "$LOCK" 2>/dev/null; do
  # Pass 4 may hold the shared lock: age alone does not prove it is stale.
  if [[ -n ${P13_WAIT_CALLBACK:-} ]]; then
    remaining=$(python3 "$P13_WAIT_CALLBACK" --wait-budget "$p13_wait_start") || exit 75
    seconds=$(python3 -c 'import sys; print(min(30.0, float(sys.argv[1])))' "$remaining")
    sleep "$seconds"
  else
    sleep 30
  fi
done
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
if [[ -n ${P13_WAIT_CALLBACK:-} ]]; then
  python3 "$P13_WAIT_CALLBACK" --record-wait "$p13_wait_start" "$(date +%s)" || exit 75
fi
bash tools/art_pipeline/run_bounded.sh "$limit" "$@"

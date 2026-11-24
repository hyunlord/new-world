#!/bin/bash
# matrix.sh <play-root worktree> <out-dir>: the PERF2-0 render matrix in three mux2-lock slices (60 s apart).
set -uo pipefail
export PATH=/opt/homebrew/bin:$PATH
ROOT=${1:?worktree}; OUT=${2:?out}; H="$(cd "$(dirname "$0")/../../.." && pwd)"
# Queue behind other lock holders: retry the non-blocking acquisition every 2 min (never preempt or signal owners).
lock() {
  local err; err=$(mktemp)
  while :; do
    python3 "$H/tools/v8/mux2_gate_lock.py" -- bash "$H/tools/v8/perf2/render.sh" "$ROOT" "$OUT" "$@" 2> "$err"
    grep -q '^MUX2_GATE_LOCK_FAIL .*Resource temporarily unavailable' "$err" || break
    echo "$(date -Iseconds) lock busy, retry in 120 s" >> "$OUT/runs.log"; sleep 120
  done
  cat "$err" >&2; rm -f "$err"
}
lock stress_1000-d30:0.5 stress_1000-d30:1 stress_1000-d30:3 stress_1000-d30:4 stress_1000-day:1 stress_1000-day:4
sleep 60
lock stress_village_300-day:0.5 stress_village_300-day:1 stress_village_300-day:3 stress_village_300-day:4 \
  stress_1000-night:1 stress_1000-night:3 stress_1000-winter:1 stress_1000-winter:3 stress_village_300-night:1 stress_village_300-winter:1
sleep 60
lock "stress_1000-d30:1:--freeze 1" "stress_1000-d30:0.5:--freeze 1" "stress_1000-d30:1:--freeze 1 --hide Env:shadows" \
  "stress_1000-d30:0.5:--freeze 1 --hide Env:shadows" "stress_1000-d30:4:--freeze 1" "stress_1000-d30:1:--paused 1" \
  "stress_1000-winter:1:--freeze 1" "stress_1000-night:1:--freeze 1"
echo MATRIX_DONE >> "$OUT/runs.log"

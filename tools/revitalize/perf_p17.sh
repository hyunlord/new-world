#!/usr/bin/env bash
# usage: perf_p17.sh <out_dir> <final_bin_dir> <baseline_bin_dir> <a|b>
# Run inside quiet_lock_p17.sh. Each stage compares three interleaved same-build
# repeats (a: v21/v20; b: v22/v21), then before/after quick and three-size slopes.
set -uo pipefail
(( $# == 4 )) || { echo 'usage: perf_p17.sh <out> <final_bin> <baseline_bin> <a|b>' >&2; exit 2; }
OUT="$1"; FINAL="$2"; BASE="$3"; STAGE="$4"
case "$STAGE" in a) OLD=game_v20; NEW=game_v21 ;; b) OLD=game_v21; NEW=game_v22 ;; *) echo 'stage must be a or b' >&2; exit 2 ;; esac
[[ -n "${P17_QUIET_PID:-}" && "${P17_QUIET_OWNER:-}" == 'pass17-sim (Codex, new-world-pass4)' ]] || { echo 'timed performance requires quiet_lock_p17.sh' >&2; exit 2; }
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
case "$OUT/" in "$ROOT/docs/revitalize/"*) ;; "$ROOT/"*) echo 'performance outputs must be outside the repository or in docs/revitalize' >&2; exit 2 ;; esac
status=0
run() {
    bash "$SCRIPT_DIR/wait_mac_p17.sh" "$OUT/RUNLOG.txt" "stage $STAGE performance" || return 1
    local info=/Users/rexxa/work/.mac_quiet.lock/info.txt
    [[ "${P17_QUIET_PID:-}" =~ ^[0-9]+$ ]] && kill -0 "$P17_QUIET_PID" 2>/dev/null &&
        grep -Fxq -- "owner=$P17_QUIET_OWNER" "$info" && grep -Fxq -- "pid=$P17_QUIET_PID" "$info" &&
        grep -Fxq -- "started=${P17_QUIET_STARTED:-}" "$info" || { echo 'owned quiet lock is no longer held' >&2; return 1; }
    printf '%s contended=%s ' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${PERF_CONTENDED:-0}" >> "$OUT/runs.log"
    printf '%q ' "$@" >> "$OUT/runs.log"; printf '\n' >> "$OUT/runs.log"
    "$@" >> "$OUT/runs.log" 2>&1 || status=1
}
for repeat in 1 2 3; do
    for rules in "$OLD" "$NEW"; do
        run "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules "$rules" --out "$OUT/a/r$repeat" || exit 1
    done
done
run "$BASE/worldsim_perf" quick --out "$OUT/quick_base" || exit 1
run "$FINAL/worldsim_perf" quick --out "$OUT/quick_final" || exit 1
for rules in "$NEW" "$OLD"; do
    run "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed --rules "$rules" --out "$OUT/b_$rules" || exit 1
done
echo "status $status" >> "$OUT/runs.log"
exit "$status"

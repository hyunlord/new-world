#!/usr/bin/env bash
# usage: perf_attr_p18.sh OUT PLAIN_BIN SUB_BIN SHA
# Pass 18 section 6.2, run inside quiet_lock_p18.sh: for three interleaved
# repeats, each of game_v20/v21/v22 runs the 500-agent fed stress day on the
# plain build (subprofile off) and then on the subprofile build. The plain
# runs give the reference timings; the pairs give the feature's overhead.
set -uo pipefail
(( $# == 4 )) || { echo 'usage: perf_attr_p18.sh OUT PLAIN_BIN SUB_BIN SHA' >&2; exit 2; }
OUT="$1"; PLAIN="$2"; SUB="$3"; SHA="$4"
[[ -n "${P18_QUIET_PID:-}" && "${P18_QUIET_OWNER:-}" == 'pass18-sim (Claude Code, new-world-pass4)' ]] || { echo 'timed performance requires quiet_lock_p18.sh' >&2; exit 2; }
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
[[ ! -e "$OUT/runs.log" ]] || { echo 'refusing to reuse performance evidence directory' >&2; exit 2; }
{
    echo "sha $SHA"
    shasum -a 256 "$PLAIN/worldsim_perf" "$SUB/worldsim_perf"
    echo "contended ${PERF_CONTENDED:-0}"
    echo "quiet $(tr '\n' ' ' < /Users/rexxa/work/.mac_quiet.lock/info.txt)"
} > "$OUT/provenance.txt"
status=0
run() {
    local build="$1" bin="$2" rules="$3" repeat="$4" code=0
    bash "$SCRIPT_DIR/wait_mac_p18.sh" "$OUT/RUNLOG.txt" 'p18-1 attribution' || return 1
    grep -Fxq -- "pid=$P18_QUIET_PID" /Users/rexxa/work/.mac_quiet.lock/info.txt || { echo 'owned quiet lock is no longer held' >&2; return 1; }
    printf '%s contended=%s %s %s r%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${PERF_CONTENDED:-0}" "$build" "$rules" "$repeat" >> "$OUT/runs.log"
    WORLDSIM_GIT_SHA="$SHA" "$bin/worldsim_perf" stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules "$rules" --out "$OUT/$build/r$repeat" >> "$OUT/runs.log" 2>&1 || code=$?
    (( code == 0 )) || status=1
    return 0
}
for repeat in 1 2 3; do
    for rules in game_v20 game_v21 game_v22; do
        run plain "$PLAIN" "$rules" "$repeat" || exit 1
        run sub "$SUB" "$rules" "$repeat" || exit 1
    done
done
echo "status $status" >> "$OUT/runs.log"
exit "$status"

#!/usr/bin/env bash
# usage: hold_p18_part2.sh OUT_ROOT QUIET_LOG P2_SHA BASE_SHA BIN_ROOT [c]
# One quiet-lock hold (run inside quiet_lock_p18.sh) for a Part 2 candidate:
# its attribution (perf_attr_p18.sh on BIN_ROOT/plain and BIN_ROOT/sub), then
# perf_p18.sh stages a (v21/v20) and b (v22/v21) with final = BIN_ROOT/plain;
# with a trailing `c`, also stage c (v23/v22) for Part 3.
set -uo pipefail
(( $# == 5 || $# == 6 )) || { echo 'usage: hold_p18_part2.sh OUT_ROOT QUIET_LOG P2_SHA BASE_SHA BIN_ROOT [c]' >&2; exit 2; }
OUT="$1"; QUIET_LOG="$2"; P2="$3"; BASE="$4"; BIN="$5"; STAGE_C="${6:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_BIN=/Users/rexxa/work/new-world-p18-base/rust/target/release
status=0
bash "$SCRIPT_DIR/perf_attr_p18.sh" "$OUT/attr" "$BIN/plain" "$BIN/sub" "$P2" || status=1
for stage in a b ${STAGE_C:+c}; do
    bash "$SCRIPT_DIR/perf_p18.sh" "$OUT/$stage" "$BIN/plain" "$BASE_BIN" "$stage" "$P2" "$BASE" "$QUIET_LOG" || status=1
done
exit "$status"

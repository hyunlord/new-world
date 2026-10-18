#!/usr/bin/env bash
# usage: hold_p18_parts12.sh OUT_ROOT QUIET_LOG P1_SHA P2_SHA BASE_SHA
# One quiet-lock hold (run inside quiet_lock_p18.sh) for sections 6.2 and 7.4:
#   1. Part 1 attribution: perf_attr_p18.sh on the Part 1 plain/subprofile builds
#   2. Part 2 attribution: the same on the Part 2 builds (after the fixes)
#   3. R5 re-judgement: perf_p18.sh stage a (game_v21 vs game_v20), final = Part 2
#   4. F8 re-judgement: perf_p18.sh stage b (game_v22 vs game_v21), final = Part 2
# Binaries are the frozen copies under /Users/rexxa/work/p18-data.
set -uo pipefail
(( $# == 5 )) || { echo 'usage: hold_p18_parts12.sh OUT_ROOT QUIET_LOG P1_SHA P2_SHA BASE_SHA' >&2; exit 2; }
OUT="$1"; QUIET_LOG="$2"; P1="$3"; P2="$4"; BASE="$5"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA=/Users/rexxa/work/p18-data
status=0
bash "$SCRIPT_DIR/perf_attr_p18.sh" "$OUT/p18-1_attr" "$DATA/bin_p1/plain" "$DATA/bin_p1/sub" "$P1" || status=1
bash "$SCRIPT_DIR/perf_attr_p18.sh" "$OUT/p18-2_attr" "$DATA/bin_p2/plain" "$DATA/bin_p2/sub" "$P2" || status=1
bash "$SCRIPT_DIR/perf_p18.sh" "$OUT/p18-2_a" "$DATA/bin_p2/plain" /Users/rexxa/work/new-world-p18-base/rust/target/release a "$P2" "$BASE" "$QUIET_LOG" || status=1
bash "$SCRIPT_DIR/perf_p18.sh" "$OUT/p18-2_b" "$DATA/bin_p2/plain" /Users/rexxa/work/new-world-p18-base/rust/target/release b "$P2" "$BASE" "$QUIET_LOG" || status=1
exit "$status"

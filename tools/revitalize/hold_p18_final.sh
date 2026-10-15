#!/usr/bin/env bash
# usage: hold_p18_final.sh OUT_ROOT QUIET_LOG P2_SHA P3_SHA BASE_SHA
# The single official quiet-lock hold (run inside quiet_lock_p18.sh) of
# sections 7.4 and 8.5 G6, after the final Part 2 build's I1-I6 proof:
#   attr  perf_attr_p18.sh on the final Part 2 plain/subprofile builds
#   a     perf_p18.sh stage a (game_v21 vs game_v20), final = Part 2   (R5)
#   b     perf_p18.sh stage b (game_v22 vs game_v21), final = Part 2   (F8)
#   c     perf_p18.sh stage c (game_v23 vs game_v22), final = Part 3   (G6)
# Each stage also runs quick on the base and final builds and the three-size
# slope runs. Binaries are the frozen copies under /Users/rexxa/work/p18-data.
set -uo pipefail
(( $# == 5 )) || { echo 'usage: hold_p18_final.sh OUT_ROOT QUIET_LOG P2_SHA P3_SHA BASE_SHA' >&2; exit 2; }
OUT="$1"; QUIET_LOG="$2"; P2="$3"; P3="$4"; BASE="$5"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA=/Users/rexxa/work/p18-data
BASE_BIN=/Users/rexxa/work/new-world-p18-base/rust/target/release
status=0
bash "$SCRIPT_DIR/perf_attr_p18.sh" "$OUT/attr" "$DATA/bin_p2f/plain" "$DATA/bin_p2f/sub" "$P2" || status=1
bash "$SCRIPT_DIR/perf_p18.sh" "$OUT/a" "$DATA/bin_p2f/plain" "$BASE_BIN" a "$P2" "$BASE" "$QUIET_LOG" || status=1
bash "$SCRIPT_DIR/perf_p18.sh" "$OUT/b" "$DATA/bin_p2f/plain" "$BASE_BIN" b "$P2" "$BASE" "$QUIET_LOG" || status=1
bash "$SCRIPT_DIR/perf_p18.sh" "$OUT/c" "$DATA/bin_p3f" "$BASE_BIN" c "$P3" "$BASE" "$QUIET_LOG" || status=1
exit "$status"

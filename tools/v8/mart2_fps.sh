#!/usr/bin/env bash
# usage: mart2_fps.sh <out_dir> <day20 .wsave> [warmup_s] [seconds]
#
# M-ART2 FPS runs (goal §4, MART2_DECISIONS D4): the seed 0 colony at day 20 at 1×, ½×, ¼× (budget ≥ 120 FPS) and the
# 1,000-person stress world at ½× and ¼× with LOD (budget ≥ 60 FPS), windowed 1920×1080, vsync off, each run its own
# Godot process (tools/v8/mart2_fps.gd). The release sim-bridge library is copied to the debug path Godot loads
# (its SHA-256 checked) and the debug library is restored on exit, whatever happens. Writes <out_dir>/fps.jsonl (one
# MART2_FPS line per run with the 1-minute load before and after) and prints it. A run whose load before was ≥ 8 is
# marked contended.
set -euo pipefail
OUT="${1:?out dir}"
SAVE="${2:?day 20 save}"
WARMUP="${3:-30}"
SECONDS_RUN="${4:-60}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source tools/v8/godot_bin.sh
mkdir -p "$OUT"
(cd rust && cargo build --release -p sim-bridge >/dev/null 2>&1)
DEBUG_LIB="rust/target/debug/libsim_bridge.dylib"
RELEASE_LIB="rust/target/release/libsim_bridge.dylib"
# macOS kills a process that maps a signed dylib overwritten in place (same inode; exit 247, no output): always
# remove the target first so the copy is a new file.
cp "$DEBUG_LIB" "$OUT/libsim_bridge.debug.backup"
restore() { rm -f "$DEBUG_LIB"; cp "$OUT/libsim_bridge.debug.backup" "$DEBUG_LIB"; }
trap restore EXIT
rm -f "$DEBUG_LIB"
cp "$RELEASE_LIB" "$DEBUG_LIB"
want=$(shasum -a 256 "$RELEASE_LIB" | cut -d' ' -f1)
have=$(shasum -a 256 "$DEBUG_LIB" | cut -d' ' -f1)
[[ "$want" == "$have" ]] || { echo "release library copy differs"; exit 1; }
echo "{\"release_lib_sha256\": \"$want\"}" > "$OUT/fps.jsonl"
load1() { sysctl -n vm.loadavg | awk '{print $2}'; }
run() {
    local label="$1"; shift
    local before after line
    before=$(load1)
    line=$(bash tools/v8/run_bounded.sh $((WARMUP + SECONDS_RUN + 120)) "$GODOT_BIN" --path . --rendering-driver opengl3 \
        --resolution 1920x1080 --script res://tools/v8/mart2_fps.gd -- "$@" --warmup "$WARMUP" --seconds "$SECONDS_RUN" \
        2>/dev/null | grep '^MART2_FPS ' | sed 's/^MART2_FPS //' || true)
    after=$(load1)
    if [[ -z "$line" ]]; then line='{"error": "no result"}'; fi
    python3 -c "import json,sys; d=json.loads(sys.argv[1]); d.update({'run': sys.argv[2], 'load_before': float(sys.argv[3]), 'load_after': float(sys.argv[4]), 'contended': float(sys.argv[3]) >= 8}); print(json.dumps(d))" \
        "$line" "$label" "$before" "$after" | tee -a "$OUT/fps.jsonl"
}
run colony_1x --load "$SAVE" --zoom 1
run colony_half --load "$SAVE" --zoom 0.5
run colony_quarter --load "$SAVE" --zoom 0.25
run stress1000_half --stress 1000 --zoom 0.5
run stress1000_quarter --stress 1000 --zoom 0.25

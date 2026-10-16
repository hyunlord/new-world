#!/usr/bin/env bash
# usage: identity_p18.sh <release_bin_dir> <summary_out_dir> <data_dir> <git_sha> [--i56-only|--no-i56]
# Pass 17's identity procedure with I6 added. I1/I3/I4/I5/I6 are sequential
# probes with the identical p16/p17 flags and four workers total.
# I2 records game_v19 seeds 12000..12002 for 120 days, every 60 ticks, workers=4.
# Default runs I1, I3, I4, I2, I5 (game_v21) and I6 (game_v22); --i56-only runs
# only I5/I6; --no-i56 runs only I1-I4.
# Raw probe/recorder data never enters the repository; only summaries and array
# sha256 lists are copied to <summary_out_dir>. Existing raw runs are preserved.
set -euo pipefail
(( $# >= 4 && $# <= 5 )) || { echo 'usage: identity_p18.sh <bin> <summaries> <data> <sha> [--i56-only|--no-i56]' >&2; exit 2; }
BIN="$1"; OUT="$2"; DATA="$3"; SHA="$4"; MODE="${5:-all}"
case "$MODE" in all|--i56-only|--no-i56) ;; *) echo "unknown mode: $MODE" >&2; exit 2 ;; esac
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
mkdir -p "$OUT" "$DATA"
DATA="$(cd "$DATA" && pwd -P)"
case "$DATA/" in "$ROOT/"*) echo 'raw identity data must be outside the repository' >&2; exit 2 ;; esac
export WORLDSIM_GIT_SHA="$SHA" CARGO_BUILD_JOBS=6
FLAGS=(--seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism)
probe() {
    local label="$1" scene="$2"
    [[ ! -e "$DATA/probe/$label" ]] || { echo "raw run already exists: $DATA/probe/$label" >&2; exit 2; }
    bash "$SCRIPT_DIR/wait_mac_p18.sh" "$DATA/RUNLOG.txt" "identity $label"
    printf '%s %s start\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$label" >> "$DATA/RUNLOG.txt"
    "$BIN/worldsim_probe" --scene "$scene" "${FLAGS[@]}" --out "$DATA/probe/$label" > "$DATA/$label.log" 2>&1
    mkdir -p "$OUT/$label"
    cp "$DATA/probe/$label/summary.json" "$OUT/$label/summary.json"
    printf '%s %s end\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$label" >> "$DATA/RUNLOG.txt"
}
if [[ "$MODE" != --i56-only ]]; then
    for entry in i1_v19:game_v19 i3_v16:game_v16 i4_v20:game_v20; do
        probe "${entry%%:*}" "${entry#*:}"
    done
    [[ ! -e "$DATA/i2" ]] || { echo "raw run already exists: $DATA/i2" >&2; exit 2; }
    bash "$SCRIPT_DIR/wait_mac_p18.sh" "$DATA/RUNLOG.txt" 'identity I2 recorder'
    printf '%s I2 start\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$DATA/RUNLOG.txt"
    "$BIN/worldsim_wm" --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 4 --out "$DATA/i2" > "$DATA/i2.log" 2>&1
    (cd "$DATA/i2" && find . -type f \( -name '*.npy' -o -name interventions.json \) | LC_ALL=C sort | while IFS= read -r file; do shasum -a 256 "$file"; done) > "$OUT/i2_sha256.txt"
    [[ -s "$OUT/i2_sha256.txt" ]] || { echo 'I2 produced no arrays' >&2; exit 1; }
    printf '%s I2 end\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$DATA/RUNLOG.txt"
fi
if [[ "$MODE" != --no-i56 ]]; then
    probe i5_v21 game_v21
    probe i6_v22 game_v22
fi

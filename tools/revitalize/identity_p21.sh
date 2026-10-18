#!/usr/bin/env bash
# usage: identity_p21.sh <release_bin_dir> <summary_out_dir> <data_dir> <git_sha> [--i5678-only|--no-i5678]
# Pass 21 (brief 4.1): identity_p20.sh with I9 (game_v25) added; --i5678-only runs I5-I9.
# Pass 19's identity procedure (identity_p19.sh) with I8 (game_v24) added. I1/I3/I4/I5/I6/I7/I8 are
# sequential probes with the identical p16-p18 flags and four workers total.
# I2 records game_v19 seeds 12000..12002 for 120 days, every 60 ticks, workers=4.
# Default runs I1, I3, I4, I2, I5 (game_v21), I6 (game_v22), I7 (game_v23) and
# I8 (game_v24); --i5678-only runs only I5-I8; --no-i5678 runs only I1-I4.
# Every probe also writes --chronicle-out (it never changes the summary); the
# chronicle files are compared too (brief 4.1 requires I7/I8), and their sha256 list is
# copied to <summary_out_dir>.
# Pass 21 (brief 1.2, as Pass 20): each probe and the recorder run in their own process
# group under run_watched_p21.sh (SIGSTOP while a foreign quiet lock exists).
# Raw probe/recorder data never enters the repository; only summaries and array
# sha256 lists are copied to <summary_out_dir>. Existing raw runs are preserved.
set -euo pipefail
(( $# >= 4 && $# <= 5 )) || { echo 'usage: identity_p21.sh <bin> <summaries> <data> <sha> [--i5678-only|--no-i5678]' >&2; exit 2; }
BIN="$1"; OUT="$2"; DATA="$3"; SHA="$4"; MODE="${5:-all}"
case "$MODE" in all|--i5678-only|--no-i5678) ;; *) echo "unknown mode: $MODE" >&2; exit 2 ;; esac
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
mkdir -p "$OUT" "$DATA"
DATA="$(cd "$DATA" && pwd -P)"
case "$DATA/" in "$ROOT/"*) echo 'raw identity data must be outside the repository' >&2; exit 2 ;; esac
export WORLDSIM_GIT_SHA="$SHA" CARGO_BUILD_JOBS=6
FLAGS=(--seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism)
# Run "$@" in a watched process group with stdout/stderr to $1.
watched() {
    local logfile="$1" purpose="$2"; shift 2
    bash "$SCRIPT_DIR/run_watched_p21.sh" "$DATA/RUNLOG.txt" "$purpose" \
        bash -c 'out="$1"; shift; exec "$@" > "$out" 2>&1' _ "$logfile" "$@"
}
probe() {
    local label="$1" scene="$2"
    [[ ! -e "$DATA/probe/$label" ]] || { echo "raw run already exists: $DATA/probe/$label" >&2; exit 2; }
    printf '%s %s start\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$label" >> "$DATA/RUNLOG.txt"
    watched "$DATA/$label.log" "identity $label" \
        "$BIN/worldsim_probe" --scene "$scene" "${FLAGS[@]}" --out "$DATA/probe/$label" --chronicle-out "$DATA/chronicle/$label"
    mkdir -p "$OUT/$label"
    cp "$DATA/probe/$label/summary.json" "$OUT/$label/summary.json"
    (cd "$DATA/chronicle/$label" && find . -type f -name 'chronicle_*.json' | LC_ALL=C sort | while IFS= read -r file; do shasum -a 256 "$file"; done) > "$OUT/$label/chronicle_sha256.txt"
    printf '%s %s end\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$label" >> "$DATA/RUNLOG.txt"
}
if [[ "$MODE" != --i5678-only ]]; then
    for entry in i1_v19:game_v19 i3_v16:game_v16 i4_v20:game_v20; do
        probe "${entry%%:*}" "${entry#*:}"
    done
    [[ ! -e "$DATA/i2" ]] || { echo "raw run already exists: $DATA/i2" >&2; exit 2; }
    printf '%s I2 start\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$DATA/RUNLOG.txt"
    watched "$DATA/i2.log" 'identity I2 recorder' \
        "$BIN/worldsim_wm" --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 4 --out "$DATA/i2"
    (cd "$DATA/i2" && find . -type f \( -name '*.npy' -o -name interventions.json \) | LC_ALL=C sort | while IFS= read -r file; do shasum -a 256 "$file"; done) > "$OUT/i2_sha256.txt"
    [[ -s "$OUT/i2_sha256.txt" ]] || { echo 'I2 produced no arrays' >&2; exit 1; }
    printf '%s I2 end\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$DATA/RUNLOG.txt"
fi
if [[ "$MODE" != --no-i5678 ]]; then
    probe i5_v21 game_v21
    probe i6_v22 game_v22
    probe i7_v23 game_v23
    probe i8_v24 game_v24
    probe i9_v25 game_v25
fi

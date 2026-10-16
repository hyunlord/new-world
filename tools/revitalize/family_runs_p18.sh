#!/usr/bin/env bash
# usage: family_runs_p18.sh <release_bin_dir> <data_dir> <git_sha> tuning|eval [scene...]
# Pass 18 sections 8.4/8.6: traced paired cohorts with Pass 17 Stage B's flags
# (hourly clock samples, --agent-trace, determinism twin, four workers),
# game_v23 then game_v22 unless scenes are given. Tuning = 19100..19115 for 60
# days, evaluation = 19000..19031 for 120 days. Raw data stays outside the repo.
set -euo pipefail
(( $# >= 4 )) || { echo 'usage: family_runs_p18.sh <bin> <data> <sha> tuning|eval [scene...]' >&2; exit 2; }
BIN="$1"; DATA="$2"; SHA="$3"; MODE="$4"; shift 4
SCENES=("$@"); (( ${#SCENES[@]} )) || SCENES=(game_v23 game_v22)
case "$MODE" in tuning) SEEDS=19100..19116; DAYS=60 ;; eval) SEEDS=19000..19032; DAYS=120 ;; *) echo 'mode: tuning|eval' >&2; exit 2 ;; esac
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
mkdir -p "$DATA"
DATA="$(cd "$DATA" && pwd -P)"
case "$DATA/" in "$ROOT/"*) echo 'raw family data must be outside the repository' >&2; exit 2 ;; esac
HOURS=00:00,01:00,02:00,03:00,04:00,05:00,06:00,07:00,08:00,09:00,10:00,11:00,12:00,13:00,14:00,15:00,16:00,17:00,18:00,19:00,20:00,21:00,22:00,23:00
export WORLDSIM_GIT_SHA="$SHA" CARGO_BUILD_JOBS=6
(cd "$BIN" && shasum -a 256 worldsim_probe) > "$DATA/binary.sha256"
for scene in "${SCENES[@]}"; do
    [[ ! -e "$DATA/$scene" ]] || { echo "raw run already exists: $DATA/$scene" >&2; exit 2; }
    bash "$SCRIPT_DIR/wait_mac_p18.sh" "$DATA/RUNLOG.txt" "family $MODE $scene"
    printf '%s %s start\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$scene" >> "$DATA/RUNLOG.txt"
    "$BIN/worldsim_probe" --scene "$scene" --seeds "$SEEDS" --days "$DAYS" --sample-every 120 --clock-samples "$HOURS" \
        --workers 4 --check-determinism --agent-trace --out "$DATA/$scene" > "$DATA/$scene.log" 2>&1
    printf '%s %s end\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$scene" >> "$DATA/RUNLOG.txt"
done

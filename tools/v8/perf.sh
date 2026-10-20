#!/usr/bin/env bash
# usage: perf.sh <bin_dir> <out_dir> <base|candidate>
#
# V8-0 performance comparison (goal 5.3, 11.1 step 12; report only).
#   natural: Pass 21's natural measurement (docs at v7-final-art:
#            docs/revitalize/PERF_BUDGET.md, "Pass 21"): `worldsim_perf natural
#            --scene game_v26 --days 120` on seeds 19600, 19603, 19605, 19606,
#            three interleaved repeats, run sequentially.
#   quick:   `worldsim_perf quick` as the binary defines it. At base this is
#            the v19 stress world (seed 15100, 500 founders, 64x64, fed, 1 day);
#            from V8-0 Part 3 it is the same world under the baseline rules.
#   stress:  `stress --rules game_v26` on that same world (seed 15100, n 500,
#            side 64, 1 day, fed) — the base-side twin of the V8 `quick`.
# perf_summary.py reduces the JSON files to <out_dir>/perf_summary.json.
set -euo pipefail
(( $# == 3 )) || { echo 'usage: perf.sh <bin_dir> <out_dir> <base|candidate>' >&2; exit 2; }
BIN="$1"; OUT="$2"; ROLE="$3"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[[ ! -e "$OUT/natural_r1" ]] || { echo "refusing to reuse $OUT" >&2; exit 2; }
mkdir -p "$OUT"
{ echo "role $ROLE"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; echo "load $(uptime | sed 's/.*load averages*: //')"; } > "$OUT/RUNLOG.txt"
for repeat in 1 2 3; do
    for seed in 19600 19603 19605 19606; do
        "$BIN/worldsim_perf" natural --scene game_v26 --seeds "$seed..$((seed + 1))" --days 120 --out "$OUT/natural_r$repeat" >> "$OUT/perf.log" 2>&1
    done
    echo "natural repeat $repeat done $(date '+%H:%M:%S') load $(uptime | sed 's/.*load averages*: //')" >> "$OUT/RUNLOG.txt"
done
"$BIN/worldsim_perf" quick --out "$OUT/quick" >> "$OUT/perf.log" 2>&1
if [[ "$ROLE" == base ]]; then
    "$BIN/worldsim_perf" stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules game_v26 --out "$OUT/stress_v26" >> "$OUT/perf.log" 2>&1
fi
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') load $(uptime | sed 's/.*load averages*: //')" >> "$OUT/RUNLOG.txt"
python3 "$SCRIPT_DIR/perf_summary.py" "$OUT" > "$OUT/perf_summary.json"
cat "$OUT/perf_summary.json"

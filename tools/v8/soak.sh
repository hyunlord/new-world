#!/usr/bin/env bash
# usage: soak.sh <out_dir> [<probe>]
#
# The economy soak fixture (M3 goal 3.4; gate step `soak`): the colony probe on seeds 0..4 for 20 days with
# tools/v8/m3_soak_script.json and --check-determinism, then checks every seed's summary: 0 ledger and 0
# reservation violation ticks, 0 deaths of any cause, every scripted workshop built and crafts made (so the
# check cannot pass on an idle colony). Prints SOAK PASS or SOAK FAIL lines.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${1:?usage: soak.sh <out_dir> [<probe>]}"
PROBE="${2:-$ROOT/rust/target/release/worldsim_probe}"
if [[ $# -lt 2 ]]; then
    (cd "$ROOT/rust" && cargo build --release -p sim-test --bin worldsim_probe) || exit 1
fi
mkdir -p "$OUT"
"$PROBE" --scene colony --seeds 0..4 --days 20 --workers 4 --commands "$ROOT/tools/v8/m3_soak_script.json" \
    --check-determinism --out "$OUT" || { echo "SOAK FAIL probe exit"; exit 1; }
python3 - "$OUT/summary.json" <<'PY'
import json, sys
seeds = json.load(open(sys.argv[1]))["seeds"]
bad = []
if len(seeds) != 4:
    bad.append(f"expected 4 seeds, got {len(seeds)}")
for s in seeds:
    sid = s["seed"]
    if s["ledger_violation_ticks"] or s["reservation_violation_ticks"]:
        bad.append(f"seed {sid}: ledger {s['ledger_violation_ticks']} reservations {s['reservation_violation_ticks']} {s['first_violation']}")
    deaths = sum(s["deaths"].values())
    if deaths:
        bad.append(f"seed {sid}: {deaths} deaths {s['deaths']}")
    if s["workshops_built"] < 2 or s["crafts_done"] == 0 or s["trees_felled"] == 0:
        bad.append(f"seed {sid}: idle colony (workshops {s['workshops_built']}, crafts {s['crafts_done']}, felled {s['trees_felled']})")
    print(f"seed {sid}: crafts {s['crafts_done']} felled {s['trees_felled']} workshops {s['workshops_built']} deaths {deaths} alive {s['alive_end']}")
for b in bad:
    print("SOAK FAIL", b)
if bad:
    sys.exit(1)
print("SOAK PASS (4 seeds x 20 days: ledger exact, reservations exact, 0 deaths, determinism)")
PY

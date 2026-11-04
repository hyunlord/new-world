#!/usr/bin/env bash
# usage: soak.sh <out_dir> [<probe>] [<paired-base-probe>]
#
# The economy soak fixture (M3 goal 3.4; gate step `soak`): the colony probe on seeds 0..4 for 20 days with
# tools/v8/soak_script.json and --check-determinism, then checks every seed's summary: 0 ledger and 0
# reservation violation ticks, 0 deaths of any cause, 0 KnowledgeLost (M4), every scripted workshop built and crafts made (so the
# check cannot pass on an idle colony). Prints SOAK PASS or SOAK FAIL lines. M5: the storyteller runs without beasts
# (`--story no-beasts`): the soak checks the economy under weather and illness, the `story` gate step the packs.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${1:?usage: soak.sh <out_dir> [<probe>] [<paired-base-probe>]}"
PROBE="${2:-$ROOT/rust/target/release/worldsim_probe}"
if [[ $# -lt 2 ]]; then
    (cd "$ROOT/rust" && cargo build --release -p sim-test --bin worldsim_probe) || exit 1
fi
BASE_PROBE="${3:-}"
if [[ -n "$BASE_PROBE" ]]; then
    [[ -n "${READ_GATE_CONFIG:-}" && " ${READ_CHANGED_FIXTURES:-} " == *" soak_economy "* ]] || { echo "SOAK FAIL paired mode requires declared soak_economy and READ config"; exit 1; }
    python3 - "$ROOT/tools/v8" "$READ_GATE_CONFIG" "$BASE_PROBE" <<'PYVERIFY'
import sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
from read_gate_checks import load_config, verify_binary
config = load_config(sys.argv[2])
if Path(sys.argv[3]).resolve() != Path(config["base_probe"]).resolve():
    raise ValueError("paired soak BASE path does not match pinned config")
verify_binary(sys.argv[3], config["base_probe_sha256"])
PYVERIFY
    [[ $? == 0 ]] || exit 1
fi
mkdir -p "$OUT"
"$PROBE" --scene colony --seeds 0..4 --days 20 --workers 4 --commands "$ROOT/tools/v8/soak_script.json" \
    --story no-beasts --check-determinism --out "$OUT" || { echo "SOAK FAIL probe exit"; exit 1; }
if [[ -n "$BASE_PROBE" ]]; then
    "$BASE_PROBE" --scene colony --seeds 0..4 --days 20 --workers 4 --commands "$ROOT/tools/v8/soak_script.json" \
        --story no-beasts --check-determinism --out "$OUT/base" || { echo "SOAK FAIL BASE probe exit"; exit 1; }
fi
python3 - "$OUT/summary.json" "$ROOT/tools/v8" "${BASE_PROBE:+$OUT/base/summary.json}" <<'PY'
import sys
sys.path.insert(0, sys.argv[2])
from read_gate_checks import compare_fixture, load_json
head = load_json(sys.argv[1])
base = load_json(sys.argv[3]) if sys.argv[3] else None
bad = []
for label, summary in ([("BASE", base), ("HEAD", head)] if base is not None else [("HEAD", head)]):
    seeds = summary["seeds"]
    if len(seeds) != 4:
        bad.append(f"{label}: expected 4 seeds, got {len(seeds)}")
    for s in seeds:
        sid = s["seed"]
        if s["ledger_violation_ticks"] or s["reservation_violation_ticks"]:
            bad.append(f"{label} seed {sid}: ledger {s['ledger_violation_ticks']} reservations {s['reservation_violation_ticks']} {s['first_violation']}")
        deaths = sum(s["deaths"].values())
        if deaths and base is None:
            bad.append(f"seed {sid}: {deaths} deaths {s['deaths']}")
        if s["workshops_built"] < 2 or s["crafts_done"] == 0 or s["trees_felled"] == 0:
            bad.append(f"{label} seed {sid}: idle colony (workshops {s['workshops_built']}, crafts {s['crafts_done']}, felled {s['trees_felled']})")
        lost = s.get("m4", {}).get("lost")
        if lost is None:
            bad.append(f"{label} seed {sid}: no m4 block")
        elif lost and base is None:
            bad.append(f"seed {sid}: KnowledgeLost {lost}")
        print(f"{label} seed {sid}: crafts {s['crafts_done']} felled {s['trees_felled']} workshops {s['workshops_built']} deaths {deaths} alive {s['alive_end']} lost {len(lost or [])}")
for b in bad:
    print("SOAK FAIL", b)
if bad:
    sys.exit(1)
if base is not None:
    compare_fixture("soak_economy", base, head, None, {"soak_economy"})
    safety = "BASE-relative survival/KnowledgeLost"
else:
    safety = "0 deaths, 0 KnowledgeLost"
print(f"SOAK PASS (4 seeds x 20 days: ledger exact, reservations exact, activity, {safety}, determinism)")
PY

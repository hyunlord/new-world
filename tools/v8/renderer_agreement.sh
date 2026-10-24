#!/usr/bin/env bash
# usage: renderer_agreement.sh <log_dir>
#
# M1.5 goal 8.1 (contract clause C2 "view independence"; future gate step `renderer_agreement`): the text
# renderer and the Godot view must draw the people on the same tiles. On seed 100 with the U-script
# (tools/v8/u_script.json) at tick 900:
#   1. build the bridge Godot loads (debug) and the release probe;
#   2. the probe writes the concrete commands the U-script resolves to (--commands-out) and the text
#      renderer's person tiles (--ascii --tick 900 --people-json; one entry per person);
#   3. tools/v8/renderer_agreement.gd boots res://scenes/main.tscn headless, replays those concrete commands
#      through the bridge's cmd_* methods, advances to tick 900 and dumps the tiles of the people the agent
#      layer draws;
#   4. the two multisets of tiles are compared, and (M-ART2) every person's look and face variant by AgentId
#      (appearance agreement: the text renderer's `appearance` rows = what the agent layer draws).
# Prints `RENDERER_AGREEMENT PASS n=<people>` (exit 0) or `RENDERER_AGREEMENT FAIL <reason>` with the
# differences (exit 1). Logs and intermediate files go to <log_dir> (keep it outside the repository).
# RENDERER_AGREEMENT_TICK=T checks tick T instead of 900 (T > 360, after the U-script orders; at most 1440).
# Godot runs bounded (tools/v8/run_bounded.sh) from tools/v8/godot_bin.sh; never run it beside another Godot
# process on this project (the strict check edits project.godot).
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG="${1:?usage: renderer_agreement.sh <log_dir>}"
SEED=100
TICK="${RENDERER_AGREEMENT_TICK:-900}"
mkdir -p "$LOG" || { echo "RENDERER_AGREEMENT FAIL cannot create $LOG"; exit 1; }
LOG="$(cd "$LOG" && pwd)"
source "$SCRIPT_DIR/godot_bin.sh"
cd "$ROOT"

fail() { echo "RENDERER_AGREEMENT FAIL $*"; exit 1; }

(cd rust && cargo build -p sim-bridge && cargo build --release -p sim-test --bin worldsim_probe) > "$LOG/build.log" 2>&1 \
    || fail "build failed (see $LOG/build.log)"
PROBE="$ROOT/rust/target/release/worldsim_probe"

rm -rf "$LOG/commands" "$LOG/probe"
bash "$SCRIPT_DIR/run_bounded.sh" 600 "$PROBE" --scene colony --seeds "$SEED..$((SEED + 1))" --days 1 --workers 1 \
    --commands tools/v8/u_script.json --commands-out "$LOG/commands" --out "$LOG/probe" > "$LOG/probe.log" 2>&1 \
    || fail "colony probe failed (see $LOG/probe.log)"
COMMANDS="$LOG/commands/colony_s${SEED}_commands.json"
[[ -s "$COMMANDS" ]] || fail "no command log $COMMANDS"

bash "$SCRIPT_DIR/run_bounded.sh" 600 "$PROBE" --scene colony --seeds "$SEED..$((SEED + 1))" --ascii --tick "$TICK" \
    --commands tools/v8/u_script.json --out "$LOG/ascii_t$TICK.txt" --people-json "$LOG/text_people.json" \
    > "$LOG/ascii.log" 2>&1 || fail "text renderer failed (see $LOG/ascii.log)"

if [[ ! -f .godot/extension_list.cfg ]]; then
    for pass in 1 2; do
        bash "$SCRIPT_DIR/run_bounded.sh" 600 "$GODOT_BIN" --headless --path . --import --editor --quit \
            > "$LOG/import_$pass.log" 2>&1
    done
fi
rm -f "$LOG/godot_people.json"
bash "$SCRIPT_DIR/run_bounded.sh" 600 "$GODOT_BIN" --headless --path . --script res://tools/v8/renderer_agreement.gd -- \
    --out "$LOG/godot_people.json" --commands "$COMMANDS" --seed "$SEED" --tick "$TICK" > "$LOG/godot.log" 2>&1
godot_status=$?
script_errors=$(grep -c 'SCRIPT ERROR' "$LOG/godot.log")
done_lines=$(grep -c '^RENDERER_AGREEMENT_GODOT_DONE' "$LOG/godot.log")
(( godot_status == 0 && script_errors == 0 && done_lines == 1 )) \
    || fail "Godot run: exit $godot_status, $script_errors SCRIPT ERROR lines, $done_lines completion lines (see $LOG/godot.log)"

python3 - "$LOG/text_people.json" "$LOG/godot_people.json" "$TICK" <<'PY'
import collections
import json
import sys

text = json.load(open(sys.argv[1]))
godot = json.load(open(sys.argv[2]))
tick = int(sys.argv[3])
problems = []
if text.get("tick") != tick or godot.get("tick") != tick:
    problems.append(f"ticks: text {text.get('tick')}, godot {godot.get('tick')}, wanted {tick}")
if not godot.get("settled", False):
    problems.append("the agent layer never settled (a person was still between two tiles)")
a = collections.Counter(tuple(t) for t in text.get("people", []))
b = collections.Counter(tuple(t) for t in godot.get("people", []))
n = sum(a.values())
if n == 0:
    problems.append("no people in the text renderer's set")
only_text = sorted((a - b).elements())
only_godot = sorted((b - a).elements())
if only_text or only_godot:
    problems.append(f"text {n} people, godot {sum(b.values())}; only in text {only_text}; only in godot {only_godot}")
# M-ART2 (goal 3.2): appearance agreement — every person's look and face variant drawn by the Godot agent layer equal
# the text renderer's (both from sim_bridge::ffi::collect_appearance), by AgentId.
looks_text = {int(r["agent_id"]): (int(r["look"]), int(r["variant"])) for r in text.get("appearance", [])}
looks_godot = {int(r["agent_id"]): (int(r["look"]), int(r["variant"])) for r in godot.get("detail", []) if "look" in r}
if not looks_text:
    problems.append("no appearance rows in the text renderer's people JSON")
appearance_bad = sorted(k for k in set(looks_text) | set(looks_godot) if looks_text.get(k) != looks_godot.get(k))
if appearance_bad:
    problems.append(f"appearance differs for {len(appearance_bad)} people: " + ", ".join(
        f"{k}: text {looks_text.get(k)} godot {looks_godot.get(k)}" for k in appearance_bad[:5]))
if problems:
    print("RENDERER_AGREEMENT FAIL " + "; ".join(problems))
    raise SystemExit(1)
print(f"RENDERER_AGREEMENT PASS n={n} tiles={len(a)} tick={tick} appearance={len(looks_text)} agree")
PY

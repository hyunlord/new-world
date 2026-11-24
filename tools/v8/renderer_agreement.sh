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
#      through the bridge's cmd_* methods, advances to tick 900 and dumps the tiles of the people the 3D People
#      layer draws (M3D-SWITCH E2; the tile under each figure's drawn feet, once every figure finished its step);
#   4. the two multisets of tiles are compared, every drawn figure must stand on its own tile and be settled, and
#      every person's drawn look and face variant (M-ART2) and the real scene's action/phase/target rows
#      (ACTION1) agree with the text probe by AgentId.
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

(cd rust && cargo build -p sim-bridge --profile game && cargo build --release -p sim-test --bin worldsim_probe) > "$LOG/build.log" 2>&1 \
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
    problems.append("the People layer never settled (a figure was still between two tiles)")
detail = godot.get("detail", [])
if len(detail) != len(godot.get("people", [])):
    problems.append(f"{len(detail)} detail rows for {len(godot.get('people', []))} drawn people")
astray = [r for r in detail if r.get("tile") != r.get("layer_tile") or not r.get("settled") or int(r.get("agent_id", -1)) < 0]
if astray:
    problems.append(f"{len(astray)} figures off their own tile, unsettled or without an AgentId: {astray[:3]}")
a = collections.Counter(tuple(t) for t in text.get("people", []))
b = collections.Counter(tuple(t) for t in godot.get("people", []))
n = sum(a.values())
if n == 0:
    problems.append("no people in the text renderer's set")
only_text = sorted((a - b).elements())
only_godot = sorted((b - a).elements())
if only_text or only_godot:
    problems.append(f"text {n} people, godot {sum(b.values())}; only in text {only_text}; only in godot {only_godot}")
# M-ART2 (goal 3.2): appearance agreement — every person's look and face variant drawn by the Godot People layer equal
# the text renderer's (both from sim_bridge::ffi::collect_appearance), by AgentId.
looks_text = {int(r["agent_id"]): (int(r["look"]), int(r["variant"])) for r in text.get("appearance", [])}
looks_godot = {int(r["agent_id"]): (int(r["look"]), int(r["variant"])) for r in godot.get("detail", []) if "look" in r}
if not looks_text:
    problems.append("no appearance rows in the text renderer's people JSON")
appearance_bad = sorted(k for k in set(looks_text) | set(looks_godot) if looks_text.get(k) != looks_godot.get(k))
if appearance_bad:
    problems.append(f"appearance differs for {len(appearance_bad)} people: " + ", ".join(
        f"{k}: text {looks_text.get(k)} godot {looks_godot.get(k)}" for k in appearance_bad[:5]))
text_actions = text.get("visible_actions", [])
godot_actions = godot.get("visible_actions", [])
expected_ids = sorted(looks_text)
for label, rows in (("text", text_actions), ("Godot", godot_actions)):
    ids = [row["agent_id"] for row in rows]
    if not ids or ids != expected_ids:
        problems.append(f"{label} visible actions do not match the ordered live AgentIds: {ids}")
if text_actions != godot_actions:
    mismatches = [(left, right) for left, right in zip(text_actions, godot_actions) if left != right]
    problems.append(f"visible action rows disagree: {mismatches[:3]}")
targeted = sum(row["phase"] > 0 and row["target_kind"] >= 0 for row in text_actions)
if tick == 900 and not targeted:
    problems.append("tick 900 lacks a visible action with a target")
if problems:
    print("RENDERER_AGREEMENT FAIL " + "; ".join(problems))
    raise SystemExit(1)
print(f"RENDERER_AGREEMENT PASS n={n} tiles={len(a)} tick={tick} appearance={len(looks_text)} actions={len(text_actions)} targeted={targeted} agree")
PY

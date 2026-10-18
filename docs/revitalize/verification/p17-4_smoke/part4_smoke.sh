#!/usr/bin/env bash
set -euo pipefail
cd /Users/rexxa/work/new-world-pass4
(( $# == 3 )) || { echo 'usage: part4_smoke.sh BIN_DIR PRODUCER_SHA OUTPUT_DIR' >&2; exit 2; }
P17_BIN="$1"
P17_SHA="$2"
P17_SMOKE="$3"
[[ ! -e "$P17_SMOKE" ]]
[[ "$(cat "$P17_BIN/source_commit.txt")" == "$P17_SHA" ]]
[[ -z "$(git diff --name-only "$P17_SHA" -- rust)" ]]
[[ -z "$(git ls-files --others --exclude-standard -- rust)" ]]
mkdir -p "$P17_SMOKE"
cp "$P17_BIN/source_commit.txt" "$P17_BIN/binaries.sha256" "$P17_SMOKE/"
(cd "$P17_BIN" && shasum -a 256 -c binaries.sha256) > "$P17_SMOKE/binaries_verified.txt"
export WORLDSIM_GIT_SHA="$P17_SHA" CARGO_BUILD_JOBS=6
P17_HOURS=00:00,01:00,02:00,03:00,04:00,05:00,06:00,07:00,08:00,09:00,10:00,11:00,12:00,13:00,14:00,15:00,16:00,17:00,18:00,19:00,20:00,21:00,22:00,23:00
trap 'p17_exit=$?; printf "%s smoke runner end exit=%s\n" "$(date -u +%FT%TZ)" "$p17_exit" >> "$P17_SMOKE/RUNLOG.txt"; printf "%s\n" "$p17_exit" > "$P17_SMOKE/runner.exit"' EXIT
for p17_scene in game_v21 game_v22; do
 for p17_mode in plain trace; do
  p17_flags=(--check-determinism)
  [[ "$p17_mode" != trace ]] || p17_flags+=(--agent-trace)
  bash tools/revitalize/wait_mac_p17.sh "$P17_SMOKE/RUNLOG.txt" "Part4 $p17_scene $p17_mode"
  printf '%s %s %s start\n' "$(date -u +%FT%TZ)" "$p17_scene" "$p17_mode" >> "$P17_SMOKE/RUNLOG.txt"
  "$P17_BIN/worldsim_probe" --scene "$p17_scene" --seeds 42..44 --days 3 --sample-every 120 --clock-samples "$P17_HOURS" --workers 4 "${p17_flags[@]}" --out "$P17_SMOKE/${p17_scene}_${p17_mode}" > "$P17_SMOKE/${p17_scene}_${p17_mode}.log" 2>&1
  printf '%s %s %s end\n' "$(date -u +%FT%TZ)" "$p17_scene" "$p17_mode" >> "$P17_SMOKE/RUNLOG.txt"
 done
 python3 tools/revitalize/compare_identity_pass17.py "$P17_SMOKE/${p17_scene}_plain/summary.json" "$P17_SMOKE/${p17_scene}_trace/summary.json" > "$P17_SMOKE/${p17_scene}_trace_identity.txt"
done
bash tools/revitalize/wait_mac_p17.sh "$P17_SMOKE/RUNLOG.txt" 'Part4 exact day60 and family trace smoke'
printf '%s v22 day60 smoke start\n' "$(date -u +%FT%TZ)" >> "$P17_SMOKE/RUNLOG.txt"
"$P17_BIN/worldsim_probe" --scene game_v22 --seeds 42..43 --days 60 --sample-every 120 --clock-samples "$P17_HOURS" --workers 4 --check-determinism --agent-trace --out "$P17_SMOKE/game_v22_day60" > "$P17_SMOKE/game_v22_day60.log" 2>&1
printf '%s v22 day60 smoke end\n' "$(date -u +%FT%TZ)" >> "$P17_SMOKE/RUNLOG.txt"
PYTHONDONTWRITEBYTECODE=1 python3 - "$P17_SMOKE" <<'PYCODE'
from pathlib import Path
import sys
sys.path.insert(0, 'tools/revitalize')
from evaluate_pass17_data import CohortSpec
from evaluate_pass17_read import load_cohort
from evaluate_pass17_family_read import load_families
root=Path(sys.argv[1])
lines=[]
for scene,folder,seeds,days in [('game_v21','game_v21_trace',(42,43),3),('game_v22','game_v22_trace',(42,43),3),('game_v22','game_v22_day60',(42,),60)]:
 cohort=load_cohort(root/folder,CohortSpec(scene,seeds,days))
 lines.append(f'{folder}: routine loader PASS worlds={len(cohort.worlds)} days={days}')
 if scene=='game_v22':
  families=load_families(cohort)
  for family in families:
   lines.append(f'family seed={family.seed} people={len(family.people)} hours={len(family.snapshots)} births={len(family.births)} fissions={len(family.fissions)} day60={family.day60 is not None}')
(root/'loader_verification.txt').write_text('\n'.join(lines)+'\n')
print('\n'.join(lines))
PYCODE
python3 /Users/rexxa/work/p17-data/trace_recompute_corrected.py "$P17_SMOKE/game_v21_trace" 42,43 > "$P17_SMOKE/routine_v21_recomputed.txt"
python3 /Users/rexxa/work/p17-data/trace_recompute_stage_b.py "$P17_SMOKE/game_v22_trace" 42,43 > "$P17_SMOKE/routine_v22_recomputed.txt"
python3 /Users/rexxa/work/p17-data/trace_recompute_stage_b.py "$P17_SMOKE/game_v22_day60" 42 > "$P17_SMOKE/routine_v22_day60_recomputed.txt"

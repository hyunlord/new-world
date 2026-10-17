#!/bin/bash
# WM-2 W2 (METRICS.md 3): the noise floor and the floor-matched scores of every trained run and the baselines, once,
# from the pre-registration commit (_setup/prereg_sha). Runs whose train.exit is not 0 are left out.
set -u
W=/home/hyunlord/work/wm1-runs/wm2
SHA=$(cat $W/_setup/prereg_sha)
D=$W/_code/${SHA:0:8}
[ -d "$D" ] || { git -C /home/hyunlord/work/new-world-wm1 fetch -q origin; git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1; }
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
ok() { [ "$(cat $W/$1/train.exit 2>/dev/null)" = "0" ] && echo $W/$1; }
R19="$(for r in wm2_m1bact_v19_s0 wm2_m1bact_v19_s1 wm2_m1bact_v19_s2 wm2_m1long_v19_s0 wm2_m1long_v19_s1; do ok $r; done)"
R16="$(ok wm2_m1b_v16_s0)"
for spec in "game_v19 v2_v19_fact v19" "game_v16 v2_v16 v16"; do
  set -- $spec
  RUNS=$R19; [ $3 = v16 ] && RUNS=$R16
  O=$W/floor_$3; mkdir -p $O
  [ -e $O/floor.json ] && continue
  date +%s > $O/job.start
  python3 tools/worldmodel/wm2/floor.py --scene $1 --dataset $2 --selection $W/calendar_val_$3/calendar_val.json --run $RUNS --out $O > $O/job.out 2>&1
  echo $? > $O/job.exit; date +%s > $O/job.end
done

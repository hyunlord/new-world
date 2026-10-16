#!/bin/bash
# WM-4: dry runs of the training and evaluation code paths on train / val data only (before METRICS.md part b): an M1
# reference on the obs val families, 120-s CRN and CT trainings with their obs-val and v2-val effects, a 3-step M1b-obs
# training. Outputs under wm4/_dry/ (never test data). usage: dryrun.sh SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm4
R=/home/hyunlord/work/wm1-runs
D=$(bash $W/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
export WM1_CODE_COMMIT=$SHA WM4_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data
O=$W/_dry; rm -rf $O; mkdir -p $O
run() { local n=$1; shift; local t=$(date +%s); "$@" > $O/$n.out 2>&1; echo "$n exit $? $(( $(date +%s) - t ))s"; }
run m1_val python3 tools/worldmodel/wm4/effects4.py m1 --run $R/wm2/wm2_m1bact_v19_s0 --role val --out $O/m1_val
run crn_train python3 tools/worldmodel/wm4/causal.py train --method crn --dataset obs_v19 --seed 0 --out $O/crn --max-seconds 120
run crn_val python3 tools/worldmodel/wm4/causal.py effects --run $O/crn --split obs --role val --out $O/crn
run ct_train python3 tools/worldmodel/wm4/causal.py train --method ct --dataset obs_v19 --seed 0 --out $O/ct --max-seconds 120
run ct_val python3 tools/worldmodel/wm4/causal.py effects --run $O/ct --split obs --role val --out $O/ct
run ct_v2val python3 tools/worldmodel/wm4/causal.py effects --run $O/ct --split v2 --role val --cases $R/wm2/effects_base/effects_base.json --out $O/ct_v2
run m1obs python3 tools/worldmodel/wm4/m1_obs.py train --out $O/m1obs --seed 0 --max-steps 3 --eval-every 3 --max-seconds 900
echo dry done

#!/bin/bash
# WM-2 Part 4 cost check (GOAL.md 9.1, 7.6): 200 steps of each model kind at k = 168 from step 1 (--ramp-steps 1),
# one validation pass at step 200 (every validation start), alone on the GPU; the run is stopped once that evaluation is
# logged (the end-of-training val metrics are not part of the check). Peak per-process GPU memory sampled every 10 s.
# usage: cost.sh CODE_SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm2
O=$W/_cost
mkdir -p $O
D=$W/_code/${SHA:0:8}
[ -d "$D" ] || { git -C /home/hyunlord/work/new-world-wm1 fetch -q origin; git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1; }
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
RECIPE="--window 336 --eval-every 200 --patience 12 --batch 32 --clamp-nonneg 1 --k-max 168 --roll-min-context 167 \
  --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --ramp-steps 1 --max-steps 200 --max-seconds 7200"
for spec in "tf m1_tf.py" "gru m1_gru.py --burn-in full"; do
  set -- $spec; NAME=$1; SCRIPT=$2; shift 2
  OUT=$O/$NAME; rm -rf $OUT; mkdir -p $OUT
  date +%s > $OUT/start
  WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data python3 tools/worldmodel/wm1/$SCRIPT train \
    --dataset v2_v19 --out $OUT --variant sev-blind_action-aware --seed 0 $RECIPE "$@" > $OUT/train.out 2>&1 &
  P=$!
  MAXMEM=0
  while kill -0 $P 2>/dev/null; do
    sleep 10
    M=$(nvidia-smi --query-compute-apps=pid,used_memory --format=csv,noheader,nounits 2>/dev/null | awk -v p=$P -F', ' '$1==p{print $2}')
    [ -n "$M" ] && [ "$M" -gt "$MAXMEM" ] && MAXMEM=$M
    if grep -q '"eval": true' $OUT/train_log.jsonl 2>/dev/null; then date +%s > $OUT/eval_logged; kill $P; sleep 5; kill -9 $P 2>/dev/null; break; fi
  done
  echo $MAXMEM > $OUT/max_gpu_mib
  date +%s > $OUT/end
done

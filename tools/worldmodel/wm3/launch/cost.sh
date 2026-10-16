#!/bin/bash
# WM-3 Part 4 cost check (GOAL.md 9.2): 100 fine-tuning steps of wm3_full and of wm3_ctrl from wm2_m1bact_v19_s0, one
# evaluation at step 0 and one at step 100 (val_t2 + val_eff), alone on the GPU; peak per-process GPU memory sampled
# every 10 s. Train and val only. usage: cost.sh SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs
O=$W/wm3/_cost${COST_TAG:-}
D=$(bash $W/wm3/_setup/code.sh $SHA) || exit 9
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
COMMON="--init $W/wm2/wm2_m1bact_v19_s0 --seed 0 --lr 1e-4 --max-steps 100 --eval-every 100 --patience 6 --max-seconds 3600 \
  --base $W/wm2/effects_base/effects_base.json"
for spec in "full --config full --n-plain 16 --n-cast 16 --partners 1 --pair 1.0 --hazard 10.0" \
            "ctrl --config ctrl --n-plain 32 --n-cast 0 --partners 0 --pair 0 --hazard 0"; do
  set -- $spec; NAME=$1; shift
  OUT=$O/$NAME; rm -rf $OUT; mkdir -p $OUT
  date +%s > $OUT/start
  WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data python3 tools/worldmodel/wm3/finetune.py \
    --out $OUT $COMMON "$@" > $OUT/train.out 2>&1 &
  P=$!
  MAXMEM=0
  while kill -0 $P 2>/dev/null; do
    sleep 10
    M=$(nvidia-smi --query-compute-apps=pid,used_memory --format=csv,noheader,nounits 2>/dev/null | awk -v p=$P -F', ' '$1==p{print $2}')
    [ -n "$M" ] && [ "$M" -gt "$MAXMEM" ] && MAXMEM=$M
  done
  wait $P; echo $? > $OUT/exit
  echo $MAXMEM > $OUT/max_gpu_mib
  date +%s > $OUT/end
done

#!/bin/bash
# WM-1.3 Part 4 training slot: runs its runs one after another (two slots at most = two trainings at once).
# usage: slot.sh SLOT_NAME CODE_SHA run1:dataset:seed run2:dataset:seed ...
# Each run: m1_tf.py train with the r3 long recipe from a git worktree at CODE_SHA, --max-seconds =
# min(30000, DEADLINE - now - 3600); not started when that is below MIN_SECONDS. After a run ends, its test
# evaluations (final, the WM-1.2-rule checkpoint, the legacy test, the probe) are launched in the background from
# the pre-registration commit in _setup/prereg_sha when that file exists (else they are left to the operator).
set -u
SLOT=$1; SHA=$2; shift 2
W=/home/hyunlord/work/wm1-runs
DEADLINE=$(cat $W/_setup/deadline_epoch)          # 2026-10-14 23:27:14 KST + recorded pauses
MIN_SECONDS=10800
LOG=$W/_queue/$SLOT.log
mkdir -p $W/_queue
code_dir() {   # a detached worktree of the DGX code clone at $1
  local d=$W/_code/${1:0:8}
  if [ ! -d "$d" ]; then git -C /home/hyunlord/work/new-world-wm1 fetch -q origin && \
     git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$d" "$1" >/dev/null 2>&1; fi
  echo "$d"
}
source /home/hyunlord/work/wm1-venv/bin/activate
for spec in "$@"; do
  IFS=: read RUN DS SEED <<< "$spec"
  NOW=$(date +%s)
  MAXS=$(( DEADLINE - NOW - 3600 )); [ $MAXS -gt 30000 ] && MAXS=30000
  if [ $MAXS -lt $MIN_SECONDS ]; then echo "$(date +%F_%T) $RUN not started: max_seconds $MAXS < $MIN_SECONDS" >> $LOG; continue; fi
  if [ -e $W/$RUN ]; then echo "$(date +%F_%T) $RUN exists: skipped" >> $LOG; continue; fi
  CD=$(code_dir $SHA)
  mkdir -p $W/$RUN
  echo "$(date +%F_%T) $RUN start ds=$DS seed=$SEED code=$SHA max_seconds=$MAXS" >> $LOG
  date +%s > $W/$RUN/train.start
  ( cd $CD && WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data python3 tools/worldmodel/wm1/m1_tf.py train \
      --dataset $DS --out $W/$RUN --variant sev-blind_action-blind --seed $SEED --batch 32 --clamp-nonneg 1 \
      --eval-every 150 --k-max 168 --max-steps 9000 --ramp-steps 1500 --patience 12 --roll-min-context 167 \
      --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --window 336 --max-seconds $MAXS ) > $W/$RUN/train.out 2>&1
  echo $? > $W/$RUN/train.exit
  date +%s > $W/$RUN/train.end
  echo "$(date +%F_%T) $RUN end exit=$(cat $W/$RUN/train.exit)" >> $LOG
  if [ -s $W/_setup/prereg_sha ]; then
    nohup bash $W/_setup/finals_w13.sh $RUN $DS > /dev/null 2>&1 &
    echo "$(date +%F_%T) $RUN finals launched from $(cat $W/_setup/prereg_sha)" >> $LOG
  fi
done
echo "$(date +%F_%T) slot $SLOT done" >> $LOG

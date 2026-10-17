#!/bin/bash
# WM-2 Part 4 training queue (GOAL.md 9.1): one training at a time, in the order of _setup/queue.txt, re-read before
# every run (lines: RUN SCRIPT DATASET VARIANT SEED MAX_STEPS [EXTRA FLAGS...]; '#' lines are skipped, so a dropped run
# is commented out). Each run: the WM-1.3 M1b recipe from a git worktree at CODE_SHA, --max-seconds =
# min(21600, DEADLINE - now - 3600) with DEADLINE re-read from _setup/deadline_epoch before every run (WM-1.3 R2-9);
# no run starts below MIN_SECONDS. Watchdog (WM-1.3 DECISIONS G4.3): a run whose train_log.jsonl is silent for
# HANG_SECONDS is killed, moved to _aborted/<run>_hang_<HHMM> and started once more. After a run ends, its test
# evaluations (finals.sh) start in the background when _setup/prereg_sha exists.
# usage: queue.sh CODE_SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm2
MIN_SECONDS=${MIN_SECONDS:-7200}
HANG_SECONDS=1800
LOG=$W/_queue/queue.log
mkdir -p $W/_queue $W/_code
code_dir() {
  local d=$W/_code/${1:0:8}
  if [ ! -d "$d" ]; then git -C /home/hyunlord/work/new-world-wm1 fetch -q origin && \
     git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$d" "$1" >/dev/null 2>&1; fi
  echo "$d"
}
source /home/hyunlord/work/wm1-venv/bin/activate
RECIPE="--window 336 --ramp-steps 1500 --eval-every 150 --patience 12 --batch 32 --clamp-nonneg 1 --k-max 168 \
  --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0"
while true; do
  LINE=$(grep -v '^#' $W/_setup/queue.txt | grep -v '^ *$' | while read r rest; do [ -e $W/$r ] || { echo "$r $rest"; break; }; done)
  [ -z "$LINE" ] && break
  read RUN SCRIPT DS VARIANT SEED MAXSTEPS EXTRA <<< "$LINE"
  for attempt in 1 2; do
    DEADLINE=$(cat $W/_setup/deadline_epoch)
    NOW=$(date +%s)
    MAXS=$(( DEADLINE - NOW - 3600 )); [ $MAXS -gt 21600 ] && MAXS=21600
    if [ $MAXS -lt $MIN_SECONDS ]; then
      echo "$(date +%F_%T) $RUN not started: max_seconds $MAXS < $MIN_SECONDS; queue stops" >> $LOG; exit 0
    fi
    CD=$(code_dir $SHA)
    mkdir -p $W/$RUN
    echo "$(date +%F_%T) $RUN start $SCRIPT ds=$DS variant=$VARIANT seed=$SEED max_steps=$MAXSTEPS extra='$EXTRA' code=$SHA max_seconds=$MAXS" >> $LOG
    date +%s > $W/$RUN/train.start
    ( cd $CD && WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data exec python3 tools/worldmodel/wm1/$SCRIPT train \
        --dataset $DS --out $W/$RUN --variant $VARIANT --seed $SEED $RECIPE --max-steps $MAXSTEPS --max-seconds $MAXS $EXTRA ) > $W/$RUN/train.out 2>&1 &
    TPID=$!
    echo $TPID > $W/$RUN/train.pid
    HUNG=0
    while kill -0 $TPID 2>/dev/null; do
      sleep 60
      LAST=$(stat -c %Y $W/$RUN/train_log.jsonl 2>/dev/null || cat $W/$RUN/train.start)
      if kill -0 $TPID 2>/dev/null && [ $(( $(date +%s) - LAST )) -gt $HANG_SECONDS ]; then HUNG=1; kill $TPID; sleep 15; kill -9 $TPID 2>/dev/null; break; fi
    done
    wait $TPID; RC=$?
    if [ $HUNG = 1 ]; then
      echo "$(date +%F_%T) $RUN hung (train_log.jsonl silent > $HANG_SECONDS s): killed, moved to _aborted" >> $LOG
      mkdir -p $W/_aborted; mv $W/$RUN $W/_aborted/${RUN}_hang_$(date +%H%M)
      continue
    fi
    echo $RC > $W/$RUN/train.exit
    date +%s > $W/$RUN/train.end
    break
  done
  if [ ! -e $W/$RUN/train.exit ]; then
    echo "$(date +%F_%T) $RUN hung twice: left in _aborted, next run" >> $LOG; mkdir -p $W/$RUN; echo hung > $W/$RUN/train.exit; continue
  fi
  echo "$(date +%F_%T) $RUN end exit=$(cat $W/$RUN/train.exit)" >> $LOG
  if [ -s $W/_setup/prereg_sha ]; then
    nohup bash $W/_setup/finals.sh $RUN > /dev/null 2>&1 &
    echo "$(date +%F_%T) $RUN finals launched from $(cat $W/_setup/prereg_sha)" >> $LOG
  fi
done
echo "$(date +%F_%T) queue done" >> $LOG

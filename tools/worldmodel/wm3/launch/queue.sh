#!/bin/bash
# WM-3 Part 4 training queue (GOAL.md 9.1): one fine-tuning at a time, in the order of _setup/queue.txt, re-read before
# every run (lines: RUN INIT CONFIG SEED N_PLAIN N_CAST PARTNERS PAIR HAZARD; '#' lines skipped, so a dropped run is
# commented out). Each run: wm3/finetune.py from a worktree at CODE_SHA, --max-seconds = min(14400, DEADLINE - now - 3600)
# with DEADLINE re-read from _setup/deadline_epoch before every run; no run starts below MIN_SECONDS. Watchdog (WM-2): a
# run whose train_log.jsonl is silent for HANG_SECONDS is killed, moved to _aborted/<run>_hang_<HHMM> and started once
# more. After a run ends, its evaluations (finals.sh) start in the background when _setup/prereg_sha exists.
# usage: queue.sh CODE_SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm3
R=/home/hyunlord/work/wm1-runs
MIN_SECONDS=${MIN_SECONDS:-3600}
HANG_SECONDS=1800
EVAL_EVERY=$(cat $W/_setup/eval_every)
LOG=$W/_queue/queue.log
mkdir -p $W/_queue
source /home/hyunlord/work/wm1-venv/bin/activate
while true; do
  LINE=$(grep -v '^#' $W/_setup/queue.txt | grep -v '^ *$' | while read r rest; do [ -e $W/$r ] || { echo "$r $rest"; break; }; done)
  [ -z "$LINE" ] && break
  read RUN INIT CONFIG SEED NP NC PART PAIR HAZ <<< "$LINE"
  for attempt in 1 2; do
    DEADLINE=$(cat $W/_setup/deadline_epoch)
    NOW=$(date +%s)
    MAXS=$(( DEADLINE - NOW - 3600 )); [ $MAXS -gt 14400 ] && MAXS=14400
    if [ $MAXS -lt $MIN_SECONDS ]; then
      echo "$(date +%F_%T) $RUN not started: max_seconds $MAXS < $MIN_SECONDS; queue stops" >> $LOG; exit 0
    fi
    CD=$(bash $W/_setup/code.sh $SHA) || { echo "$(date +%F_%T) code.sh failed" >> $LOG; exit 9; }
    mkdir -p $W/$RUN
    echo "$(date +%F_%T) $RUN start init=$INIT config=$CONFIG seed=$SEED plain=$NP cast=$NC partners=$PART pair=$PAIR hazard=$HAZ eval_every=$EVAL_EVERY code=$SHA max_seconds=$MAXS load=$(cut -d' ' -f1-3 /proc/loadavg)" >> $LOG
    date +%s > $W/$RUN/train.start
    ( cd $CD && WM1_CODE_COMMIT=$SHA WM3_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data exec python3 tools/worldmodel/wm3/finetune.py \
        --init $R/wm2/$INIT --out $W/$RUN --config $CONFIG --seed $SEED --n-plain $NP --n-cast $NC --partners $PART \
        --pair $PAIR --hazard $HAZ --lr 1e-4 --max-steps 4000 --eval-every $EVAL_EVERY --patience 6 --max-seconds $MAXS \
        --base $R/wm2/effects_base/effects_base.json ) > $W/$RUN/train.out 2>&1 &
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

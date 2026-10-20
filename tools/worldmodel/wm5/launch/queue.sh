#!/bin/bash
# WM-5 Part 4 training queue (GOAL.md 9.1, 0.3.7; WM-4 queue.sh with wm5 paths): one training job at a time, in the order of _setup/queue.txt,
# re-read before every run. Lines: RUN SHA BUDGET CMD... ('#' lines skipped, so a dropped run is commented out); CMD is
# run in a worktree at SHA (code.sh) with @OUT@ = the run directory, @MAXS@ = min(BUDGET, DEADLINE - now - 3600)
# (DEADLINE re-read from _setup/deadline_epoch before every run), @R@ = /home/hyunlord/work/wm1-runs. No run starts
# below MIN_SECONDS. When no line is pending the queue waits (lines may be appended later) until _setup/queue.stop exists.
# Watchdog (WM-2/WM-3): a run whose OUT/train_log.jsonl is silent for HANG_SECONDS gets SIGUSR1 (every WM-5 / WM-4 / WM-3
# training script dumps all thread stacks to train.out on it), then is killed, moved to _aborted/<run>_hang_<HHMM> and
# started once more. No evaluation is launched from here (test evaluations wait for METRICS.md part b, finals5.sh).
# usage: queue.sh
set -u
W=/home/hyunlord/work/wm1-runs/wm5
R=/home/hyunlord/work/wm1-runs
MIN_SECONDS=${MIN_SECONDS:-1800}
HANG_SECONDS=1800
LOG=$W/_queue/queue.log
mkdir -p $W/_queue
source /home/hyunlord/work/wm1-venv/bin/activate
while true; do
  LINE=$(grep -v '^#' $W/_setup/queue.txt | grep -v '^ *$' | while read r rest; do [ -e $W/$r ] || { echo "$r $rest"; break; }; done)
  if [ -z "$LINE" ]; then
    [ -e $W/_setup/queue.stop ] && break
    sleep 60; continue
  fi
  read RUN SHA BUDGET CMD <<< "$LINE"
  for attempt in 1 2; do
    DEADLINE=$(cat $W/_setup/deadline_epoch)
    NOW=$(date +%s)
    MAXS=$(( DEADLINE - NOW - 3600 )); [ $MAXS -gt $BUDGET ] && MAXS=$BUDGET
    if [ $MAXS -lt $MIN_SECONDS ]; then
      echo "$(date +%F_%T) $RUN not started: max_seconds $MAXS < $MIN_SECONDS; queue stops" >> $LOG; exit 0
    fi
    CD=$(bash $W/_setup/code.sh $SHA) || { echo "$(date +%F_%T) code.sh failed" >> $LOG; exit 9; }
    mkdir -p $W/$RUN
    C=${CMD//@OUT@/$W/$RUN}; C=${C//@MAXS@/$MAXS}; C=${C//@R@/$R}
    echo "$(date +%F_%T) $RUN start code=$SHA max_seconds=$MAXS load=$(cut -d' ' -f1-3 /proc/loadavg) cmd: $C" >> $LOG
    echo "$C" > $W/$RUN/train.cmd
    date +%s > $W/$RUN/train.start
    ( cd $CD && WM1_CODE_COMMIT=$SHA WM3_CODE_COMMIT=$SHA WM4_CODE_COMMIT=$SHA WM5_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data \
        exec $C ) > $W/$RUN/train.out 2>&1 &
    TPID=$!
    echo $TPID > $W/$RUN/train.pid
    HUNG=0
    while kill -0 $TPID 2>/dev/null; do
      sleep 60
      LAST=$(stat -c %Y $W/$RUN/train_log.jsonl 2>/dev/null || cat $W/$RUN/train.start)
      if kill -0 $TPID 2>/dev/null && [ $(( $(date +%s) - LAST )) -gt $HANG_SECONDS ]; then
        HUNG=1; kill -USR1 $TPID; sleep 10; kill $TPID; sleep 15; kill -9 $TPID 2>/dev/null; break
      fi
    done
    wait $TPID; RC=$?
    if [ $HUNG = 1 ]; then
      echo "$(date +%F_%T) $RUN hung (train_log.jsonl silent > $HANG_SECONDS s): stack dumped, killed, moved to _aborted" >> $LOG
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
done
echo "$(date +%F_%T) queue done" >> $LOG

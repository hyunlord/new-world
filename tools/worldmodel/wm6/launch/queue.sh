#!/bin/bash
# WM-6 Part 4 training queue (GOAL.md 9.1, 0.3.7; WM-5's queue.sh with wm6 paths): one training job at a time, in the
# order of _setup/queue.txt, re-read before every run. Lines: RUN SHA BUDGET CMD... ('#' lines skipped); CMD is run in a
# worktree at SHA (code.sh) with @OUT@ = the run directory, @MAXS@ = min(BUDGET, DEADLINE - now - 600) (DEADLINE re-read
# from _setup/deadline_epoch before every run), @R@ = /home/hyunlord/work/wm1-runs. No run starts below MIN_SECONDS.
# When no line is pending the queue waits (lines may be appended later) until _setup/queue.stop exists.
# Recording handshake (GOAL.md 0.3.7: recording uses 8 workers with no training, 4 beside one): before a run the queue
# touches _setup/training.want and waits until the recorder (record_v2x.sh) runs <= 4 workers (_record/rec.load) or is
# not running; the marker is removed when no line is pending.
# Watchdog (WM-2 … WM-5): a run whose OUT/train_log.jsonl (or, before it exists, train.start) is older than HANG_SECONDS
# gets SIGUSR1, then is killed, moved to _aborted/<run>_hang_<HHMM> and started once more.
# No evaluation is launched from here (test evaluations wait for METRICS.md, finals6.sh).
# usage: queue.sh
set -u
W=/home/hyunlord/work/wm1-runs/wm6
R=/home/hyunlord/work/wm1-runs
MIN_SECONDS=${MIN_SECONDS:-600}
HANG_SECONDS=1800
LOG=$W/_queue/queue.log
mkdir -p $W/_queue
source /home/hyunlord/work/wm1-venv/bin/activate
pending() { grep -v '^#' $W/_setup/queue.txt | grep -v '^ *$' | while read r rest; do [ -e $W/$r ] || { echo "$r $rest"; break; }; done; }
recorder_busy() {   # true while the recorder runs more than 4 workers
  local pid; pid=$(cat $W/_logs/record_v2x.pid 2>/dev/null) || return 1
  kill -0 $pid 2>/dev/null || return 1
  [ -e $W/_record/rec.done ] && return 1
  [ "$(cat $W/_record/rec.load 2>/dev/null || echo 0)" -gt 4 ]
}
while true; do
  LINE=$(pending)
  if [ -z "$LINE" ]; then
    rm -f $W/_setup/training.want
    [ -e $W/_setup/queue.stop ] && break
    sleep 60; continue
  fi
  read RUN SHA BUDGET CMD <<< "$LINE"
  touch $W/_setup/training.want
  WAITED=0
  while recorder_busy; do sleep 10; WAITED=$((WAITED + 10)); done
  [ $WAITED -gt 0 ] && echo "$(date +%F_%T) $RUN waited ${WAITED}s for the recorder to drop to 4 workers" >> $LOG
  for attempt in 1 2; do
    DEADLINE=$(cat $W/_setup/deadline_epoch)
    NOW=$(date +%s)
    MAXS=$(( DEADLINE - NOW - 600 )); [ $MAXS -gt $BUDGET ] && MAXS=$BUDGET
    if [ $MAXS -lt $MIN_SECONDS ]; then
      echo "$(date +%F_%T) $RUN not started: max_seconds $MAXS < $MIN_SECONDS; queue stops" >> $LOG
      rm -f $W/_setup/training.want; exit 0
    fi
    CD=$(bash $W/_setup/code.sh $SHA) || { echo "$(date +%F_%T) code.sh failed" >> $LOG; exit 9; }
    mkdir -p $W/$RUN
    C=${CMD//@OUT@/$W/$RUN}; C=${C//@MAXS@/$MAXS}; C=${C//@R@/$R}
    echo "$(date +%F_%T) $RUN start code=$SHA max_seconds=$MAXS load=$(cut -d' ' -f1-3 /proc/loadavg) rec_load=$(cat $W/_record/rec.load 2>/dev/null) cmd: $C" >> $LOG
    echo "$C" > $W/$RUN/train.cmd
    date +%s > $W/$RUN/train.start
    ( cd $CD && WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=/home/hyunlord/work/worldsim-wm-data exec $C ) > $W/$RUN/train.out 2>&1 &
    TPID=$!
    echo $TPID > $W/$RUN/train.pid
    HUNG=0
    while kill -0 $TPID 2>/dev/null; do
      sleep 20
      LAST=$(stat -c %Y $W/$RUN/train_log.jsonl 2>/dev/null || cat $W/$RUN/train.start)
      if kill -0 $TPID 2>/dev/null && [ $(( $(date +%s) - LAST )) -gt $HANG_SECONDS ]; then
        HUNG=1; kill -USR1 $TPID; sleep 10; kill $TPID; sleep 15; kill -9 $TPID 2>/dev/null; break
      fi
    done
    wait $TPID; RC=$?
    if [ $HUNG = 1 ]; then
      echo "$(date +%F_%T) $RUN hung (silent > $HANG_SECONDS s): killed, moved to _aborted" >> $LOG
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
  echo "$(date +%F_%T) $RUN end exit=$(cat $W/$RUN/train.exit) seconds=$(( $(cat $W/$RUN/train.end) - $(cat $W/$RUN/train.start) ))" >> $LOG
done
rm -f $W/_setup/training.want
echo "$(date +%F_%T) queue done" >> $LOG

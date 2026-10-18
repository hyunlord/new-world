#!/bin/bash
# WM-5: run finals5.sh items as their trainings end. Items are KIND:ARG; a causal / m1obs2 item waits for
# <run>/train.exit (skipped with a log line if it is not 0); any other item runs at once. Items run one at a time, in
# order of readiness; the loop ends when every item is done or skipped. FINALS_SHA passes through to finals5.sh.
# usage: finals_watch5.sh ITEM...
set -u
W=/home/hyunlord/work/wm1-runs/wm5
LOG=$W/_queue/finals.log
declare -A DONE
while true; do
  left=0
  for it in "$@"; do
    [ -n "${DONE[$it]:-}" ] && continue
    K=${it%%:*}; A=${it#*:}
    if [ "$K" = causal ] || [ "$K" = m1obs2 ]; then
      if [ ! -e $W/$A/train.exit ]; then left=1; continue; fi
      if [ "$(cat $W/$A/train.exit)" != "0" ]; then echo "$(date +%F_%T) watch: $A train exit != 0, skipped" >> $LOG; DONE[$it]=1; continue; fi
    fi
    echo "$(date +%F_%T) watch: start $it (FINALS_SHA=${FINALS_SHA:-prereg})" >> $LOG
    bash $W/_setup/finals5.sh $K $A
    DONE[$it]=1
  done
  [ $left = 0 ] && break
  sleep 120
done
echo "$(date +%F_%T) watch: all items done ($*)" >> $LOG

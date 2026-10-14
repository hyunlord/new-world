#!/usr/bin/env bash
# PASS14_GOAL.md section 1: before a launch, if the WM run trains (m1_tf|m1_gru) and the 1-min load average > nproc, wait.
# Logs every check to p14/logs/waits.log.
W=/home/hyunlord/work/worldsim-wm-data/p14/logs/waits.log
n=$(nproc)
while true; do
  load=$(cut -d" " -f1 /proc/loadavg)
  wm=$(pgrep -f "m1_tf|m1_gru" | grep -v $$ | wc -l)
  if [ "$wm" -gt 0 ] && awk -v l=$load -v n=$n "BEGIN{exit !(l>n)}"; then
    echo "$(date "+%F %T") $1 wait load $load nproc $n wm $wm" >> $W; sleep 60
  else
    echo "$(date "+%F %T") $1 go load $load nproc $n wm $wm" >> $W; break
  fi
done

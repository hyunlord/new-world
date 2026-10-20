#!/bin/bash
# Mac side: copy the small files of WM-3 runs and their evaluations from the DGX into docs/worldmodel/wm3/runs/
# (never checkpoints). usage: fetch.sh SSH_CONFIG RUN [RUN ...]
set -u
CFG=$1; shift
W=/home/hyunlord/work/wm1-runs/wm3
DST=$(cd "$(dirname "$0")/../../../.." && pwd)/docs/worldmodel/wm3/runs
for RUN in "$@"; do
  for D in $RUN ${RUN}_effects ${RUN}_split ${RUN}_fact ${RUN}_calib; do
    ssh -F $CFG dgx "[ -d $W/$D ]" || continue
    mkdir -p $DST/$D
    FILES=$(ssh -F $CFG dgx "cd $W/$D && ls | grep -v '\.pt$'")
    for f in $FILES; do scp -q -F $CFG dgx:$W/$D/$f $DST/$D/; done
  done
  if ssh -F $CFG dgx "[ -e $W/diag/effect_lag_$RUN.json ]"; then
    mkdir -p $DST/diag; scp -q -F $CFG dgx:$W/diag/effect_lag_$RUN.json $DST/diag/
  fi
done

#!/bin/bash
# WM-4 Part 4 cost check (GOAL.md 9.1): one short training alone on the GPU (train and val data only), peak per-process
# GPU memory sampled every 10 s. Run from the queue (one job at a time). usage: cost4.sh OUT CMD...
set -u
OUT=$1; shift
mkdir -p $OUT
date +%s > $OUT/cost.start
"$@" > $OUT/cost.out 2>&1 &
P=$!
MAXMEM=0
while kill -0 $P 2>/dev/null; do
  sleep 10
  M=$(nvidia-smi --query-compute-apps=pid,used_memory --format=csv,noheader,nounits 2>/dev/null | awk -v p=$P -F', ' '$1==p{print $2}')
  [ -n "$M" ] && [ "$M" -gt "$MAXMEM" ] && MAXMEM=$M
done
wait $P; RC=$?
echo $MAXMEM > $OUT/max_gpu_mib
date +%s > $OUT/cost.end
touch $OUT/train_log.jsonl
exit $RC

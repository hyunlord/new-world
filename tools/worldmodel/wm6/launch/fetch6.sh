#!/bin/bash
# WM-6 (Mac): copy every run's small files (never checkpoints, *.pt) from the DGX into docs/worldmodel/wm6/runs/
# (GOAL.md 9.6). usage: fetch6.sh SSH_CONFIG [RUN ...]   (no RUN: every run directory except _code, _logs, _setup)
set -u
CFG=$1; shift
REPO=$(cd "$(dirname "$0")/../../../.." && pwd)
DST=$REPO/docs/worldmodel/wm6/runs
SRC=/home/hyunlord/work/wm1-runs/wm6
if [ $# -eq 0 ]; then
  set -- $(ssh -F $CFG dgx "cd $SRC && ls -d */ | tr -d / | grep -v -E '^(_code|_logs|_setup|_record|_queue|_aborted|_prep|_smoke|v2x_manifest)$'")
fi
for R in "$@"; do
  mkdir -p $DST/$R
  rsync -a -e "ssh -F $CFG" --exclude '*.pt' --exclude '*.pid' --exclude 'gbm_*.txt' --exclude '*.npz' "dgx:$SRC/$R/" $DST/$R/
done
echo "fetched $# runs"

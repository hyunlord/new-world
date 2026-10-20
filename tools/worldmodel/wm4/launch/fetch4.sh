#!/bin/bash
# Mac side: copy the small files of WM-4 runs and evaluations from the DGX into docs/worldmodel/wm4/runs/ (never
# checkpoints, never the grid members' checkpoints). usage: fetch4.sh SSH_CONFIG DIR [DIR ...]  (DIR under wm1-runs/wm4)
set -u
CFG=$1; shift
W=/home/hyunlord/work/wm1-runs/wm4
DST=$(cd "$(dirname "$0")/../../../.." && pwd)/docs/worldmodel/wm4/runs
for RUN in "$@"; do
  ssh -F $CFG dgx "[ -d $W/$RUN ]" || { echo "missing $RUN"; continue; }
  FILES=$(ssh -F $CFG dgx "cd $W/$RUN && find . -type f ! -name '*.pt' ! -name 'train.pid' | sed 's#^\./##'")
  for f in $FILES; do
    mkdir -p "$DST/$RUN/$(dirname $f)"
    scp -q -F $CFG "dgx:$W/$RUN/$f" "$DST/$RUN/$f"
  done
  echo "fetched $RUN ($(echo $FILES | wc -w) files)"
done

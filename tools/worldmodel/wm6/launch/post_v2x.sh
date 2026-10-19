#!/bin/bash
# WM-6 Part 1 → Part 4 hand-off (DGX): waits for the v2x recording (_record/rec.done; a failed recording step stops it),
# then the manifest (data6.py manifest: every episode verified), the v2x features (effects6.py prepare --part v2x), the
# dataset description (data6.py describe: counts, distributions, KS vs WM-2's train worlds), and finally appends
# _setup/queue_post.txt (the N = 800 / 1,600 / 2,000 trainings) to the training queue. Each step logs start / end / exit
# in _record/post_<step>.*; a failed step stops the script.
# usage: post_v2x.sh CODE_SHA
set -u
export OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1      # DECISIONS L2.9
SHA=$1
W6=/home/hyunlord/work/wm1-runs/wm6
L=$W6/_record
DATA=/home/hyunlord/work/worldsim-wm-data
BASE=/home/hyunlord/work/wm1-runs/wm2/effects_base/effects_base.json
PY=/home/hyunlord/work/wm1-venv/bin/python
D=$(bash $W6/_setup/code.sh $SHA) || exit 9
cd $D
export WM1_CODE_COMMIT=$SHA WM_DATA_ROOT=$DATA
step() {
  local n=post_$1; shift
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then echo "$(date +%F_%T) $n done before: skipped"; return 0; fi
  date +%s > $L/$n.start; echo "$(date +%F_%T) start $n"
  nice -n 10 "$@" > $L/$n.log 2>&1; local rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; echo "$(date +%F_%T) end $n exit $rc"
  [ $rc -eq 0 ] || exit $rc
}
until [ -e $L/rec.done ]; do
  for f in families link check; do [ -e $L/$f.exit ] && [ "$(cat $L/$f.exit)" != 0 ] && { echo "recording step $f failed"; exit 3; }; done
  sleep 60
done
step manifest $PY tools/worldmodel/wm6/data6.py manifest --root $DATA --out $W6/v2x_manifest/v2x_manifest.json
step prep $PY tools/worldmodel/wm6/effects6.py prepare --part v2x --base $BASE --out $W6/_prep
step describe $PY tools/worldmodel/wm6/data6.py describe --prep $W6/_prep --base $BASE \
  --manifest $W6/v2x_manifest/v2x_manifest.json --timeline $DATA/v2x/timeline_out/timeline.json --out $W6/describe/describe.json
cat $W6/_setup/queue_post.txt >> $W6/_setup/queue.txt
echo "$(date +%F_%T) queue lines appended: $(grep -vc '^#' $W6/_setup/queue_post.txt)"

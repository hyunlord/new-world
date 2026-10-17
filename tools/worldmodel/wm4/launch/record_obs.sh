#!/bin/bash
# WM-4 Part 1 (GOAL.md 6.2): the observational split of game_v19 46000-46559 with the frozen WM-2 recorder.
# Stages (one per call, so METRICS.md part a — the terciles — is committed between `factual` and `policy`):
#   factual   6.2.1  factual episodes (no schedule) into obs/factual/
#   policy    6.2.3  wm4/policy.py on every world's factual episode -> obs/policy/policy.json, obs/schedules/obs/
#   observed  6.2.4  every world with --schedule-dir obs/schedules/obs --include-factual into obs/observed/
#   families  6.2.5  val and test worlds: one recorder call per world (fork at its reference row, R = 1) into
#                    obs/families/{val,test}/flat/, then per-world directories (families_v2.link)
# nice 10; WORKERS (default 4 beside a training, 8 when none runs). WORLDSIM_GIT_SHA = the recorder's source (WM-2).
# Each step logs start / end / exit in $L; a step whose .exit is 0 is skipped; a failed step stops the job.
# usage: record_obs.sh STAGE CODE_SHA
set -u
STAGE=$1; SHA=$2
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/obs
L=/home/hyunlord/work/wm1-runs/wm4/_record
PY=/home/hyunlord/work/wm1-venv/bin/python
C=$(bash /home/hyunlord/work/wm1-runs/wm4/_setup/code.sh $SHA) || exit 9
export WORLDSIM_GIT_SHA=f5373ab2b7679c4a0d3a5c86b256971c86f93d41
W=${WORKERS:-4}
mkdir -p "$L" "$D"
step() {   # step NAME CMD...: skipped when NAME.exit is 0 already
  local n=$1; shift
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then echo "$(date +%F_%T) $n done before: skipped"; return 0; fi
  date +%s > $L/$n.start; echo "$(date +%F_%T) start $n (workers $W, load $(cut -d' ' -f1-3 /proc/loadavg))"
  "$@" > $L/$n.log 2>&1; local rc=$?
  echo $rc > $L/$n.exit; date +%s > $L/$n.end; echo "$(date +%F_%T) end $n exit $rc"
  [ $rc -eq 0 ] || exit $rc
}
case $STAGE in
factual)
  step factual nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 46000..46560 --days 120 --every 60 --workers $W --out $D/factual
  step factual_verify $PY $C/tools/worldmodel/wm4/obs_split.py verify --dir $D/factual --expect 560 --out $L/factual_verify.json
  step terciles $PY $C/tools/worldmodel/wm4/obs_split.py terciles --factual $D/factual --out $L/terciles.json ;;
policy)
  step policy $PY $C/tools/worldmodel/wm4/obs_split.py policy --factual $D/factual --metrics $C/docs/worldmodel/wm4/METRICS.md \
    --out $D/policy/policy.json --schedules $D/schedules/obs ;;
observed)
  step observed nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 46000..46560 --days 120 --every 60 \
    --schedule-dir $D/schedules/obs --include-factual --workers $W --out $D/observed
  step observed_check $PY $C/tools/worldmodel/wm4/obs_split.py observed-check --factual $D/factual --observed $D/observed \
    --policy $D/policy/policy.json --out $L/observed_check.json ;;
families)
  step fam_schedules $PY $C/tools/worldmodel/wm4/obs_split.py fam-schedules --policy $D/policy/policy.json --out $D/schedules/fam
  for ROLE in val test; do
    for S in $($PY $C/tools/worldmodel/wm4/obs_split.py seeds --role $ROLE); do
      CT=$($PY -c "import json;print(json.load(open('$D/schedules/fam/s$S/ref.json'))['ref_tick'])")
      step fam_$S nice -n 10 $B/worldsim_wm --scene game_v19 --seeds $S..$((S+1)) --days 120 --every 60 --fork-tick $CT \
        --replicates 1 --schedule-dir $D/schedules/fam/s$S --include-factual --workers $W --out $D/families/$ROLE/flat
    done
    step fam_link_$ROLE $PY $C/tools/worldmodel/wm2/families_v2.py link --flat $D/families/$ROLE/flat --fam $D/families/$ROLE/fam
  done
  step fam_check $PY $C/tools/worldmodel/wm4/obs_split.py fam-check --root $D --policy $D/policy/policy.json \
    --out $L/fam_check.json ;;
*) echo "unknown stage $STAGE"; exit 2 ;;
esac
echo "$(date +%F_%T) record_obs $STAGE done"

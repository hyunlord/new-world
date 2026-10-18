#!/bin/bash
# WM-5 Part 1 (GOAL.md 6.2): the obs2 split of game_v19 47000-48639 with the frozen WM-2 recorder.
# Stages (one per call):
#   r0        6.2.1  round 0, factual (no schedule) into obs2/rounds/r0/, verified
#   round K   6.2.3  round K >= 1: wm5/obs2.py round-schedules writes obs2/schedules/r<K>/ (the worlds with a new cast),
#                    the recorder records them (--schedule-dir, --include-factual is not needed: every seed of the call
#                    has a file; one call per round over the listed seeds), then obs2.py round-check (prefix checks)
#   families  6.2.5  val and test worlds: one recorder call per (world, ref) into obs2/families/<role>/c<ref>/flat/
# nice 10; WORKERS (default 4 beside a training, 8 when none runs). WORLDSIM_GIT_SHA = the recorder's source (WM-2).
# Each step logs start / end / exit in $L; a step whose .exit is 0 is skipped; a failed step stops the job.
# usage: record_obs2.sh STAGE CODE_SHA [K]
set -u
STAGE=$1; SHA=$2; K=${3:-}
B=/home/hyunlord/work/wm1-runs/_build/src_wm2/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/obs2
L=/home/hyunlord/work/wm1-runs/wm5/_record
PY=/home/hyunlord/work/wm1-venv/bin/python
C=$(bash /home/hyunlord/work/wm1-runs/wm5/_setup/code.sh $SHA) || exit 9
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
one() {   # one SEED-LEVEL recorder call, skipped when its .exit is 0: one NAME OUTDIR ARGS...
  local n=$1 out=$2; shift 2
  if [ "$(cat $L/$n.exit 2>/dev/null)" = "0" ]; then return 0; fi
  nice -n 10 $B/worldsim_wm --scene game_v19 --days 120 --every 60 --workers 1 --out $out "$@" > $L/$n.log 2>&1
  echo $? > $L/$n.exit
}
export -f one; export B L
case $STAGE in
r0)
  step r0 nice -n 10 $B/worldsim_wm --scene game_v19 --seeds 47000..48640 --days 120 --every 60 --workers $W --out $D/rounds/r0
  step r0_verify $PY $C/tools/worldmodel/wm5/obs2.py verify --dir $D/rounds/r0 --expect 1640 --out $L/r0_verify.json ;;
round)
  # round K >= 1: schedules from round K-1's episodes, then one recorder call per world with a new cast (W in parallel)
  step round${K}_schedules $PY $C/tools/worldmodel/wm5/obs2.py round-schedules --root $D --round $K --out $L/round${K}_schedules.json
  mkdir -p $L/r$K
  date +%s > $L/round${K}_record.start
  cat $D/policy/round_${K}_seeds.txt | xargs -P $W -I{} bash -c "one r$K/s{} $D/rounds/r$K --seeds {}..\$(({}+1)) --schedule-dir $D/schedules/r$K"
  date +%s > $L/round${K}_record.end
  BAD=$(for S in $(cat $D/policy/round_${K}_seeds.txt); do [ "$(cat $L/r$K/s$S.exit 2>/dev/null)" = "0" ] || echo $S; done | wc -l)
  echo "$(date +%F_%T) round $K recorded: $(wc -l < $D/policy/round_${K}_seeds.txt) worlds, $BAD failed"
  [ "$BAD" = "0" ] || exit 3
  step round${K}_check $PY $C/tools/worldmodel/wm5/obs2.py round-check --root $D --round $K --out $L/round${K}_check.json ;;
finalize)
  step finalize $PY $C/tools/worldmodel/wm5/obs2.py finalize --root $D --out $L/finalize.json ;;
families)
  # GOAL.md 6.2.5: one recorder call per (world, ref), W in parallel; then per-world directories (families_v2.link)
  step fam_schedules $PY $C/tools/worldmodel/wm5/obs2.py fam-schedules --root $D --out $L/fam_schedules.json
  for ROLE in val test; do
    mkdir -p $L/fam_$ROLE
    date +%s > $L/fam_${ROLE}_record.start
    $PY $C/tools/worldmodel/wm5/obs2.py fam-jobs --root $D --role $ROLE | xargs -P $W -L 1 bash -c \
      "one fam_$ROLE/\$0_s\$1 $D/families/$ROLE/\$0/flat --seeds \$1..\$((\$1+1)) --fork-tick \$2 --replicates \$3 --schedule-dir $D/schedules/fam/$ROLE/\$0/s\$1 --include-factual"
    date +%s > $L/fam_${ROLE}_record.end
    BAD=$(ls $L/fam_$ROLE/*.exit | while read f; do [ "$(cat $f)" = "0" ] || echo $f; done | wc -l)
    echo "$(date +%F_%T) families $ROLE recorded: $(ls $L/fam_$ROLE/*.exit | wc -l) calls, $BAD failed"
    [ "$BAD" = "0" ] || exit 3
    for REF in c1 c2; do
      step fam_link_${ROLE}_$REF $PY $C/tools/worldmodel/wm2/families_v2.py link --flat $D/families/$ROLE/$REF/flat --fam $D/families/$ROLE/$REF/fam
    done
  done ;;
unforked)
  # GOAL.md 6.2.5 third check: pre_r0 of 3 sampled test worlds (non-empty pre at c2) against a fresh unforked recording
  for S in $UNFORKED_SEEDS; do
    mkdir -p $D/families/_unforked/test/c2 $D/schedules/fam/_unforked/s$S
    cp $D/schedules/fam/test/c2/s$S/s${S}_pre.json $D/schedules/fam/_unforked/s$S/
    one unforked_s$S $D/families/_unforked/test/c2 --seeds $S..$((S+1)) --schedule-dir $D/schedules/fam/_unforked/s$S
    [ "$(cat $L/unforked_s$S.exit)" = "0" ] || exit 3
  done
  step fam_check $PY $C/tools/worldmodel/wm5/obs2.py fam-check --root $D --unforked $D/families/_unforked --out $L/fam_check.json ;;
*) echo "unknown stage $STAGE"; exit 2 ;;
esac
echo "$(date +%F_%T) record_obs2 $STAGE done"

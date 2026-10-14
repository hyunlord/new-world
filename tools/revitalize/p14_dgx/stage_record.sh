#!/usr/bin/env bash
# usage: stage_record.sh STAGE SEEDS SCHED_SUBDIRS... [-- EXTRA_FLAGS]
# Pass 14 Stage A / B recording (PASS14_GOAL.md 6.1): one recorder job per schedule subdirectory, in order, each gated
# by wait_load.sh; the first with --include-factual records the factual too, later ones add --resume so the verified
# factual episodes are kept. nice 10, 4 workers, fork at 14,280 (display day 11, 06:00), R = 2, 120 days, rows every 60.
set -uo pipefail
B=/home/hyunlord/work/worldsim-wm-data/p14/bin; D=/home/hyunlord/work/worldsim-wm-data/p14
export WORLDSIM_GIT_SHA=2ba871c09819c240904510bbf26b0fb2a59a79b0
stage=$1; seeds=$2; shift 2
extra=""; dirs=()
while [ $# -gt 0 ]; do if [ "$1" = "--" ]; then shift; extra="$*"; break; fi; dirs+=("$1"); shift; done
first=1
for sub in "${dirs[@]}"; do
  resume=""; [ $first -eq 0 ] && resume="--resume"
  $B/wait_load.sh "${stage}_$sub"; date "+start ${stage}_$sub %F %T"
  nice -n 10 $B/wm_frozen --scene game_v19 --seeds $seeds --days 120 --every 60 --schedule-dir $D/schedules_$stage/$sub --include-factual --fork-tick 14280 --replicates 2 --workers 4 --out $D/stage$stage $resume $extra
  code=$?; echo "${stage}_$sub exit $code"; date "+end ${stage}_$sub %F %T"
  [ $code -ne 0 ] && exit $code
  first=0
done

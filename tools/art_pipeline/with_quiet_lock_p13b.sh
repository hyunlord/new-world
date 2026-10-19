#!/usr/bin/env bash
# One measured session per quiet hold. Foreign processes are never signalled.
set -euo pipefail
if (( $# < 3 )) || [[ ! $2 =~ ^[0-9]+$ ]] || (( $2 < 1 || $2 > 3600 )); then
  echo "usage: $0 PURPOSE HOLD_SECONDS COMMAND [ARG ...] (1..3600)" >&2
  exit 2
fi
purpose=$1
limit=$2
shift 2
repo=$(cd "$(dirname "$0")/../.." && pwd)
cd "$repo"
: "${P13_REFERENCE:?calibration JSON required}"
: "${P13_VALIDITY_OUT:?unique session JSON required}"
lock=/Users/rexxa/work/.mac_quiet.lock
export P13_HOLDS="$repo/docs/art/production13b/perf/holds.md" P13_PURPOSE="$purpose"
export P13_ATTEMPT_LEDGER="${P13_ATTEMPT_LEDGER:-$repo/docs/art/production13b/perf/attempts.json}"
mkdir -p "$(dirname "$P13_HOLDS")"
log() { printf -- '- %s pid=%s %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$$" "$*" >> "$P13_HOLDS"; }
owned=0
child=
watcher=
cleanup() {
  local rc=$?
  trap - EXIT INT TERM HUP
  if [[ -n $child ]]; then
    kill -TERM -- "-$child" 2>/dev/null || true
    sleep 1
    kill -KILL -- "-$child" 2>/dev/null || true
    wait "$child" 2>/dev/null || true
  fi
  if [[ -n $watcher ]]; then
    kill -TERM -- "-$watcher" 2>/dev/null || true
    wait "$watcher" 2>/dev/null || true
  fi
  python3 "$repo/tools/art_pipeline/session_validity_p13.py" --finish "$P13_VALIDITY_OUT" || true
  if (( owned )); then
    log "HOLD_END purpose=$purpose exit=$rc"
    if [[ -f $lock/info.txt ]] && grep -qx "pid=$$" "$lock/info.txt"; then
      rm "$lock/info.txt"
      rmdir "$lock"
    else
      log 'OWNERSHIP_CHANGED lock preserved'
    fi
  fi
  exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP
wait_start=$(date +%s)
log "WAIT_START purpose=$purpose"
while ! mkdir "$lock" 2>/dev/null; do
  now=$(date +%s)
  if (( now-wait_start >= 10800 || (${P13_DEADLINE_EPOCH:-0} > 0 && now >= ${P13_DEADLINE_EPOCH:-0}) )); then
    python3 "$repo/tools/art_pipeline/session_validity_p13.py" --record-wait "$wait_start" "$now" || true
    log "WAIT_EXHAUSTED purpose=$purpose"
    exit 75
  fi
  remaining=$(python3 "$repo/tools/art_pipeline/session_validity_p13.py" --wait-budget "$wait_start") || exit 75
  seconds=$(python3 -c 'import sys; print(min(30.0, float(sys.argv[1])))' "$remaining")
  sleep "$seconds"
done
owned=1
started=$(date +%s)
printf 'owner=codex-art-pass13\npid=%s\npurpose=%s\nstarted=%s\n' "$$" "$purpose" "$started" > "$lock/info.txt"
python3 "$repo/tools/art_pipeline/session_validity_p13.py" --record-wait "$wait_start" "$started"
log "HOLD_START purpose=$purpose limit=$limit waited=$((started-wait_start))"
export P13_HOLD_START="$started" P13_HOLD_OWNER=codex-art-pass13 P13_LOCK_WAIT_SECONDS=$((started-wait_start))
export P13_CONTENDED=0 P10_CONTENDED=0
export P13_WAIT_CALLBACK="$repo/tools/art_pipeline/session_validity_p13.py"
set -m
bash -c 'trap "exit 143" TERM HUP; source "$1" "$2" "${@:3}"' _ \
  "$repo/tools/art_pipeline/with_window_lock.sh" "$limit" python3 "$repo/tools/art_pipeline/session_validity_p13.py" "$@" &
child=$!
(
  remaining=$((started+limit-1-$(date +%s)))
  (( remaining > 0 )) && sleep "$remaining"
  log "HOLD_DEADLINE purpose=$purpose child=$child"
  kill -TERM -- "-$child" 2>/dev/null || true
  sleep 1
  kill -KILL -- "-$child" 2>/dev/null || true
) &
watcher=$!
set +e
wait "$child"
rc=$?
set -e
exit "$rc"

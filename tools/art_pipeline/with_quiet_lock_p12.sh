#!/usr/bin/env bash
# One invocation is one measured world / Godot launch. Never use for builds or gates.
set -euo pipefail
if (( $# < 3 )) || [[ ! $2 =~ ^[0-9]+$ ]] || (( $2 < 1 || $2 > 3600 )); then
  echo "usage: $0 PURPOSE HOLD_SECONDS COMMAND [ARG ...] (1..3600 seconds)" >&2
  exit 2
fi
purpose=$1
limit=$2
shift 2
deadline=${P12_DEADLINE_EPOCH:-0}
if [[ ! $deadline =~ ^[0-9]+$ ]]; then
  echo 'P12_DEADLINE_EPOCH must be a nonnegative epoch integer' >&2
  exit 2
fi
repo=$(cd "$(dirname "$0")/../.." && pwd)
lock=/Users/rexxa/work/.mac_quiet.lock
perf="$repo/docs/art/production12/perf"
holds="$perf/holds.md"
decisions="$repo/docs/art/production12/DECISIONS.md"
owner=codex-art-pass12
cd "$repo"
mkdir -p "$perf"
iso() { date -u '+%Y-%m-%dT%H:%M:%SZ'; }
epoch() { date '+%s'; }
load_now() {
  local value
  value=$(sysctl -n vm.loadavg | awk '{print $2}')
  [[ $value =~ ^[0-9]+([.][0-9]+)?$ ]] || return 1
  printf '%s\n' "$value"
}
log() { printf -- '- %s pid=%s %s\n' "$(iso)" "$$" "$*" >> "$holds"; }
check_deadline() {
  if (( deadline > 0 && $(epoch) >= deadline )); then
    log "TASK_DEADLINE purpose=$purpose deadline=$deadline"
    exit 124
  fi
}
wait_bounded() {
  local seconds=$1 left
  check_deadline
  if (( deadline > 0 )); then
    left=$((deadline-$(epoch)))
    (( left < seconds )) && seconds=$left
  fi
  (( seconds > 0 )) && sleep "$seconds"
  check_deadline
}
other_processes() {
  # Exact process-name matching, including every worldsim_* executable.
  local name ids
  for name in cargo rustc Godot; do
    ids=$(pgrep -x "$name" || true)
    if [[ -n $ids ]]; then ps -p "$(echo "$ids" | paste -sd, -)" -o pid=,ppid=,comm= || true; fi
  done
  ids=$(pgrep -x 'worldsim_.*' || true)
  if [[ -n $ids ]]; then ps -p "$(echo "$ids" | paste -sd, -)" -o pid=,ppid=,comm= || true; fi
}
owned=0
child=
watcher=
started=
load_before=
contended=false
cleanup() {
  local rc=$? end load_after
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
  if (( owned )); then
    end=$(epoch)
    load_after=$(load_now || echo unavailable)
    log "HOLD_END owner=$owner purpose=$purpose start=$started end=$end elapsed=$((end-started)) load_before=$load_before load_after=$load_after contended=$contended exit=$rc"
    # Persist cooldown before releasing; another invocation reads this marker.
    printf '<!-- art-p12-last-release %s -->\n' "$end" >> "$holds"
    if [[ -f $lock/info.txt ]] && grep -qx "pid=$$" "$lock/info.txt"; then
      rm "$lock/info.txt"
      rmdir "$lock" || true
    else
      log 'ERROR quiet lock ownership changed; did not remove it'
    fi
  fi
  exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP
wait_start=$(epoch)
log "WAIT_START owner=$owner purpose=$purpose hold_limit=$limit"
while :; do
  check_deadline
  now=$(epoch)
  if [[ -d $lock ]]; then
    modified=$(stat -f '%m' "$lock")
    log "WAIT_OTHER_LOCK age=$((now-modified)) info=$(tr '\n' ';' < "$lock/info.txt" 2>/dev/null || true) next_poll=30s"
    wait_bounded 30
    continue
  fi
  load_before=$(load_now)
  processes=$(other_processes)
  log "CHECK load=$load_before processes=${processes//$'\n'/;} waited=$((now-wait_start))s"
  if [[ -n $processes ]] || ! awk -v load="$load_before" 'BEGIN {exit !(load+0 < 6)}'; then
    if (( now-wait_start < 60 )); then
      log 'WAIT_BUSY next_poll=30s'
      wait_bounded 30
      continue
    fi
    contended=true
  else
    contended=false
  fi
  check_deadline
  if mkdir "$lock" 2>/dev/null; then
    owned=1
    started=$(epoch)
    printf 'owner=%s\npid=%s\npurpose=%s\nstarted=%s\n' "$owner" "$$" "$purpose" "$(iso)" > "$lock/info.txt"
    check_deadline
    # The task deadline closes admission; an admitted hold retains its own cap.
    break
  fi
  log 'ACQUIRE_RACE retry'
done
log "HOLD_START owner=$owner purpose=$purpose start=$started load_before=$load_before contended=$contended waited=$((started-wait_start))s"
# Job control gives each child its own process group: signals never target a
# foreign process. The outer deadline also bounds waiting for the window lock.
export P12_CONTENDED=0 P12_HOLD_START="$started" P12_HOLD_OWNER="$owner"
[[ $contended == true ]] && export P12_CONTENDED=1
export P10_CONTENDED="$P12_CONTENDED"
: "${P12_VALIDITY_OUT:?requires unique contention snapshot output}"
set -m
bash -c 'trap "exit 143" TERM HUP; source "$1" "$2" "${@:3}"' _ \
  "$repo/tools/art_pipeline/with_window_lock.sh" "$limit" python3 "$repo/tools/art_pipeline/session_validity_p12.py" "$@" &
child=$!
(
  remaining=$((started+limit-1-$(epoch)))
  (( remaining > 0 )) && sleep "$remaining"
  log "HOLD_DEADLINE child=$child limit=$limit"
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

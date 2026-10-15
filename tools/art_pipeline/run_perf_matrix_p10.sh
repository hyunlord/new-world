#!/usr/bin/env bash
# Run or resume one complete matrix, preserving every bounded world-session attempt.
set -euo pipefail
if (( $# != 3 )) || [[ $1 != baseline && $1 != after ]]; then
  echo "usage: $0 baseline|after CHECKOUT EVIDENCE_DIR" >&2
  exit 2
fi
phase=$1
checkout=$2
evidence=$3
root=$(cd "$(dirname "$0")/../.." && pwd)
godot=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
mkdir -p "$evidence"
runner_lock="$evidence/.p10-matrix-$phase.lock"
if ! mkdir "$runner_lock"; then
  echo "Another matrix runner owns $runner_lock; inspect its process before retrying" >&2
  exit 2
fi
trap 'rmdir "$runner_lock"' EXIT

validate_session() {
  local output=$1 log=$2 status=$3 world=$4
  if [[ ! -f $output || ! -f $log ]]; then
    echo "Incomplete prior session: $output / $log" >&2
    return 1
  fi
  if grep -Eq '(^|[^[:alnum:]_])ERROR([[:space:]:]|$)' "$log"; then
    echo "Rejected session with ERROR log: $log" >&2
    return 1
  fi
  if [[ -e $status ]] && [[ $(cat "$status") != 0 ]]; then
    echo "Rejected nonzero session status: $status" >&2
    return 1
  fi
  python3 - "$root/tools/art_pipeline" "$output" "$world" "$phase" <<'PY'
import sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
from summarize_perf_p10 import load_session
session = load_session(Path(sys.argv[2]))
if session.world != sys.argv[3] or session.phase != sys.argv[4]:
    raise SystemExit("Rejected session with mismatched world or phase")
if session.payload.get("valid") is not True or session.payload.get("errors") != []:
    raise SystemExit("Rejected session without valid=true and empty errors")
print(f"Validated session: {sys.argv[2]}")
PY
}

accepted=()
for world in W0 W500 W1 W200 W1000; do
  if [[ $world == W1000 && ( $phase != after || ${P10_REQUIRED_ONLY:-1} == 1 ) ]]; then
    echo "Deferred optional world: $phase $world (after-only opt-in: P10_REQUIRED_ONLY=0)" >&2
    continue
  fi
  success=0
  highest=0
  selected=""
  for attempt in 1 2 3; do
    output="$evidence/$phase-$world-attempt$attempt.json"
    log="$evidence/$phase-$world-attempt$attempt.log"
    status="$evidence/$phase-$world-attempt$attempt.exit"
    if [[ -e $output || -e $log || -e $status ]]; then
      highest=$attempt
      if validate_session "$output" "$log" "$status" "$world"; then
        if [[ -z $selected ]]; then selected=$output; fi
      else
        echo "Retained rejected prior attempt: $phase $world $attempt" >&2
      fi
    fi
  done
  if [[ -n $selected ]]; then
    accepted+=("$selected")
    echo "Resuming with accepted evidence: $selected"
    continue
  fi
  for (( attempt=highest+1; attempt<=3; attempt++ )); do
    if [[ -n ${P10_DEADLINE_EPOCH:-} ]] && (( $(date +%s) >= P10_DEADLINE_EPOCH )); then
      echo "Twelve-hour deadline reached; no new session for $world" >&2
      break
    fi
    output="$evidence/$phase-$world-attempt$attempt.json"
    log="$evidence/$phase-$world-attempt$attempt.log"
    status="$evidence/$phase-$world-attempt$attempt.exit"
    if [[ -e $output || -e $log || -e $status ]]; then
      echo "Refusing to overwrite concurrently created evidence: $output" >&2
      exit 2
    fi
    code=0
    if (set -o noclobber; bash "$root/tools/art_pipeline/with_quiet_lock_p10.sh" "$phase-$world-attempt$attempt" 3600 \
      "$godot" --path "$checkout" --resolution 1920x1080 \
      --script res://scripts/test/art/perf_frames_p10.gd -- \
      --world "$world" --phase "$phase" --out "$output" > "$log" 2>&1); then
      code=0
    else
      code=$?
    fi
    (set -o noclobber; printf '%s\n' "$code" > "$status")
    if (( code == 0 )) && validate_session "$output" "$log" "$status" "$world"; then
      accepted+=("$output")
      success=1
      break
    fi
    echo "Retained failed session: $log" >&2
  done
  if (( ! success )); then
    echo "No accepted session for $phase $world (up to three attempts); evidence retained" >&2
  fi
done
python3 "$root/tools/art_pipeline/summarize_perf_p10.py" \
  "$root/docs/art/production10/perf/$phase" "${accepted[@]}"

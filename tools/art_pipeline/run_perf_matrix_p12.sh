#!/usr/bin/env bash
# Run or resume one complete matrix, preserving every bounded world-session attempt.
set -euo pipefail
if (( $# != 3 )) || [[ $1 != baseline && $1 != after ]]; then
  echo "usage: $0 baseline|after CHECKOUT EVIDENCE_DIR" >&2
  exit 2
fi
phase=$1
checkout=$(cd "$2" && pwd -P)
mkdir -p "$3"
evidence=$(cd "$3" && pwd -P)
root=$(cd "$(dirname "$0")/../.." && pwd)
godot=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
export P12_MEASURE_CHECKOUT="$checkout"
export P12_MEASURE_HEAD
P12_MEASURE_HEAD=$(git -C "$checkout" rev-parse HEAD)
if [[ $phase == baseline && ( -z ${P12_BASELINE_SHA:-} || $P12_MEASURE_HEAD != "$P12_BASELINE_SHA" ) ]]; then
  echo "Set P12_BASELINE_SHA to the Part 0 tooling commit; baseline checkout must match" >&2
  exit 2
fi
library="$checkout/rust/target/debug/libsim_bridge.dylib"
collector=perf_frames_p12.gd
validator=summarize_perf_p12
export P12_BUILD_PROFILE=${P12_BUILD_PROFILE:-release}
[[ $P12_BUILD_PROFILE == release ]] || exit 2
release_library="$checkout/rust/target/release/libsim_bridge.dylib"
[[ $(shasum -a 256 "$library" | awk '{print $1}') == $(shasum -a 256 "$release_library" | awk '{print $1}') ]] || exit 2
export P12_MEASURE_DYLIB_SHA256
P12_MEASURE_DYLIB_SHA256=$(shasum -a 256 "$library" | awk '{print $1}')
export P12_MEASURE_SCRIPT_SHA256
P12_MEASURE_SCRIPT_SHA256=$(shasum -a 256 "$checkout/scripts/test/art/$collector" | awk '{print $1}')
verify_checkout() {
  if [[ $P12_BUILD_PROFILE == release ]]; then
    [[ $(shasum -a 256 "$release_library" | awk '{print $1}') == "$P12_MEASURE_DYLIB_SHA256" ]] || return 1
  fi
  [[
     $(git -C "$checkout" rev-parse HEAD) == "$P12_MEASURE_HEAD" &&
     $(shasum -a 256 "$library" | awk '{print $1}') == "$P12_MEASURE_DYLIB_SHA256" &&
     $(shasum -a 256 "$checkout/scripts/test/art/$collector" | awk '{print $1}') == "$P12_MEASURE_SCRIPT_SHA256" ]] || {
    echo "Checkout/build changed during matrix; refusing mixed provenance" >&2
    return 1
  }
}
export P12_SESSION_VALIDATOR="$validator"
runner_lock="$evidence/.p12-matrix-$phase.lock"
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
  if [[ ! -f $status ]] || [[ $(cat "$status") != 0 ]]; then
    echo "Rejected missing/nonzero session status: $status" >&2
    return 1
  fi
  verify_checkout || return 1
  python3 - "$root/tools/art_pipeline" "$output" "$world" "$phase" <<'PY'
import sys
import os
from pathlib import Path
sys.path.insert(0, sys.argv[1])
from importlib import import_module
load_session = import_module(os.environ["P12_SESSION_VALIDATOR"]).load_session
session = load_session(Path(sys.argv[2]))
if session.world != sys.argv[3] or session.phase != sys.argv[4]:
    raise SystemExit("Rejected session with mismatched world or phase")
if session.payload.get("valid") is not True or session.payload.get("errors") != []:
    raise SystemExit("Rejected session without valid=true and empty errors")
for field, variable in (("checkout", "P12_MEASURE_CHECKOUT"), ("build_commit", "P12_MEASURE_HEAD"), ("build_sha256", "P12_MEASURE_DYLIB_SHA256"), ("collector_sha256", "P12_MEASURE_SCRIPT_SHA256")):
    if session.payload.get(field) != os.environ[variable]:
        raise SystemExit(f"Rejected session with mismatched {field}")
print(f"Validated session: {sys.argv[2]}")
PY
}

accepted=()
any_accepted=0
for world in ${P12_WORLDS:-W0 W500 W1 W200 W-multi}; do
  if [[ $world == W1000 && ( $phase != after || ${P12_REQUIRED_ONLY:-1} == 1 ) ]]; then
    echo "Deferred optional world: $phase $world (after-only opt-in: P12_REQUIRED_ONLY=0)" >&2
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
    any_accepted=1
    echo "Resuming with accepted evidence: $selected"
    continue
  fi
  for (( attempt=highest+1; attempt<=3; attempt++ )); do
    if [[ -n ${P12_DEADLINE_EPOCH:-} ]] && (( $(date +%s) >= P12_DEADLINE_EPOCH )); then
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
    verify_checkout
    export P12_VALIDITY_OUT="$output.validity.json"
    code=0
    if [[ $world == W1 && ${P12_SPLIT_W1:-0} == 1 ]]; then
      fragments=()
      for block in frozen running_2x_0 running_2x_1 running_2x_2; do
        if [[ -n ${P12_DEADLINE_EPOCH:-} ]] && (( $(date +%s) >= P12_DEADLINE_EPOCH )); then code=124; break; fi
        fragment="${output%.json}-$block.json"
        fragments+=("$fragment")
        if [[ $block == frozen ]]; then export P12_FROZEN_ONLY=1 P12_RUNNING_BLOCK="";
        else export P12_FROZEN_ONLY=0 P12_RUNNING_BLOCK="$block"; fi
        export P12_VALIDITY_OUT="$fragment.validity.json"
        part_code=0
        bash "$root/tools/art_pipeline/with_quiet_lock_p12.sh" "$phase-$world-$block-attempt$attempt" 3600 \
          "$godot" --path "$checkout" --resolution 1920x1080 --script "res://scripts/test/art/$collector" -- \
          --world "$world" --phase "$phase" --out "$fragment" > "${fragment%.json}.log" 2>&1 || part_code=$?
        printf '%s\n' "$part_code" > "${fragment%.json}.exit"
        cat "${fragment%.json}.log" >> "$log"
        if (( part_code != 0 )); then code=$part_code; break; fi
        verify_checkout
      done
      unset P12_FROZEN_ONLY P12_RUNNING_BLOCK
      if (( code == 0 )); then
        python3 - "$output" "${fragments[@]}" <<'PYMERGE'
import hashlib
import json
import sys
from pathlib import Path
paths = [Path(name) for name in sys.argv[2:]]
parts = [json.loads(path.read_text()) for path in paths]
for part in parts:
    if part.get("valid") is not True or part.get("errors"):
        raise SystemExit("Invalid split fragment")
    for key in ("build_commit", "build_sha256", "collector_sha256", "release_sha256", "world", "phase", "initial"):
        if part[key] != parts[0][key]:
            raise SystemExit("Split fragment provenance/state mismatch: " + key)
merged = dict(parts[0])
merged["blocks"] = [block for part in parts for block in part["blocks"]]
merged["final"] = parts[-1]["final"]
merged["finished"] = parts[-1]["finished"]
merged["pooled"] = {}
merged["split_fragments"] = [{"source": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "load_before": part["load_before"], "load_after": part["load_after"]} for path, part in zip(paths, parts)]
merged["contended"] = any(part["contended"] for part in parts)
Path(sys.argv[1]).write_text(json.dumps(merged, indent=2) + "\n")
PYMERGE
      fi
    elif (set -o noclobber; bash "$root/tools/art_pipeline/with_quiet_lock_p12.sh" "$phase-$world-attempt$attempt" 3600 \
      "$godot" --path "$checkout" --resolution 1920x1080 \
      --script "res://scripts/test/art/$collector" -- \
      --world "$world" --phase "$phase" --out "$output" > "$log" 2>&1); then
      code=0
    else
      code=$?
    fi
    (set -o noclobber; printf '%s\n' "$code" > "$status")
    if (( code == 0 )) && validate_session "$output" "$log" "$status" "$world"; then
      accepted+=("$output")
      any_accepted=1
      success=1
      break
    fi
    echo "Retained failed session: $log" >&2
  done
  if (( ! success )); then
    echo "No accepted session for $phase $world (up to three attempts); evidence retained" >&2
  fi
done
if (( ! any_accepted )); then
  echo "No accepted sessions; matrix remains incomplete" >&2
  exit 1
fi
python3 "$root/tools/art_pipeline/$validator.py" \
  "$root/docs/art/production12/perf/${P12_OUTPUT_PREFIX:-$phase}" "${accepted[@]}"

#!/usr/bin/env bash
# Usage: bash tools/art_pipeline/run_identity_p13b.sh CHECKOUT EVIDENCE_DIR A|Aprime|B
set -euo pipefail
: "${P13B_WALKER_KERNEL:?requires explicit P13B_WALKER_KERNEL=0 or 1}"
[[ "$P13B_WALKER_KERNEL" == 0 || "$P13B_WALKER_KERNEL" == 1 ]] || { echo "invalid P13B_WALKER_KERNEL" >&2; exit 2; }
export P13B_WALKER_KERNEL
self=$(cd "$(dirname "$0")" && pwd -P)/$(basename "$0")
provenance_helper=$(dirname "$self")/run_running_identity_p13b.sh
snapshot() {
  bash "$provenance_helper" --snapshot "$1" | python3 -c '
import hashlib, json, sys
from pathlib import Path
value = json.load(sys.stdin)
value["p3_plan_sha256"] = hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest()
value["p3_target_tick"] = sys.argv[2]
value["runner_sha256"] = hashlib.sha256(Path(sys.argv[3]).read_bytes()).hexdigest()
value["provenance_helper_sha256"] = hashlib.sha256(Path(sys.argv[4]).read_bytes()).hexdigest()
print(json.dumps(value, sort_keys=True, indent=2))
' "$2" "$3" "$self" "$provenance_helper"
}
if [[ ${1:-} == --snapshot ]]; then
  [[ $# -eq 4 ]] || exit 2
  snapshot "$2" "$3" "$4"
  exit
fi
if [[ $# -ne 3 || ! "$3" =~ ^(A|Aprime|B)$ ]]; then
  echo 'usage: run_identity_p13b.sh CHECKOUT EVIDENCE_DIR A|Aprime|B' >&2
  exit 2
fi
checkout=$(cd "$1" && pwd -P)
mkdir -p "$2"
evidence=$(cd "$2" && pwd -P)
label=$3
output="$evidence/$label"
godot=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
quiet=${P13_QUIET_LOCK:-/Users/rexxa/work/.mac_quiet.lock}
limit=${P13_CAPTURE_LIMIT:-1800}
if [[ ! "$limit" =~ ^[1-9][0-9]{0,3}$ || ! -x "$godot" ]] || (( limit > 3600 )); then
  echo 'requires an executable GODOT_BIN and P13_CAPTURE_LIMIT between 1 and 3600' >&2
  exit 2
fi
for required in project.godot scripts/test/art/capture_identity_p13b.gd scripts/test/art/capture_framings_p13b.gd tools/art_pipeline/with_window_lock.sh tools/art_pipeline/run_bounded.sh; do
  if [[ ! -f "$checkout/$required" ]]; then
    echo "missing required checkout file: $required" >&2
    exit 2
  fi
done
# Atomic refusal preserves every earlier run, including failed captures.
mkdir "$output"
mkdir "$output/logs" "$output/provenance"
: "${P13_P3_PLAN:?requires derived P3 plan path}"
: "${P13_P3_TARGET_TICK:?requires derived P3 tick}"
cp "$P13_P3_PLAN" "$output/p3-plan.json"
snapshot "$checkout" "$output/p3-plan.json" "$P13_P3_TARGET_TICK" > "$output/provenance/initial.json"
printf 'framing\tstarted_utc\tfinished_utc\texit_code\tartifacts\tlog_errors\tprovenance\n' > "$output/launches.tsv"
printf 'checkout=%s\nlabel=%s\nfixed_fps=60\nresolution=1920x1080\nlimit_seconds=%s\nquiet_lock=%s\n' "$checkout" "$label" "$limit" "$quiet" > "$output/runner.txt"
cd "$checkout"
failed=0
for framing in P1 P2 P3 P4 P5 P6; do
  while [[ -d "$quiet" ]]; do
    printf '%s %s waiting for quiet lock %s\n' "$(date -u +%FT%TZ)" "$framing" "$quiet" >> "$output/logs/quiet-waits.log"
    sleep 30
  done
  snapshot "$checkout" "$output/p3-plan.json" "$P13_P3_TARGET_TICK" > "$output/provenance/$framing.before.json"
  if ! cmp -s "$output/provenance/initial.json" "$output/provenance/$framing.before.json"; then
    echo "source/build/HEAD/plan changed before $framing; refusing mixed capture" >&2
    exit 1
  fi
  args=("$godot" --path "$checkout" --disable-render-loop --fixed-fps 60 --resolution 1920x1080)
  if [[ "$framing" == P3 ]]; then
    args+=(--script res://scripts/test/art/capture_framings_p13b.gd -- --plan art_society --out "$output")
  else
    args+=(--script res://scripts/test/art/capture_identity_p13b.gd)
  fi
  started=$(date -u +%FT%TZ)
  rc=0
  # Recheck after the potentially long window-lock wait; never acquire a quiet hold.
  # with_window_lock invokes run_bounded, including this recheck in the time limit.
  env P13_IDENTITY_ID="$framing" P13_IDENTITY_OUT="$output" \
    P13_P3_TARGET_TICK="$P13_P3_TARGET_TICK" CAPTURE_IDENTITY_P13=1 CAPTURE_PLAN_JSON="$output/p3-plan.json" CAPTURE_SOCIETY_PERF_ONLY=0 \
    bash tools/art_pipeline/with_window_lock.sh "$limit" \
    bash -c 'set -euo pipefail; quiet=$1; self=$2; checkout=$3; before=$4; inside=$5; plan=$6; tick=$7; shift 7
      while [[ -d "$quiet" ]]; do printf "waiting for quiet lock %s\n" "$quiet"; sleep 30; done
      bash "$self" --snapshot "$checkout" "$plan" "$tick" > "$inside"
      cmp "$before" "$inside"
      exec "$@"' identity-launch "$quiet" "$self" "$checkout" \
      "$output/provenance/$framing.before.json" "$output/provenance/$framing.inside.json" \
      "$output/p3-plan.json" "$P13_P3_TARGET_TICK" "${args[@]}" > "$output/logs/$framing.log" 2>&1 || rc=$?
  printf '%s\n' "$rc" > "$output/logs/$framing.exit"
  provenance=unchanged
  if ! snapshot "$checkout" "$output/p3-plan.json" "$P13_P3_TARGET_TICK" > "$output/provenance/$framing.after.json" || ! cmp -s "$output/provenance/$framing.before.json" "$output/provenance/$framing.after.json"; then
    provenance=changed
    failed=1
  fi
  log_errors=none
  if grep -Eq 'SCRIPT ERROR|ERROR:' "$output/logs/$framing.log"; then
    log_errors=detected
    failed=1
  fi
  artifacts=present
  if [[ ! -s "$output/$framing.png" || ! -s "$output/$framing.json" ]]; then
    artifacts=missing
    failed=1
  fi
  if [[ $rc -ne 0 ]]; then failed=1; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$framing" "$started" "$(date -u +%FT%TZ)" "$rc" "$artifacts" "$log_errors" "$provenance" >> "$output/launches.tsv"
  if [[ $failed -ne 0 ]]; then break; fi
done
exit "$failed"

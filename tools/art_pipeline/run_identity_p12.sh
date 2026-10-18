#!/usr/bin/env bash
# Usage: bash tools/art_pipeline/run_identity_p12.sh CHECKOUT EVIDENCE_DIR A|Aprime|B
set -euo pipefail
if [[ $# -ne 3 || ! "$3" =~ ^(A|Aprime|B)$ ]]; then
  echo 'usage: run_identity_p12.sh CHECKOUT EVIDENCE_DIR A|Aprime|B' >&2
  exit 2
fi
checkout=$(cd "$1" && pwd -P)
mkdir -p "$2"
evidence=$(cd "$2" && pwd -P)
label=$3
output="$evidence/$label"
godot=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
quiet=${P12_QUIET_LOCK:-/Users/rexxa/work/.mac_quiet.lock}
limit=${P12_CAPTURE_LIMIT:-1800}
if [[ ! "$limit" =~ ^[1-9][0-9]{0,3}$ || ! -x "$godot" ]] || (( limit > 3600 )); then
  echo 'requires an executable GODOT_BIN and P12_CAPTURE_LIMIT between 1 and 3600' >&2
  exit 2
fi
for required in project.godot scripts/test/art/capture_identity_p12.gd scripts/test/art/capture_framings_p12.gd tools/art_pipeline/with_window_lock.sh tools/art_pipeline/run_bounded.sh; do
  if [[ ! -f "$checkout/$required" ]]; then
    echo "missing required checkout file: $required" >&2
    exit 2
  fi
done
# Atomic refusal preserves every earlier run, including failed captures.
mkdir "$output"
mkdir "$output/logs"
: "${P12_P3_PLAN:?requires derived P3 plan path}"
: "${P12_P3_TARGET_TICK:?requires derived P3 tick}"
cp "$P12_P3_PLAN" "$output/p3-plan.json"
printf 'framing\tstarted_utc\tfinished_utc\texit_code\tartifacts\tlog_errors\n' > "$output/launches.tsv"
printf 'checkout=%s\nlabel=%s\nfixed_fps=60\nresolution=1920x1080\nlimit_seconds=%s\nquiet_lock=%s\n' "$checkout" "$label" "$limit" "$quiet" > "$output/runner.txt"
cd "$checkout"
failed=0
for framing in P1 P2 P3 P4 P5 P6; do
  while [[ -d "$quiet" ]]; do
    printf '%s %s waiting for quiet lock %s\n' "$(date -u +%FT%TZ)" "$framing" "$quiet" >> "$output/logs/quiet-waits.log"
    sleep 30
  done
  args=("$godot" --path "$checkout" --disable-render-loop --fixed-fps 60 --resolution 1920x1080)
  if [[ "$framing" == P3 ]]; then
    args+=(--script res://scripts/test/art/capture_framings_p12.gd -- --plan art_society --out "$output")
  else
    args+=(--script res://scripts/test/art/capture_identity_p12.gd)
  fi
  started=$(date -u +%FT%TZ)
  rc=0
  # Recheck after the potentially long window-lock wait; never acquire a quiet hold.
  # with_window_lock invokes run_bounded, including this recheck in the time limit.
  env P12_IDENTITY_ID="$framing" P12_IDENTITY_OUT="$output" \
    P12_P3_TARGET_TICK="$P12_P3_TARGET_TICK" CAPTURE_IDENTITY_P12=1 CAPTURE_PLAN_JSON="$output/p3-plan.json" CAPTURE_SOCIETY_PERF_ONLY=0 \
    bash tools/art_pipeline/with_window_lock.sh "$limit" \
    bash -c 'while [[ -d "$1" ]]; do printf "waiting for quiet lock %s\n" "$1"; sleep 30; done; shift; exec "$@"' \
    identity-launch "$quiet" "${args[@]}" > "$output/logs/$framing.log" 2>&1 || rc=$?
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
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$framing" "$started" "$(date -u +%FT%TZ)" "$rc" "$artifacts" "$log_errors" >> "$output/launches.tsv"
  if [[ $failed -ne 0 ]]; then break; fi
done
exit "$failed"

#!/usr/bin/env bash
set -euo pipefail
snapshot() {
  python3 - "$1" <<'PY'
import hashlib
import json
import subprocess
import sys
from pathlib import Path
root = Path(sys.argv[1])
files = {root / "project.godot"}
for directory in ("scripts", "scenes", "shaders", "data", "assets", "localization"):
    files.update(p for p in (root / directory).rglob("*") if p.is_file())
for pattern in ("*.rs", "*.ron", "Cargo.toml", "Cargo.lock"):
    files.update(p for p in (root / "rust").rglob(pattern) if "target" not in p.relative_to(root / "rust").parts and p.is_file())
files.update(root.glob("*.gdextension"))
files.update((root / "rust").glob("*.gdextension"))
sources = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}
print(json.dumps({"checkout": str(root), "head": subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip(), "debug_sha256": hashlib.sha256((root / "rust/target/debug/libsim_bridge.dylib").read_bytes()).hexdigest(), "source_sha256": hashlib.sha256(json.dumps(sources, sort_keys=True).encode()).hexdigest(), "sources": sources}, sort_keys=True, indent=2))
PY
}
if [[ ${1:-} == --snapshot ]]; then
  [[ $# -eq 2 ]] || exit 2
  snapshot "$2"
  exit
fi
if [[ $# -ne 2 ]]; then
  echo 'usage: run_running_identity_p13.sh CHECKOUT NEW_OUTPUT_DIR' >&2
  exit 2
fi
checkout=$(cd "$1" && pwd -P)
self=$(cd "$(dirname "$0")" && pwd -P)/$(basename "$0")
godot=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
quiet=${P13_QUIET_LOCK:-/Users/rexxa/work/.mac_quiet.lock}
[[ -x "$godot" ]] || { echo 'GODOT_BIN is not executable' >&2; exit 2; }
for required in project.godot rust/target/debug/libsim_bridge.dylib scripts/test/art/capture_running_identity_p13.gd tools/art_pipeline/with_window_lock.sh tools/art_pipeline/run_bounded.sh; do
  [[ -f "$checkout/$required" ]] || { echo "missing $required" >&2; exit 2; }
done
mkdir "$2"
output=$(cd "$2" && pwd -P)
mkdir "$output/logs" "$output/provenance"
snapshot "$checkout" > "$output/provenance/initial.json"
read -r head library_hash < <(python3 - "$output/provenance/initial.json" <<'PY'
import json, sys
value = json.load(open(sys.argv[1]))
print(value["head"], value["debug_sha256"])
PY
)
printf 'world\tstarted_utc\tfinished_utc\texit_code\tartifacts\tlog_errors\tprovenance\n' > "$output/launches.tsv"
printf 'checkout=%s\nhead=%s\ndebug_sha256=%s\nfixed_fps=60\nresolution=1920x1080\nlimit_seconds=1800\nquiet_lock=%s\n' "$checkout" "$head" "$library_hash" "$quiet" > "$output/runner.txt"
cd "$checkout"
failed=0
for world in ${P13_RUNNING_WORLDS:-W0 W200 W-multi W500_FROZEN W500_2X}; do
  while [[ -d "$quiet" ]]; do
    printf '%s %s waiting for quiet lock %s\n' "$(date -u +%FT%TZ)" "$world" "$quiet" >> "$output/logs/quiet-waits.log"
    sleep 30
  done
  snapshot "$checkout" > "$output/provenance/$world.before.json"
  if ! cmp -s "$output/provenance/initial.json" "$output/provenance/$world.before.json"; then
    echo "source/build/HEAD changed before $world; refusing mixed capture" >&2
    exit 1
  fi
  started=$(date -u +%FT%TZ)
  rc=0
  # Recheck inside the lock: source/build may change while waiting for its owner.
  env P13_RUNNING_WORLD="$world" P13_RUNNING_OUT="$output" P13_BUILD_COMMIT="$head" P13_BUILD_SHA256="$library_hash" \
    bash tools/art_pipeline/with_window_lock.sh 1800 \
    bash -c 'set -euo pipefail; quiet=$1; self=$2; checkout=$3; before=$4; inside=$5; shift 5
      while [[ -d "$quiet" ]]; do printf "waiting for quiet lock %s\n" "$quiet"; sleep 30; done
      bash "$self" --snapshot "$checkout" > "$inside"
      cmp "$before" "$inside"
      exec "$@"' running-launch "$quiet" "$self" "$checkout" \
      "$output/provenance/$world.before.json" "$output/provenance/$world.inside.json" \
      "$godot" --path "$checkout" --fixed-fps 60 --resolution 1920x1080 \
      --script res://scripts/test/art/capture_running_identity_p13.gd \
      > "$output/logs/$world.log" 2>&1 || rc=$?
  finished=$(date -u +%FT%TZ)
  printf '%s\n' "$rc" > "$output/logs/$world.exit"
  provenance=unchanged
  if ! snapshot "$checkout" > "$output/provenance/$world.after.json" || ! cmp -s "$output/provenance/$world.before.json" "$output/provenance/$world.after.json"; then
    provenance=changed
    failed=1
  fi
  log_errors=none
  if grep -Eq 'SCRIPT ERROR|ERROR:' "$output/logs/$world.log"; then log_errors=detected; failed=1; fi
  artifacts=present
  if [[ ! -s "$output/$world.json" || ! -s "$output/$world.jsonl" ]]; then artifacts=missing; failed=1; fi
  if [[ $rc -ne 0 ]]; then failed=1; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$world" "$started" "$finished" "$rc" "$artifacts" "$log_errors" "$provenance" >> "$output/launches.tsv"
  if [[ $provenance != unchanged || $rc -eq 124 ]]; then
    echo 'Stopping after provenance change or timeout; inspect owned processes before retrying in a fresh directory' >&2
    break
  fi
done
exit "$failed"

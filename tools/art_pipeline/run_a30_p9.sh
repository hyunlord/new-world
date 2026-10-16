#!/usr/bin/env bash
# One A30 launch per invocation; with_quiet_lock_p9 owns window locking and timeout.
set -euo pipefail
if (( $# != 3 )) || [[ $1 != before && $1 != after ]]; then
  echo "usage: $0 before|after CHECKOUT NEW_OUTPUT_DIR" >&2
  exit 2
fi
phase=$1
checkout=$(cd "$2" && pwd)
root=$(cd "$(dirname "$0")/../.." && pwd)
case "$checkout" in
  "$root"|/Users/rexxa/work/art-p9-base) ;;
  *) echo "A30 checkout must be the art clone or its authorized base worktree" >&2; exit 2 ;;
esac
if [[ $phase == before && $checkout != /Users/rexxa/work/art-p9-base ]] || [[ $phase == after && $checkout != "$root" ]]; then
  echo "before must use art-p9-base; after must use the current art clone" >&2
  exit 2
fi
godot=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
if [[ ! -x $godot || ! -f $checkout/scripts/test/capture_framings.gd || ! -f $checkout/rust/target/debug/libsim_bridge.dylib ]]; then
  echo "A30 requires Godot, the original capture script and a built debug bridge" >&2
  exit 2
fi
mkdir -p "$(dirname "$3")"
if ! mkdir "$3"; then
  echo "Refusing to overwrite A30 evidence; use a new output directory" >&2
  exit 2
fi
output=$(cd "$3" && pwd)
sha=$(git -C "$checkout" rev-parse HEAD)
started=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
code=0
if bash "$root/tools/art_pipeline/with_quiet_lock_p9.sh" "a30-$phase" 3600 \
  env CAPTURE_SOCIETY_PERF_ONLY=1 WORLDSIM_GIT_SHA="$sha" \
  "$godot" --path "$checkout" --resolution 1920x1080 \
  --script res://scripts/test/capture_framings.gd -- --plan art_society --out "$output" \
  > "$output/session.log" 2>&1; then
  code=0
else
  code=$?
fi
printf '%s\n' "$code" > "$output/session.exit"
python3 - "$output" "$phase" "$checkout" "$sha" "$started" "$code" <<'PY'
import datetime
import json
import pathlib
import sys
output = pathlib.Path(sys.argv[1])
metadata = {"phase": sys.argv[2], "checkout": sys.argv[3], "git_sha": sys.argv[4],
            "started": sys.argv[5], "finished": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "exit_code": int(sys.argv[6]), "build_profile": "debug", "fixed_fps": False,
            "plan": "art_society", "CAPTURE_SOCIETY_PERF_ONLY": "1",
            "measurement": str(output / "frame_times_soc_layers.json")}
(output / "session_metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
PY
if (( code != 0 )); then
  echo "A30 failed with exit $code; preserved $output" >&2
  exit "$code"
fi
if grep -Eq '(^|[^[:alnum:]_])ERROR([[:space:]:]|$)' "$output/session.log"; then
  echo "A30 log contains ERROR; preserved $output" >&2
  exit 1
fi
python3 - "$root/tools/art_pipeline" "$output/frame_times_soc_layers.json" <<'PY'
import sys
sys.path.insert(0, sys.argv[1])
from compare_perf_p9 import a30
result = a30(sys.argv[2])
if not result:
    raise SystemExit("A30 measurement missing or invalid; original evidence retained")
print("Validated A30 artifact:", sys.argv[2])
print(result)
PY

#!/bin/bash
# render.sh <play-root worktree> <out-dir> [run-spec ...]: PERF2-0 Mac frame profiles under the mux2 GPU lock.
# A run spec is "scene:zoom[:extra-args]" (scene = basename of a .wsave in $SCENES); each run is one windowed Godot
# process of tools/v8/perf2_frame.gd (warmup 10 s, 30 s measured, simulation at 1x, WORLDSIM_MUX2_PROFILE=1).
# Records the 1-minute load before each run. Never run outside the lock (the caller wraps it: mux2_gate_lock.py).
set -uo pipefail
export PATH=/opt/homebrew/bin:$PATH
BOUNDED="$(cd "$(dirname "$0")/.." && pwd)/run_bounded.sh"
ROOT=${1:?worktree}; OUT=${2:?out}; shift 2
SCENES=${SCENES:-/Users/rexxa/work/art-img-data/perf2/data/scenes}
GODOT=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
mkdir -p "$OUT"
for spec in "$@"; do
  IFS=: read -r scene zoom extra <<< "$spec"
  name="$scene-z$zoom${extra:+-$(echo "$extra" | tr -c 'a-zA-Z0-9' '_')}"
  echo "$(date -Iseconds) start $name load $(sysctl -n vm.loadavg)" >> "$OUT/runs.log"
  start=$(python3 -c 'import time;print(time.time())')
  WORLDSIM_MUX2_PROFILE=1 bash "$BOUNDED" 300 "$GODOT" --disable-vsync --path "$ROOT" -s res://tools/v8/perf2_frame.gd -- \
    --load "$SCENES/$scene.wsave" --zoom "$zoom" --warmup "${WARMUP:-10}" --seconds "${SECONDS_MEASURED:-30}" \
    --out "$OUT/$name.json" $extra > "$OUT/$name.log" 2>&1
  code=$?
  echo "$(date -Iseconds) end $name exit $code wall $(python3 -c "import time;print(round(time.time()-$start,1))") load $(sysctl -n vm.loadavg)" >> "$OUT/runs.log"
done

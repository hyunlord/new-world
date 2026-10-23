#!/usr/bin/env bash
# Run the isolated graphical art load with immutable input hashes.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
source "$ROOT/tools/v8/godot_bin.sh"
OUT="${1:-/Users/rexxa/work/artstruct-data/fps/final}"
mkdir -p "$OUT"
if [[ -e "$OUT/measurement.json" ]]; then
    echo "Refusing to overwrite measured run: $OUT" >&2
    exit 2
fi
bash "$ROOT/tools/v8/run_bounded.sh" 90 "$GODOT_BIN" --path "$ROOT/art/proto/struct" --rendering-driver opengl3 -- --out "$OUT" --seconds 60 > "$OUT/godot.log" 2>&1
if rg 'SCRIPT ERROR|Parse Error|ERROR:' "$OUT/godot.log"; then exit 1; fi
rg 'STRUCT_CROWD_DONE verified=true' "$OUT/godot.log"

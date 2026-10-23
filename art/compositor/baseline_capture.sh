#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
task_project="${ENV1_PROBE_WORKTREE:-/Users/rexxa/work/artenv1-data/probe-worktree}"
task_out="${1:-/Users/rexxa/work/artenv1-data/baseline}"
task_godot="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
test "$(git -C "$task_project" rev-parse HEAD)" = "3a3b236c125c0c25e9b74261be88536473fc01e9"
test -f "$task_project/rust/target/debug/libsim_bridge.dylib"
mkdir -p "$task_out"
bash "$task_project/tools/v8/run_bounded.sh" 180 caffeinate -i -d "$task_godot" \
  --path "$task_project" --rendering-driver opengl3 --resolution 1600x900 \
  --script "$task_root/art/compositor/baseline_capture.gd" -- --out "$task_out" \
  2>&1 | tee "$task_out/capture.log"
if rg -n 'SCRIPT ERROR|Parse Error|ERROR:' "$task_out/capture.log"; then
  exit 1
fi
rg -q '^ENV1_BASELINE_CAPTURE_DONE 6$' "$task_out/capture.log"

#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
PLAN="${1:-}"
case "$PLAN" in baseline|final) ;; *) echo "Usage: capture.sh baseline|final" >&2; exit 2;; esac
if ! command -v xvfb-run >/dev/null 2>&1; then
    echo "CAPTURE SKIPPED: xvfb-run unavailable; no screenshots produced." >&2
    exit 77
fi
GODOT_BIN="${GODOT_BIN:-/home/hyunlord/tools/godot-4.6/Godot_v4.6-stable_linux.arm64}"
test -x "$GODOT_BIN"
export CARGO_BUILD_JOBS=10
for attempt in {1..11}; do
    free -g
    available="$(free -g | awk '/^Mem:/ {print $7}')"
    if (( available >= 24 )); then break; fi
    if (( attempt == 11 )); then
        export CARGO_BUILD_JOBS=4
        echo "Capture memory wait exhausted; reducing build jobs to 4." >&2
        break
    fi
    sleep 60
done
df -h /home
available_kb="$(df -Pk /home | awk 'NR==2 {print $4}')"
if (( available_kb < 15 * 1024 * 1024 )); then
    echo "Capture blocked: less than 15 GiB available on /home." >&2
    exit 1
fi
# Serialize capture invocations; do not kill or touch other users' processes.
mkdir -p .harness/evidence/revitalize
exec 9>.harness/evidence/revitalize/capture.lock
flock -n 9 || { echo "Another capture is already running." >&2; exit 1; }
(cd rust && cargo build -p sim-bridge)
timeout 600 "$GODOT_BIN" --headless --path . --import --editor --quit
export WORLDSIM_GIT_SHA="$(git rev-parse HEAD)"
out=".harness/evidence/revitalize/shots/$PLAN"
timeout 600 xvfb-run -a -s "-screen 0 1920x1080x24" \
    "$GODOT_BIN" --path . --rendering-driver opengl3 --resolution 1920x1080 \
    --script res://scripts/test/capture_framings.gd -- --out "$ROOT/$out" --plan "$PLAN"
python3 tools/revitalize/downscale.py "$out" "docs/revitalize/shots/$PLAN"

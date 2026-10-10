#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
PLAN="${1:-final}"
case "$PLAN" in baseline|final) ;; *) echo 'Usage: capture.sh baseline|final' >&2; exit 2;; esac
EVIDENCE="${CAPTURE_EVIDENCE:-$ROOT/.harness/evidence/revitalize-pass2}"
mkdir -p "$EVIDENCE"
LOCK="$EVIDENCE/capture.lockdir"
mkdir "$LOCK" 2>/dev/null || { echo 'Capture lock exists; another invocation may be running.' >&2; exit 1; }
xvfb_pid=''
cleanup() {
    if [[ -n "$xvfb_pid" ]]; then kill "$xvfb_pid" 2>/dev/null || true; wait "$xvfb_pid" 2>/dev/null || true; fi
    rmdir "$LOCK"
}
trap cleanup EXIT
run_bounded() {
    python3 - "$@" <<'PY'
import subprocess
import sys
try:
    raise SystemExit(subprocess.run(sys.argv[1:], timeout=600, check=False).returncode)
except subprocess.TimeoutExpired:
    print('Godot exceeded the 600-second limit.', file=sys.stderr)
    raise SystemExit(124)
PY
}
case "$(uname -s)" in
    Darwin)
        [[ "$(launchctl managername)" == Aqua ]] || { echo 'Tier A requires an Aqua session.' >&2; exit 77; }
        GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
        export CARGO_BUILD_JOBS=8
        ;;
    Linux)
        GODOT_BIN="${GODOT_BIN:-/home/hyunlord/tools/godot-4.6/Godot_v4.6-stable_linux.arm64}"
        export CARGO_BUILD_JOBS=10
        for attempt in {1..11}; do
            free -g
            available="$(free -g | awk '/^Mem:/ {print $7}')"
            if (( available >= 24 )); then break; fi
            if (( attempt == 11 )); then export CARGO_BUILD_JOBS=5; break; fi
            sleep 60
        done
        available_kb="$(df -Pk /home | awk 'NR==2 {print $4}')"
        (( available_kb >= 15 * 1024 * 1024 )) || { echo 'Less than 15 GiB free on /home.' >&2; exit 1; }
        if [[ -z "${DISPLAY:-}" ]]; then
            X="${XVFB_LOCAL_ROOT:-/home/hyunlord/tools/xvfb-local/root}"
            if [[ -x "$X/usr/bin/Xvfb" && -x /usr/bin/xkbcomp ]]; then
                [[ ! -e /tmp/.X11-unix/X97 && ! -e /tmp/.X97-lock ]] || { echo 'Display :97 occupied; choose another tier.' >&2; exit 77; }
                export LD_LIBRARY_PATH="$X/usr/lib/aarch64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
                "$X/usr/bin/Xvfb" :97 -screen 0 1920x1080x24 -nolisten tcp > "$EVIDENCE/xvfb.log" 2>&1 &
                xvfb_pid=$!
                export DISPLAY=:97
                for attempt in {1..20}; do [[ -S /tmp/.X11-unix/X97 ]] && break; sleep 0.25; done
            else
                echo 'No display; prepare user-space Xvfb or use Mac tier A.' >&2
                exit 77
            fi
        fi
        display_number="${DISPLAY#:}"
        display_number="${display_number%%.*}"
        [[ "$display_number" =~ ^[0-9]+$ ]] || { echo 'Only a local owned X display is allowed.' >&2; exit 77; }
        socket="/tmp/.X11-unix/X$display_number"
        [[ -S "$socket" && "$(stat -c %U "$socket")" == "$(id -un)" ]] || { echo 'Display socket is absent or not owned by current user.' >&2; exit 77; }
        ;;
    *) echo 'Unsupported capture host.' >&2; exit 77;;
esac
[[ -x "$GODOT_BIN" ]]
if [[ "${CAPTURE_SKIP_BUILD:-0}" != 1 ]]; then (cd rust && cargo build -p sim-bridge); fi
run_bounded "$GODOT_BIN" --headless --path . --import --editor --quit
export WORLDSIM_GIT_SHA="$(git rev-parse HEAD)"
out="$EVIDENCE/shots/$PLAN"
run_bounded "$GODOT_BIN" --path . --rendering-driver opengl3 --resolution 1920x1080 \
    --script res://scripts/test/capture_framings.gd -- --out "$out" --plan "$PLAN"
if [[ "$PLAN" == final ]]; then
    qa_args=("$out" "docs/revitalize/shots/final")
    if [[ -n "${BASELINE_RAW:-}" ]]; then qa_args+=("$BASELINE_RAW"); fi
    python3 tools/revitalize/visual_qa.py "${qa_args[@]}"
fi

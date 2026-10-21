#!/usr/bin/env bash
# usage: test_play.sh [--quick]
#
# Tests for tools/play.sh (V8-PLAY goal 2.3; gate step 21). Each case checks an exact exit code:
#   unknown version → 3; GODOT_BIN=/nonexistent → 2; a bad --speed → 4;
#   --headless-check main, m1 and v7 → 0 (each builds/imports its worktree under PLAY_ROOT when needed,
#   then runs 300 headless frames and requires 0 SCRIPT ERROR lines).
# --quick runs only the three exit-code cases (no Godot, no build).
# Prints PLAY_TEST_PASS/PLAY_TEST_FAIL per case and PLAY_TEST_DONE failures=N; exits non-zero on any failure.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
PLAY="$ROOT/tools/play.sh"
failures=0

expect() {
    local want="$1" label="$2"; shift 2
    local out code
    out="$("$@" 2>&1)"
    code=$?
    if (( code == want )); then
        echo "PLAY_TEST_PASS $label (exit $code)"
    else
        echo "PLAY_TEST_FAIL $label: exit $code, wanted $want"
        printf '%s\n' "$out" | tail -15 | sed 's/^/    /'
        failures=$((failures + 1))
    fi
}

expect 3 "unknown version" /bin/bash "$PLAY" no-such-version-v8play
expect 2 "GODOT_BIN=/nonexistent" env GODOT_BIN=/nonexistent /bin/bash "$PLAY" m1 --headless-check
expect 4 "bad --speed" /bin/bash "$PLAY" m1 --speed 7
if [[ "${1:-}" != --quick ]]; then
    for v in main m1 v7; do
        expect 0 "--headless-check $v" /bin/bash "$PLAY" "$v" --headless-check
    done
fi
echo "PLAY_TEST_DONE failures=$failures"
(( failures == 0 ))

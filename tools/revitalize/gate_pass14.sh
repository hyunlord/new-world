#!/usr/bin/env bash
# usage: gate_pass14.sh <log_dir>
# The Pass 14 gate (PASS14_GOAL.md 4.2), run on the Mac from the repository root that contains this script:
# both Python unittest discoveries (tools/revitalize, tools/worldmodel) with WM_BIN=/nonexistent and
# PROBE_BIN=/nonexistent (no recorder or probe runs on the Mac in this pass; the tests that need a binary skip),
# and the check that `git diff --name-only 2ba871c0..HEAD -- rust/` is empty (so Pass 13's Rust gate on 2ba871c0
# still covers the code). Every step runs; logs and exit codes land in <log_dir>, <log_dir>/gate.txt summarises
# them with the discovery counts. Exit 0 only when every step passed.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
LOG="${1:?usage: gate_pass14.sh <log_dir>}"
mkdir -p "$LOG"
SUMMARY="$LOG/gate.txt"
{ echo "head $(git rev-parse HEAD)"; echo "started $(date '+%Y-%m-%d %H:%M:%S %Z')"; } > "$SUMMARY"
status=0
step() {
    local name="$1"; shift
    "$@" > "$LOG/$name.log" 2>&1
    local code=$?
    echo "$name exit $code" >> "$SUMMARY"
    (( code == 0 )) || status=1
}
counts() {
    local ran skipped
    ran=$(grep -E '^Ran [0-9]+ test' "$LOG/$1.log" | awk '{print $2}')
    skipped=$(grep -oE 'skipped=[0-9]+' "$LOG/$1.log" | cut -d= -f2)
    echo "$1 ran ${ran:-?} skipped ${skipped:-0} result $(grep -E "^(OK|FAILED)" "$LOG/$1.log" | tail -1)" >> "$SUMMARY"
}
export WM_BIN=/nonexistent PROBE_BIN=/nonexistent
step python_unittest python3 -m unittest discover -s tools/revitalize -p 'test_*.py'
counts python_unittest
step python_worldmodel python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
counts python_worldmodel
step rust_unchanged bash -c 'changed=$(git diff --name-only 2ba871c09819c240904510bbf26b0fb2a59a79b0..HEAD -- rust/); echo "rust/ files changed since 2ba871c0: ${changed:-none}"; [ -z "$changed" ]'
echo "finished $(date '+%Y-%m-%d %H:%M:%S %Z') status $status" >> "$SUMMARY"
exit "$status"

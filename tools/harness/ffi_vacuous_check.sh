#!/bin/bash
# ffi_vacuous_check.sh — V7 Hook Governance v3.3 §4.4
# FFI Vacuous Check: sim-bridge 변경 0 검증
#
# Usage: ffi_vacuous_check.sh <diff-files-newline-list>
#   stdin alternative: echo "$files" | ffi_vacuous_check.sh -
#
# Returns:
#   0  vacuous confirmed (no sim-bridge changes) → full FFI credit
#   1  not vacuous (≥1 sim-bridge change) → run normal FFI verify
#
# Reference: .harness/prompts/governance_v3_3.md §4 + §4.4

set -euo pipefail

if [[ "$#" -lt 1 ]]; then
    echo "usage: $0 <diff-files-newline-list> | $0 -" >&2
    exit 2
fi

if [[ "$1" == "-" ]]; then
    DIFF_FILES="$(cat)"
else
    DIFF_FILES="$1"
fi

sim_bridge_changes=0
while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if [[ "$f" =~ ^rust/crates/sim-bridge/ ]]; then
        sim_bridge_changes=$((sim_bridge_changes + 1))
    fi
done <<< "$DIFF_FILES"

if [[ "$sim_bridge_changes" -eq 0 ]]; then
    echo "[ffi-vacuous] CONFIRMED: No sim-bridge changes (full FFI credit)"
    exit 0
else
    echo "[ffi-vacuous] NOT vacuous: $sim_bridge_changes sim-bridge file change(s)" >&2
    exit 1
fi

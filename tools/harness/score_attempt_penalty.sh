#!/bin/bash
# score_attempt_penalty.sh — V7 Hook Governance v3.3 §3.2 + §3.3
# Attempt 분류 후 penalty 산출 (per-attempt cap -5)
#
# Usage: score_attempt_penalty.sh <attempts-dir>
#   attempts-dir 안에 attempt-1/, attempt-2/, ... 각 안에 verdict 파일 (또는 동등명)
#
# Output (stdout, key=value lines):
#   PENALTY=<int>           sum of per-attempt penalties (≤0)
#   EFFECTIVE_ATTEMPTS=<n>  count of LOCK_VIOLATION + OUT_OF_SCOPE + OTHER attempts
#
# Penalty per category:
#   LOCK_VIOLATION  -5
#   OUT_OF_SCOPE    -5
#   TEST_RIGOR       0  (정상 refinement)
#   STYLE            0  (정상 refinement)
#   OTHER           -2  (보수적)
#
# Per-attempt cap: -5 (한 attempt 안 multiple violation 있어도 -5 cap; 자동 enforced
#                       because case branch picks single category, and -5 is its max)
#
# Reference: .harness/prompts/governance_v3_3.md §3.2 + §3.3

set -euo pipefail

ATTEMPTS_DIR="${1:?usage: $0 <attempts-dir>}"

if [[ ! -d "$ATTEMPTS_DIR" ]]; then
    echo "PENALTY=0"
    echo "EFFECTIVE_ATTEMPTS=0"
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLASSIFIER="$SCRIPT_DIR/classify_recode.sh"

if [[ ! -x "$CLASSIFIER" ]]; then
    echo "[score_attempt_penalty] missing classifier: $CLASSIFIER" >&2
    exit 2
fi

total_penalty=0
effective_count=0

shopt -s nullglob
for attempt_dir in "$ATTEMPTS_DIR"/attempt-*/; do
    [[ ! -d "$attempt_dir" ]] && continue

    # accept either verdict or verdict.txt or verdict.md
    verdict=""
    for cand in "$attempt_dir/verdict" "$attempt_dir/verdict.txt" "$attempt_dir/verdict.md"; do
        if [[ -f "$cand" ]]; then verdict="$cand"; break; fi
    done
    [[ -z "$verdict" ]] && continue

    attempt_num="$(basename "$attempt_dir" | sed 's|/$||; s/^attempt-//')"
    category="$("$CLASSIFIER" "$verdict" "$attempt_num")"

    case "$category" in
        LOCK_VIOLATION|OUT_OF_SCOPE)
            total_penalty=$((total_penalty - 5))
            effective_count=$((effective_count + 1))
            ;;
        TEST_RIGOR|STYLE)
            : ;;
        OTHER)
            total_penalty=$((total_penalty - 2))
            effective_count=$((effective_count + 1))
            ;;
    esac
done

echo "PENALTY=$total_penalty"
echo "EFFECTIVE_ATTEMPTS=$effective_count"

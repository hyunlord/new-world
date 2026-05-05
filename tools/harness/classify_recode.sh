#!/bin/bash
# classify_recode.sh — V7 Hook Governance v3.3 §3.3
# RE-CODE 분류기: Generator self-report + Evaluator verdict regex 분류
#
# Usage: classify_recode.sh <verdict-file> <attempt-num>
#
# Output (stdout): one of
#   LOCK_VIOLATION
#   OUT_OF_SCOPE
#   TEST_RIGOR
#   STYLE
#   OTHER
#
# Reference: .harness/prompts/governance_v3_3.md §3.1.2 + §3.3

set -euo pipefail

VERDICT_FILE="${1:?usage: $0 <verdict-file> <attempt-num>}"
ATTEMPT_NUM="${2:?attempt number}"

# Empty / missing → OTHER (conservative)
if [[ ! -f "$VERDICT_FILE" ]]; then
    echo "OTHER"
    exit 0
fi

content="$(cat "$VERDICT_FILE")"
if [[ -z "$content" ]]; then
    echo "OTHER"
    exit 0
fi

# Priority order (가장 강한 signal 부터 — 한 attempt가 multiple match 시 LOCK > OOS > TEST > STYLE > OTHER)
if echo "$content" | grep -qiE 'lock|cardinality|prompt §3|forbidden rationale|"more flexible"|"reasonable"|"future-proof"|"more idiomatic"|"more rust-idiomatic"'; then
    echo "LOCK_VIOLATION"
elif echo "$content" | grep -qiE 'out of scope|workspace\.members|new crate|sim-test|sim-bridge.*new|harness\.rs|prompt §6'; then
    echo "OUT_OF_SCOPE"
elif echo "$content" | grep -qiE 'test count|edge case|boundary|coverage|test rigor|insufficient test|test보강'; then
    echo "TEST_RIGOR"
elif echo "$content" | grep -qiE 'rustfmt|clippy.*style|naming|doc comment|verbosity|cosmetic'; then
    echo "STYLE"
else
    echo "OTHER"
fi

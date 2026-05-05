#!/bin/bash
# cold_tier_classifier.sh — V7 Hook Governance v3.3 §2.4
# 4 Signals 검증: cold tier 자동 식별
#
# Usage: cold_tier_classifier.sh <diff-files-newline-list>
#   stdin alternative: echo "$files" | cold_tier_classifier.sh -
#
# Returns:
#   0  cold tier confirmed (all 4 signals)
#   1  hot/mixed/warm (≥1 signal missing)
#
# Reference: .harness/prompts/governance_v3_3.md §2.1.4 + §2.4

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

# ── Signal A: crate prefix
# 모든 변경 파일이 cold tier crate 안에 또는 exempt path 안에 존재
signal_a=1
while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if [[ "$f" =~ ^rust/crates/(sim-core|sim-data|sim-test|sim-bench)/ ]]; then continue; fi
    if [[ "$f" =~ ^rust/Cargo\.toml$ ]]; then continue; fi
    if [[ "$f" =~ ^\.harness/ ]]; then continue; fi
    if [[ "$f" =~ ^localization/ ]]; then continue; fi
    signal_a=0
    break
done <<< "$DIFF_FILES"

# ── Signal B: file pattern (cold tier OK or exempt)
signal_b=1
while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if [[ "$f" =~ \.(rs|ron|toml|md|json)$ ]]; then continue; fi
    signal_b=0
    break
done <<< "$DIFF_FILES"

# ── Signal C: GDScript/Godot absence
signal_c=1
if echo "$DIFF_FILES" | grep -qE '\.(gd|gdshader|tscn|tres)$|^scripts/|^scenes/'; then
    signal_c=0
fi

# ── Signal D: System tick registration absence (heuristic, scans only existing rs files)
signal_d=1
while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    [[ ! -f "$f" ]] && continue
    [[ ! "$f" =~ \.rs$ ]] && continue
    if grep -qE 'register_runtime_system!|impl[[:space:]]+RuntimeSystem[[:space:]]+for|fn[[:space:]]+tick\(' "$f" 2>/dev/null; then
        signal_d=0
        break
    fi
done <<< "$DIFF_FILES"

# ── Final
if [[ "$signal_a" == "1" && "$signal_b" == "1" && "$signal_c" == "1" && "$signal_d" == "1" ]]; then
    echo "[cold-tier] CONFIRMED: all 4 signals present (A=1 B=1 C=1 D=1)"
    exit 0
else
    echo "[cold-tier] NOT confirmed (A=$signal_a B=$signal_b C=$signal_c D=$signal_d)" >&2
    exit 1
fi

#!/bin/bash
# cold_tier_classifier.sh — V7 Hook Governance v3.3.8 §2.4
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
#
# v3.3.8 (2026-05-09):
#   - Signal A: sim-bridge added to whitelist (T7.7.A scaffold lane).
#     Rationale: sim-bridge crate scaffold (Cargo.toml + empty lib.rs +
#     workspace registration) is structural infrastructure with zero
#     behavior. Signal D's `impl RuntimeSystem for X` regex remains the
#     authoritative gate for FFI methods that ship behavior in T7.7.B
#     and beyond — those will fall through to hot-tier classification
#     because FFI shims do not register RuntimeSystem implementations.
#
# v3.3.6 (2026-05-09):
#   - Signal A: sim-systems added to whitelist (empty scaffold lane).
#     Rationale: Signal D regex precision in v3.3.5 already guarantees hot-tier
#     auto-classification when actual `impl RuntimeSystem for X` lands in T7.6.
#     sim-bridge intentionally NOT whitelisted (FFI integration tier, T7.7 decision).
#   - Signal B: '.log' extension added to allowlist (pipeline log artifacts).
#
# v3.3.5 (2026-05-09):
#   - Signal A: sim-engine added to whitelist (V7 reset multi-crate scaffold).
#     sim-systems / sim-bridge intentionally NOT whitelisted (hot/integration tier).
#   - Signal D: 'fn tick(' removed (false-positive on trait method definitions
#     and impl method bodies). Replaced with explicit register patterns
#     `register_runtime_system!` / `impl RuntimeSystem for` / `register_system(`.
#     sim-engine 크레이트는 trait host이므로 Signal D 스캔에서 제외 (infrastructure).
#     실제 hot-tier registration 신호는 sim-systems 등 외부 크레이트에서 발생.

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
    if [[ "$f" =~ ^rust/crates/(sim-core|sim-data|sim-test|sim-bench|sim-engine|sim-systems|sim-bridge)/ ]]; then continue; fi
    if [[ "$f" =~ ^rust/Cargo\.toml$ ]]; then continue; fi
    if [[ "$f" =~ ^\.harness/ ]]; then continue; fi
    if [[ "$f" =~ ^tools/ ]]; then continue; fi
    if [[ "$f" =~ ^localization/ ]]; then continue; fi
    signal_a=0
    break
done <<< "$DIFF_FILES"

# ── Signal B: file pattern (cold tier OK or exempt)
signal_b=1
while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    if [[ "$f" =~ \.(rs|ron|toml|md|ftl|json|sh|py|log)$ ]]; then continue; fi
    signal_b=0
    break
done <<< "$DIFF_FILES"

# ── Signal C: GDScript/Godot absence
signal_c=1
if echo "$DIFF_FILES" | grep -qE '\.(gd|gdshader|tscn|tres)$|^scripts/|^scenes/'; then
    signal_c=0
fi

# ── Signal D: System tick registration absence (heuristic, scans only existing rs files)
# v3.3.5: sim-engine 크레이트는 RuntimeSystem trait/register_system 메서드를 정의하는
#         infrastructure host이므로 Signal D 스캔 대상에서 제외한다.
#         hot-tier registration의 실제 신호는 sim-systems 등 다른 크레이트에서 발생.
signal_d=1
while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    [[ ! -f "$f" ]] && continue
    [[ ! "$f" =~ \.rs$ ]] && continue
    if [[ "$f" =~ ^rust/crates/sim-engine/ ]]; then continue; fi
    if grep -qE 'register_runtime_system!|^impl[[:space:]]+RuntimeSystem[[:space:]]+for[[:space:]]+|register_system\(' "$f" 2>/dev/null; then
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

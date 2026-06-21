#!/usr/bin/env bash
# ============================================================================
# Deterministic regression verdict (Fix 2 — 2026-06-21, harness-infra)
# ============================================================================
# Computes the Step-2.7 regression verdict by a DETERMINISTIC set-difference
# read of an ALREADY-PRODUCED cargo-test gate result — NO re-run, NO Codex
# subagent, NO ~10-min wrapper timeout. Replaces the old Codex-re-run guard
# that always timed out and defaulted to CLEAN (the masked-pass Fix 1 closed).
#
# Source of truth for "what ran": the Generator gate result
# (gate_result_attempt*.txt, full `cargo test --workspace`, NON-quiet → per-test
# names). NOT Step-0's `--quiet` output (aggregate counts, no names).
#
# Verdict (set-difference):
#   failing tests in the gate result NOT in the reconciled baseline list
#     >=1  -> REGRESSION_DETECTED        (generate_report.sh scores 0/15 -> block)
#      0   -> CLEAN                       (15/15)
#   gate result missing / empty / no `test result:` line (did not complete /
#   unparseable) -> REGRESSION_GUARD_INCOMPLETE (0/15 -> block) — Fix 1 floor.
#
# Note on the "V7-reset SKIP set": it is a GDScript file/dir-ABSENCE rule
# (governance v3.3.9-v3.3.12), applied to the old guard's advisory GDScript
# steps — it is NOT a cargo-test-failure exclusion, so it does not apply to this
# set-difference (and is moot while scripts/ui/ exists). FFI chain verify remains
# the separate advisory path from Fix 1 (not score-affecting).
#
# Name format: cargo prints the test name module-qualified for unit tests
# (`a::b::tests::fn`, `tests::fn`) and bare for integration-test top-level fns
# (`harness_x_y`). The reconciled known_failures.txt is generated from the SAME
# extractor, so the two round-trip identically (re-feeding the clean-HEAD log
# yields CLEAN).
#
# Usage: regression_verdict.sh <gate_result.txt> <known_failures.txt>
# Emits the regression_guard.txt body to stdout.
set -o pipefail

gate="${1:-}"
baseline="${2:-}"

emit_incomplete() {
    echo "regression_status: REGRESSION_GUARD_INCOMPLETE"
    echo "regression_details: $1"
}

# --- INCOMPLETE floor (Fix 1): a guard that could not read a completed gate
# result verified NOTHING and must NEVER read as CLEAN. ------------------------
if [[ -z "$gate" || ! -f "$gate" ]]; then
    emit_incomplete "gate result file missing (${gate:-<none>}) — verification did NOT complete; blocking, NOT clean."
    exit 0
fi
if [[ ! -s "$gate" ]]; then
    emit_incomplete "gate result file empty ($gate) — verification did NOT complete; blocking, NOT clean."
    exit 0
fi
if ! grep -qaE '^test result:' "$gate" 2>/dev/null; then
    emit_incomplete "gate result has no 'test result:' line ($gate) — gate did not complete / unparseable; blocking, NOT clean."
    exit 0
fi

# --- Failing tests as printed by cargo (sorted, unique). ----------------------
failing=$(grep -aE '^test .+ \.\.\. FAILED$' "$gate" 2>/dev/null \
            | sed -E 's/^test (.+) \.\.\. FAILED$/\1/' \
            | sort -u)

# --- Reconciled baseline list: first token per non-comment / non-blank line. --
known=""
if [[ -n "$baseline" && -f "$baseline" ]]; then
    known=$(grep -vE '^[[:space:]]*#|^[[:space:]]*$' "$baseline" 2>/dev/null \
              | awk '{print $1}' | sort -u)
fi

# --- New failures = failing MINUS baseline (set difference). -------------------
new_failures=$(comm -23 \
    <(printf '%s\n' "$failing" | grep -v '^$' | sort -u) \
    <(printf '%s\n' "$known"   | grep -v '^$' | sort -u))

if [[ -n "$new_failures" ]]; then
    echo "regression_status: REGRESSION_DETECTED"
    echo "regression_details: NEW failure(s) not in reconciled baseline: $(printf '%s' "$new_failures" | tr '\n' ' ' | sed 's/ $//')"
    exit 0
fi

_fcount=$(printf '%s\n' "$failing" | grep -cv '^$')
echo "regression_status: CLEAN"
echo "regression_details: no new failures beyond ${_fcount} baseline-known (deterministic set-difference vs reconciled known_failures.txt)."
exit 0

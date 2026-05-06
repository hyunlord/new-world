#!/usr/bin/env bash
# STRUCTURAL-COMMIT Authorization Script (V7 Hook Governance v3.3.4)
#
# Usage:
#   bash tools/harness/authorize_structural_commit.sh \
#       --feature <slug> \
#       --reason "<rationale>" \
#       --authorized-by "<operator name>"
#
# Establishes a one-shot, time-limited (≤2 h) authorization for direct-
# implementation commits in cold-tier crates (sim-core/sim-data/sim-test/
# sim-bench schema work) without running the full harness pipeline.
#
# Use case (post-V.4 cascade):
#   - T1 (cargo deps + lib.rs scaffolding) succeeded pre-V.4 (no lane needed)
#   - T2-T5 (material module) succeeded pre-V.4
#   - T6.1~T6.5 (100 RON files) need a formal lane post-V.4 — that is THIS lane
#
# Hot-tier crates (sim-systems/sim-engine/sim-bridge) are explicitly DENIED
# — they require the full harness pipeline (Plan debate + Visual Verify +
# Evaluator). Cold tier signals: A=crate prefix, B=ext allowlist,
# C=no GDScript, D=no RuntimeSystem registration.
#
# Prerequisites enforced by this script:
#   1. cargo test --workspace must pass cleanly
#   2. cargo clippy --workspace --all-targets -- -D warnings must be clean
#   3. cold_tier_classifier.sh must confirm all 4 signals (A=1 B=1 C=1 D=1)
#   4. Staged files must be non-empty (something to commit)
#
# Side effects:
#   - Creates .harness/audit/structural_commit_active marker (consumed once
#     by tools/harness/hooks/pre-commit-check.sh on next commit)
#   - Appends entry to .harness/audit/structural_commits.log
#   - Writes manual_verification.log + clippy_full.log to evidence dir
#
# After authorization the next commit MUST:
#   - Include [STRUCTURAL] tag in subject line
#   - Reference the feature slug + rationale in the commit body
set -uo pipefail

FEATURE=""
REASON=""
AUTHORIZED_BY=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature)
            FEATURE="$2"; shift 2 ;;
        --reason)
            REASON="$2"; shift 2 ;;
        --authorized-by)
            AUTHORIZED_BY="$2"; shift 2 ;;
        *)
            printf 'Unknown argument: %s\n' "$1" >&2
            exit 1 ;;
    esac
done

if [[ -z "$FEATURE" || -z "$REASON" || -z "$AUTHORIZED_BY" ]]; then
    cat >&2 <<USAGE
Usage: $0 --feature <slug> --reason "<rationale>" --authorized-by "<operator>"

All three flags are required.
USAGE
    exit 1
fi

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [[ -z "$PROJECT_ROOT" ]]; then
    printf '[authorize-structural] not in a git repository\n' >&2
    exit 1
fi

cd "$PROJECT_ROOT"

EVIDENCE_DIR="$PROJECT_ROOT/.harness/evidence/$FEATURE"
AUDIT_DIR="$PROJECT_ROOT/.harness/audit"
MARKER="$AUDIT_DIR/structural_commit_active"
LOG="$AUDIT_DIR/structural_commits.log"

mkdir -p "$EVIDENCE_DIR" "$AUDIT_DIR"

printf '\n=== STRUCTURAL-COMMIT Authorization for %s ===\n' "$FEATURE"
printf 'Reason: %s\n' "$REASON"
printf 'Authorized by: %s\n\n' "$AUTHORIZED_BY"

# Step 1: cargo test --workspace (must pass cleanly — no baseline tolerance)
printf '[1/4] Running cargo test --workspace ...\n'
TEST_LOG="$EVIDENCE_DIR/manual_verification.log"
( cd "$PROJECT_ROOT/rust" && cargo test --workspace --no-fail-fast ) >"$TEST_LOG" 2>&1
TEST_EXIT=$?

if [[ $TEST_EXIT -ne 0 ]]; then
    printf '[authorize-structural] cargo test FAILED (exit %s). Full log: %s\n' "$TEST_EXIT" "$TEST_LOG" >&2
    tail -40 "$TEST_LOG" >&2
    printf '\nSTRUCTURAL-COMMIT requires zero test failures. Fix before authorizing.\n' >&2
    exit 1
fi
printf '       PASS — cargo test --workspace clean.\n'
printf '       Log saved to %s\n' "$TEST_LOG"

# Step 2: cargo clippy --workspace --all-targets -- -D warnings (must be clean)
printf '[2/4] Running cargo clippy --workspace --all-targets -- -D warnings ...\n'
CLIPPY_LOG="$EVIDENCE_DIR/clippy_full.log"
( cd "$PROJECT_ROOT/rust" && cargo clippy --workspace --all-targets -- -D warnings ) >"$CLIPPY_LOG" 2>&1
CLIPPY_EXIT=$?

if [[ $CLIPPY_EXIT -ne 0 ]]; then
    printf '[authorize-structural] cargo clippy FAILED (exit %s). Full log: %s\n' "$CLIPPY_EXIT" "$CLIPPY_LOG" >&2
    tail -40 "$CLIPPY_LOG" >&2
    printf '\nSTRUCTURAL-COMMIT requires zero clippy issues. Fix before authorizing.\n' >&2
    exit 1
fi
printf '       PASS — cargo clippy clean.\n'
printf '       Log saved to %s\n' "$CLIPPY_LOG"

# Step 3: cold_tier_classifier — must confirm all 4 signals on staged files
printf '[3/4] Running cold_tier_classifier on staged files ...\n'
STAGED=$(git diff --cached --name-only 2>/dev/null || echo "")
if [[ -z "$STAGED" ]]; then
    printf '[authorize-structural] no staged files. Stage your changes before authorizing.\n' >&2
    exit 1
fi

CLASSIFIER="$PROJECT_ROOT/tools/harness/cold_tier_classifier.sh"
if [[ ! -x "$CLASSIFIER" ]]; then
    printf '[authorize-structural] cold_tier_classifier.sh missing or not executable: %s\n' "$CLASSIFIER" >&2
    exit 1
fi

CLASSIFIER_LOG="$EVIDENCE_DIR/cold_tier_classifier.log"
echo "$STAGED" | bash "$CLASSIFIER" - >"$CLASSIFIER_LOG" 2>&1
CLASSIFIER_EXIT=$?

if [[ $CLASSIFIER_EXIT -ne 0 ]]; then
    printf '[authorize-structural] cold_tier NOT confirmed. STRUCTURAL-COMMIT denied (Hot tier requires harness pipeline).\n' >&2
    cat "$CLASSIFIER_LOG" >&2
    printf '\nStaged files:\n%s\n' "$STAGED" >&2
    exit 1
fi
printf '       PASS — cold tier 4 signals confirmed.\n'
printf '       Log saved to %s\n' "$CLASSIFIER_LOG"

# Step 4: Write marker + audit log entry
printf '[4/4] Writing marker + audit log entry ...\n'

NOW_EPOCH=$(date +%s)
NOW_ISO=$(date -u +%Y-%m-%dT%H:%M:%SZ)

{
    printf 'feature: %s\n' "$FEATURE"
    printf 'reason: %s\n' "$REASON"
    printf 'authorized_at: %s\n' "$NOW_ISO"
    printf 'authorized_epoch: %s\n' "$NOW_EPOCH"
    printf 'authorized_by: %s\n' "$AUTHORIZED_BY"
    printf 'cargo_test_result: PASS\n'
    printf 'clippy_result: CLEAN\n'
    printf 'cold_tier: CONFIRMED\n'
    printf 'evidence_dir: %s\n' "$EVIDENCE_DIR"
} > "$MARKER"

{
    printf '%s|%s|%s|%s\n' \
        "$NOW_ISO" \
        "$FEATURE" \
        "$AUTHORIZED_BY" \
        "$REASON"
} >> "$LOG"

printf '       Marker:  %s\n' "$MARKER"
printf '       Audit:   %s\n' "$LOG"

printf '\n=== AUTHORIZED ===\n'
printf 'Validity: 2 hours from %s\n' "$NOW_ISO"
printf '\nNext commit MUST:\n'
printf '  - Include [STRUCTURAL] tag in subject line\n'
printf '  - Reference feature slug: %s\n' "$FEATURE"
printf '  - Reference reason: %s\n' "$REASON"
printf '\nMarker is one-shot — consumed by pre-commit hook on next commit.\n'

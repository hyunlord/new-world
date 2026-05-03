#!/usr/bin/env bash
# ENV-BYPASS Authorization Script (CLAUDE.md Rule 7.1)
#
# Usage: bash tools/harness/authorize_env_bypass.sh <feature_slug> <env_block_reason>
#
# Establishes a one-shot, time-limited (≤2 h) authorization for committing
# code while a harness pipeline cannot run due to an *environmental* block
# (Claude API rate limit, network outage, MCP unavailable, etc.).
#
# Prerequisites enforced by this script (v3.2.1 — baseline-aware):
#   1. cargo test --workspace must pass OR fail only on locations registered in
#      .harness/baseline/known_failures.txt (saved to manual_verification.log)
#   2. cargo clippy --workspace --all-targets -- -D warnings must be clean OR
#      fail only on file:line locations registered in
#      .harness/baseline/clippy_baseline_raw.txt (saved to clippy_full.log,
#      diff in clippy_baseline_diff.txt). Tolerates Rust toolchain drift.
#   3. Interactive operator confirmation (typed "yes")
#
# Side effects:
#   - Creates .harness/audit/env_bypass_active marker (read by stop-check.sh)
#   - Appends an entry to .harness/audit/env_bypass.log audit trail
#
# After authorization the next commit MUST:
#   - Be tagged with [ENV-BYPASS] in the subject line
#   - Be followed within 7 days by a formal harness re-run
set -uo pipefail

if [[ $# -lt 2 ]]; then
    printf 'Usage: %s <feature_slug> <env_block_reason>\n' "$0" >&2
    exit 1
fi

FEATURE="$1"
REASON="$2"

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [[ -z "$PROJECT_ROOT" ]]; then
    printf '[authorize-env-bypass] not in a git repository\n' >&2
    exit 1
fi

cd "$PROJECT_ROOT"

EVIDENCE_DIR="$PROJECT_ROOT/.harness/evidence/$FEATURE"
AUDIT_DIR="$PROJECT_ROOT/.harness/audit"
MARKER="$AUDIT_DIR/env_bypass_active"
LOG="$AUDIT_DIR/env_bypass.log"

mkdir -p "$EVIDENCE_DIR" "$AUDIT_DIR"

printf '\n=== ENV-BYPASS Authorization for %s ===\n' "$FEATURE"
printf 'Reason: %s\n\n' "$REASON"

# Step 1: cargo test --workspace (baseline-aware — see CLAUDE.md Rule 7.1)
printf '[1/3] Running cargo test --workspace ...\n'
TEST_LOG="$EVIDENCE_DIR/manual_verification.log"
# cargo test exits non-zero when any test fails. Capture the full log unconditionally;
# we classify the failures against the baseline registry below.
( cd "$PROJECT_ROOT/rust" && cargo test --workspace --no-fail-fast ) >"$TEST_LOG" 2>&1 || true

if ! grep -E 'test result:' "$TEST_LOG" >/dev/null; then
    printf '[authorize-env-bypass] cargo test produced no test results — likely a build failure. See %s\n' "$TEST_LOG" >&2
    tail -30 "$TEST_LOG" >&2
    exit 1
fi

# Extract every FAILED test name. Format from rustc:
#   "test <path::name> ... FAILED"
ACTUAL_FAILS=$(grep -E '^test [^ ]+ \.\.\. FAILED$' "$TEST_LOG" \
    | sed -E 's/^test ([^ ]+) \.\.\. FAILED$/\1/' \
    | sort -u)

# Load baseline registry (strip comments + blank lines).
BASELINE_FILE="$PROJECT_ROOT/.harness/baseline/known_failures.txt"
if [[ -f "$BASELINE_FILE" ]]; then
    BASELINE=$(grep -Ev '^\s*(#|$)' "$BASELINE_FILE" | sort -u)
else
    BASELINE=""
fi

# NEW_FAILS = ACTUAL_FAILS \ BASELINE (regressions vs baseline).
NEW_FAILS=$(comm -23 <(printf '%s\n' "$ACTUAL_FAILS") <(printf '%s\n' "$BASELINE"))
# RESOLVED = BASELINE \ ACTUAL_FAILS (baseline entries that no longer fail).
RESOLVED=$(comm -13 <(printf '%s\n' "$ACTUAL_FAILS") <(printf '%s\n' "$BASELINE"))

ACTUAL_COUNT=$(printf '%s\n' "$ACTUAL_FAILS" | grep -cve '^$' || true)
BASELINE_COUNT=$(printf '%s\n' "$BASELINE" | grep -cve '^$' || true)
NEW_COUNT=$(printf '%s\n' "$NEW_FAILS" | grep -cve '^$' || true)
RESOLVED_COUNT=$(printf '%s\n' "$RESOLVED" | grep -cve '^$' || true)

printf '       Failures observed: %s (baseline registered: %s)\n' "$ACTUAL_COUNT" "$BASELINE_COUNT"
if [[ "$RESOLVED_COUNT" -gt 0 ]]; then
    printf '       Resolved baseline entries (consider removing from registry):\n'
    printf '%s\n' "$RESOLVED" | sed 's/^/         - /'
fi
if [[ "$NEW_COUNT" -gt 0 ]]; then
    printf '[authorize-env-bypass] %s NEW regression(s) not in baseline registry:\n' "$NEW_COUNT" >&2
    printf '%s\n' "$NEW_FAILS" | sed 's/^/  - /' >&2
    printf '\nFix the regressions or document them in %s before authorizing.\n' "$BASELINE_FILE" >&2
    exit 1
fi
printf '       PASS — only documented baseline failures present (%s of %s).\n' \
    "$ACTUAL_COUNT" "$BASELINE_COUNT"
printf '       Log saved to %s\n' "$TEST_LOG"

# Step 2: per-crate clippy (baseline-aware)
# v3.2.1: Tolerates pre-existing baseline clippy issues (toolchain drift) but
# blocks any NEW issue introduced by the change. See CLAUDE.md Rule 7.1.
#
# Per-crate iteration is mandatory: `cargo clippy --workspace --all-targets`
# short-circuits when the first crate's lib-test errors abort dependent
# compilation, leaving downstream crates (sim-bridge, sim-test) unchecked
# and producing a falsely small ACTUAL set.
printf '[2/3] Running per-crate cargo clippy --all-targets -- -D warnings ...\n'
CLIPPY_LOG="$EVIDENCE_DIR/clippy_full.log"
CLIPPY_DIFF="$EVIDENCE_DIR/clippy_baseline_diff.txt"
: > "$CLIPPY_LOG"
for crate in sim-core sim-data sim-engine sim-systems sim-bridge sim-test; do
    printf '       -- %s --\n' "$crate" | tee -a "$CLIPPY_LOG" >/dev/null
    ( cd "$PROJECT_ROOT/rust" && cargo clippy -p "$crate" --all-targets -- -D warnings ) \
        >>"$CLIPPY_LOG" 2>&1 || true
done

# Extract (file, lint_name) fingerprints from the per-crate logs. Using
# fingerprints (not file:line:col) keeps the registry stable across line
# shifts when patches modify baseline-affected files. The Python extractor
# resolves each error block's lint via the help-URL anchor, which is more
# reliable than the `-D clippy::xxx` note (clippy emits the note only once
# per lint kind per compilation run).
EXTRACTOR="$PROJECT_ROOT/tools/harness/extract_clippy_fingerprints.py"
ACTUAL_CLIPPY=$(python3 "$EXTRACTOR" "$CLIPPY_LOG" 2>>"$CLIPPY_LOG" | sort -u)

CLIPPY_BASELINE_FILE="$PROJECT_ROOT/.harness/baseline/clippy_baseline_raw.txt"
if [[ -f "$CLIPPY_BASELINE_FILE" ]]; then
    CLIPPY_BASELINE=$(grep -Ev '^\s*(#|$)' "$CLIPPY_BASELINE_FILE" | sort -u)
else
    CLIPPY_BASELINE=""
fi

# NEW_CLIPPY = ACTUAL_CLIPPY \ CLIPPY_BASELINE (regressions vs baseline).
NEW_CLIPPY=$(comm -23 <(printf '%s\n' "$ACTUAL_CLIPPY") <(printf '%s\n' "$CLIPPY_BASELINE"))
# RESOLVED_CLIPPY = CLIPPY_BASELINE \ ACTUAL_CLIPPY (baseline entries no longer present).
RESOLVED_CLIPPY=$(comm -13 <(printf '%s\n' "$ACTUAL_CLIPPY") <(printf '%s\n' "$CLIPPY_BASELINE"))

ACTUAL_CLIPPY_COUNT=$(printf '%s\n' "$ACTUAL_CLIPPY" | grep -cve '^$' || true)
BASELINE_CLIPPY_COUNT=$(printf '%s\n' "$CLIPPY_BASELINE" | grep -cve '^$' || true)
NEW_CLIPPY_COUNT=$(printf '%s\n' "$NEW_CLIPPY" | grep -cve '^$' || true)
RESOLVED_CLIPPY_COUNT=$(printf '%s\n' "$RESOLVED_CLIPPY" | grep -cve '^$' || true)

# Persist diff for evidence trail
{
    printf '# Clippy baseline diff for %s\n' "$FEATURE"
    printf '# Generated: %s\n\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf '## Counts\n'
    printf 'baseline: %s\n' "$BASELINE_CLIPPY_COUNT"
    printf 'actual:   %s\n' "$ACTUAL_CLIPPY_COUNT"
    printf 'new:      %s\n' "$NEW_CLIPPY_COUNT"
    printf 'resolved: %s\n\n' "$RESOLVED_CLIPPY_COUNT"
    printf '## NEW issues (would block)\n'
    printf '%s\n' "$NEW_CLIPPY"
    printf '\n## RESOLVED baseline entries (consider regenerating registry)\n'
    printf '%s\n' "$RESOLVED_CLIPPY"
} > "$CLIPPY_DIFF"

printf '       Issues observed: %s (baseline registered: %s)\n' \
    "$ACTUAL_CLIPPY_COUNT" "$BASELINE_CLIPPY_COUNT"
if [[ "$RESOLVED_CLIPPY_COUNT" -gt 0 ]]; then
    printf '       Resolved baseline entries (consider regenerating registry):\n'
    printf '%s\n' "$RESOLVED_CLIPPY" | sed 's/^/         - /'
fi
if [[ "$NEW_CLIPPY_COUNT" -gt 0 ]]; then
    printf '[authorize-env-bypass] %s NEW clippy issue(s) not in baseline registry:\n' "$NEW_CLIPPY_COUNT" >&2
    printf '%s\n' "$NEW_CLIPPY" | sed 's/^/  - /' >&2
    printf '\nFix the new issues or document them in %s before authorizing.\n' "$CLIPPY_BASELINE_FILE" >&2
    printf 'Full clippy log: %s\n' "$CLIPPY_LOG" >&2
    printf 'Diff report:     %s\n' "$CLIPPY_DIFF" >&2
    exit 1
fi
printf '       PASS — only documented baseline issues present (%s of %s).\n' \
    "$ACTUAL_CLIPPY_COUNT" "$BASELINE_CLIPPY_COUNT"
printf '       Log saved to %s\n' "$CLIPPY_LOG"
printf '       Diff saved to %s\n' "$CLIPPY_DIFF"

# Step 3: Interactive confirmation
printf '\n[3/3] Operator confirmation\n'
printf 'Local verification PASSED. Authorize ENV-BYPASS commit for %s?\n' "$FEATURE"
printf 'Type "yes" to confirm, anything else aborts: '
read -r REPLY
if [[ "$REPLY" != "yes" ]]; then
    printf '[authorize-env-bypass] operator declined. No marker written.\n' >&2
    exit 1
fi

NOW_EPOCH=$(date +%s)
NOW_ISO=$(date -u +%Y-%m-%dT%H:%M:%SZ)
USER_ID="${USER:-unknown}"

# Write marker (consumed once by stop-check.sh, then deleted)
{
    printf 'feature: %s\n' "$FEATURE"
    printf 'reason: %s\n' "$REASON"
    printf 'authorized_at: %s\n' "$NOW_ISO"
    printf 'authorized_epoch: %s\n' "$NOW_EPOCH"
    printf 'authorized_by: %s\n' "$USER_ID"
    printf 'evidence_dir: %s\n' "$EVIDENCE_DIR"
} > "$MARKER"

# Append to audit log
{
    printf '%s|%s|%s|%s\n' \
        "$NOW_ISO" \
        "$FEATURE" \
        "$USER_ID" \
        "$REASON"
} >> "$LOG"

printf '\n=== AUTHORIZED ===\n'
printf 'Marker:  %s\n' "$MARKER"
printf 'Audit:   %s\n' "$LOG"
printf 'Validity: 2 hours from %s\n' "$NOW_ISO"
printf '\nNext commit MUST:\n'
printf '  - Include [ENV-BYPASS] in the subject line\n'
printf '  - Reference reason: %s\n' "$REASON"
printf '  - Be followed within 7 days by a formal harness re-run\n'
printf '\nUse: HARNESS_SKIP=1 git commit ... (the stop hook honours the marker)\n'

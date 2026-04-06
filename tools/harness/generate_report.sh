#!/usr/bin/env bash
# Generate pipeline report — immutable audit trail of every step.
# Called by harness_pipeline.sh at the APPROVE step.
# Usage: bash tools/harness/generate_report.sh <feature>
set -uo pipefail

FEATURE="${1:?Usage: generate_report.sh <feature>}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

HARNESS_DIR="$PROJECT_ROOT/.harness"
REPORT_DIR="$HARNESS_DIR/reports/$FEATURE"
PLAN_DIR="$HARNESS_DIR/plans/$FEATURE"
RESULT_DIR="$HARNESS_DIR/results/$FEATURE"
EVIDENCE_DIR="$HARNESS_DIR/evidence/$FEATURE"
REVIEW_DIR="$HARNESS_DIR/reviews/$FEATURE"

mkdir -p "$REPORT_DIR"

REPORT="$REPORT_DIR/pipeline_report.md"

# Count artifacts
PLAN_ROUNDS=$(ls "$PLAN_DIR"/quality_review_round*.md 2>/dev/null | wc -l | tr -d ' ')
CODE_ATTEMPTS=$(ls "$RESULT_DIR"/gen_result_attempt*.md 2>/dev/null | wc -l | tr -d ' ')
SCREENSHOTS=$(find "$EVIDENCE_DIR" -name "screenshot_*.png" 2>/dev/null | wc -l | tr -d ' ')

# Extract verdicts
QC_VERDICTS=""
for qr in "$PLAN_DIR"/quality_review_round*.md; do
    [[ -f "$qr" ]] || continue
    v=$(sed 's/\*//g; s/_//g' "$qr" | grep -i "^verdict:" | head -1 | awk '{print $2}')
    QC_VERDICTS+="$(basename "$qr"): $v, "
done

EVAL_VERDICTS=""
for er in "$REVIEW_DIR"/review_attempt*.md; do
    [[ -f "$er" ]] || continue
    v=$(sed 's/\*//g; s/_//g' "$er" | grep -i "^verdict:" | head -1 | awk '{print $2}')
    EVAL_VERDICTS+="$(basename "$er"): $v, "
done

VISUAL_VERDICT="SKIPPED"
if [[ -f "$EVIDENCE_DIR/visual_analysis.txt" ]]; then
    VISUAL_VERDICT=$(grep -i "VISUAL_OK\|VISUAL_WARNING\|VISUAL_FAIL" "$EVIDENCE_DIR/visual_analysis.txt" | tail -1 | grep -o "VISUAL_[A-Z]*" || echo "UNKNOWN")
fi

# Drafter summary
DRAFTER_ASSERTIONS=0
if [[ -f "$PLAN_DIR/plan_draft.md" ]]; then
    DRAFTER_ASSERTIONS=$(grep -c "^### Assertion" "$PLAN_DIR/plan_draft.md" 2>/dev/null || echo "0")
fi

# Challenger summary
CHALLENGER_ISSUES=0
CHALLENGER_GAMING=0
if [[ -f "$PLAN_DIR/challenge_report.md" ]]; then
    CHALLENGER_ISSUES=$(grep -c "\[ISSUE\]" "$PLAN_DIR/challenge_report.md" 2>/dev/null || echo "0")
    CHALLENGER_GAMING=$(grep -ci "gaming\|cheat\|hardcoded\|bypass" "$PLAN_DIR/challenge_report.md" 2>/dev/null || echo "0")
fi

# Final verdict
FINAL_VERDICT=$(head -1 "$REVIEW_DIR/verdict" 2>/dev/null || echo "UNKNOWN")

# Generator files changed
GEN_FILES=""
LATEST_GEN=$(ls "$RESULT_DIR"/gen_result_attempt*.md 2>/dev/null | tail -1)
if [[ -f "$LATEST_GEN" ]]; then
    GEN_FILES=$(grep -A50 "## Files Changed" "$LATEST_GEN" | grep "^- " | head -15 || echo "(none)")
fi

# Revision diff check
REVISION_STATUS="NOT GENERATED"
if [[ -f "$PLAN_DIR/plan_revised.md" ]]; then
    if diff -q "$PLAN_DIR/plan_draft.md" "$PLAN_DIR/plan_revised.md" >/dev/null 2>&1; then
        REVISION_STATUS="IDENTICAL to draft (Challenger feedback not incorporated)"
    else
        REVISION_STATUS="DIFFERENT from draft (Challenger feedback incorporated)"
    fi
fi

# Artifact inventory
ARTIFACT_LIST=$(find "$HARNESS_DIR" -path "*/$FEATURE/*" -type f 2>/dev/null | sort | sed 's/^/  /' || echo "  (none)")

cat > "$REPORT" << REPORT_EOF
# Pipeline Report: $FEATURE

> Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
> Pipeline: harness v3.1

## Summary

| Metric | Value |
|--------|-------|
| Final verdict | **$FINAL_VERDICT** |
| Plan debate rounds | $PLAN_ROUNDS |
| Code attempts | $CODE_ATTEMPTS |
| Visual verdict | $VISUAL_VERDICT |
| Screenshots | $SCREENSHOTS |

## Step 1: Planning Phase

### 1a Drafter
- Assertions: $DRAFTER_ASSERTIONS
- Plan: $PLAN_DIR/plan_draft.md

### 1b Challenger
- Issues found: $CHALLENGER_ISSUES
- Gaming vectors: $CHALLENGER_GAMING
- Report: $PLAN_DIR/challenge_report.md

### 1c Drafter Revision
- Revision: $REVISION_STATUS

### 1d Quality Checker
- Rounds: $PLAN_ROUNDS
- Verdicts: ${QC_VERDICTS:-"none"}

## Step 2: Implementation

### Generator
- Attempts: $CODE_ATTEMPTS
- Files changed:
$GEN_FILES

## Step 2.5: Visual Verification

- Screenshots: $SCREENSHOTS
- Visual verdict: $VISUAL_VERDICT

## Step 3: Evaluation

- Verdicts: ${EVAL_VERDICTS:-"none"}
- Final: **$FINAL_VERDICT**

## Artifact Inventory

\`\`\`
$ARTIFACT_LIST
\`\`\`
REPORT_EOF

echo "$REPORT"

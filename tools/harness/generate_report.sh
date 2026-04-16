#!/usr/bin/env bash
# Generate pipeline report v2 — scoring system + process summary + performance data.
# Called by harness_pipeline.sh at pipeline completion (APPROVE, RE-CODE, RE-PLAN, FAIL).
# Usage: bash tools/harness/generate_report.sh <feature> [--mode <mode>]
set -uo pipefail

FEATURE="${1:?Usage: generate_report.sh <feature> [--mode <mode>]}"
MODE="${3:---quick}"  # default if not passed

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

HARNESS_DIR="$PROJECT_ROOT/.harness"
REPORT_DIR="$HARNESS_DIR/reports/$FEATURE"
PLAN_DIR="$HARNESS_DIR/plans/$FEATURE"
RESULT_DIR="$HARNESS_DIR/results/$FEATURE"
EVIDENCE_DIR="$HARNESS_DIR/evidence/$FEATURE"
REVIEW_DIR="$HARNESS_DIR/reviews/$FEATURE"
PROGRESS_FILE="$HARNESS_DIR/progress/$FEATURE/progress.md"

mkdir -p "$REPORT_DIR"

REPORT="$REPORT_DIR/pipeline_report.md"

# ── Data Collection ──────────────────────────────────────────────────────

# Count artifacts
PLAN_ROUNDS=$(ls "$PLAN_DIR"/quality_review_round*.md 2>/dev/null | wc -l | tr -d ' ')
CODE_ATTEMPTS=$(ls "$RESULT_DIR"/gen_result_attempt*.md 2>/dev/null | wc -l | tr -d ' ')
SCREENSHOTS=$(find "$EVIDENCE_DIR" -name "screenshot_*.png" 2>/dev/null | wc -l | tr -d ' ')

# Extract QC verdicts
QC_VERDICTS=""
for qr in "$PLAN_DIR"/quality_review_round*.md; do
    [[ -f "$qr" ]] || continue
    v=$(sed 's/\*//g; s/_//g' "$qr" | grep -i "^verdict:" | head -1 | awk '{print $2}')
    QC_VERDICTS+="$(basename "$qr"): $v, "
done

# Extract evaluator verdicts per attempt
EVAL_VERDICTS=""
for er in "$REVIEW_DIR"/review_attempt*.md; do
    [[ -f "$er" ]] || continue
    v=$(sed 's/\*//g; s/_//g' "$er" | grep -i "^verdict:" | head -1 | awk '{print $2}')
    EVAL_VERDICTS+="$(basename "$er"): $v, "
done

# Visual verdict
VISUAL_VERDICT="SKIPPED"
if [[ -f "$EVIDENCE_DIR/visual_analysis.txt" ]]; then
    VISUAL_VERDICT=$(grep -o "VISUAL_[A-Z]*" "$EVIDENCE_DIR/visual_analysis.txt" | tail -1 || echo "UNKNOWN")
fi

# Drafter assertions
DRAFTER_ASSERTIONS=0
if [[ -f "$PLAN_DIR/plan_draft.md" ]]; then
    DRAFTER_ASSERTIONS=$(grep -cE "^#{2,3} Assertion|\*\*Assertion" "$PLAN_DIR/plan_draft.md" 2>/dev/null || true)
    DRAFTER_ASSERTIONS="${DRAFTER_ASSERTIONS:-0}"
fi

# Challenger stats
CHALLENGER_ISSUES=0
if [[ -f "$PLAN_DIR/challenge_report.md" ]]; then
    CHALLENGER_ISSUES=$(grep -c "\[ISSUE\]\|^### Issue\|^- Issue" "$PLAN_DIR/challenge_report.md" 2>/dev/null || echo "0")
fi

# Final verdict — cross-check verdict file with actual latest review
FINAL_VERDICT=$(head -1 "$REVIEW_DIR/verdict" 2>/dev/null || echo "UNKNOWN")
LATEST_REVIEW_FILE=$(ls "$REVIEW_DIR"/review_attempt*.md 2>/dev/null | tail -1)
if [[ -f "$LATEST_REVIEW_FILE" ]]; then
    LATEST_REVIEW_VERDICT=$(sed 's/\*//g; s/_//g' "$LATEST_REVIEW_FILE" | grep -i "^verdict:" | head -1 | awk '{print $2}')
    # If verdict file disagrees with actual latest review, trust the review
    if [[ -n "$LATEST_REVIEW_VERDICT" ]]; then
        case "$FINAL_VERDICT" in
            APPROVED|APPROVE)
                if [[ "$LATEST_REVIEW_VERDICT" != "APPROVE" && "$LATEST_REVIEW_VERDICT" != "APPROVED" ]]; then
                    FINAL_VERDICT="$LATEST_REVIEW_VERDICT"
                fi
                ;;
        esac
    fi
fi

# Gate data — test count from gate_result (most complete), fallback to step0_test
GATE_TEST_COUNT="0"
GATE_TEST_STATUS="UNKNOWN"
LATEST_GATE=$(ls "$RESULT_DIR"/gate_result_attempt*.txt 2>/dev/null | tail -1)
if [[ -f "$LATEST_GATE" ]]; then
    GATE_TEST_COUNT=$(grep -c "^test .*\.\.\. ok$" "$LATEST_GATE" 2>/dev/null || echo "0")
    if grep -q "FAILED\|^test .* FAILED" "$LATEST_GATE"; then
        GATE_TEST_STATUS="FAIL"
    else
        GATE_TEST_STATUS="PASS"
    fi
elif [[ -f "$RESULT_DIR/step0_test.txt" ]]; then
    if grep -q "test result: ok" "$RESULT_DIR/step0_test.txt" && ! grep -q "FAILED" "$RESULT_DIR/step0_test.txt"; then
        GATE_TEST_STATUS="PASS"
    else
        GATE_TEST_STATUS="FAIL"
    fi
fi

CLIPPY_STATUS="UNKNOWN"
if [[ -f "$RESULT_DIR/step0_clippy.txt" ]]; then
    if grep -q "^error\[" "$RESULT_DIR/step0_clippy.txt"; then
        CLIPPY_STATUS="FAIL"
    else
        CLIPPY_STATUS="clean"
    fi
fi

FFI_STATUS="UNKNOWN"
if [[ -f "$RESULT_DIR/step0_ffi.txt" ]]; then
    if grep -qi "OK\|PASS\|COMPLETE" "$RESULT_DIR/step0_ffi.txt"; then
        FFI_STATUS="OK"
    else
        FFI_STATUS="FAIL"
    fi
fi

# Performance data
AVG_TICK="N/A"
EST_TPS="N/A"
FPS_VAL="N/A"
if [[ -f "$EVIDENCE_DIR/performance.txt" ]]; then
    AVG_TICK=$(grep -i "avg tick" "$EVIDENCE_DIR/performance.txt" | grep -oE '[0-9]+\.[0-9]+' | head -1 || echo "N/A")
    EST_TPS=$(grep -i "Est\. TPS\|TPS" "$EVIDENCE_DIR/performance.txt" | grep -oE '[0-9]+\.[0-9]+' | head -1 || echo "N/A")
    FPS_VAL=$(grep -i "frames_per_second\|FPS" "$EVIDENCE_DIR/performance.txt" | grep -oE '[0-9]+' | head -1 || echo "N/A")
fi

# Regression guard
REGRESSION_STATUS="NOT_RUN"
if [[ -f "$REVIEW_DIR/regression_guard.txt" ]]; then
    if grep -qi "CLEAN\|NO_REGRESSION" "$REVIEW_DIR/regression_guard.txt"; then
        REGRESSION_STATUS="CLEAN"
    else
        REGRESSION_STATUS="REGRESSION_DETECTED"
    fi
fi

# FFI chain verify
FFI_CHAIN="NOT_RUN"
if [[ -f "$EVIDENCE_DIR/ffi_chain_verify.txt" ]]; then
    FFI_CHAIN=$(grep -o "ALL_COMPLETE\|HAS_BROKEN" "$EVIDENCE_DIR/ffi_chain_verify.txt" | tail -1 || echo "UNKNOWN")
fi

# New harness test count — from the latest gate_result (most reliable)
NEW_HARNESS_TESTS=0
if [[ -f "$LATEST_GATE" ]]; then
    NEW_HARNESS_TESTS=$(grep -c "^test tests::harness_.*ok$" "$LATEST_GATE" 2>/dev/null || true)
    NEW_HARNESS_TESTS="${NEW_HARNESS_TESTS:-0}"
fi

# Duration from progress file
DURATION="N/A"
if [[ -f "$PROGRESS_FILE" ]]; then
    local_start=$(grep "start_time:" "$PROGRESS_FILE" 2>/dev/null | head -1 | awk '{print $2}')
    local_end=$(grep "end_time:" "$PROGRESS_FILE" 2>/dev/null | tail -1 | awk '{print $2}')
    if [[ -n "$local_start" && -n "$local_end" && "$local_end" -gt "$local_start" ]] 2>/dev/null; then
        duration_sec=$((local_end - local_start))
    else
        # Fallback: parse first and last HH:MM:SS timestamps from progress table
        first_ts=$(grep "^|" "$PROGRESS_FILE" | grep -v "Time\|---" | head -1 | awk -F'|' '{print $2}' | tr -d ' ')
        last_ts=$(grep "^|" "$PROGRESS_FILE" | grep -v "Time\|---" | tail -1 | awk -F'|' '{print $2}' | tr -d ' ')
        if [[ -n "$first_ts" && -n "$last_ts" ]]; then
            # Convert HH:MM:SS to seconds
            IFS=: read -r h1 m1 s1 <<< "$first_ts"
            IFS=: read -r h2 m2 s2 <<< "$last_ts"
            start_secs=$(( 10#$h1 * 3600 + 10#$m1 * 60 + 10#$s1 ))
            end_secs=$(( 10#$h2 * 3600 + 10#$m2 * 60 + 10#$s2 ))
            # Handle day wraparound (e.g. 23:50 → 00:10)
            if [[ $end_secs -lt $start_secs ]]; then
                end_secs=$((end_secs + 86400))
            fi
            duration_sec=$((end_secs - start_secs))
        fi
    fi
    # Format duration_sec → "Xh Ym Zs"
    if [[ -n "${duration_sec:-}" && "$duration_sec" -gt 0 ]] 2>/dev/null; then
        dur_h=$((duration_sec / 3600))
        dur_m=$(( (duration_sec % 3600) / 60 ))
        dur_s=$((duration_sec % 60))
        if [[ $dur_h -gt 0 ]]; then
            DURATION="${dur_h}h ${dur_m}m ${dur_s}s"
        elif [[ $dur_m -gt 0 ]]; then
            DURATION="${dur_m}m ${dur_s}s"
        else
            DURATION="${dur_s}s"
        fi
    fi
fi

# Changed files — prefer git diff --numstat, fallback to gen_result extraction
CHANGES_TABLE=""
if git -C "$PROJECT_ROOT" rev-parse HEAD~1 &>/dev/null; then
    # Parse git diff --numstat: "added\tdeleted\tfile"
    while IFS=$'\t' read -r added deleted file; do
        [[ -z "$file" ]] && continue
        CHANGES_TABLE+="- \`$file\` (+${added}/-${deleted})"$'\n'
    done < <(git -C "$PROJECT_ROOT" diff --numstat HEAD~1 -- 2>/dev/null)
fi
# Fallback if git diff produced nothing
if [[ -z "$CHANGES_TABLE" ]]; then
    LATEST_GEN=$(ls "$RESULT_DIR"/gen_result_attempt*.md 2>/dev/null | tail -1 || true)
    if [[ -n "$LATEST_GEN" && -f "$LATEST_GEN" ]]; then
        CHANGES_TABLE=$(awk '/^## Files Changed|^## Changed Files/{found=1;next} /^## [^F]/{if(found)exit} found && /^- /' "$LATEST_GEN" 2>/dev/null || true)
    fi
fi

# ── Score Calculation ────────────────────────────────────────────────────

# 1. Mechanical Gate (10)
# Tests must pass for ANY gate points. clippy/FFI are bonus only if tests pass.
SCORE_GATE=0
GATE_DETAIL=""
if [[ "$GATE_TEST_STATUS" == "PASS" ]]; then
    SCORE_GATE=6
    GATE_DETAIL="test ${GATE_TEST_COUNT} passed"
    if [[ "$CLIPPY_STATUS" == "clean" ]]; then
        SCORE_GATE=$((SCORE_GATE + 2))
        GATE_DETAIL+=", clippy clean"
    else
        GATE_DETAIL+=", clippy $CLIPPY_STATUS"
    fi
    if [[ "$FFI_STATUS" == "OK" ]]; then
        SCORE_GATE=$((SCORE_GATE + 2))
        GATE_DETAIL+=", FFI OK"
    else
        GATE_DETAIL+=", FFI $FFI_STATUS"
    fi
else
    # Gate = 0 when tests fail — no partial credit
    SCORE_GATE=0
    GATE_DETAIL="test FAIL"
    [[ "$CLIPPY_STATUS" == "clean" ]] && GATE_DETAIL+=", clippy clean"
    [[ "$FFI_STATUS" == "OK" ]] && GATE_DETAIL+=", FFI OK"
fi

# 2. Plan Quality (5)
SCORE_PLAN=0
PLAN_DETAIL=""
if [[ "$MODE" == "--quick" || "$MODE" == "--light" ]]; then
    SCORE_PLAN=5
    PLAN_DETAIL="auto (${MODE} mode)"
else
    if [[ $PLAN_ROUNDS -gt 0 ]]; then
        SCORE_PLAN=$((SCORE_PLAN + 2))
        PLAN_DETAIL="$PLAN_ROUNDS debate round(s)"
    else
        PLAN_DETAIL="no debate"
    fi
    if echo "$QC_VERDICTS" | grep -qi "APPROVED\|PLAN_APPROVED\|PLANAPPROVED"; then
        SCORE_PLAN=$((SCORE_PLAN + 3))
        PLAN_DETAIL+=", QC APPROVED"
    else
        PLAN_DETAIL+=", QC ${QC_VERDICTS:-none}"
    fi
fi

# 3. Code Quality (15)
# Only APPROVE earns points. RE-CODE/RE-PLAN/FAIL = 0 (pipeline failed to resolve).
SCORE_CODE=0
CODE_DETAIL=""
case "$FINAL_VERDICT" in
    APPROVED|APPROVE)
        if [[ $CODE_ATTEMPTS -eq 1 ]]; then
            SCORE_CODE=15; CODE_DETAIL="APPROVE on attempt 1"
        elif [[ $CODE_ATTEMPTS -eq 2 ]]; then
            SCORE_CODE=11; CODE_DETAIL="APPROVE on attempt 2"
        else
            SCORE_CODE=8; CODE_DETAIL="APPROVE on attempt $CODE_ATTEMPTS"
        fi
        ;;
    RE-CODE|RECODE)
        SCORE_CODE=0; CODE_DETAIL="RE-CODE after $CODE_ATTEMPTS attempt(s)"
        ;;
    RE-PLAN|REPLAN)
        SCORE_CODE=0; CODE_DETAIL="RE-PLAN after $CODE_ATTEMPTS attempt(s)"
        ;;
    *)
        SCORE_CODE=0; CODE_DETAIL="$FINAL_VERDICT"
        ;;
esac

# 4. Test Coverage (20)
SCORE_TESTS=0
TEST_DETAIL=""
test_cap=$NEW_HARNESS_TESTS
if [[ $test_cap -gt 10 ]]; then test_cap=10; fi
SCORE_TESTS=$((test_cap * 2))
TEST_DETAIL="$NEW_HARNESS_TESTS new harness tests"

# 5. Visual Verify (20)
SCORE_VISUAL=0
VISUAL_DETAIL=""
# Screenshots: min(count * 2, 8)
ss_score=$((SCREENSHOTS * 2))
if [[ $ss_score -gt 8 ]]; then ss_score=8; fi
SCORE_VISUAL=$ss_score
# VLM verdict: VISUAL_OK=7, VISUAL_WARNING=4, VISUAL_FAIL=0
case "$VISUAL_VERDICT" in
    VISUAL_OK) SCORE_VISUAL=$((SCORE_VISUAL + 7)); VISUAL_DETAIL="$SCREENSHOTS screenshots, VLM OK" ;;
    VISUAL_WARNING) SCORE_VISUAL=$((SCORE_VISUAL + 4)); VISUAL_DETAIL="$SCREENSHOTS screenshots, VLM WARNING" ;;
    VISUAL_FAIL) SCORE_VISUAL=$((SCORE_VISUAL + 0)); VISUAL_DETAIL="$SCREENSHOTS screenshots, VLM FAIL" ;;
    SKIPPED) SCORE_VISUAL=$((SCORE_VISUAL + 0)); VISUAL_DETAIL="$SCREENSHOTS screenshots, VLM skipped" ;;
    *) SCORE_VISUAL=$((SCORE_VISUAL + 0)); VISUAL_DETAIL="$SCREENSHOTS screenshots, VLM $VISUAL_VERDICT" ;;
esac
# Interactive scenarios: PASS=5, else 0
INTERACTIVE_PASS=false
if [[ -f "$EVIDENCE_DIR/interactive_results.txt" ]]; then
    if grep -qi "PASS\|SUCCESS\|ALL.*PASS" "$EVIDENCE_DIR/interactive_results.txt" 2>/dev/null; then
        SCORE_VISUAL=$((SCORE_VISUAL + 5))
        VISUAL_DETAIL+=", interactive PASS"
        INTERACTIVE_PASS=true
    else
        VISUAL_DETAIL+=", interactive FAIL"
    fi
fi
if [[ $SCORE_VISUAL -gt 20 ]]; then SCORE_VISUAL=20; fi

# 6. Regression (15)
SCORE_REGRESSION=0
REGRESSION_DETAIL=""
case "$REGRESSION_STATUS" in
    CLEAN) SCORE_REGRESSION=15; REGRESSION_DETAIL="CLEAN" ;;
    NOT_RUN) SCORE_REGRESSION=5; REGRESSION_DETAIL="not run" ;;
    *) SCORE_REGRESSION=0; REGRESSION_DETAIL="$REGRESSION_STATUS" ;;
esac

# 7. Evaluator (15)
# Only APPROVE earns points. Any non-APPROVE verdict = 0.
SCORE_EVALUATOR=0
EVALUATOR_DETAIL=""
case "$FINAL_VERDICT" in
    APPROVED|APPROVE) SCORE_EVALUATOR=15; EVALUATOR_DETAIL="APPROVE" ;;
    RE-CODE|RECODE) SCORE_EVALUATOR=0; EVALUATOR_DETAIL="RE-CODE" ;;
    RE-PLAN|REPLAN) SCORE_EVALUATOR=0; EVALUATOR_DETAIL="RE-PLAN" ;;
    *) SCORE_EVALUATOR=0; EVALUATOR_DETAIL="$FINAL_VERDICT" ;;
esac

# Total
SCORE_TOTAL=$((SCORE_GATE + SCORE_PLAN + SCORE_CODE + SCORE_TESTS + SCORE_VISUAL + SCORE_REGRESSION + SCORE_EVALUATOR))

# Grade
if [[ $SCORE_TOTAL -ge 95 ]]; then GRADE="A — Ship it!"
elif [[ $SCORE_TOTAL -ge 85 ]]; then GRADE="B — Acceptable"
elif [[ $SCORE_TOTAL -ge 70 ]]; then GRADE="C — Needs work"
else GRADE="F — Reject"
fi

# ── Process Summary Extraction ───────────────────────────────────────────

# Plan summary
PLAN_SUMMARY=""
if [[ "$MODE" == "--quick" ]]; then
    PLAN_SUMMARY="Skipped (--quick mode). $DRAFTER_ASSERTIONS assertions planned."
elif [[ "$MODE" == "--light" ]]; then
    PLAN_SUMMARY="Skipped (--light mode). Prompt used as plan directly."
else
    PLAN_SUMMARY="$DRAFTER_ASSERTIONS assertions drafted. $PLAN_ROUNDS debate round(s)."
    if [[ $CHALLENGER_ISSUES -gt 0 ]]; then
        PLAN_SUMMARY+=" Challenger raised $CHALLENGER_ISSUES issue(s)."
    fi
    if [[ -n "$QC_VERDICTS" ]]; then
        PLAN_SUMMARY+=" QC: ${QC_VERDICTS%%, }"
    fi
fi

# Implementation summary
GEN_SUMMARY="$CODE_ATTEMPTS attempt(s)."
if [[ $CODE_ATTEMPTS -gt 1 ]]; then
    # Extract issues from evaluator reviews
    for i in $(seq 1 $((CODE_ATTEMPTS - 1))); do
        review_file="$REVIEW_DIR/review_attempt${i}.md"
        if [[ -f "$review_file" ]]; then
            attempt_verdict=$(sed 's/\*//g; s/_//g' "$review_file" | grep -i "^verdict:" | head -1 | awk '{print $2}')
            GEN_SUMMARY+=" Attempt $i → $attempt_verdict."
        fi
    done
fi

# Evaluator summary (1-2 sentences from latest review)
EVALUATOR_SUMMARY=""
LATEST_REVIEW=$(ls "$REVIEW_DIR"/review_attempt*.md 2>/dev/null | tail -1)
if [[ -f "$LATEST_REVIEW" ]]; then
    # Extract "Overall Assessment" section or first 2 sentences after verdict
    EVALUATOR_SUMMARY=$(awk '/Overall Assessment/{found=1;next} /^###/{found=0} found' "$LATEST_REVIEW" 2>/dev/null | head -3 | tr '\n' ' ' | sed 's/^[[:space:]]*//' || echo "")
    if [[ -z "$EVALUATOR_SUMMARY" ]]; then
        EVALUATOR_SUMMARY=$(grep -A2 "^verdict:" "$LATEST_REVIEW" 2>/dev/null | tail -2 | tr '\n' ' ' | sed 's/^[[:space:]]*//' || echo "")
    fi
fi

# FFI detail
FFI_DETAIL=""
if [[ -f "$EVIDENCE_DIR/ffi_chain_verify.txt" ]]; then
    ffi_count=$(grep -c "COMPLETE\|BROKEN" "$EVIDENCE_DIR/ffi_chain_verify.txt" 2>/dev/null || echo "0")
    FFI_DETAIL="$FFI_CHAIN ($ffi_count methods checked)"
else
    FFI_DETAIL="not run"
fi

# Regression detail — supports both structured (key: value) and prose formats
REG_DETAIL_TEXT=""
if [[ -f "$REVIEW_DIR/regression_guard.txt" ]]; then
    # Try structured format first (harness_passed: N)
    reg_passed=$(grep -oE 'harness_passed: [0-9]+|Passed: `[0-9]+`' "$REVIEW_DIR/regression_guard.txt" | grep -oE '[0-9]+' | head -1 || echo "?")
    reg_total=$(grep -oE 'harness_total_matched: [0-9]+|Total harness tests run: `[0-9]+`' "$REVIEW_DIR/regression_guard.txt" | grep -oE '[0-9]+' | head -1 || echo "?")
    gate_passed=$(grep -oE 'gate_total_passed: [0-9]+|Total passed: `[0-9]+`' "$REVIEW_DIR/regression_guard.txt" | grep -oE '[0-9]+' | head -1 || echo "?")
    REG_DETAIL_TEXT="$REGRESSION_STATUS. Gate: $gate_passed passed. Harness: $reg_passed/$reg_total."
else
    REG_DETAIL_TEXT="Not run."
fi

# Root cause analysis (only if RE-CODE or RE-PLAN occurred)
ROOT_CAUSE=""
if [[ $CODE_ATTEMPTS -gt 1 ]] || echo "$EVAL_VERDICTS" | grep -qi "RE-CODE\|RE-PLAN\|RECODE\|REPLAN"; then
    ROOT_CAUSE+=$'\n'"## Root Cause Analysis"$'\n'
    for i in $(seq 1 "$CODE_ATTEMPTS"); do
        review_file="$REVIEW_DIR/review_attempt${i}.md"
        [[ -f "$review_file" ]] || continue
        attempt_verdict=$(sed 's/\*//g; s/_//g' "$review_file" | grep -i "^verdict:" | head -1 | awk '{print $2}')
        if [[ "$attempt_verdict" != "APPROVE" && "$attempt_verdict" != "APPROVED" ]]; then
            ROOT_CAUSE+=$'\n'"### Attempt $i → $attempt_verdict"$'\n'
            # Extract issues section
            issues=$(awk '/^### Issues/{found=1;next} /^### [^I]/{found=0} found' "$review_file" 2>/dev/null | head -10 || echo "")
            if [[ -n "$issues" ]]; then
                ROOT_CAUSE+="$issues"$'\n'
            fi
        else
            ROOT_CAUSE+=$'\n'"### Attempt $i → APPROVE"$'\n'
            ROOT_CAUSE+="Implementation accepted."$'\n'
        fi
    done
fi

# ── Generate Report ──────────────────────────────────────────────────────

cat > "$REPORT" << REPORT_EOF
# Pipeline Report: $FEATURE

> Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
> Pipeline: harness v4
> Mode: $MODE
> Duration: $DURATION
> Grade: **$GRADE** ($SCORE_TOTAL/100)

---

## Score Breakdown

| Category | Score | Max | Detail |
|----------|:-----:|:---:|--------|
| Mechanical Gate | $SCORE_GATE | 10 | $GATE_DETAIL |
| Plan Quality | $SCORE_PLAN | 5 | $PLAN_DETAIL |
| Code Quality | $SCORE_CODE | 15 | $CODE_DETAIL |
| Test Coverage | $SCORE_TESTS | 20 | $TEST_DETAIL |
| Visual Verify | $SCORE_VISUAL | 20 | $VISUAL_DETAIL |
| Regression | $SCORE_REGRESSION | 15 | $REGRESSION_DETAIL |
| Evaluator | $SCORE_EVALUATOR | 15 | $EVALUATOR_DETAIL |
| **TOTAL** | **$SCORE_TOTAL** | **100** | **$GRADE** |

---

## Process Summary

### Step 0: Mechanical Gate
$GATE_TEST_STATUS: cargo test $GATE_TEST_COUNT passed, clippy $CLIPPY_STATUS, FFI $FFI_STATUS

### Step 1: Planning
$PLAN_SUMMARY

### Step 2: Implementation
$GEN_SUMMARY

### Step 2.5: Visual Verification
$SCREENSHOTS screenshots captured. VLM: $VISUAL_VERDICT.

### Step 2.5c: FFI Chain
$FFI_DETAIL

### Step 2.7: Regression Guard
$REG_DETAIL_TEXT

### Step 3: Evaluator
Verdict: **$FINAL_VERDICT**
$EVALUATOR_SUMMARY
$ROOT_CAUSE
---

## Changes

$CHANGES_TABLE

---

## Performance

| Metric | Value |
|--------|-------|
| Pipeline duration | $DURATION |
| Workspace tests | $GATE_TEST_COUNT passed |
| New harness tests | $NEW_HARNESS_TESTS |
| Avg tick (ms) | $AVG_TICK |
| Est. TPS | $EST_TPS |
REPORT_EOF

# Append FPS line only if not headless (FPS > 1)
if [[ "$FPS_VAL" != "1" && "$FPS_VAL" != "N/A" ]]; then
    # Insert FPS row before the trailing blank line
    sed -i '' '/^| Est\. TPS/a\
| FPS | '"$FPS_VAL"' |
' "$REPORT"
fi

echo "$REPORT"

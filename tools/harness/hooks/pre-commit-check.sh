#!/usr/bin/env bash
# Layer 1: Claude Code PreToolUse hook for Bash tool
# Blocks git commit/push if code files are staged without harness approval.
# Exit 0 = allow, Exit 2 = hard block
# NOTE: This runs on EVERY Bash tool call. Must exit 0 fast for non-git commands.
set -uo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

# Read stdin JSON to get the command
INPUT=$(cat 2>/dev/null || echo "{}")
CMD=$(echo "$INPUT" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    print(d.get('tool_input', {}).get('command', ''))
except:
    print('')
" 2>/dev/null || echo "")

# Only care about git commit and git push
if ! echo "$CMD" | grep -qE '^\s*git\s+(commit|push)'; then
    exit 0
fi

# ENV-BYPASS (CLAUDE.md Rule 7.1 v3.2.1): a valid env_bypass_active marker
# (≤2 h old, with evidence dir present) authorises the next git commit
# without requiring HARNESS_SKIP=1 or a harness verdict. Marker is one-shot
# and consumed by Layer 2 (hooks/pre-commit-harness) on actual commit.
PROJECT_ROOT_FOR_BYPASS="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"
BYPASS_MARKER="$PROJECT_ROOT_FOR_BYPASS/.harness/audit/env_bypass_active"
if [[ -f "$BYPASS_MARKER" ]] && echo "$CMD" | grep -qE '^\s*git\s+commit'; then
    bypass_mtime=$(stat -f %m "$BYPASS_MARKER" 2>/dev/null || stat -c %Y "$BYPASS_MARKER" 2>/dev/null || echo 0)
    bypass_age=$(( $(date +%s) - bypass_mtime ))
    if [[ $bypass_age -lt 7200 ]]; then
        bypass_feature=$(grep '^feature:' "$BYPASS_MARKER" 2>/dev/null | head -1 | awk '{print $2}')
        evidence_dir="$PROJECT_ROOT_FOR_BYPASS/.harness/evidence/$bypass_feature"
        if [[ -s "$evidence_dir/manual_verification.log" && -s "$evidence_dir/clippy_full.log" ]]; then
            echo "[pre-commit] ENV-BYPASS active for $bypass_feature (age=${bypass_age}s) — allowing commit" >&2
            exit 0
        fi
    fi
fi

# HARNESS_SKIP=1 is FORBIDDEN per CLAUDE.md Rule 9
if [[ "${HARNESS_SKIP:-}" == "1" ]]; then
    echo "BLOCKED: HARNESS_SKIP=1 is FORBIDDEN per CLAUDE.md Rule 9." >&2
    echo "Diagnose the score gap and fix root cause. Do not bypass." >&2
    exit 2
fi

# It's a git commit/push — check for staged code files
STAGED=$(git diff --cached --name-only 2>/dev/null || echo "")

if [[ -z "$STAGED" ]]; then
    exit 0
fi

# Check if any staged file requires approval
NEEDS_APPROVAL=false
CODE_FILES=""
while IFS= read -r file; do
    [[ -z "$file" ]] && continue
    case "$file" in
        *.md|*.txt|localization/*.json|localization/*/*.json|localization/*/*/*.json|\
localization/*/*.ftl|localization/*/*/*.ftl|\
tools/harness/*|.claude/*|hooks/*|.gitignore|.editorconfig|.gitattributes|\
.harness/baseline/*|.harness/audit/*|.harness/prompts/*)
            ;;
        *)
            NEEDS_APPROVAL=true
            CODE_FILES+="  $file"$'\n'
            ;;
    esac
done <<< "$STAGED"

if [[ "$NEEDS_APPROVAL" != "true" ]]; then
    exit 0
fi

# Check for valid verdict (within last hour) + score threshold
HARNESS_DIR=".harness"

# v3.3.3 §2.4: Tier-aware threshold (Hook = pure consumer, D4α 채택)
#   Hot tier (cold-tier 4 Signal 미충족):
#     max 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15)
#     threshold 90 (raw 100 × 90%, v3.3.3 §2.2)
#     VLM SKIP +8 environmental cost retained (Rule 7)
#   Cold tier (4 Signal 충족 — sim-core/sim-data/sim-test/sim-bench schema work):
#     max 100 (Visual 20 auto credit by generate_report.sh, Tests 20 포함)
#     threshold 75 (raw 100 × 75%, v3.3.3 §2.2)
#     VLM SKIP +0 (cold tier auto credit이 보정 대체)
# Hook은 verdict line 4 또는 pipeline_report.md에서 score를 pure consumer로 추출.
# Visual auto credit은 generate_report.sh 책임 (V.4.5에서 처리, hook에서 산출 X).
TIER="hot"
if [[ -x "tools/harness/cold_tier_classifier.sh" ]]; then
    if echo "$STAGED" | bash tools/harness/cold_tier_classifier.sh - >/dev/null 2>&1; then
        TIER="cold"
    fi
fi

if [[ "$TIER" == "cold" ]]; then
    SCORE_THRESHOLD=75   # v3.3.3 §2.4 정정 (54 → 75, raw 100 × 75%)
    APPLY_VLM_COST=0
else
    SCORE_THRESHOLD=90   # v3.3.3 §2.4 정정 (72 → 90, raw 100 × 90%, dimension 100 일관)
    APPLY_VLM_COST=1
fi
APPROVED_FEATURE=""

if [[ -d "$HARNESS_DIR/reviews" ]]; then
    while IFS= read -r -d '' verdict_file; do
        first_line=$(head -1 "$verdict_file" 2>/dev/null || echo "")
        if [[ "$first_line" == "APPROVED" ]]; then
            file_epoch=$(sed -n '3p' "$verdict_file" 2>/dev/null || echo "0")
            if [[ "$file_epoch" =~ ^[0-9]+$ ]]; then
                now_epoch=$(date +%s)
                age=$(( now_epoch - file_epoch ))
                if [[ $age -lt 3600 ]]; then
                    APPROVED_FEATURE=$(sed -n '2p' "$verdict_file" 2>/dev/null || echo "")
                    # Score gate: REQUIRED — no score evidence = BLOCK (no silent pass)
                    score=""
                    score_source=""
                    if [[ -n "$APPROVED_FEATURE" ]]; then
                        # 1st: verdict line 4 (pipeline writes score there)
                        verdict_score=$(sed -n '4p' "$verdict_file" 2>/dev/null || echo "")
                        if [[ "$verdict_score" =~ ^[0-9]+$ ]]; then
                            score="$verdict_score"
                            score_source="verdict:line4"
                        fi
                        # 2nd: legacy pipeline_report.md
                        if [[ -z "$score" ]]; then
                            report_file="$HARNESS_DIR/reports/$APPROVED_FEATURE/pipeline_report.md"
                            if [[ -f "$report_file" ]]; then
                                score=$(grep -oE '\*\*[0-9]+\*\*' "$report_file" | head -1 | tr -d '*' || echo "")
                                [[ -n "$score" ]] && score_source="pipeline_report.md"
                            fi
                        fi
                        # No score found — BLOCK (no silent pass)
                        if [[ -z "$score" ]]; then
                            echo "BLOCKED: No score evidence for feature '$APPROVED_FEATURE'." >&2
                            echo "Verdict has no score (line 4 empty) and pipeline_report.md missing." >&2
                            echo "Run full pipeline: bash tools/harness/harness_pipeline.sh $APPROVED_FEATURE <prompt>" >&2
                            exit 2
                        fi
                        # Adjusted score: add back VLM environmental costs per CLAUDE.md Rule 7.
                        # "VLM WARNING alone never blocks merge. This is policy, not bug."
                        # v3.3 §5.4: Cold tier excludes visual dimension entirely → no VLM cost.
                        # Hot tier retains v3.2.1 behaviour (VLM SKIP/WARNING = +8).
                        adjusted_score="$score"
                        vlm_env_cost=0
                        if [[ "$APPLY_VLM_COST" == "1" ]]; then
                            vlm_analysis_file="$HARNESS_DIR/evidence/$APPROVED_FEATURE/visual_analysis.txt"
                            if [[ ! -f "$vlm_analysis_file" ]]; then
                                vlm_env_cost=8
                            elif grep -qE "^VISUAL_WARNING" "$vlm_analysis_file" 2>/dev/null; then
                                vlm_env_cost=8
                            fi
                        fi
                        if [[ "$vlm_env_cost" -gt 0 ]]; then
                            adjusted_score=$((score + vlm_env_cost))
                        fi
                        if [[ "$adjusted_score" -lt "$SCORE_THRESHOLD" ]] 2>/dev/null; then
                            echo "BLOCKED: Pipeline score ${score} (adjusted ${adjusted_score}) below ${TIER}-tier threshold ${SCORE_THRESHOLD}." >&2
                            echo "Feature: $APPROVED_FEATURE (source: $score_source)" >&2
                            if [[ "$vlm_env_cost" -gt 0 ]]; then
                                echo "  Adjustment +${vlm_env_cost} (VLM env cost per Rule 7) applied but score still below threshold." >&2
                            fi
                            exit 2
                        fi
                        if [[ "$vlm_env_cost" -gt 0 ]]; then
                            echo "[hook] ${TIER}-tier: score $score → adjusted $adjusted_score (+${vlm_env_cost} VLM env cost) ≥ $SCORE_THRESHOLD ✓" >&2
                        else
                            echo "[hook] ${TIER}-tier: score $score ≥ $SCORE_THRESHOLD ✓" >&2
                        fi
                    fi
                    exit 0
                fi
            fi
        fi
    done < <(find "$HARNESS_DIR/reviews" -name "verdict" -print0 2>/dev/null)
fi

# No valid approval — BLOCK
echo "BLOCKED: Harness pipeline approval required for git commit/push." >&2
echo "Staged code/asset files:" >&2
echo "$CODE_FILES" >&2
echo "Run: bash tools/harness/harness_pipeline.sh <feature> <prompt.md>" >&2
exit 2

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

# HARNESS_SKIP=1 is FORBIDDEN per CLAUDE.md Rule 9
if [[ "${HARNESS_SKIP:-}" == "1" ]]; then
    echo "BLOCKED: HARNESS_SKIP=1 is FORBIDDEN per CLAUDE.md Rule 9." >&2
    echo "Diagnose the score gap and fix root cause. Do not bypass." >&2
    exit 2
fi

# Only care about git commit and git push
if ! echo "$CMD" | grep -qE '^\s*git\s+(commit|push)'; then
    exit 0
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
        *.md|*.txt|localization/*.json|tools/harness/*|.claude/*|hooks/*|.gitignore|.editorconfig|.gitattributes)
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
# Score threshold: 90 (was 95, lowered 2026-04-24).
# Rationale: VLM login environment issues cause a systematic -8 Visual
# WARNING in every pipeline regardless of code quality. APPROVE verdict +
# CLEAN regression guard + passing tests + clippy clean remain required;
# this change only tolerates the environmental Visual WARNING cost.
# Planned: restore to 95 once vlm-login-env-fix lands.
SCORE_THRESHOLD=90
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
                        if [[ "$score" -lt "$SCORE_THRESHOLD" ]] 2>/dev/null; then
                            echo "BLOCKED: Pipeline score ${score}/100 below ${SCORE_THRESHOLD} threshold." >&2
                            echo "Feature: $APPROVED_FEATURE (source: $score_source)" >&2
                            exit 2
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

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
SCORE_THRESHOLD=95
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
                    # Score gate: check pipeline report score
                    if [[ -n "$APPROVED_FEATURE" ]]; then
                        report_file="$HARNESS_DIR/reports/$APPROVED_FEATURE/pipeline_report.md"
                        if [[ -f "$report_file" ]]; then
                            score=$(grep -oE '\*\*[0-9]+\*\*' "$report_file" | head -1 | tr -d '*' || echo "0")
                            if [[ -n "$score" && "$score" -lt "$SCORE_THRESHOLD" ]] 2>/dev/null; then
                                echo "BLOCKED: Pipeline score ${score}/100 below ${SCORE_THRESHOLD} threshold (feature: $APPROVED_FEATURE)." >&2
                                exit 2
                            fi
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

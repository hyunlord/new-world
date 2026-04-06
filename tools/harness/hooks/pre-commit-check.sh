#!/usr/bin/env bash
# Layer 1: Claude Code PreToolUse hook
# Blocks git commit/push if code files are staged without harness approval.
# Exit 0 = allow, Exit 2 = block (exit 1 = non-blocking warning in Claude Code)
set -uo pipefail

# Ensure we're in the project root
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

# Get staged files from git
STAGED=$(git diff --cached --name-only 2>/dev/null || echo "")

if [[ -z "$STAGED" ]]; then
    exit 0
fi

# Check if any staged file requires approval
NEEDS_APPROVAL=false
CODE_FILES=""
while IFS= read -r file; do
    case "$file" in
        *.md|*.txt|localization/*.json|tools/harness/*|.claude/*|hooks/*|.gitignore|.editorconfig|.gitattributes)
            ;;  # exempt
        *)
            NEEDS_APPROVAL=true
            CODE_FILES+="  $file"$'\n'
            ;;
    esac
done <<< "$STAGED"

if [[ "$NEEDS_APPROVAL" != "true" ]]; then
    exit 0
fi

# Check for valid verdict
HARNESS_DIR=".harness"
if [[ -d "$HARNESS_DIR/reviews" ]]; then
    while IFS= read -r -d '' verdict_file; do
        first_line=$(head -1 "$verdict_file" 2>/dev/null || echo "")
        if [[ "$first_line" == "APPROVED" ]]; then
            file_epoch=$(sed -n '3p' "$verdict_file" 2>/dev/null || echo "0")
            if [[ "$file_epoch" =~ ^[0-9]+$ ]]; then
                now_epoch=$(date +%s)
                age=$(( now_epoch - file_epoch ))
                if [[ $age -lt 3600 ]]; then
                    exit 0  # Valid approval found
                fi
            fi
        fi
    done < <(find "$HARNESS_DIR/reviews" -name "verdict" -print0 2>/dev/null)
fi

# No valid approval — BLOCK
echo ""
echo "BLOCKED: Harness pipeline approval required."
echo "Staged code/asset files:"
echo "$CODE_FILES"
echo "Run: bash tools/harness/harness_pipeline.sh <feature> <prompt.md>"
echo ""
exit 2

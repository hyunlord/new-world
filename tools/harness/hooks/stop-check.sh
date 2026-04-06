#!/usr/bin/env bash
# Layer 2: Claude Code Stop hook
# If code/asset files are modified but no pipeline was run, prevent stopping.
# Exit 0 = allow stop, Exit 2 = force continue
set -uo pipefail

# Check for modified (unstaged + staged) code/asset files
MODIFIED=$(git diff --name-only HEAD 2>/dev/null; git diff --cached --name-only 2>/dev/null)
MODIFIED=$(echo "$MODIFIED" | sort -u)

if [[ -z "$MODIFIED" ]]; then
    exit 0  # Nothing changed — can stop
fi

# Check if any modified file requires pipeline
NEEDS_PIPELINE=false
while IFS= read -r file; do
    [[ -z "$file" ]] && continue
    case "$file" in
        *.md|*.txt|localization/*.json|tools/harness/*|.claude/*|hooks/*|.gitignore|.editorconfig|.gitattributes)
            ;;  # exempt
        *)
            NEEDS_PIPELINE=true
            break
            ;;
    esac
done <<< "$MODIFIED"

if [[ "$NEEDS_PIPELINE" != "true" ]]; then
    exit 0
fi

# Code files modified — check for valid verdict
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
                    exit 0  # Valid approval — can stop
                fi
            fi
        fi
    done < <(find "$HARNESS_DIR/reviews" -name "verdict" -print0 2>/dev/null)
fi

# Code changed but no approval — BLOCK STOP
# Read from stdin to check stop_hook_active flag (prevent infinite loop)
INPUT=$(cat)
STOP_HOOK_ACTIVE=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('stop_hook_active', False))" 2>/dev/null || echo "False")

if [[ "$STOP_HOOK_ACTIVE" == "True" || "$STOP_HOOK_ACTIVE" == "true" ]]; then
    # Already retried — let it stop to avoid infinite loop
    echo "WARNING: Code/asset files modified without harness approval. Allowing stop to prevent loop."
    exit 0
fi

echo ""
echo "CANNOT STOP: Code/asset files modified without harness pipeline approval."
echo "Run the harness pipeline before finishing:"
echo "  bash tools/harness/harness_pipeline.sh <feature> <prompt.md>"
echo ""
exit 2

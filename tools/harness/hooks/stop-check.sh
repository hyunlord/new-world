#!/usr/bin/env bash
# Layer 2: Claude Code Stop hook
# If code/asset files are modified but no pipeline was run, prevent stopping.
# Exit 0 = allow stop, Exit 2 = force continue
set -uo pipefail

if [[ "${HARNESS_SKIP:-}" == "1" ]]; then
    exit 0
fi

# Ensure we're in the project root
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

# Check for modified (unstaged + staged) code/asset files
MODIFIED=$(git diff --name-only HEAD 2>/dev/null; git diff --cached --name-only 2>/dev/null)
MODIFIED=$(echo "$MODIFIED" | sort -u)

if [[ -z "$MODIFIED" ]]; then
    exit 0  # Nothing changed — can stop
fi

# Classify by required tier: none < light < quick < full
TIER="none"
while IFS= read -r file; do
    [[ -z "$file" ]] && continue
    case "$file" in
        rust/crates/sim-core/*.rs|rust/crates/sim-core/**/*.rs|\
rust/crates/sim-systems/*.rs|rust/crates/sim-systems/**/*.rs|\
rust/crates/sim-engine/*.rs|rust/crates/sim-engine/**/*.rs)
            TIER="full"
            ;;
        *.gd|*.gdshader)
            [[ "$TIER" != "full" ]] && TIER="quick"
            ;;
        rust/crates/sim-test/*.rs|rust/crates/sim-test/**/*.rs|\
rust/crates/sim-data/*.rs|rust/crates/sim-data/**/*.rs|\
rust/crates/sim-bridge/*.rs|rust/crates/sim-bridge/**/*.rs)
            [[ "$TIER" != "full" ]] && TIER="quick"
            ;;
        *.png|*.svg|*.wav|*.tscn|*.tres|*.ron)
            [[ "$TIER" == "none" ]] && TIER="light"
            ;;
        *.md|*.txt|localization/*.json|tools/*|tools/*/*|.claude/*|.claude/*/*|\
hooks/*|.gitignore|.editorconfig|.gitattributes|*.toml|*.cfg|*.py)
            ;;
        *)
            [[ "$TIER" == "none" || "$TIER" == "light" ]] && TIER="quick"
            ;;
    esac
done <<< "$MODIFIED"

# none and light tiers don't block stop
# (light auto-approves via VLM verdict in --light mode)
if [[ "$TIER" == "none" || "$TIER" == "light" ]]; then
    exit 0
fi

# quick or full — check for valid verdict
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
INPUT=$(cat 2>/dev/null || echo "{}")
STOP_HOOK_ACTIVE=$(echo "$INPUT" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    print(str(d.get('stop_hook_active', False)).lower())
except:
    print('false')
" 2>/dev/null || echo "false")

if [[ "$STOP_HOOK_ACTIVE" == "true" ]]; then
    echo "WARNING: Code files modified without harness approval (tier: $TIER). Allowing stop to prevent loop."
    exit 0
fi

echo ""
echo "CANNOT STOP: Code files modified without harness pipeline approval (tier: $TIER)."
echo "Run the harness pipeline before finishing:"
echo "  bash tools/harness/harness_pipeline.sh <feature> <prompt.md> --${TIER}"
echo ""
echo "Or bypass: HARNESS_SKIP=1"
echo ""
exit 2

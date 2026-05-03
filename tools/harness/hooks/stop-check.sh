#!/usr/bin/env bash
# Layer 2: Claude Code Stop hook
# If code/asset files are modified but no pipeline was run, prevent stopping.
# Exit 0 = allow stop, Exit 2 = force continue
set -uo pipefail

# ENV-BYPASS (CLAUDE.md Rule 7.1 v3.2.1): one-shot authorized bypass for env blocks.
# Requires .harness/audit/env_bypass_active marker (≤2 h old) AND
# manual_verification.log + clippy_full.log under .harness/evidence/<feature>/.
# clippy_full.log is the per-crate log produced by authorize_env_bypass.sh and is
# diff-checked against .harness/baseline/clippy_baseline_raw.txt fingerprints.
PROJECT_ROOT_FOR_BYPASS="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"
BYPASS_MARKER="$PROJECT_ROOT_FOR_BYPASS/.harness/audit/env_bypass_active"
if [[ -f "$BYPASS_MARKER" ]]; then
    bypass_mtime=$(stat -f %m "$BYPASS_MARKER" 2>/dev/null || stat -c %Y "$BYPASS_MARKER" 2>/dev/null || echo 0)
    bypass_age=$(( $(date +%s) - bypass_mtime ))
    if [[ $bypass_age -ge 7200 ]]; then
        printf '[stop-check] ENV-BYPASS expired (>2h, age=%ss). Re-authorize to continue.\n' "$bypass_age" >&2
        rm -f "$BYPASS_MARKER"
        exit 2
    fi
    bypass_feature=$(grep '^feature:' "$BYPASS_MARKER" 2>/dev/null | head -1 | awk '{print $2}')
    if [[ -z "$bypass_feature" ]]; then
        printf '[stop-check] ENV-BYPASS marker malformed (missing feature:). Aborting.\n' >&2
        exit 2
    fi
    evidence_dir="$PROJECT_ROOT_FOR_BYPASS/.harness/evidence/$bypass_feature"
    if [[ ! -s "$evidence_dir/manual_verification.log" ]]; then
        printf '[stop-check] ENV-BYPASS missing %s/manual_verification.log\n' "$evidence_dir" >&2
        exit 2
    fi
    if [[ ! -s "$evidence_dir/clippy_full.log" ]]; then
        printf '[stop-check] ENV-BYPASS missing %s/clippy_full.log\n' "$evidence_dir" >&2
        exit 2
    fi
    printf '[stop-check] ENV-BYPASS active for %s (age=%ss). Allowing stop.\n' "$bypass_feature" "$bypass_age" >&2
    rm -f "$BYPASS_MARKER"  # one-shot
    exit 0
fi

if [[ "${HARNESS_SKIP:-}" == "1" ]]; then
    # Record SKIP event to track budget usage
    PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"
    state_dir="$PROJECT_ROOT/.harness/state"
    mkdir -p "$state_dir"
    skip_log="$state_dir/skip_history.log"
    feature_guess=$(find "$PROJECT_ROOT/.harness/evidence" -maxdepth 1 -type d 2>/dev/null \
        | sort -t/ -k1 | tail -1 | xargs basename 2>/dev/null || echo "unknown")
    printf '%s|%s|%s\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        "${feature_guess:-unknown}" \
        "${HARNESS_SKIP_REASON:-user-skip}" \
        >> "$skip_log"

    # Warn if the last 3 commits all used SKIP
    recent_count=$(tail -n 3 "$skip_log" 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$recent_count" -ge 3 ]]; then
        printf '\n' >&2
        printf '⚠️  HARNESS SKIP BUDGET WARNING ⚠️\n' >&2
        printf 'Last 3 commits all used HARNESS_SKIP=1.\n' >&2
        printf 'Recent skip history:\n' >&2
        tail -n 3 "$skip_log" | sed 's/^/  /' >&2
        printf '\nNext feature MUST pass harness without SKIP to restore confidence.\n' >&2
        printf 'See tools/harness/HARNESS_INFRA_TODO.md for context.\n\n' >&2
    fi
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

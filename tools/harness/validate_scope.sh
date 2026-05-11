#!/bin/bash
# validate_scope.sh — V7 Hook Governance v3.3.15 §4-γ
# Pre-Integrator working-tree scope validator (Generator scope enforcement, 4th layer).
#
# Existing 3 layers (already in place):
#   1. Contract  (harness-generator.md L61 "Stay in scope")     — soft instruction
#   2. Detection (classify_recode.sh OUT_OF_SCOPE regex)        — post-fact classifier
#   3. Penalty   (score_attempt_penalty.sh OUT_OF_SCOPE -5)     — post-fact deduction
# This script (NEW, v3.3.15):
#   4. Pre-gate  — surfaces working-tree files that aren't declared in the prompt
#                  BEFORE Integrator commits them. Warn-only first iteration; block
#                  reserved for v3.3.16+ if false-positive rate is acceptable.
#
# Usage: validate_scope.sh <feature>
#
# Exit codes:
#   0  always (warn mode — does not block commits)
#   2  reserved (block mode, deferred to v3.3.16+)
#
# Stdout: warning lines (one per out-of-scope file) prefixed with [scope:WARN].
# Stderr: diagnostic info.
#
# Mechanism:
#   1. Read .harness/prompts/<feature>.md
#   2. Extract candidate file paths via regex (relative paths with known extensions)
#   3. Read `git diff --cached --name-only` (staged files only)
#   4. For each staged code file: warn if not appearing in prompt as a declared path
#   5. Exempt paths under .harness/, tools/harness/, .claude/, docs (*.md / *.txt)
#      because pre-commit-check.sh already excludes them from the approval gate.
#
# Reference: governance discussion 2026-05-11 Issues 1+2+4, Option 4-γ.

set -uo pipefail

FEATURE="${1:?usage: $0 <feature>}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROMPT_FILE="$PROJECT_ROOT/.harness/prompts/${FEATURE}.md"

if [[ ! -f "$PROMPT_FILE" ]]; then
    echo "[scope] prompt not found: $PROMPT_FILE — skipping scope validation" >&2
    exit 0
fi

# Staged files (relative to repo root)
STAGED=$(git diff --cached --name-only 2>/dev/null || true)
if [[ -z "$STAGED" ]]; then
    exit 0
fi

# Extract declared file paths from prompt. Best-effort regex:
#   - matches relative paths containing a directory separator and a known extension
#   - tolerates surrounding markdown formatting (backticks, parens, asterisks)
DECLARED=$(grep -oE '[A-Za-z0-9_./-]+\.(rs|gd|gdshader|ron|json|ftl|toml|tres|tscn|wav|png|md)' "$PROMPT_FILE" \
    | sort -u || true)

# Build awk-friendly declared set
declared_match() {
    local target="$1"
    [[ -z "$DECLARED" ]] && return 1
    # Exact match OR target is suffix of a declared path OR declared is suffix of target
    while IFS= read -r d; do
        [[ -z "$d" ]] && continue
        if [[ "$target" == "$d" ]] || [[ "$target" == */"$d" ]] || [[ "$d" == */"$target" ]]; then
            return 0
        fi
    done <<< "$DECLARED"
    return 1
}

WARN_COUNT=0
while IFS= read -r file; do
    [[ -z "$file" ]] && continue
    # Skip paths excluded from approval gate (harness infra, audit, prompts, etc.)
    case "$file" in
        .harness/*|tools/harness/*|.claude/*|hooks/*|*.md|*.txt|\
.gitignore|.editorconfig|.gitattributes|\
localization/*) continue ;;
    esac

    if ! declared_match "$file"; then
        echo "[scope:WARN] $file — not declared in $PROMPT_FILE"
        WARN_COUNT=$((WARN_COUNT + 1))
    fi
done <<< "$STAGED"

if [[ $WARN_COUNT -gt 0 ]]; then
    echo "[scope] $WARN_COUNT staged file(s) not declared in prompt scope (warn-only; commit proceeds)." >&2
    echo "[scope] Declared paths in prompt: $(echo "$DECLARED" | wc -l | tr -d ' ') unique." >&2
fi

exit 0

#!/usr/bin/env bash
# Pre-flight check for harness pipeline.
# Verifies external dependencies before a full pipeline run.
# Usage: bash tools/harness/hooks/pre-flight-check.sh [--full|--quick|--light]
# Exit codes: 0 = pass, 1 = hard failure, 2 = soft warning (non-fatal)

set -uo pipefail

MODE="${1:-unknown}"

# Check 1: claude CLI available
if ! command -v claude >/dev/null 2>&1; then
    echo "❌ claude CLI not found — install Claude Code and retry"
    exit 1
fi
echo "✅ claude CLI found"

# Check 2: Codex auth (only matters for --full mode which uses Codex Evaluator)
if [[ "$MODE" == "--full" || "$MODE" == "unknown" ]]; then
    if command -v codex >/dev/null 2>&1; then
        if timeout 10 codex exec --sandbox=danger-full-network -s read-only echo "ok" > /dev/null 2>&1; then
            echo "✅ Codex auth valid"
        else
            echo "⚠️  Codex auth may be expired or network unreachable"
            echo "   If expired: codex auth login"
            echo "   Continuing — Evaluator will fall back to claude harness-evaluator if Codex is unreachable"
            # Non-fatal: the pipeline has a fallback path for Codex failures
        fi
    else
        echo "⚠️  codex CLI not found — Evaluator will fall back to claude harness-evaluator"
    fi
fi

# Check 3: Disk space (warn if < 500 MB free for evidence storage)
_evidence_root="${HARNESS_DIR:-.harness}"
_free_kb=$(df -Pk "$_evidence_root" 2>/dev/null | tail -1 | awk '{print $4}')
if [[ -n "$_free_kb" && "$_free_kb" =~ ^[0-9]+$ && "$_free_kb" -lt 512000 ]]; then
    echo "⚠️  Less than 500 MB free on disk — evidence collection may fail"
fi

echo "Pre-flight check passed"
exit 0

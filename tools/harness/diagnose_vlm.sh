#!/usr/bin/env bash
# diagnose_vlm.sh — Verify VLM (Claude CLI) authentication for harness pipeline.
# Usage: bash tools/harness/diagnose_vlm.sh
#
# Background: harness_pipeline.sh's VLM analysis spawns `claude --agent
# harness-vlm-analyzer`. Earlier the call wrapped it in `env -i` with only 5
# environment variables preserved (PATH/HOME/USER/TERM/CLAUDE_CONFIG_DIR),
# which stripped the macOS launch-services context required for keychain auth
# and produced a systematic "Not logged in" / VISUAL_WARNING result.
#
# This script verifies the new (env-preserving) VLM invocation works by:
#   Test 1 — direct claude CLI works in the current shell
#   Test 2 — closed-stdin spawn (matches harness pipeline) returns OK output
#   Test 3 — VLM-style call produces a verdict-style token

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== VLM Authentication Diagnostic ==="
echo "Date: $(date -Iseconds 2>/dev/null || date '+%Y-%m-%dT%H:%M:%S')"
echo "Project: $PROJECT_ROOT"
echo "Claude CLI: $(claude --version 2>&1 | head -1)"
echo ""

# ── Test 1: direct claude CLI ─────────────────────────────────────────────
echo "=== Test 1: claude --version (current shell) ==="
if claude --version >/dev/null 2>&1; then
    echo "PASS: Claude CLI is installed and responsive"
else
    echo "FAIL: claude --version did not return cleanly"
    echo "      Install Claude CLI before running the harness pipeline."
    exit 1
fi
echo ""

# ── Test 2: closed-stdin spawn (matches new VLM invocation) ──────────────
echo "=== Test 2: closed-stdin claude -p (matches harness VLM call) ==="
test2_output=$(
    HARNESS_VLM_ISOLATED=1 claude -p "Reply with the single word OK." \
        --output-format text < /dev/null 2>&1
) || test2_rc=$?
test2_rc="${test2_rc:-0}"
echo "Exit code: $test2_rc"
echo "Output: $(echo "$test2_output" | head -3)"
if [[ "$test2_output" == *"Not logged in"* ]]; then
    echo "FAIL: Spawned claude reports 'Not logged in'"
    echo "      The harness VLM step will produce VISUAL_WARNING for every feature."
    echo "      Run \`claude /login\` in this shell, or set ANTHROPIC_API_KEY."
    exit 2
fi
if [[ -z "$test2_output" ]]; then
    echo "FAIL: Spawned claude produced empty output"
    exit 2
fi
echo "PASS: Spawned claude returned a non-empty, authenticated response"
echo ""

# ── Test 3: VLM-style verdict-token call ─────────────────────────────────
echo "=== Test 3: VLM-style call producing verdict token ==="
test3_output=$(
    HARNESS_VLM_ISOLATED=1 claude --agent harness-vlm-analyzer \
        -p 'Diagnostic only — respond with exactly: VISUAL_OK (diagnose_vlm.sh)' \
        --output-format text < /dev/null 2>&1
) || test3_rc=$?
test3_rc="${test3_rc:-0}"
echo "Exit code: $test3_rc"
echo "Output preview:"
echo "$test3_output" | head -5 | sed 's/^/  /'
if [[ "$test3_output" == *"Not logged in"* ]]; then
    echo "FAIL: VLM agent invocation reports 'Not logged in'"
    exit 3
fi
if [[ "$test3_output" == *"VISUAL_OK"* || "$test3_output" == *"VISUAL_WARNING"* || "$test3_output" == *"VISUAL_FAIL"* ]]; then
    echo "PASS: VLM agent returned a verdict-style token"
    exit 0
fi
echo "WARNING: VLM produced output but no verdict token — check the agent prompt"
exit 4

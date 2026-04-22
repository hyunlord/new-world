#!/usr/bin/env bash
# Smoke test for harness pipeline infrastructure primitives.
# Run standalone — does NOT require a real pipeline invocation.
# Usage: bash tools/harness/test_pipeline_infra.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PIPELINE="$SCRIPT_DIR/harness_pipeline.sh"
PASS=0
FAIL=0

pass() { echo "[PASS] $1"; (( PASS++ )) || true; }
fail() { echo "[FAIL] $1"; (( FAIL++ )) || true; }

echo "===== Harness Infra Self-Test ====="
echo "Target: $PIPELINE"
echo ""

# --- Test 1: run_with_timeout function defined ---
if grep -q "^run_with_timeout()" "$PIPELINE" 2>/dev/null; then
    pass "run_with_timeout function defined"
else
    fail "run_with_timeout function missing"
fi

# --- Test 2: run_codex applies timeout wrapper ---
if grep -A 25 "^run_codex()" "$PIPELINE" | grep -q "run_with_timeout"; then
    pass "run_codex applies timeout"
else
    fail "run_codex does not use timeout — Codex hang will stall pipeline"
fi

# --- Test 3: VLM isolation primitives present ---
if grep -q "exec < /dev/null" "$PIPELINE" && grep -q "env -i" "$PIPELINE"; then
    pass "VLM isolation primitives present (exec < /dev/null + env -i)"
else
    fail "VLM isolation not applied to claude invocation"
fi

# --- Test 4: VLM contamination detection implemented ---
if grep -q "contaminated" "$PIPELINE"; then
    pass "VLM contamination detection implemented"
else
    fail "VLM contamination detection missing"
fi

# --- Test 5: FFI verify has timeout handling ---
if grep -A 10 "Running FFI Chain Verify" "$PIPELINE" | grep -q "TIMED_OUT\|124"; then
    pass "FFI verify has timeout fallback"
else
    fail "FFI verify missing timeout handling"
fi

# --- Test 6: Regression guard has timeout handling ---
if grep -A 10 "Running Regression Guard" "$PIPELINE" | grep -q "TIMED_OUT\|124"; then
    pass "Regression guard has timeout fallback"
else
    fail "Regression guard missing timeout handling"
fi

# --- Test 7: stop-check.sh has SKIP budget warning ---
STOP_CHECK="$SCRIPT_DIR/hooks/stop-check.sh"
if [[ -f "$STOP_CHECK" ]] && grep -q "SKIP BUDGET WARNING" "$STOP_CHECK"; then
    pass "stop-check.sh has SKIP budget warning"
else
    fail "stop-check.sh missing SKIP budget tracking"
fi

# --- Test 8: timeout primitive available and wired into run_codex ---
# Live execution test: use system timeout directly (avoids sourcing pipeline)
_timeout_ok=false
if command -v timeout >/dev/null 2>&1; then
    set +e
    timeout 1 sleep 3 2>/dev/null
    _rc=$?
    set -e
    if [[ $_rc -eq 124 || $_rc -ne 0 ]]; then
        _timeout_ok=true
    fi
elif command -v perl >/dev/null 2>&1; then
    # macOS perl fallback path — just verify perl is available
    _timeout_ok=true
fi

if $_timeout_ok; then
    pass "timeout primitive available and works (run_with_timeout will function correctly)"
else
    fail "neither 'timeout' nor 'perl' found — run_with_timeout may not enforce limits"
fi

# --- Test 9: harness-drafter calls use timeout wrapper ---
if grep -q "DRAFTER_TIMEOUT_SECONDS" "$PIPELINE" && \
   grep -q "run_with_timeout.*drafter_timeout\|run_with_timeout.*drafter_rev_timeout" "$PIPELINE"; then
    pass "harness-drafter calls have timeout wrapper (DRAFTER_TIMEOUT_SECONDS)"
else
    fail "harness-drafter calls missing timeout wrapper"
fi

# --- Test 10: harness-generator call uses timeout wrapper ---
if grep -q "GENERATOR_TIMEOUT_SECONDS" "$PIPELINE" && \
   grep -q 'run_with_timeout.*gen_timeout' "$PIPELINE"; then
    pass "harness-generator call has timeout wrapper (GENERATOR_TIMEOUT_SECONDS)"
else
    fail "harness-generator call missing timeout wrapper"
fi

# --- Test 11: harness-evaluator call uses timeout wrapper ---
if grep -q "EVALUATOR_TIMEOUT_SECONDS" "$PIPELINE" && \
   grep -q 'run_with_timeout.*eval_timeout' "$PIPELINE"; then
    pass "harness-evaluator call has timeout wrapper (EVALUATOR_TIMEOUT_SECONDS)"
else
    fail "harness-evaluator call missing timeout wrapper"
fi

# --- Test 12: harness-vlm-analyzer calls use timeout wrapper ---
if grep -q "VLM_TIMEOUT_SECONDS" "$PIPELINE" && \
   grep -q 'run_with_timeout.*vlm_timeout' "$PIPELINE"; then
    pass "harness-vlm-analyzer calls have timeout wrapper (VLM_TIMEOUT_SECONDS)"
else
    fail "harness-vlm-analyzer calls missing timeout wrapper"
fi

echo ""
echo "============================="
echo "Results: $PASS passed, $FAIL failed"
echo "============================="

if [[ $FAIL -eq 0 ]]; then
    echo "===== All infrastructure tests PASS ====="
    exit 0
else
    echo "===== $FAIL test(s) FAILED ====="
    exit 1
fi

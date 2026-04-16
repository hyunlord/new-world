#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# WorldSim Harness Pipeline v3.2 — Codex 3-Role Integration (All Code Changes)
# ============================================================
#
# Usage:
#   bash tools/harness/harness_pipeline.sh <feature_name> <feature_prompt_file> [--quick]
#
# Scope: ALL code, shader, asset, data, and scene changes go through this pipeline.
# Exempt: docs (.md/.txt), localization JSON, harness infra itself.
# The pre-commit hook enforces this — code commits without APPROVED verdict are blocked.
#
# Arguments:
#   feature_name:        Short identifier (e.g., "temperament_cognition")
#   feature_prompt_file: Path to the Claude Code prompt .md file
#   --quick:             Skip Challenger step (for Type A invariants only)
#
# Pipeline:
#   Step 1a:  PLANNER       (Claude Code agent)  → plan_draft.md
#   Step 1b:  CHALLENGER    (Claude Code -p)      → challenge_report.md  [skipped with --quick]
#   Step 1c:  PLANNER       (Claude Code agent)  → plan_final.md         [skipped with --quick]
#   Step 2:   GENERATOR     (Claude Code -p)      → code + gen_result.md
#   Step 2.5a: VISUAL VERIFY (Godot local)        → screenshots + data    [non-blocking]
#   Step 2.5b: VLM ANALYSIS  (Claude -p)          → visual_analysis.txt   [non-blocking]
#   Step 2.5c: FFI VERIFY    (Codex)              → ffi_chain_verify.txt  [if sim-bridge changed]
#   Step 2.7:  REGR. GUARD   (Codex)              → regression_guard.txt  [always]
#   Step 3:   EVALUATOR     (Codex)               → review.md + verdict   [independent from Generator]
#   Step 4:   INTEGRATOR    (script logic)        → commit / retry / stop
#
# Each step runs as a separate `claude -p` session, providing natural
# context isolation — no session can see another's reasoning.
#
# Retry logic:
#   RE-CODE:  → Step 2 (max 3 code attempts)
#   RE-PLAN:  → Step 1a (max 2 plan attempts)
#   FAIL:     → exit 1 + report
#   APPROVE:  → commit allowed

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
HARNESS_DIR="$PROJECT_ROOT/.harness"
TEMPLATES_DIR="$SCRIPT_DIR/templates"

# --- Args ---
# --- Ensure pre-commit hook is installed ---
_git_dir=$(git rev-parse --git-common-dir 2>/dev/null || git rev-parse --git-dir)
if [[ ! -f "$_git_dir/hooks/pre-commit" ]] || ! grep -q "harness approval" "$_git_dir/hooks/pre-commit" 2>/dev/null; then
    log "Installing pre-commit hook..."
    cp "$PROJECT_ROOT/hooks/pre-commit-harness" "$_git_dir/hooks/pre-commit"
    chmod +x "$_git_dir/hooks/pre-commit"
    log "Pre-commit hook installed at $_git_dir/hooks/pre-commit"
fi

FEATURE="${1:?Usage: harness_pipeline.sh <feature_name> <prompt_file> [--full|--quick|--light]}"
PROMPT_FILE="${2:?Usage: harness_pipeline.sh <feature_name> <prompt_file> [--full|--quick|--light]}"
MODE="${3:---full}"  # "--full" (default), "--quick", or "--light"

# --- Directories ---
PLAN_DIR="$HARNESS_DIR/plans/$FEATURE"
RESULT_DIR="$HARNESS_DIR/results/$FEATURE"
REVIEW_DIR="$HARNESS_DIR/reviews/$FEATURE"
EVIDENCE_DIR="$HARNESS_DIR/evidence/$FEATURE"
PROGRESS_FILE="$HARNESS_DIR/progress/$FEATURE/progress.md"

mkdir -p "$PLAN_DIR" "$RESULT_DIR" "$REVIEW_DIR" "$EVIDENCE_DIR"

# --- Counters ---
PLAN_ATTEMPT=0
MAX_PLAN_ATTEMPTS=2
CODE_ATTEMPT=0
MAX_CODE_ATTEMPTS=3

# --- Codex config ---
CODEX_MODEL="${CODEX_MODEL:-}"  # empty = codex default; override with e.g. "o3" or "gpt-4.1"
CODEX_BIN="${CODEX_BIN:-codex}"

# --- Logging ---
log() { echo "[harness $(date +%H:%M:%S)] $*"; }
die() { log "FATAL: $*"; exit 1; }

# --- Progress reporting ---
init_progress() {
    mkdir -p "$(dirname "$PROGRESS_FILE")"
    cat > "$PROGRESS_FILE" << EOF
# Pipeline Progress: $FEATURE
> Mode: $MODE | Started: $(date +"%Y-%m-%d %H:%M:%S")
start_time: $(date +%s)

| Time | Step | Status | What was done |
|------|------|--------|---------------|
EOF
}

finalize_progress() {
    echo "end_time: $(date +%s)" >> "$PROGRESS_FILE"
}

report_step() {
    local step_name="$1"
    local status="$2"  # DONE, RUNNING, SKIPPED, FAILED
    local summary="${3:-}"
    mkdir -p "$(dirname "$PROGRESS_FILE")"
    local timestamp
    timestamp=$(date +"%H:%M:%S")
    echo "| $timestamp | $step_name | **$status** | $summary |" >> "$PROGRESS_FILE"
    echo "[harness progress] $step_name: $status — $summary"
}

summarize_drafter() {
    local plan_file="$1"
    local assertion_count
    assertion_count=$(grep -c "^### Assertion\|^- Assertion\|assertion_name" "$plan_file" 2>/dev/null || echo "0")
    local assertion_names
    assertion_names=$(grep -i "assertion_name\|^### Assertion" "$plan_file" 2>/dev/null | head -3 | sed 's/.*: //;s/^### //' | tr '\n' ', ' | sed 's/, $//')
    echo "Plan: ${assertion_count} assertions — ${assertion_names:-none listed}"
}

summarize_challenger() {
    local report_file="$1"
    local issues
    issues=$(grep -c "\[ISSUE\]\|^- \|^[0-9]\." "$report_file" 2>/dev/null || echo "0")
    local gaming
    gaming=$(grep -ci "gaming\|cheat\|hardcod\|bypass\|trivial" "$report_file" 2>/dev/null || echo "0")
    local top_issue
    top_issue=$(grep -i "\[ISSUE\]\|^1\.\|^- " "$report_file" 2>/dev/null | head -1 | cut -c1-60)
    echo "Found ${issues} issues (${gaming} gaming vectors). Top: ${top_issue:-none}"
}

summarize_qc() {
    local review_file="$1"
    local verdict
    verdict=$(sed 's/\*//g; s/_//g' "$review_file" | grep -i "^verdict:" | head -1 | awk '{print $2}' || echo "UNKNOWN")
    local reason
    reason=$(grep -i "reason\|rationale\|because" "$review_file" 2>/dev/null | head -1 | cut -c1-60)
    echo "Verdict: ${verdict}. ${reason:-}"
}

summarize_generator() {
    local result_file="$1"
    local files_changed
    files_changed=$(grep "^- \|created\|modified\|added" "$result_file" 2>/dev/null | head -5 | sed 's/^- //' | tr '\n' ', ' | sed 's/, $//')
    local lines_added
    lines_added=$(grep -i "lines\|+[0-9]" "$result_file" 2>/dev/null | head -1 | cut -c1-40)
    echo "Files: ${files_changed:-unknown}. ${lines_added:-}"
}

summarize_visual() {
    local evidence_dir="$1"
    local screenshots
    screenshots=$(find "$evidence_dir" -name "screenshot_*.png" 2>/dev/null | wc -l | tr -d ' ')
    local agents=""
    if [[ -f "$evidence_dir/entity_summary.txt" ]]; then
        agents=$(grep -i "alive\|total\|count" "$evidence_dir/entity_summary.txt" 2>/dev/null | head -1 | cut -c1-40)
    fi
    local fps=""
    if [[ -f "$evidence_dir/performance.txt" ]]; then
        fps=$(grep -i "fps\|tps\|avg" "$evidence_dir/performance.txt" 2>/dev/null | head -1 | cut -c1-30)
    fi
    echo "${screenshots} screenshots. ${agents:+Agents: $agents. }${fps:+$fps}"
}

summarize_vlm() {
    local analysis_file="$1"
    local verdict
    verdict=$(grep -i "VISUAL_OK\|VISUAL_WARNING\|VISUAL_FAIL" "$analysis_file" 2>/dev/null | tail -1 | grep -o "VISUAL_[A-Z]*" || echo "UNKNOWN")
    local detail
    detail=$(grep -vi "^verdict\|^#\|^---\|^$" "$analysis_file" 2>/dev/null | head -1 | cut -c1-60)
    echo "${verdict}. ${detail:-}"
}

summarize_evaluator() {
    local review_file="$1"
    local verdict
    verdict=$(sed 's/\*//g; s/_//g' "$review_file" | grep -i "^verdict:" | head -1 | awk '{print $2}' || echo "UNKNOWN")
    local reason
    reason=$(sed 's/\*//g' "$review_file" | grep -i "reason\|rationale\|summary\|conclusion" 2>/dev/null | head -1 | cut -c1-60)
    echo "${verdict}. ${reason:-}"
}

# --- Template rendering (multiline-safe) ---
# Usage: render_template template_file output_file KEY1=val1 KEY2=@filepath ...
# Values prefixed with @ are read from file. Uses python3 for safe replacement.
render_template() {
    local template="$1"
    local output="$2"
    shift 2

    cp "$template" "$output"

    for arg in "$@"; do
        local key="${arg%%=*}"
        local val="${arg#*=}"

        # If value starts with @, read from file
        if [[ "$val" == @* ]]; then
            local filepath="${val#@}"
            if [[ -f "$filepath" ]]; then
                val=$(cat "$filepath")
            else
                val="(file not found: $filepath)"
            fi
        fi

        # Write value to temp file to avoid shell arg length limits
        local tmpval
        tmpval=$(mktemp)
        printf '%s' "$val" > "$tmpval"

        python3 -c "
import sys
key = '{{' + sys.argv[1] + '}}'
with open(sys.argv[2]) as vf:
    val = vf.read()
with open(sys.argv[3]) as f:
    content = f.read()
with open(sys.argv[3], 'w') as f:
    f.write(content.replace(key, val))
" "$key" "$tmpval" "$output"

        rm -f "$tmpval"
    done
}

# --- Timeout wrapper (GNU timeout or macOS perl fallback) ---
run_with_timeout() {
    local seconds=$1
    shift
    if command -v timeout >/dev/null 2>&1; then
        timeout "$seconds" "$@"
    else
        # macOS fallback: perl alarm with proper process management
        perl -e '
            use POSIX ":sys_wait_h";
            alarm $ARGV[0];
            $SIG{ALRM} = sub { kill "TERM", $pid; exit 142; };
            $pid = fork();
            if ($pid == 0) { exec @ARGV[1..$#ARGV]; die "exec failed: $!"; }
            waitpid($pid, 0);
            exit ($? >> 8);
        ' "$seconds" "$@"
    fi
}

# --- Check prerequisites ---
[[ -f "$PROMPT_FILE" ]] || die "Prompt file not found: $PROMPT_FILE"
command -v claude >/dev/null 2>&1 || die "claude CLI not found"
command -v "$CODEX_BIN" >/dev/null 2>&1 || die "codex CLI not found (set CODEX_BIN to override path)"
command -v python3 >/dev/null 2>&1 || die "python3 not found (needed for template rendering)"

# ── CRITICAL: Prevent nested session detection ──
# Claude Code sets CLAUDECODE=1. When harness_pipeline.sh runs
# inside a Claude Code session and calls `claude --agent`, the child
# process inherits CLAUDECODE=1 and refuses to start.
# Setting CLAUDECODE="" disables the check.
# Ref: https://github.com/anthropics/claude-agent-sdk-python/issues/573
export CLAUDECODE=""
unset CLAUDE_CODE_ENTRYPOINT 2>/dev/null || true

# ============================================================
# STEP 0: MECHANICAL GATE (0-token — cargo test + clippy + FFI)
# ============================================================
run_mechanical_gate() {
    log "=== Step 0: MECHANICAL GATE (0-token) ==="

    cd "$PROJECT_ROOT/rust"

    # 0a. cargo test
    if ! cargo test --workspace --quiet 2>&1 | tail -20 | tee "$RESULT_DIR/step0_test.txt"; then
        cd "$PROJECT_ROOT"
        die "MECHANICAL GATE FAILED: cargo test. Fix before running harness."
    fi

    # 0b. cargo clippy
    if ! cargo clippy --workspace -- -D warnings 2>&1 | tail -20 | tee "$RESULT_DIR/step0_clippy.txt"; then
        cd "$PROJECT_ROOT"
        die "MECHANICAL GATE FAILED: cargo clippy. Fix before running harness."
    fi

    cd "$PROJECT_ROOT"

    # 0c. FFI chain check (newly added #[func] methods)
    if [[ -x "$SCRIPT_DIR/ffi_chain_check.sh" ]]; then
        if ! bash "$SCRIPT_DIR/ffi_chain_check.sh" "$PROJECT_ROOT" 2>&1 | tee "$RESULT_DIR/step0_ffi.txt"; then
            log "WARNING: FFI chain issues detected (non-blocking at Step 0)"
        fi
    else
        log "FFI chain check script not found — skipping"
    fi

    log "MECHANICAL GATE: PASS"
}

# ============================================================
# STEP 1a: DRAFTER (Claude Code agent — harness-drafter)
# ============================================================
run_planner() {
    local attempt=$1
    log "=== Step 1a: PLANNER (plan attempt $attempt) ==="

    local feedback_arg=""
    if [[ -f "$REVIEW_DIR/review_latest.md" ]]; then
        feedback_arg="

PREVIOUS REVIEW FEEDBACK (plan was rejected — address these issues):
$(cat "$REVIEW_DIR/review_latest.md")"
    fi

    # Build planner input
    cat > "$PLAN_DIR/planner_input.md" << PLANNER_EOF
# Harness Test Plan Request

## Feature
$(cat "$PROMPT_FILE")

## Your Task
You are the PLANNER. Read the feature description above and produce a test plan.
You do NOT write code. You write a plan that tells the Generator WHAT to test.

## Output Format
Output your test plan directly using this exact structure:

\`\`\`
---
feature: $FEATURE
plan_attempt: $attempt
seed: 42
agent_count: 20
---

## Assertions

### Assertion 1: <name>
- metric: <what to measure>
- threshold: <value>
- type: <A|B|C|D|E>
- rationale: "<why this threshold — cite source for B, cite observed value for C>"
- ticks: <how long to simulate>
- components_read: [<ECS components the test queries>]

### Assertion 2: <name>
...

## Edge Cases
- <edge case 1>: <expected behavior>
- <edge case 2>: <expected behavior>

## NOT in Scope
- <what this test intentionally does NOT check>
\`\`\`

## Rules
- Every threshold MUST have a Type (A/B/C/D/E) and rationale
- Read .claude/skills/worldsim-harness/evaluation_criteria.md for Type definitions
- Type C thresholds: you MUST state the observed value and margin
- Do NOT include implementation hints, code snippets, or architecture suggestions
- Do NOT suggest HOW to implement — only WHAT to verify
$feedback_arg
PLANNER_EOF

    # Run planner agent — capture stdout as plan
    log "Running Planner agent..."
    claude --agent harness-drafter \
           -p "$(cat "$PLAN_DIR/planner_input.md")" \
           --output-format text \
           > "$PLAN_DIR/plan_draft.md" \
           2> >(tee "$PLAN_DIR/planner_log.txt" >&2)

    # Verify output exists and is non-empty
    [[ -s "$PLAN_DIR/plan_draft.md" ]] || die "Planner did not produce plan_draft.md"
    log "Plan draft created: $PLAN_DIR/plan_draft.md"
}

# ============================================================
# STEP 1b: CHALLENGER (agent session — isolated context)
# ============================================================
run_challenger() {
    local round=${1:-1}
    log "=== Step 1b: CHALLENGER (round $round) ==="

    # On round 2+, challenge the revised plan instead of the draft
    local plan_to_challenge="$PLAN_DIR/plan_draft.md"
    if [[ $round -gt 1 ]] && [[ -f "$PLAN_DIR/plan_revised.md" ]]; then
        plan_to_challenge="$PLAN_DIR/plan_revised.md"
    fi

    render_template \
        "$TEMPLATES_DIR/challenger_prompt.md" \
        "$PLAN_DIR/challenger_input_round${round}.md" \
        "FEATURE=$FEATURE" \
        "PLAN_DRAFT=@$plan_to_challenge"

    log "Running Challenger (isolated session, round $round)..."
    claude --agent harness-challenger \
           -p "$(cat "$PLAN_DIR/challenger_input_round${round}.md")" \
           --output-format text \
           > "$PLAN_DIR/challenge_report.md" \
           2> >(tee "$PLAN_DIR/challenger_log_round${round}.txt" >&2)

    [[ -s "$PLAN_DIR/challenge_report.md" ]] || {
        log "WARNING: Challenger produced empty output"
        echo "No challenges raised." > "$PLAN_DIR/challenge_report.md"
    }
    log "Challenge report: $PLAN_DIR/challenge_report.md"
}

# ============================================================
# STEP 1c: DRAFTER REVISION (Claude Code agent)
# ============================================================
run_planner_revision() {
    log "=== Step 1c: DRAFTER REVISION ==="

    local qc_feedback=""
    if [[ -f "$PLAN_DIR/quality_review_latest.md" ]]; then
        qc_feedback="

## Quality Checker Feedback (address these specific issues):
$(cat "$PLAN_DIR/quality_review_latest.md")

IMPORTANT: The Quality Checker found issues with your previous revision. Address EVERY item in their 'Fix These' list."
    fi

    cat > "$PLAN_DIR/revision_input.md" << REVISION_EOF
# Revise Test Plan

## Your Original Plan
$(cat "$PLAN_DIR/plan_draft.md")

## Challenger's Feedback
$(cat "$PLAN_DIR/challenge_report.md")
$qc_feedback

## Your Task
Revise the plan to address the Challenger's valid points.
If a challenge is invalid, explain why and keep the original.
Output the final revised plan directly, using the same format as the original plan.
REVISION_EOF

    log "Running Drafter revision..."
    claude --agent harness-drafter \
           -p "$(cat "$PLAN_DIR/revision_input.md")" \
           --output-format text \
           > "$PLAN_DIR/plan_revised.md" \
           2> >(tee "$PLAN_DIR/revision_log.txt" >&2)

    [[ -s "$PLAN_DIR/plan_revised.md" ]] || {
        log "WARNING: Revision produced empty output — using draft"
        cp "$PLAN_DIR/plan_draft.md" "$PLAN_DIR/plan_revised.md"
    }
    log "Revised plan: $PLAN_DIR/plan_revised.md"
}

# ============================================================
# STEP 1d: QUALITY CHECKER (separate Claude session — isolated)
# ============================================================
run_quality_checker() {
    local round=$1
    log "=== Step 1d: QUALITY CHECKER (round $round) ==="

    render_template \
        "$TEMPLATES_DIR/quality_checker_prompt.md" \
        "$PLAN_DIR/qc_input_round${round}.md" \
        "FEATURE=$FEATURE" \
        "PLAN_DRAFT=@$PLAN_DIR/plan_draft.md" \
        "CHALLENGE_REPORT=@$PLAN_DIR/challenge_report.md" \
        "PLAN_REVISED=@$PLAN_DIR/plan_revised.md"

    log "Running Quality Checker (isolated session, round $round)..."
    claude --agent harness-quality-checker \
           -p "$(cat "$PLAN_DIR/qc_input_round${round}.md")" \
           --output-format text \
           > "$PLAN_DIR/quality_review_round${round}.md" \
           2> >(tee "$PLAN_DIR/qc_log_round${round}.txt" >&2)

    [[ -s "$PLAN_DIR/quality_review_round${round}.md" ]] || {
        log "WARNING: Quality Checker produced empty output — treating as PLAN_APPROVED"
        echo "verdict: PLAN_APPROVED" > "$PLAN_DIR/quality_review_round${round}.md"
    }

    ln -sf "quality_review_round${round}.md" "$PLAN_DIR/quality_review_latest.md"
    log "Quality review: $PLAN_DIR/quality_review_round${round}.md"
}

parse_plan_verdict() {
    local review_file="$PLAN_DIR/quality_review_latest.md"
    local verdict=""

    local cleaned
    cleaned=$(sed 's/\*//g; s/_//g; s/`//g' "$review_file")

    # Standard: "verdict: PLAN_APPROVED"
    verdict=$(echo "$cleaned" | grep -i "^verdict:" | head -1 | awk '{print toupper($2)}' | sed 's/PLAN//' || echo "")

    # Regex fallback
    if [[ -z "$verdict" ]]; then
        verdict=$(echo "$cleaned" | grep -ioE "(PLAN_APPROVED|PLAN_REVISE|PLAN_FAIL|PLANAPPROVED|PLANREVISE|PLANFAIL|APPROVED|REVISE|FAIL)" | tail -1 | sed 's/PLAN_//; s/PLAN//' | tr '[:lower:]' '[:upper:]' || echo "")
    fi

    # Standalone word in last 20 lines
    if [[ -z "$verdict" ]]; then
        verdict=$(tail -20 "$review_file" | sed 's/\*//g; s/_//g' | grep -ioE "\b(APPROVED|REVISE|FAIL)\b" | tail -1 | tr '[:lower:]' '[:upper:]' || echo "")
    fi

    case "$verdict" in
        APPROVED|PLANAPPROVED)
            log "PLAN APPROVED by Quality Checker"
            return 0 ;;
        REVISE|PLANREVISE)
            log "PLAN REVISE requested by Quality Checker"
            return 1 ;;
        FAIL|PLANFAIL)
            log "PLAN FAIL — Quality Checker rejected"
            return 2 ;;
        *)
            log "WARNING: Could not parse plan verdict. Treating as PLAN_APPROVED (safe default)"
            return 0 ;;
    esac
}

# ============================================================
# STEP 2: GENERATOR (separate Claude session — isolated context)
# ============================================================
run_generator() {
    local attempt=$1
    log "=== Step 2: GENERATOR (code attempt $attempt) ==="

    # Determine which plan to use
    local plan_file="$PLAN_DIR/plan_final.md"
    [[ -f "$plan_file" ]] || plan_file="$PLAN_DIR/plan_draft.md"

    # Build feedback section for retries (information isolation: issues only, no scores/verdicts)
    local feedback_section=""
    if [[ $attempt -gt 1 ]]; then
        if [[ -f "$REVIEW_DIR/issues_latest.md" ]] && [[ -s "$REVIEW_DIR/issues_latest.md" ]]; then
            feedback_section="

## Previous Issues to Fix:
$(cat "$REVIEW_DIR/issues_latest.md")"
        elif [[ -f "$REVIEW_DIR/review_latest.md" ]]; then
            feedback_section="

## Previous Evaluator Feedback (fix these issues):
$(cat "$REVIEW_DIR/review_latest.md")"
        fi
    fi

    # Render generator prompt from template (multiline-safe)
    render_template \
        "$TEMPLATES_DIR/generator_prompt.md" \
        "$RESULT_DIR/generator_input_attempt${attempt}.md" \
        "FEATURE=$FEATURE" \
        "PLAN=@$plan_file" \
        "FEATURE_PROMPT=@$PROMPT_FILE" \
        "CODE_ATTEMPT=$attempt" \
        "FEEDBACK=$feedback_section"

    # Generator needs tool access to write code — use --dangerously-skip-permissions
    log "Running Generator (isolated session, attempt $attempt)..."
    claude --agent harness-generator \
           -p "$(cat "$RESULT_DIR/generator_input_attempt${attempt}.md")" \
           --dangerously-skip-permissions \
           --output-format text \
           2>&1 | tee "$RESULT_DIR/generator_log_attempt${attempt}.txt"

    # Run gate
    log "Running gate command..."
    cd "$PROJECT_ROOT/rust"
    if cargo test --workspace 2>&1 | tee "$RESULT_DIR/gate_result_attempt${attempt}.txt"; then
        log "Gate: cargo test PASSED"
    else
        log "Gate: cargo test FAILED"
        echo "GATE_FAILED" >> "$RESULT_DIR/gen_result_attempt${attempt}.md"
    fi

    if cargo clippy --workspace -- -D warnings 2>&1 | tee -a "$RESULT_DIR/gate_result_attempt${attempt}.txt"; then
        log "Gate: clippy PASSED"
    else
        log "Gate: clippy FAILED"
        echo "CLIPPY_FAILED" >> "$RESULT_DIR/gen_result_attempt${attempt}.md"
    fi
    cd "$PROJECT_ROOT"

    # Run harness tests specifically (|| true: failures handled by Evaluator, not set -e)
    log "Running harness tests..."
    cd "$PROJECT_ROOT/rust"
    if cargo test -p sim-test harness_ -- --nocapture 2>&1 | tee "$RESULT_DIR/harness_result_attempt${attempt}.txt"; then
        log "Harness: all tests PASSED"
    else
        log "Harness: some tests FAILED (Evaluator will review)"
    fi
    cd "$PROJECT_ROOT"

    # PostCode FFI check — verify proxy chain after Generator changes
    if changed_sim_bridge && [[ -x "$SCRIPT_DIR/ffi_chain_check.sh" ]]; then
        log "Running PostCode FFI chain check..."
        if ! bash "$SCRIPT_DIR/ffi_chain_check.sh" "$PROJECT_ROOT" 2>&1 | tee "$RESULT_DIR/postcode_ffi_attempt${attempt}.txt"; then
            log "POST-CODE FFI CHAIN BROKEN — RE-CODE"
            echo "FFI chain broken. See postcode_ffi_attempt${attempt}.txt" > "$REVIEW_DIR/issues_latest.md"
            return 1
        fi
    fi

    # Create result summary if Generator didn't
    if [[ ! -f "$RESULT_DIR/gen_result_attempt${attempt}.md" ]]; then
        cat > "$RESULT_DIR/gen_result_attempt${attempt}.md" << RESULT_EOF
---
feature: $FEATURE
code_attempt: $attempt
plan_attempt: $PLAN_ATTEMPT
---

## Gate Results
$(tail -20 "$RESULT_DIR/gate_result_attempt${attempt}.txt")

## Harness Results
$(tail -30 "$RESULT_DIR/harness_result_attempt${attempt}.txt")
RESULT_EOF
    fi

    # Symlink latest result
    ln -sf "gen_result_attempt${attempt}.md" "$RESULT_DIR/gen_result_latest.md"
    log "Generator result: $RESULT_DIR/gen_result_attempt${attempt}.md"
}

# ============================================================
# STEP 2.5a: VISUAL VERIFY (Godot — local execution)
# ============================================================
run_visual_verify() {
    log "=== Step 2.5a: VISUAL VERIFY ==="

    local evidence_dir="$HARNESS_DIR/evidence/$FEATURE"
    mkdir -p "$evidence_dir"

    # Resolve Godot binary
    local godot_bin="${GODOT:-}"
    if [[ -z "$godot_bin" ]]; then
        for candidate in \
            "/Applications/Godot.app/Contents/MacOS/Godot" \
            "$HOME/Downloads/Godot.app/Contents/MacOS/Godot" \
            "$HOME/Applications/Godot.app/Contents/MacOS/Godot"; do
            if [[ -x "$candidate" ]]; then
                godot_bin="$candidate"
                break
            fi
        done
    fi
    if [[ -z "$godot_bin" ]] && command -v godot >/dev/null 2>&1; then
        godot_bin="$(command -v godot)"
    fi

    if [[ -z "$godot_bin" ]] || [[ ! -x "$godot_bin" ]]; then
        log "WARNING: Godot not found — skipping visual verification"
        log "Set GODOT env var or install godot to enable visual verification"
        cat > "$evidence_dir/skip_reason.txt" <<'SKIP_EOF'
Godot not found — visual verification skipped
VISUAL_SKIPPED: GDScript rendering changes cannot be pixel-verified without Godot.
Evaluator should assess GDScript changes via code review only.
Do NOT RE-CODE solely because rendering parameters cannot be visually confirmed.
SKIP_EOF
        return 0
    fi

    # Determine ticks from plan
    local ticks=4380
    if [[ -f "$PLAN_DIR/plan_final.md" ]]; then
        local plan_ticks
        plan_ticks=$(grep -oP 'ticks:\s*\K\d+' "$PLAN_DIR/plan_final.md" 2>/dev/null | sort -rn | head -1 || echo "")
        if [[ -n "$plan_ticks" ]]; then
            ticks="$plan_ticks"
        fi
    fi

    # Run Godot with visual verify script (windowed for screenshots)
    log "Running Godot visual verification (ticks=$ticks)..."
    run_with_timeout 600 "$godot_bin" \
        --path "$PROJECT_ROOT" \
        --script scripts/test/harness_visual_verify.gd \
        -- --feature "$FEATURE" --ticks "$ticks" \
        2>&1 | tee "$evidence_dir/godot_output.txt" || {
            local exit_code=$?
            if [[ $exit_code -eq 124 ]] || [[ $exit_code -eq 142 ]]; then
                log "WARNING: Godot visual verification timed out (10 min)"
                echo "TIMEOUT after 10 minutes" > "$evidence_dir/timeout.txt"
            else
                log "WARNING: Godot visual verification exited with code $exit_code"
                echo "EXIT CODE: $exit_code" > "$evidence_dir/exit_error.txt"
            fi
            # Non-fatal — continue pipeline even if visual verify fails
            return 0
        }

    log "Visual evidence captured to: $evidence_dir"
    ls -la "$evidence_dir/" 2>/dev/null || true
}

# ============================================================
# STEP 2.5b: VLM ANALYSIS (Claude vision — screenshot → text)
# ============================================================
run_vlm_analysis() {
    log "=== Step 2.5b: VLM ANALYSIS ==="

    local evidence_dir="$HARNESS_DIR/evidence/$FEATURE"

    # Check if we have any screenshots
    local screenshot_count
    screenshot_count=$(find "$evidence_dir" -name "screenshot_*.png" 2>/dev/null | wc -l | tr -d ' ')

    if [[ "$screenshot_count" -eq 0 ]]; then
        log "No screenshots found — generating text-only analysis from data files"

        # Build analysis input from text data files
        local analysis_input=""
        for datafile in entity_summary.txt performance.txt console_log.txt; do
            if [[ -f "$evidence_dir/$datafile" ]]; then
                analysis_input+="
## $datafile
$(cat "$evidence_dir/$datafile")
"
            fi
        done

        if [[ -z "$analysis_input" ]]; then
            log "No visual evidence at all — skipping VLM analysis"
            echo "No visual evidence available (Godot verification was skipped or failed)" \
                > "$evidence_dir/visual_analysis.txt"
            return 0
        fi

        # Text-only analysis (no images)
        local vlm_prompt
        vlm_prompt=$(cat <<VLM_EOF
You are a visual verification analyst for a game simulation.
Analyze this data and determine if there are signs of problems.

$analysis_input

Answer with this format:
## Visual Analysis: $FEATURE

### Data Assessment
<your analysis of entity summary, performance, console log>

### Visual Verdict
VISUAL_OK | VISUAL_WARNING(<reason>) | VISUAL_FAIL(<reason>)
VLM_EOF
        )

        claude --agent harness-vlm-analyzer \
            -p "$vlm_prompt" \
            --output-format text \
            > "$evidence_dir/visual_analysis.txt" \
            2> >(tee "$evidence_dir/vlm_log.txt" >&2) || true

    else
        log "Found $screenshot_count screenshots — running VLM analysis"

        # Build image path list for the prompt (Claude reads via Read tool)
        local image_paths=""
        for img in "$evidence_dir"/screenshot_*.png; do
            image_paths+="- $img
"
        done

        # Collect text data
        local text_data=""
        for datafile in entity_summary.txt performance.txt console_log.txt; do
            if [[ -f "$evidence_dir/$datafile" ]]; then
                text_data+="
## $datafile
$(cat "$evidence_dir/$datafile")
"
            fi
        done

        # Extract feature-specific visual checks from plan if present
        local visual_checks=""
        if [[ -f "$PLAN_DIR/plan_final.md" ]]; then
            visual_checks=$(grep -A 20 "## Visual Checks\|## Visual Verification\|## In-Game" \
                "$PLAN_DIR/plan_final.md" 2>/dev/null || echo "No feature-specific visual checks in plan")
        fi

        # Render visual checklist
        render_template \
            "$TEMPLATES_DIR/visual_checklist.md" \
            "$evidence_dir/visual_checklist_rendered.md" \
            "FEATURE=$FEATURE" \
            "VISUAL_CHECKS=$visual_checks"

        # Run Claude with image paths in prompt + tool access to read them
        local vlm_input
        vlm_input=$(cat "$evidence_dir/visual_checklist_rendered.md")
        vlm_input+="

## Screenshot Files
Use the Read tool to view each screenshot image below:
$image_paths
## Data Files
$text_data

Read each screenshot file listed above, then analyze the screenshots and data.
Answer every question in the checklist."

        claude --agent harness-vlm-analyzer \
            -p "$vlm_input" \
            --dangerously-skip-permissions \
            --output-format text \
            > "$evidence_dir/visual_analysis.txt" \
            2> >(tee "$evidence_dir/vlm_log.txt" >&2) || true
    fi

    if [[ ! -s "$evidence_dir/visual_analysis.txt" ]]; then
        log "WARNING: VLM analysis produced empty output"
        echo "VLM analysis failed to produce output" > "$evidence_dir/visual_analysis.txt"
    fi

    log "Visual analysis: $evidence_dir/visual_analysis.txt"
}

# ============================================================
# STEP 2.5a-int: VLM INTERACTIVE (Godot + Python controller)
# ============================================================
run_vlm_interactive() {
    log "=== Step 2.5a-int: VLM INTERACTIVE ==="

    local evidence_dir="$HARNESS_DIR/evidence/$FEATURE"
    mkdir -p "$evidence_dir"

    # Check if prompt has interactive scenarios
    local scenarios
    scenarios=$(sed -n '/^## Interactive Scenarios/,/^## [^I]/p' "$PROMPT_FILE" 2>/dev/null | sed '$d')
    if [[ -z "$scenarios" ]]; then
        log "No interactive scenarios defined — skipping"
        return 0
    fi

    echo "$scenarios" > "$evidence_dir/interactive_scenarios.md"

    # Resolve Godot binary (same logic as run_visual_verify)
    local godot_bin="${GODOT:-}"
    if [[ -z "$godot_bin" ]]; then
        for candidate in \
            "/Applications/Godot.app/Contents/MacOS/Godot" \
            "$HOME/Downloads/Godot.app/Contents/MacOS/Godot" \
            "$HOME/Applications/Godot.app/Contents/MacOS/Godot"; do
            if [[ -x "$candidate" ]]; then
                godot_bin="$candidate"
                break
            fi
        done
    fi
    if [[ -z "$godot_bin" ]] && command -v godot >/dev/null 2>&1; then
        godot_bin="$(command -v godot)"
    fi

    if [[ -z "$godot_bin" ]] || [[ ! -x "$godot_bin" ]]; then
        log "WARNING: Godot not found — skipping VLM interactive"
        return 0
    fi

    # Start Godot with interactive mode
    log "Starting Godot in interactive mode..."
    "$godot_bin" --path "$PROJECT_ROOT" \
        --script scripts/test/harness_visual_verify.gd \
        -- --feature "$FEATURE" --ticks 2000 --interactive \
        2>&1 | tee "$evidence_dir/godot_interactive_output.txt" &
    local godot_pid=$!

    # Wait for TCP server to start (poll port 9223)
    local retries=0
    while [[ $retries -lt 20 ]]; do
        if python3 -c "
import socket, sys
s = socket.socket()
s.settimeout(0.5)
try:
    s.connect(('127.0.0.1', 9223))
    s.close()
    sys.exit(0)
except:
    sys.exit(1)
" 2>/dev/null; then
            break
        fi
        sleep 0.5
        retries=$((retries + 1))
    done

    if [[ $retries -ge 20 ]]; then
        log "WARNING: Godot interactive server did not start — skipping"
        kill "$godot_pid" 2>/dev/null
        wait "$godot_pid" 2>/dev/null
        return 0
    fi

    log "Godot interactive server ready — running controller..."

    # Run interactive controller
    run_with_timeout 120 python3 "$SCRIPT_DIR/interactive_controller.py" \
        --evidence-dir "$evidence_dir" \
        --scenarios "$evidence_dir/interactive_scenarios.md" \
        2>&1 | tee "$evidence_dir/interactive_log.txt" || {
            log "WARNING: Interactive controller exited with error"
        }

    # Cleanup Godot (quit command should have stopped it, but ensure)
    kill "$godot_pid" 2>/dev/null
    wait "$godot_pid" 2>/dev/null

    if [[ -f "$evidence_dir/interactive_results.txt" ]]; then
        log "VLM Interactive results: $evidence_dir/interactive_results.txt"
    else
        log "WARNING: No interactive results produced"
    fi
}

summarize_interactive() {
    local results_file="$1"
    if [[ ! -f "$results_file" ]]; then
        echo "SKIPPED (no scenarios)"
        return
    fi
    local pass_count fail_count
    pass_count=$(grep -c "RESULT: PASS" "$results_file" 2>/dev/null || echo "0")
    fail_count=$(grep -c "RESULT: FAIL" "$results_file" 2>/dev/null || echo "0")
    local scenarios
    scenarios=$(grep -c "^SCENARIO:" "$results_file" 2>/dev/null || echo "0")
    echo "${scenarios} scenarios: ${pass_count} PASS, ${fail_count} FAIL"
}

# ============================================================
# HELPER: Detect if sim-bridge was modified by Generator
# ============================================================
changed_sim_bridge() {
    # Check if sim-bridge lib.rs has uncommitted changes
    git diff --name-only HEAD -- rust/crates/sim-bridge/src/lib.rs 2>/dev/null | grep -q .
}

# ============================================================
# HELPER: Build codex exec command array
# ============================================================
run_codex() {
    local sandbox="$1"
    local output_file="$2"
    local prompt_file="$3"

    local cmd=("$CODEX_BIN" exec)
    if [[ -n "$CODEX_MODEL" ]]; then
        cmd+=(-m "$CODEX_MODEL")
    fi
    cmd+=(-s "$sandbox" -o "$output_file" --ephemeral)

    "${cmd[@]}" < "$prompt_file"
}

# ============================================================
# STEP 2.5c: FFI CHAIN VERIFY (Codex — conditional on sim-bridge changes)
# ============================================================
run_ffi_verify() {
    log "=== Step 2.5c: FFI CHAIN VERIFY (Codex) ==="

    local evidence_dir="$HARNESS_DIR/evidence/$FEATURE"
    mkdir -p "$evidence_dir"

    # Extract only recently changed #[func] methods (feature-scoped)
    # Pre-existing unproxied methods are tech debt, not feature regressions
    local rust_methods
    rust_methods=$(cd "$PROJECT_ROOT" && git diff HEAD~1 -- rust/crates/sim-bridge/src/lib.rs 2>/dev/null \
        | grep '^+.*fn ' | grep -v '^+++' \
        | sed 's/.*fn \([a-z_][a-z_0-9]*\).*/\1/' 2>/dev/null | sort -u || true)

    if [[ -z "$rust_methods" ]]; then
        log "No new #[func] methods detected in sim-bridge — skipping FFI verify"
        echo "ffi_overall: ALL_COMPLETE (no new methods)" > "$evidence_dir/ffi_chain_verify.txt"
        return 0
    fi

    # Build prompt
    local ffi_prompt_file
    ffi_prompt_file=$(mktemp)
    cat > "$ffi_prompt_file" << 'FFI_HEADER'
You are an FFI chain verifier for WorldSim.

Your ONLY job is to verify that every Rust #[func] method in SimBridge has a complete proxy chain through GDScript.

The chain for each method is:
1. rust/crates/sim-bridge/src/lib.rs — #[func] fn <name>
2. scripts/core/simulation/sim_bridge.gd — func <name> (may use EITHER direct call runtime.<name>() OR defensive call() pattern: runtime.call("<name>", ...))
3. scripts/core/simulation/simulation_engine.gd — func <name> (may use EITHER direct call sim_bridge.<name>() OR defensive call() pattern: sim_bridge.call("<name>", ...))

IMPORTANT: Both direct-call and call()-based proxy patterns are VALID. Search for the method name as a string literal too, not just dotted calls.

INSTRUCTIONS:
- The methods listed below are ONLY the recently added/changed methods from this feature
- For each method, check if the proxy exists in both GDScript files using grep
- Do NOT check or report on pre-existing methods not listed below

Output format (one block per broken/new method):
METHOD: <name>
  lib.rs: OK
  sim_bridge.gd: OK/MISSING
  simulation_engine.gd: OK/MISSING/N/A
  chain: COMPLETE/BROKEN

At the end, output a summary line:
ffi_overall: ALL_COMPLETE | HAS_BROKEN

FFI_HEADER

    cat >> "$ffi_prompt_file" << FFI_METHODS
These are the NEWLY ADDED #[func] methods for this feature (from git diff):
$rust_methods

Check the proxy chain for ONLY these methods. Do not scan for other methods.
FFI_METHODS

    log "Running FFI Chain Verify via Codex..."
    if run_codex "read-only" "$evidence_dir/ffi_chain_verify.txt" "$ffi_prompt_file" 2> "$evidence_dir/ffi_verify_log.txt"; then
        log "FFI chain verify complete"
    else
        log "WARNING: FFI chain verify failed (exit $?) — continuing pipeline"
        echo "FFI chain verification failed to execute" > "$evidence_dir/ffi_chain_verify.txt"
    fi

    rm -f "$ffi_prompt_file"

    # Check for BROKEN chains
    if grep -qi "BROKEN\|HAS_BROKEN" "$evidence_dir/ffi_chain_verify.txt" 2>/dev/null; then
        log "FFI CHAIN BROKEN detected — Evaluator will see this evidence"
    else
        log "FFI chains: all complete (or no new methods)"
    fi
}

summarize_ffi_verify() {
    local verify_file="$1"
    local broken
    broken=$(grep -ci "BROKEN" "$verify_file" 2>/dev/null || echo "0")
    local overall
    overall=$(grep -o "ALL_COMPLETE\|HAS_BROKEN" "$verify_file" 2>/dev/null | tail -1 || echo "UNKNOWN")
    echo "${overall}. ${broken} broken chain(s)"
}

# ============================================================
# STEP 2.7: REGRESSION GUARD (Codex — runs every attempt)
# ============================================================
run_regression_guard() {
    log "=== Step 2.7: REGRESSION GUARD (Codex) ==="

    local guard_prompt_file
    guard_prompt_file=$(mktemp)
    cat > "$guard_prompt_file" << 'GUARD_EOF'
You are a regression guard for WorldSim. Your ONLY job is to verify nothing broke.

Run these checks IN ORDER. Do NOT skip any.

1. Run the full gate:
   cd rust && cargo test --workspace 2>&1
   Report: total passed, total failed, any new failures.

2. Run all harness tests:
   cd rust && cargo test -p sim-test harness_ -- --nocapture 2>&1
   Count total harness tests. Report any FAILING tests by name.

3. Check GDScript for obvious errors:
   grep -rn 'func.*(' scripts/ui/renderers/ | head -10
   Look for obvious syntax issues (unmatched brackets, missing colons).

4. Check for broken input handling:
   grep -n '_input\|_unhandled_input\|mouse_filter' scripts/ui/renderers/entity_renderer.gd | head -10
   Verify input handlers still exist in entity_renderer and aren't accidentally removed.
   NOTE: building_renderer.gd intentionally has NO _input handler — do NOT flag its absence.

5. Check FFI evidence if it exists:
   cat .harness/evidence/*/ffi_chain_verify.txt 2>/dev/null
   cat .harness/evidence/*/ffi_verify.txt 2>/dev/null
   ONLY methods listed as BROKEN that were NEWLY ADDED for this feature = regression.
   Pre-existing WARN entries for unproxied methods are technical debt, NOT regressions.
   If ffi_overall says ALL_COMPLETE, the FFI check is CLEAN.

OUTPUT FORMAT (must be machine-parseable):
After your analysis, output EXACTLY one of these lines at the end:
regression_status: CLEAN
regression_status: REGRESSION_DETECTED

If REGRESSION_DETECTED, also output:
regression_details: <specific test or feature that broke>
GUARD_EOF

    log "Running Regression Guard via Codex..."
    if run_codex "workspace-write" "$REVIEW_DIR/regression_guard.txt" "$guard_prompt_file" 2> "$REVIEW_DIR/regression_guard_log.txt"; then
        log "Regression guard complete"
    else
        log "WARNING: Regression guard failed (exit $?) — continuing pipeline"
        echo "regression_status: CLEAN" > "$REVIEW_DIR/regression_guard.txt"
        echo "(Regression guard execution failed — defaulting to CLEAN)" >> "$REVIEW_DIR/regression_guard.txt"
    fi

    rm -f "$guard_prompt_file"

    if grep -q "REGRESSION_DETECTED" "$REVIEW_DIR/regression_guard.txt" 2>/dev/null; then
        log "REGRESSION DETECTED — Evaluator will incorporate this evidence"
        local details
        details=$(grep "regression_details:" "$REVIEW_DIR/regression_guard.txt" 2>/dev/null | head -1 || echo "")
        if [[ -n "$details" ]]; then
            log "  $details"
        fi
    else
        log "Regression guard: CLEAN"
    fi
}

summarize_regression_guard() {
    local guard_file="$1"
    local status
    status=$(grep -o "CLEAN\|REGRESSION_DETECTED" "$guard_file" 2>/dev/null | tail -1 || echo "UNKNOWN")
    local details
    details=$(grep "regression_details:" "$guard_file" 2>/dev/null | head -1 | sed 's/regression_details: //' | cut -c1-50 || echo "")
    echo "${status}${details:+: $details}"
}

# ============================================================
# STEP 3: CODEX EVALUATOR (Codex — independent from Generator)
# ============================================================
run_codex_evaluator() {
    log "=== Step 3: CODEX EVALUATOR ==="

    local plan_file="$PLAN_DIR/plan_final.md"
    [[ -f "$plan_file" ]] || plan_file="$PLAN_DIR/plan_draft.md"

    # Collect the actual test code that was written
    local test_code
    test_code=$(grep -A 200 "fn harness_.*$FEATURE" "$PROJECT_ROOT/rust/crates/sim-test/src/main.rs" 2>/dev/null || echo "No matching harness test found in sim-test")

    # Capture harness output tail
    local harness_tail
    harness_tail=$(tail -40 "$RESULT_DIR/harness_result_attempt${CODE_ATTEMPT}.txt" 2>/dev/null || echo "No harness results")

    # Collect visual analysis if available
    local visual_analysis=""
    if [[ -f "$HARNESS_DIR/evidence/$FEATURE/visual_analysis.txt" ]]; then
        visual_analysis=$(cat "$HARNESS_DIR/evidence/$FEATURE/visual_analysis.txt")
    else
        visual_analysis="Visual verification was not performed (Godot not available or failed)."
    fi

    # Collect FFI chain verify if available
    local ffi_evidence=""
    if [[ -f "$HARNESS_DIR/evidence/$FEATURE/ffi_chain_verify.txt" ]]; then
        ffi_evidence="
## FFI Chain Verification (Codex-verified)
$(cat "$HARNESS_DIR/evidence/$FEATURE/ffi_chain_verify.txt")"
    fi

    # Collect regression guard if available
    local regression_evidence=""
    if [[ -f "$REVIEW_DIR/regression_guard.txt" ]]; then
        regression_evidence="
## Regression Guard (Codex-verified)
$(cat "$REVIEW_DIR/regression_guard.txt")"
    fi

    # Build the full evaluator prompt: instructions + data
    local evaluator_prompt_file
    evaluator_prompt_file=$(mktemp)

    # Read the agent instructions
    local agent_instructions
    agent_instructions=$(cat "$PROJECT_ROOT/.claude/agents/harness-codex-evaluator.md" 2>/dev/null || echo "ERROR: Agent instructions not found")

    # Render data section from template
    render_template \
        "$TEMPLATES_DIR/evaluator_prompt.md" \
        "$REVIEW_DIR/evaluator_data_attempt${CODE_ATTEMPT}.md" \
        "FEATURE=$FEATURE" \
        "PLAN=@$plan_file" \
        "GEN_RESULT=@$RESULT_DIR/gen_result_latest.md" \
        "HARNESS_RESULT=$harness_tail" \
        "TEST_CODE=$test_code" \
        "VISUAL_ANALYSIS=$visual_analysis"

    # Combine: agent instructions + rendered data + extra evidence
    cat > "$evaluator_prompt_file" << EVAL_COMBINE
$agent_instructions

---

$(cat "$REVIEW_DIR/evaluator_data_attempt${CODE_ATTEMPT}.md")
$ffi_evidence
$regression_evidence

---

You are evaluating feature: $FEATURE (code attempt $CODE_ATTEMPT).
Working directory: $PROJECT_ROOT

Run your mandatory execution checks (cargo test, anti-circular, FFI chain).
Then follow the evaluation protocol and output format defined above.
End with verdict: APPROVE, RE-CODE, RE-PLAN, or FAIL.
EVAL_COMBINE

    # Also save the full input for audit trail
    cp "$evaluator_prompt_file" "$REVIEW_DIR/evaluator_input_attempt${CODE_ATTEMPT}.md"

    # Run via Codex (workspace-write: needs to run cargo test)
    log "Running Codex Evaluator (independent session, attempt $CODE_ATTEMPT)..."
    if run_codex "workspace-write" "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" "$evaluator_prompt_file" 2> "$REVIEW_DIR/codex_evaluator_log.txt"; then
        log "Codex Evaluator complete"
    else
        local exit_code=$?
        log "WARNING: Codex Evaluator exited with code $exit_code"
        if [[ ! -s "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" ]]; then
            die "Codex Evaluator failed and produced no output (exit $exit_code). Check $REVIEW_DIR/codex_evaluator_log.txt"
        fi
    fi

    rm -f "$evaluator_prompt_file"

    [[ -s "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" ]] || die "Codex Evaluator did not produce review"

    # Symlink latest
    ln -sf "review_attempt${CODE_ATTEMPT}.md" "$REVIEW_DIR/review_latest.md"
    log "Review: $REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md"

    # Information isolation: extract only issues, never scores/verdicts
    # When RE-CODE, Generator only sees issue descriptions — not scores or verdict rationale
    if grep -qi "RE-CODE\|RE_CODE\|RECODE" "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" 2>/dev/null; then
        # Extract actionable issues from "Issues" and "Fix These" sections
        # Uses flag-based approach (not range) because headers match both start/end patterns
        awk 'BEGIN{p=0}
            /^### Issues/{p=1;next}
            /^### If RE-CODE.*Fix/{p=1;next}
            /^### /{p=0}
            /^verdict:/{p=0}
            p{print}' "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" \
            | grep -v "^Score:\|^verdict:\|APPROVE\|RE-PLAN\|FAIL\|^§[0-9]" \
            > "$REVIEW_DIR/issues_latest.md" 2>/dev/null || true

        # Fallback: if empty or contains only headers (no actionable items), extract numbered lines
        if [[ ! -s "$REVIEW_DIR/issues_latest.md" ]] || ! grep -q '^[0-9]' "$REVIEW_DIR/issues_latest.md" 2>/dev/null; then
            grep '^[0-9]\+\.' "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" \
                > "$REVIEW_DIR/issues_latest.md" 2>/dev/null || true
        fi

        # Final fallback: write a generic message so Generator knows RE-CODE happened
        if [[ ! -s "$REVIEW_DIR/issues_latest.md" ]]; then
            echo "Evaluator requested RE-CODE but specific issues could not be extracted. Check review_attempt${CODE_ATTEMPT}.md manually." \
                > "$REVIEW_DIR/issues_latest.md"
        fi
    fi
}

# ============================================================
# STEP 3 (LEGACY): EVALUATOR via Claude Code — kept as fallback
# ============================================================
run_evaluator() {
    log "=== Step 3: EVALUATOR ==="

    local plan_file="$PLAN_DIR/plan_final.md"
    [[ -f "$plan_file" ]] || plan_file="$PLAN_DIR/plan_draft.md"

    # Collect the actual test code that was written
    local test_code
    test_code=$(grep -A 200 "fn harness_.*$FEATURE" "$PROJECT_ROOT/rust/crates/sim-test/src/main.rs" 2>/dev/null || echo "No matching harness test found in sim-test")

    # Capture harness output tail
    local harness_tail
    harness_tail=$(tail -40 "$RESULT_DIR/harness_result_attempt${CODE_ATTEMPT}.txt" 2>/dev/null || echo "No harness results")

    # Collect visual analysis if available
    local visual_analysis=""
    if [[ -f "$HARNESS_DIR/evidence/$FEATURE/visual_analysis.txt" ]]; then
        visual_analysis=$(cat "$HARNESS_DIR/evidence/$FEATURE/visual_analysis.txt")
    else
        visual_analysis="Visual verification was not performed (Godot not available or failed)."
    fi

    # Render evaluator prompt from template (multiline-safe)
    render_template \
        "$TEMPLATES_DIR/evaluator_prompt.md" \
        "$REVIEW_DIR/evaluator_input.md" \
        "FEATURE=$FEATURE" \
        "PLAN=@$plan_file" \
        "GEN_RESULT=@$RESULT_DIR/gen_result_latest.md" \
        "HARNESS_RESULT=$harness_tail" \
        "TEST_CODE=$test_code" \
        "VISUAL_ANALYSIS=$visual_analysis"

    # Run in isolated session — cannot see Generator's reasoning
    log "Running Evaluator (isolated session)..."
    claude --agent harness-evaluator \
           -p "$(cat "$REVIEW_DIR/evaluator_input.md")" \
           --output-format text \
           > "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" \
           2> >(tee "$REVIEW_DIR/evaluator_log.txt" >&2)

    [[ -s "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md" ]] || die "Evaluator did not produce review"

    # Symlink latest
    ln -sf "review_attempt${CODE_ATTEMPT}.md" "$REVIEW_DIR/review_latest.md"
    log "Review: $REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md"
}

# ============================================================
# STEP 4: PARSE VERDICT + RETRY LOGIC
# ============================================================
parse_verdict() {
    local review_file="$REVIEW_DIR/review_latest.md"
    local verdict=""

    # Strip markdown formatting
    local cleaned
    cleaned=$(sed 's/\*//g; s/_//g; s/`//g' "$review_file")

    # Tier 1: standard format "verdict: X"
    verdict=$(echo "$cleaned" | grep -i "^verdict:" | head -1 | awk '{print toupper($2)}' || echo "")

    # Tier 2: regex anywhere in file
    if [[ -z "$verdict" ]]; then
        verdict=$(echo "$cleaned" | grep -ioE "verdict:\s*(APPROVE|RE-CODE|RE-PLAN|RECODE|REPLAN|FAIL)" | head -1 | sed 's/.*://' | tr -d ' ' | tr '[:lower:]' '[:upper:]' || echo "")
    fi

    # Tier 3: standalone verdict word in last 20 lines
    if [[ -z "$verdict" ]]; then
        verdict=$(tail -20 "$review_file" | sed 's/\*//g; s/_//g; s/`//g' | grep -ioE "\b(APPROVE|RE-CODE|RE-PLAN|RECODE|REPLAN|FAIL)\b" | tail -1 | tr '[:lower:]' '[:upper:]' || echo "")
    fi

    # Normalize
    case "$verdict" in
        APPROVE|APPROVED)
            log "APPROVED by Evaluator"
            return 0
            ;;
        RECODE|RE-CODE|RE_CODE)
            log "RE-CODE requested by Evaluator"
            return 1
            ;;
        REPLAN|RE-PLAN|RE_PLAN)
            log "RE-PLAN requested by Evaluator"
            return 2
            ;;
        FAIL|FAILED)
            log "FAIL verdict from Evaluator"
            return 3
            ;;
        *)
            log "WARNING: Could not parse verdict from review. Raw last 5 lines:"
            tail -5 "$review_file" | while IFS= read -r line; do log "  $line"; done
            log "Treating as RE-CODE (safe default — will retry)"
            return 1
            ;;
    esac
}

# ============================================================
# COMMIT MESSAGE FORMATTER
# ============================================================
format_commit_message() {
    local feature="$1"
    local plan_attempts="$2"
    local code_attempts="$3"
    local qc_rounds
    qc_rounds=$(ls "$PLAN_DIR"/quality_review_round*.md 2>/dev/null | wc -l | tr -d ' ')
    local visual_verdict="SKIP"
    if [[ -f "$HARNESS_DIR/evidence/$feature/visual_analysis.txt" ]]; then
        local raw_verdict
        raw_verdict=$(grep -o "VISUAL_[A-Z]*" "$HARNESS_DIR/evidence/$feature/visual_analysis.txt" | tail -1 || echo "")
        if [[ -n "$raw_verdict" ]]; then
            visual_verdict="${raw_verdict#VISUAL_}"
        fi
    fi
    local regression="SKIP"
    if [[ -f "$REVIEW_DIR/regression_guard.txt" ]]; then
        regression=$(grep -o "CLEAN\|REGRESSION_DETECTED" "$REVIEW_DIR/regression_guard.txt" | tail -1 || echo "SKIP")
    fi
    local ffi="SKIP"
    if [[ -f "$HARNESS_DIR/evidence/$feature/ffi_chain_verify.txt" ]]; then
        ffi=$(grep -o "ALL_COMPLETE\|HAS_BROKEN" "$HARNESS_DIR/evidence/$feature/ffi_chain_verify.txt" | tail -1 || echo "SKIP")
    fi
    echo "feat($feature): implementation [harness: plan x${plan_attempts}(QC:r${qc_rounds}) code x${code_attempts} eval:APPROVE(codex) visual:${visual_verdict} ffi:${ffi} regr:${regression}]"
}

# ============================================================
# MAIN LOOP
# ============================================================
main() {
    log "=========================================="
    log "Harness Pipeline v4 — $FEATURE"
    log "Mode: $MODE"
    log "Prompt: $PROMPT_FILE"
    log "=========================================="

    init_progress

    # Step 0: Mechanical Gate (0-token — must pass before any LLM calls)
    run_mechanical_gate
    report_step "0 Mechanical Gate" "DONE" "cargo test + clippy + FFI check"

    # ============================================================
    # LIGHT MODE — Generator + Gate + Visual Verify only (2 LLM calls)
    # ============================================================
    if [[ "$MODE" == "--light" ]]; then
        log "=== LIGHT MODE — Generator + Gate + Visual Verify only ==="

        report_step "1a-1d Planning" "SKIPPED" "--light mode (prompt used as plan directly)"

        # Use prompt file directly as plan
        cp "$PROMPT_FILE" "$PLAN_DIR/plan_final.md"

        CODE_ATTEMPT=1

        run_generator $CODE_ATTEMPT
        report_step "2 Generator A1" "DONE" "$(summarize_generator "$RESULT_DIR/gen_result_attempt${CODE_ATTEMPT}.md")"

        run_visual_verify
        report_step "2.5a Visual Verify" "DONE" "$(summarize_visual "$EVIDENCE_DIR")"

        run_vlm_interactive
        report_step "2.5a-int VLM Interactive" "DONE" "$(summarize_interactive "$EVIDENCE_DIR/interactive_results.txt")"

        run_vlm_analysis
        report_step "2.5b VLM Analysis" "DONE" "$(summarize_vlm "$EVIDENCE_DIR/visual_analysis.txt")"

        report_step "3 Evaluator" "SKIPPED" "--light mode (VLM result is the verdict)"

        # VLM result is the verdict in light mode
        local vlm_result
        vlm_result=$(head -1 "$EVIDENCE_DIR/visual_analysis.txt" 2>/dev/null || echo "UNKNOWN")
        local vlm_note=""
        if [[ "$vlm_result" != *"VISUAL_OK"* ]]; then
            vlm_note=" (VLM: $vlm_result — review visual_analysis.txt)"
        fi

        echo "APPROVED" > "$REVIEW_DIR/verdict"
        echo "$FEATURE" >> "$REVIEW_DIR/verdict"
        date +%s >> "$REVIEW_DIR/verdict"

        finalize_progress
        bash "$PROJECT_ROOT/tools/harness/generate_report.sh" "$FEATURE" --mode "$MODE" 2>/dev/null || true

        echo ""
        echo "============================================"
        echo "  PIPELINE REPORT — $FEATURE"
        echo "============================================"
        cat "$HARNESS_DIR/reports/$FEATURE/pipeline_report.md" 2>/dev/null || true
        echo "============================================"

        local commit_msg
        commit_msg="feat($FEATURE): implementation [harness: light mode, visual:$(grep -o "VISUAL_[A-Z]*" "$EVIDENCE_DIR/visual_analysis.txt" 2>/dev/null | tail -1 | sed 's/VISUAL_//' || echo "SKIP")]"
        echo "$commit_msg" > "$REVIEW_DIR/commit_message.txt"
        log "==========================================="
        log "LIGHT MODE COMPLETE — $FEATURE approved${vlm_note}"
        log "Suggested commit: $commit_msg"
        log "==========================================="
        exit 0
    fi

    while [[ $PLAN_ATTEMPT -lt $MAX_PLAN_ATTEMPTS ]]; do
        PLAN_ATTEMPT=$((PLAN_ATTEMPT + 1))
        CODE_ATTEMPT=0

        # --- Step 1: Planning (debate loop) ---
        run_planner $PLAN_ATTEMPT
        report_step "1a Drafter" "DONE" "$(summarize_drafter "$PLAN_DIR/plan_draft.md")"

        if [[ "$MODE" == "--full" ]]; then
            local PLAN_ROUND=0
            local MAX_PLAN_ROUNDS=2
            local plan_approved=false

            while [[ $PLAN_ROUND -lt $MAX_PLAN_ROUNDS ]]; do
                PLAN_ROUND=$((PLAN_ROUND + 1))
                log "=== Plan debate round $PLAN_ROUND / $MAX_PLAN_ROUNDS ==="

                run_challenger $PLAN_ROUND
                report_step "1b Challenger R$PLAN_ROUND" "DONE" "$(summarize_challenger "$PLAN_DIR/challenge_report.md")"

                run_planner_revision
                report_step "1c Revision" "DONE" "Revised plan incorporating challenger feedback"

                run_quality_checker $PLAN_ROUND
                report_step "1d QC R$PLAN_ROUND" "DONE" "$(summarize_qc "$PLAN_DIR/quality_review_latest.md")"

                local plan_verdict=0
                parse_plan_verdict || plan_verdict=$?

                case $plan_verdict in
                    0)  # PLAN_APPROVED
                        cp "$PLAN_DIR/plan_revised.md" "$PLAN_DIR/plan_final.md"
                        log "Plan approved after $PLAN_ROUND debate round(s)"
                        plan_approved=true
                        break
                        ;;
                    1)  # PLAN_REVISE
                        if [[ $PLAN_ROUND -ge $MAX_PLAN_ROUNDS ]]; then
                            log "Max plan rounds ($MAX_PLAN_ROUNDS) reached — using current revision"
                            cp "$PLAN_DIR/plan_revised.md" "$PLAN_DIR/plan_final.md"
                            plan_approved=true
                            break
                        fi
                        log "Quality Checker requested revision — starting round $((PLAN_ROUND + 1))"
                        ;;
                    2)  # PLAN_FAIL
                        die "Quality Checker verdict: PLAN_FAIL. Cannot proceed.
Feature: $FEATURE
Quality review: $PLAN_DIR/quality_review_latest.md"
                        ;;
                esac
            done

            if [[ "$plan_approved" != "true" ]]; then
                cp "$PLAN_DIR/plan_revised.md" "$PLAN_DIR/plan_final.md" 2>/dev/null || \
                cp "$PLAN_DIR/plan_draft.md" "$PLAN_DIR/plan_final.md"
            fi
        else
            # --quick mode: skip debate
            log "Quick mode — skipping debate"
            cp "$PLAN_DIR/plan_draft.md" "$PLAN_DIR/plan_final.md"
            report_step "1b Challenger" "SKIPPED" "--quick mode (no debate needed)"
            report_step "1c Revision" "SKIPPED" "--quick mode"
            report_step "1d QC" "SKIPPED" "--quick mode"
        fi

        # --- Steps 2-3: Generate + Evaluate loop ---
        while [[ $CODE_ATTEMPT -lt $MAX_CODE_ATTEMPTS ]]; do
            CODE_ATTEMPT=$((CODE_ATTEMPT + 1))

            run_generator $CODE_ATTEMPT
            report_step "2 Generator A$CODE_ATTEMPT" "DONE" "$(summarize_generator "$RESULT_DIR/gen_result_attempt${CODE_ATTEMPT}.md")"

            run_visual_verify
            report_step "2.5a Visual Verify" "DONE" "$(summarize_visual "$EVIDENCE_DIR")"

            run_vlm_interactive
            report_step "2.5a-int VLM Interactive" "DONE" "$(summarize_interactive "$EVIDENCE_DIR/interactive_results.txt")"

            run_vlm_analysis
            report_step "2.5b VLM Analysis" "DONE" "$(summarize_vlm "$EVIDENCE_DIR/visual_analysis.txt")"

            # --- Codex verification steps (independent from Generator) ---

            # Step 2.5c: FFI Chain Verify — only when sim-bridge changed
            if changed_sim_bridge; then
                run_ffi_verify
                report_step "2.5c FFI Verify (Codex)" "DONE" "$(summarize_ffi_verify "$HARNESS_DIR/evidence/$FEATURE/ffi_chain_verify.txt")"
            else
                report_step "2.5c FFI Verify (Codex)" "SKIPPED" "sim-bridge not modified"
            fi

            # Step 2.7: Regression Guard — always runs
            run_regression_guard
            report_step "2.7 Regression Guard (Codex)" "DONE" "$(summarize_regression_guard "$REVIEW_DIR/regression_guard.txt")"

            # Step 3: Codex Evaluator — replaces Claude Code evaluator for bias isolation
            run_codex_evaluator
            report_step "3 Evaluator (Codex)" "DONE" "$(summarize_evaluator "$REVIEW_DIR/review_attempt${CODE_ATTEMPT}.md")"

            local verdict_code=0
            parse_verdict || verdict_code=$?

            case $verdict_code in
                0)  # APPROVE
                    # Mark as approved for pre-commit hook (Layer 1 + Layer 3 read this)
                    echo "APPROVED" > "$REVIEW_DIR/verdict"
                    echo "$FEATURE" >> "$REVIEW_DIR/verdict"
                    date +%s >> "$REVIEW_DIR/verdict"

                    # Layer 4: Generate pipeline report (immutable audit trail)
                    finalize_progress
                    local report_path
                    report_path=$(bash "$PROJECT_ROOT/tools/harness/generate_report.sh" "$FEATURE" --mode "$MODE" 2>/dev/null || echo "")
                    local score="0"
                    local pipeline_grade="PIPELINE_FAILED"
                    if [[ -n "$report_path" ]]; then
                        log "Pipeline report: $report_path"
                        score=$(grep -oE '\*\*([0-9]+)\*\*' "$report_path" 2>/dev/null | head -1 | tr -d '*' || echo "0")
                        if [[ -n "$score" && "$score" -ge 95 ]] 2>/dev/null; then
                            pipeline_grade="PIPELINE_PASSED"
                        elif [[ -n "$score" && "$score" -ge 70 ]] 2>/dev/null; then
                            pipeline_grade="PIPELINE_ACCEPTABLE"
                            log "WARNING: Score ${score}/100 — ACCEPTABLE but below 95. Pre-commit hook will block."
                        else
                            pipeline_grade="PIPELINE_FAILED"
                            log "WARNING: Score ${score}/100 — FAILED. Pre-commit hook will block."
                        fi
                    fi
                    # Append score + grade to verdict file for hook consumption
                    echo "$score" >> "$REVIEW_DIR/verdict"
                    echo "$pipeline_grade" >> "$REVIEW_DIR/verdict"

                    echo ""
                    echo "============================================"
                    echo "  PIPELINE REPORT — $FEATURE"
                    echo "============================================"
                    cat "$HARNESS_DIR/reports/$FEATURE/pipeline_report.md" 2>/dev/null || true
                    echo "============================================"

                    # Suggest commit message with evidence metadata
                    local commit_msg
                    commit_msg=$(format_commit_message "$FEATURE" "$PLAN_ATTEMPT" "$CODE_ATTEMPT")
                    echo "$commit_msg" > "$REVIEW_DIR/commit_message.txt"
                    log "=========================================="
                    log "Pipeline COMPLETE — $FEATURE approved"
                    log "Plan attempts: $PLAN_ATTEMPT, Code attempts: $CODE_ATTEMPT"
                    log "Suggested commit: $commit_msg"
                    log "=========================================="
                    exit 0
                    ;;
                1)  # RE-CODE
                    echo "RE-CODE" > "$REVIEW_DIR/verdict"
                    echo "$FEATURE" >> "$REVIEW_DIR/verdict"
                    date +%s >> "$REVIEW_DIR/verdict"
                    if [[ $CODE_ATTEMPT -ge $MAX_CODE_ATTEMPTS ]]; then
                        log "Max code attempts ($MAX_CODE_ATTEMPTS) reached — escalating to RE-PLAN"
                        break  # Break inner loop → re-plan
                    fi
                    log "Retrying Generator with Evaluator feedback (attempt $((CODE_ATTEMPT+1)))"
                    ;;
                2)  # RE-PLAN
                    echo "RE-PLAN" > "$REVIEW_DIR/verdict"
                    echo "$FEATURE" >> "$REVIEW_DIR/verdict"
                    date +%s >> "$REVIEW_DIR/verdict"
                    log "Re-planning from Step 1a"
                    break  # Break inner loop → re-plan
                    ;;
                3)  # FAIL
                    echo "FAIL" > "$REVIEW_DIR/verdict"
                    echo "$FEATURE" >> "$REVIEW_DIR/verdict"
                    date +%s >> "$REVIEW_DIR/verdict"
                    finalize_progress
                    bash "$PROJECT_ROOT/tools/harness/generate_report.sh" "$FEATURE" --mode "$MODE" 2>/dev/null || true
                    echo ""
                    echo "============================================"
                    echo "  PIPELINE REPORT — $FEATURE"
                    echo "============================================"
                    cat "$HARNESS_DIR/reports/$FEATURE/pipeline_report.md" 2>/dev/null || true
                    echo "============================================"
                    die "Evaluator verdict: FAIL. Manual intervention required.
Feature: $FEATURE
Review: $REVIEW_DIR/review_latest.md
Plan: $PLAN_DIR/plan_final.md"
                    ;;
            esac
        done
    done

    # Write final verdict for report generator (last evaluator result)
    local last_review_verdict="FATAL"
    local last_review_file
    last_review_file=$(ls "$REVIEW_DIR"/review_attempt*.md 2>/dev/null | tail -1)
    if [[ -f "$last_review_file" ]]; then
        last_review_verdict=$(sed 's/\*//g; s/_//g' "$last_review_file" | grep -i "^verdict:" | head -1 | awk '{print $2}' || echo "FATAL")
    fi
    echo "$last_review_verdict" > "$REVIEW_DIR/verdict"
    echo "$FEATURE" >> "$REVIEW_DIR/verdict"
    date +%s >> "$REVIEW_DIR/verdict"

    finalize_progress
    bash "$PROJECT_ROOT/tools/harness/generate_report.sh" "$FEATURE" --mode "$MODE" 2>/dev/null || true
    echo ""
    echo "============================================"
    echo "  PIPELINE REPORT — $FEATURE"
    echo "============================================"
    cat "$HARNESS_DIR/reports/$FEATURE/pipeline_report.md" 2>/dev/null || true
    echo "============================================"
    die "Max plan attempts ($MAX_PLAN_ATTEMPTS) exhausted. Manual intervention required.
Feature: $FEATURE
Last review: $REVIEW_DIR/review_latest.md"
}

main

#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# WorldSim Harness Pipeline v3.1 — Enforced Multi-Agent (All Code Changes)
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
#   Step 3:   EVALUATOR     (Claude Code -p)      → review.md + verdict
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
FEATURE="${1:?Usage: harness_pipeline.sh <feature_name> <prompt_file> [--quick]}"
PROMPT_FILE="${2:?Usage: harness_pipeline.sh <feature_name> <prompt_file> [--quick]}"
MODE="${3:-full}"  # "full" or "--quick"

# --- Directories ---
PLAN_DIR="$HARNESS_DIR/plans/$FEATURE"
RESULT_DIR="$HARNESS_DIR/results/$FEATURE"
REVIEW_DIR="$HARNESS_DIR/reviews/$FEATURE"

mkdir -p "$PLAN_DIR" "$RESULT_DIR" "$REVIEW_DIR"

# --- Counters ---
PLAN_ATTEMPT=0
MAX_PLAN_ATTEMPTS=2
CODE_ATTEMPT=0
MAX_CODE_ATTEMPTS=3

# --- Logging ---
log() { echo "[harness $(date +%H:%M:%S)] $*"; }
die() { log "FATAL: $*"; exit 1; }

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
command -v python3 >/dev/null 2>&1 || die "python3 not found (needed for template rendering)"

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
    local verdict
    verdict=$(sed 's/\*//g; s/_//g' "$review_file" | grep -i "^verdict:" | head -1 | awk '{print toupper($2)}' || echo "UNKNOWN")

    case "$verdict" in
        PLANAPPROVED|PLAN-APPROVED|APPROVED)
            log "PLAN APPROVED by Quality Checker"
            return 0 ;;
        PLANREVISE|PLAN-REVISE|REVISE)
            log "PLAN REVISE requested by Quality Checker"
            return 1 ;;
        PLANFAIL|PLAN-FAIL|FAIL)
            log "PLAN FAIL — Quality Checker rejected the plan"
            return 2 ;;
        *)
            log "Unknown plan verdict: $verdict — treating as PLAN_APPROVED"
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

    # Build feedback section for retries
    local feedback_section=""
    if [[ -f "$REVIEW_DIR/review_latest.md" ]] && [[ $attempt -gt 1 ]]; then
        feedback_section="

## Previous Evaluator Feedback (fix these issues):
$(cat "$REVIEW_DIR/review_latest.md")"
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
    claude -p "$(cat "$RESULT_DIR/generator_input_attempt${attempt}.md")" \
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

    # Run harness tests specifically
    log "Running harness tests..."
    cd "$PROJECT_ROOT/rust"
    cargo test -p sim-test harness_ -- --nocapture 2>&1 | tee "$RESULT_DIR/harness_result_attempt${attempt}.txt"
    cd "$PROJECT_ROOT"

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
        echo "Godot not found — visual verification skipped" > "$evidence_dir/skip_reason.txt"
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

        claude -p "$vlm_prompt" \
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

        claude -p "$vlm_input" \
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
# STEP 3: EVALUATOR (separate Claude session — isolated context)
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
    claude -p "$(cat "$REVIEW_DIR/evaluator_input.md")" \
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
    local verdict

    # Extract verdict line (strip markdown bold/italic markers before matching)
    verdict=$(sed 's/\*//g; s/_//g' "$review_file" | grep -i "^verdict:" | head -1 | awk '{print toupper($2)}' || echo "UNKNOWN")

    case "$verdict" in
        APPROVE)
            log "APPROVED — ready to commit"
            return 0
            ;;
        RE-CODE|RECODE|RE_CODE)
            log "RE-CODE requested"
            return 1
            ;;
        RE-PLAN|REPLAN|RE_PLAN)
            log "RE-PLAN requested"
            return 2
            ;;
        FAIL)
            log "FAIL — cannot be resolved automatically"
            return 3
            ;;
        *)
            log "Unknown verdict: $verdict — treating as RE-CODE"
            return 1
            ;;
    esac
}

# ============================================================
# MAIN LOOP
# ============================================================
main() {
    log "=========================================="
    log "Harness Pipeline v3 — $FEATURE"
    log "Mode: $MODE"
    log "Prompt: $PROMPT_FILE"
    log "=========================================="

    while [[ $PLAN_ATTEMPT -lt $MAX_PLAN_ATTEMPTS ]]; do
        PLAN_ATTEMPT=$((PLAN_ATTEMPT + 1))
        CODE_ATTEMPT=0

        # --- Step 1: Planning (debate loop) ---
        run_planner $PLAN_ATTEMPT

        if [[ "$MODE" != "--quick" ]]; then
            local PLAN_ROUND=0
            local MAX_PLAN_ROUNDS=2
            local plan_approved=false

            while [[ $PLAN_ROUND -lt $MAX_PLAN_ROUNDS ]]; do
                PLAN_ROUND=$((PLAN_ROUND + 1))
                log "=== Plan debate round $PLAN_ROUND / $MAX_PLAN_ROUNDS ==="

                run_challenger $PLAN_ROUND
                run_planner_revision

                run_quality_checker $PLAN_ROUND

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
            log "Quick mode — skipping debate"
            cp "$PLAN_DIR/plan_draft.md" "$PLAN_DIR/plan_final.md"
        fi

        # --- Steps 2-3: Generate + Evaluate loop ---
        while [[ $CODE_ATTEMPT -lt $MAX_CODE_ATTEMPTS ]]; do
            CODE_ATTEMPT=$((CODE_ATTEMPT + 1))

            run_generator $CODE_ATTEMPT
            run_visual_verify
            run_vlm_analysis
            run_evaluator

            local verdict_code=0
            parse_verdict || verdict_code=$?

            case $verdict_code in
                0)  # APPROVE
                    # Mark as approved for pre-commit hook
                    echo "APPROVED" > "$REVIEW_DIR/verdict"
                    echo "$FEATURE" >> "$REVIEW_DIR/verdict"
                    date +%s >> "$REVIEW_DIR/verdict"
                    log "=========================================="
                    log "Pipeline COMPLETE — $FEATURE approved"
                    log "Plan attempts: $PLAN_ATTEMPT, Code attempts: $CODE_ATTEMPT"
                    log "=========================================="
                    exit 0
                    ;;
                1)  # RE-CODE
                    if [[ $CODE_ATTEMPT -ge $MAX_CODE_ATTEMPTS ]]; then
                        log "Max code attempts ($MAX_CODE_ATTEMPTS) reached — escalating to RE-PLAN"
                        break  # Break inner loop → re-plan
                    fi
                    log "Retrying Generator with Evaluator feedback (attempt $((CODE_ATTEMPT+1)))"
                    ;;
                2)  # RE-PLAN
                    log "Re-planning from Step 1a"
                    break  # Break inner loop → re-plan
                    ;;
                3)  # FAIL
                    die "Evaluator verdict: FAIL. Manual intervention required.
Feature: $FEATURE
Review: $REVIEW_DIR/review_latest.md
Plan: $PLAN_DIR/plan_final.md"
                    ;;
            esac
        done
    done

    die "Max plan attempts ($MAX_PLAN_ATTEMPTS) exhausted. Manual intervention required.
Feature: $FEATURE
Last review: $REVIEW_DIR/review_latest.md"
}

main

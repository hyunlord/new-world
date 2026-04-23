# Harness Infrastructure — Issue Tracker

## Completed

### ✅ Issue 1: Codex MCP timeout (harness-infra-stabilization, 7fb939b)
- `run_codex` now wrapped with `run_with_timeout` (CODEX_TIMEOUT_SECONDS, default 600s)
- Fallback to `claude harness-evaluator` if Codex unreachable

### ✅ Issue 2: VLM isolation + contamination detection (7fb939b)
- VLM runs in `exec < /dev/null; env -i` subshell — stop-hook text cannot leak in
- Contamination detection quarantines tainted output and emits VISUAL_WARNING

### ✅ Issue 3: SKIP budget tracking (7fb939b)
- `stop-check.sh` warns when HARNESS_SKIP budget (3/5 features) is exceeded

### ✅ Issue 4: All `claude --agent` calls have timeout (harness-infra-v2)
- 7 agent calls now wrapped with `run_with_timeout`:
  - harness-drafter ×2 (DRAFTER_TIMEOUT_SECONDS, 600s)
  - harness-challenger (CHALLENGER_TIMEOUT_SECONDS, 600s)
  - harness-quality-checker (QC_TIMEOUT_SECONDS, 600s)
  - harness-generator (GENERATOR_TIMEOUT_SECONDS, 900s — runs cargo test)
  - harness-vlm-analyzer ×2 (VLM_TIMEOUT_SECONDS, 600s)
  - harness-evaluator (EVALUATOR_TIMEOUT_SECONDS, 600s)
- Graceful fallback: Challenger/QC/VLM timeout → empty output → existing fallback path
- Hard failure: Drafter/Generator/Evaluator timeout → `die` (output required)
- Motivation: Round 3 Integration Generator hung at 42 log lines — required manual kill

### ✅ Issue 5: Codex auth renewal procedure documented (harness-infra-v2)
- Pre-flight check script: `tools/harness/hooks/pre-flight-check.sh`
- Probes Codex auth (10s timeout) and warns if expired
- Non-fatal: pipeline continues with claude fallback

## Known Issues

### Godot headless subprocess hang (from sprite-infra)
- `scripts/test/harness_sprite_infra_picker.gd` not wired to sim-test
- Root cause: Godot headless mode doesn't terminate cleanly after script execution
- Status: Low priority until tile grid rendering (Feature 5+)

## Monitoring

If any single agent times out 3+ times in a week, revisit that agent's timeout value.
Env vars allow tuning without code changes (e.g. `GENERATOR_TIMEOUT_SECONDS=1200`).

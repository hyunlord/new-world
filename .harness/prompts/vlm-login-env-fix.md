# Feature: vlm-login-env-fix

## Summary

Fix the systematic Visual WARNING that has plagued every harness pipeline run
today. Root cause: `harness_pipeline.sh`'s VLM analysis wrapped its
`claude --agent harness-vlm-analyzer` invocation in `env -i` with only 5
environment variables preserved (PATH/HOME/USER/TERM/CLAUDE_CONFIG_DIR), which
stripped the macOS launch-services context required for keychain access. The
spawned Claude CLI then reported "Not logged in" and the pipeline appended
`VISUAL_WARNING`, costing −8 points (score ceiling 92).

Two layers of leak protection were already in place:
- `exec < /dev/null` (closes stdin → blocks any stop-hook *text*)
- A dedicated subshell

The `env -i` was therefore overzealous — it removed something the auth path
needed without protecting against any vector that the stdin redirect didn't
already cover.

## Changes Made

### tools/harness/harness_pipeline.sh
Both VLM call sites (text-only mode + image+text mode) replaced their
`env -i ... claude` invocation with a plain `claude` call inside the existing
closed-stdin subshell. `HARNESS_VLM_ISOLATED=1` is still exported so the
spawned process can detect it is running under the harness. Comment updated to
explain why `env -i` was removed.

### tools/harness/diagnose_vlm.sh (new, +x)
Three-test diagnostic suite for verifying VLM authentication:
- Test 1: `claude --version` works in the current shell
- Test 2: closed-stdin spawn (matches the new harness call) returns
  authenticated output
- Test 3: VLM agent invocation produces a verdict-style token
  (VISUAL_OK / VISUAL_WARNING / VISUAL_FAIL)

Exits non-zero on each failure mode for use in CI.

## Verification

- `bash tools/harness/diagnose_vlm.sh` — 3/3 PASS (VISUAL_OK observed)
- `cd rust && cargo test --workspace` — 1145+ passed, 0 failed (no Rust changes)
- `cd rust && cargo clippy --workspace -- -D warnings` — clean

## Scope

- No simulation logic changes
- No Rust changes (workspace tests untouched)
- No score / pipeline structure changes
- VLM prompt / agent / parsing unchanged
- Hook threshold restoration (90 → 95) intentionally deferred — it should land
  in a separate commit only after the *next* feature confirms VISUAL_OK is
  reliably reached under the new VLM call.

## Roadmap v4 Status (unchanged)

| Prereq | State |
|--------|-------|
| A-3 Effect Primitive | DONE |
| A-4 Causal Tracking | DONE |
| A-5 System Frequency Tiering | DONE |
| A-6 Room BFS | DONE |
| A-8 Temperament | DONE |

This commit is environment infrastructure, not a roadmap prereq itself.

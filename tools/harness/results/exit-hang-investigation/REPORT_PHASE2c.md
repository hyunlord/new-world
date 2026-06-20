# Generator Exit-Hang — PHASE 2c: H3 REFUTED (MCP fix necessary-but-insufficient)

**Date:** 2026-06-20 (post-reboot re-run)
**Run:** `harness_pipeline.sh inventory-2-2-pickup .harness/prompts/inventory-2-2-pickup.md --full`
**Generator:** code attempt 1 → FATAL timeout (rc 142) → **no Evaluator verdict → no commit**
**HEAD:** unchanged at `fffd7bfc` (PHASE 2b MCP fix). NO ENV-BYPASS.

## TL;DR

The PHASE 2b MCP-disable fix (`fffd7bfc`) **worked exactly as designed** — it eliminated all 6
MCP-server child processes. But the Generator **hung again with the identical signature**. This
**refutes H3** (the hypothesis that wedged MCP children caused the hang). The true, constant root
cause is the `claude --agent --output-format text` process **not draining its Anthropic API
connection pool on agent completion**, leaving libuv's event loop parked in `kevent64` forever.

## Evidence — before/after, airtight

Watchdog capture: `tools/harness/results/exit-hang-investigation/live_capture_20260620_033151_attempt1/`
(`outcome.txt: completed gen_rc=142 log_bytes=0`)

| Signal | PRIOR (pre-fix, PID 42587, 06-15) | THIS (post-fix, PID 85577, 06-20) |
|---|---|---|
| `--strict-mcp-config` flag | absent | **present** (harness_pipeline.sh:1098 + all 8 `claude --agent` sites) |
| **Descendant processes** | **6** (omc-bridge, team-mcp, server-github, agentmemory, firecrawl, codex-mcp) — all wedged in `start (in dyld)+6992` | **0 (EMPTY)** ✅ |
| **ESTABLISHED sockets** | 14 (≈9 Anthropic API + googleusercontent/MCP + …) | **6 — ALL Anthropic API** (160.79.104.10:https), FDs 9/11/12/13/23/26 |
| **Main thread** | parked in `kevent64` | **4099/4134 samples = 99.2% in `kevent64`** (frame `start (in dyld)+6992` → libuv loop → kevent64 — byte-identical) |
| stdout/stderr | 0-byte log | 0-byte `generator_log_attempt1.txt` (FDs 1/2/4/10) |
| Outcome | rc 142 @ 1800s | rc 142 @ 1800s |

The MCP fix removed the 6 children AND the extra MCP-related sockets (14→6 ESTABLISHED;
googleusercontent socket gone). The hang did not change.

## Diagnosis

- **H3 (wedged MCP children) — REFUTED.** With 0 MCP children, the 6 Anthropic keep-alive sockets
  alone keep the loop alive. The MCP children in the prior capture were a *co-symptom* of the same
  pattern (referenced libuv handles preventing event-loop drain), not the cause.
- **H1 (orphan / main dead) — ruled out.** Main `claude` alive throughout (parent = perl wrapper).
- **H2 (mid-read API stall) — not the picture.** Main is parked in `kevent64` (idle event loop
  waiting for events), not blocked in a `read` on a socket. The 6 sockets are idle keep-alive, not
  an in-flight streaming read (130s→1800s with no data delivered).
- **TRUE ROOT CAUSE (exit-drain):** `claude --agent --output-format text` finishes the agent's work
  (all Gather code written to disk) but does **not** close its Anthropic API connection pool
  (undici/fetch keep-alive) on completion. Those ~6 referenced socket handles keep libuv's event
  loop live → process never exits cleanly → `--output-format text` (which flushes only at clean
  exit) never flushes → 0-byte log → perl SIGALRM kill at the 1800s deadline.

## Two SEPARATE infra issues, now cleanly distinguished

1. **Generator exit-hang** (this report): exit-drain of the API connection pool. NOT fixed by
   MCP-disable. **Next target = F3.**
2. **dyld test-binary tax**: Step-0 mechanical gate took **~59 min** this run (02:25:55 →
   03:24:58). Reboot reduced it (was ~2.8h) but did not eliminate it — ~262 `deps/harness_*`
   binaries each ~20-28s in dyld startup. Separate, lower-priority infra target.

## Recommended fix — F3 (completion-signal early-exit), NOT a bypass

`text` output can never work here (only flushes on a clean exit that never happens). Switch the
`claude --agent` calls to **`--output-format stream-json`**:
- `claude` flushes incrementally and emits a terminal `{"type":"result",...}` message.
- The harness/watchdog detects that marker → the agent's work is provably complete.
- Then gracefully SIGTERM the lingering process (it is only holding idle API keep-alive sockets)
  and proceed to Visual Verify + Evaluator.

This is a genuine fix to the root cause (detect real completion + reap a provably-done process),
**not** a timeout raise, **not** `HARNESS_SKIP_GENERATOR`, **not** ENV-BYPASS. It requires its own
change to `harness_pipeline.sh` (harness-infra, pipeline-exempt) and a clean re-run to validate.

## State left behind

- HEAD: `fffd7bfc` (unchanged). Working tree: fresh Gather code on disk, **uncommitted +
  unverified** (latest/best implementation, for a future F3-fixed re-run to regenerate or verify).
- `stash@{0}` = prior run's Gather copy (fallback). `stash@{1}` = old pre-reform WIP.
- No settlement regression observed (pipeline never reached the regression/Evaluator stage; the
  Step-0 mechanical gate — full `cargo test --workspace` — PASSED with 0 failures vs baseline 27).

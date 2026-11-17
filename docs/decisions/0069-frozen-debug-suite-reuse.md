# ADR 0069 — Reuse complete debug evidence for identical frozen inputs

## Context

Pre-gate and full gate repeat the same multi-hour workspace debug suite.
Historical logs contain nested subprocess results, so a sum of every libtest
summary cannot prove complete outer target coverage. Planner D-GATE-2 permits
one complete debug pass per frozen HEAD.

## Decision

Pre-gate step4 captures inputs before the original command and creates an atomic
receipt only after successful complete execution and unchanged inputs. Full
gate step4 verifies a supplied `GATE_DEBUG_RECEIPT` against the current frozen
commit/tree, clean worktree, lockfile, toolchain, command, profile/features,
configuration, data/fixtures and relevant hashed environment. An independent
manifest-derived inventory guards against missing targets. The original log
and per-target final results remain content-bound evidence.

Accepted reuse is explicitly `PASS_REUSED` with original receipt path, digest
and wall time. Missing, malformed, incomplete or mismatched evidence runs the
original complete debug suite. A partial pre-gate never authorizes publication;
release and all other full-gate steps retain their requirements.

## Alternatives considered

- Unconditional debug skip: rejects input provenance and complete coverage.
- Repeat debug unconditionally: retains hours of duplicate work for frozen inputs.
- Change optimization or remove release checks: outside this pass's scope.

## Consequences

This retains one complete debug result, but does not provide a second independent
run for flaky-test detection. Strict bindings may conservatively rerun debug.
Historical receipts are not manufactured and a running gate is never altered.
Python tool tests are the validation for this branch while the machine lock is
held; real gate admission remains a later integration check.

## Date

2026-11-16.

## Source

`docs/v8/c1/C1_DECISIONS.md` D-GATE-2;
`tools/v8/gate.sh`; `tools/v8/gate_test_evidence.py`.

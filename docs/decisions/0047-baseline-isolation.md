# ADR 0047 — Isolate the frozen baseline after the colony switch

Status: production switch committed as `fbc9a9babc9bb5df1d06ca1e655901e480521f66` (p6). Baseline isolation is committed as p7 `be12050c951dfb025f3440d34b657eec0663b817`: pure move verified; live old-shadow retirement and dead colony-gate/seam removal implemented. Systems/feature/formatting/strict-Clippy and source/registration isolation checks pass. Post-isolation identity capture PASS (D20); final evaluation is RUNNING; full-goal acceptance and publication remain pending. The optional standalone crate is not adopted.

## Context

ADR 0014 requires historical baseline identity. M-CORE authorizes a new colony decision policy, not a rewrite of the research world. Before the explicit switch commit, colony OFF and steward-ON fixtures must also retain identity. Moving the old selector too early or removing its gates while it still serves colony construction would violate that contract.

## Decision

Maintain an explicit world registration table in `sim-engine/src/registration.rs`, checked against real constructors. The working-tree table now assigns `AgentDecisionSystem` to Baseline and `ColonyMindSystem` to Colony. Fresh generation and production save loading use requested/decoded rules; the live old-shadow factory is retired in p7. Frozen old-runner source and binary manifests remain external for reproducibility. Post-isolation identity remains a separate check.

The p6 switch registers `ColonyMindSystem` for colony and the legacy selector for baseline at the existing priority/cadence. P7 moves the legacy selector to `sim-systems/src/runtime/baseline/decision/`, then separately removes dead colony gates and `jobs/seam.rs`. The initial two-file pure move has exact hash proof in `/Users/rexxa/work/core-data/baseline-dev/pure-move-proof.txt`; that proof does not cover subsequent cleanup. Source import-boundary and registration checks pass; post-isolation identity capture passes for the supported corpus (D20). Preservation covers supported baseline constructors and the saved baseline corpus, not arbitrary injected colony Job/draft/flee states.

Shared mechanical helpers live in `runtime/agent_actions/`: consumption, needs/arrival, forage and wood targeting, memory bias and routine lifecycle. Public mechanical API reexports continue to refer to neutral implementations. Sharing a helper does not authorize a colony import of the baseline selector, a call to legacy policy from facts, or changes to legacy random draws. The `baseline_isolation` gate must reject colony imports of `runtime::baseline::*` and verify world registration. Save/reload must preserve each world's actual system set and baseline fixtures.

A separate `sim-baseline` crate is dropped from this milestone under the goal's optional drop order. The moved selector imports `pub(crate)` neutral `agent_actions` modules plus settlement food/firewood and chronicle helpers; `sim-systems` also owns world registration. Moving only the selector into a crate that depends back on these helpers while `sim-systems` registers it would introduce a circular package dependency. Avoiding that would require another helper/registrar split and visibility changes beyond this isolation boundary. Keep module isolation and its gate instead; no runtime or compile-time speed benefit is claimed.

## Consequences

Baseline/research tags remain intact. Every pre-switch commit requires baseline plus colony OFF/ON identity; the switch alone changes colony behavior and records old/new digests. Final isolation requires baseline identity after the move and cleanup, current constructor/registration agreement and an import-boundary check. None is inferred from this ADR or from compilation alone. Held-out evaluation, shadow results and timing stay in their dedicated M-CORE records.

## Alternatives considered

Rejected alternatives: deleting the baseline to avoid maintenance (violates identity); retaining a hidden old-policy call beneath colony facts (defeats separation); deleting the seam before switching real constructors (changes the live pre-switch path).

## Source

[CORE_GOAL §§2–5](../v8/core/CORE_GOAL.md), [colony mind](0046-colony-mind.md), [authoring](../v8/core/MIND_AUTHORING.md), `sim-engine/src/registration.rs`, `sim-systems/src/runtime/{baseline/decision,agent_actions,mind}/` (the original `runtime/decision` path is historical).

## Date

2026-10-26.

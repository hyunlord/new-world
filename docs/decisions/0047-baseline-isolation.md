# ADR 0047 — Isolate the frozen baseline after the colony switch

Status: accepted staging boundary; neutral helper extraction and world-registration inventory exist. The production switch, final baseline directory move and deletion of the obsolete colony seam are pending. No standalone `sim-baseline` crate or completed isolation gate is claimed.

## Context

ADR 0014 requires historical baseline identity. M-CORE authorizes a new colony decision policy, not a rewrite of the research world. Before the explicit switch commit, colony OFF and steward-ON fixtures must also retain identity. Moving the old selector too early or removing its gates while it still serves colony construction would violate that contract.

## Decision

Maintain an explicit world registration table in `sim-engine/src/registration.rs`, checked against real constructors. At this ADR's preparation, `AgentDecisionSystem` is still registered for Both; `ColonyMindSystem` is implemented but not the production replacement. The table is a current-state artifact, not a declaration that the switch happened.

After shadow acceptance and the authorized switch, register `ColonyMindSystem` for colony and the legacy selector for baseline at the existing priority/cadence. Move the legacy selector and baseline-only helpers to `sim-systems/src/runtime/baseline/decision/` by a separately reviewable pure move. Then remove only colony gates proven dead under baseline-only registration and the unused `jobs/seam.rs`, retaining baseline identity. The move, dead-gate removal and their evidence must remain distinguishable; source simplification is not proof of identity.

Shared mechanical helpers live in `runtime/agent_actions/`: consumption, needs/arrival, forage and wood targeting, memory bias and routine lifecycle. Existing legacy API paths re-export these neutral implementations. Sharing a helper does not authorize a colony import of the baseline selector, a call to legacy policy from facts, or changes to legacy random draws. The `baseline_isolation` gate must reject colony imports of `runtime::baseline::*` and verify world registration. Save/reload must preserve each world's actual system set and baseline fixtures.

A separate `sim-baseline` crate is optional under the goal's drop order. Do not force a crate split through dependency cycles merely to claim separation. If deferred, record that limitation while retaining module-level isolation and its gate.

## Consequences

Baseline/research tags remain intact. Every pre-switch commit requires baseline plus colony OFF/ON identity; the switch alone changes colony behavior and records old/new digests. Final isolation requires baseline identity after the move and cleanup, current constructor/registration agreement and an import-boundary check. None is inferred from this ADR or from compilation alone. Held-out evaluation, shadow results and timing stay in their dedicated M-CORE records.

## Alternatives considered

Rejected alternatives: deleting the baseline to avoid maintenance (violates identity); retaining a hidden old-policy call beneath colony facts (defeats separation); deleting the seam before switching real constructors (changes the live pre-switch path).

## Source

[CORE_GOAL §§2–5](../v8/core/CORE_GOAL.md), [colony mind](0046-colony-mind.md), [authoring](../v8/core/MIND_AUTHORING.md), `sim-engine/src/registration.rs`, `sim-systems/src/runtime/{decision,agent_actions,mind}/`.

## Date

2026-10-26.

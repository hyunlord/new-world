# ADR 0046 — Give the colony a data-defined mind

Status: design accepted for M-CORE; compiler, providers, lifecycle effects and recorder implemented in the working tree. Round-4 shadow/calibration checks passed; the working-tree production registrar now selects the mind for colony and the legacy selector for baseline (D19). Switch commit/captures, final baseline isolation, held-out evaluation and performance acceptance remain pending. This ADR records architecture, not a passing gate.

## Context

The legacy decision cascade shares mutation ordering across needs, jobs, gathering and routine replay. Adding future mind, society and body policy as more branches would deepen that coupling. ADR 0039's refusal to split that cascade remains valid for the frozen baseline; it does not require a new colony policy to inherit the cascade. ADR 0033 separates content data from executable behaviour.

## Decision

Compile `sim-core/data/colony/mind.ron` once against registered fact and behaviour names. Schema 1 has six ordered bands: critical, duty, need, work, leisure, idle. The first eligible band wins. Within it, checked i64 scores sum a constant and integer fact × weight-permille terms; ties follow authored order. The running node's hysteresis margin resists switches within its band. Inactive node cadence is authored; running eligibility is checked each tick. A node name is an identity and random domain, never a special-case policy dispatch key.

`sim-core/src/colony/mind/` owns the pure compiler, columnar facts and selector. `sim-systems/src/runtime/mind/` binds factual providers and lifecycle callbacks. Providers observe state and expose units; they do not choose actions. The coordinator processes AgentId order, resolves registered targets, dispatches start/continue/finish, refreshes invalidated factual domains, handles routine replay, then applies deferred target and claim effects in their required order. `runtime/agent_actions/` contains neutral mechanics shared with the legacy path. Jobs still assign and execute work; flight and drafting retain their existing owners. The mind chooses whether an owner may continue, without copying those systems into the selector.

New behaviour effects must preserve arrival/consumption boundaries, social handshake snapshots, reservation release and dynamic claim ordering. Errors such as a stale target or unprepared factual context are explicit; a target-bound rest cannot pretend that Idle plus a SeekTarget moves a person. Hearth rest uses the registered Activity-backed SitByFire lifecycle and a real reachable hearth.

Random selection context is keyed by seed, agent, tick and node domain, never mutable RNG consumption or hash iteration. The new routine path's keyed duration is an explicit replacement; the legacy routine draw remains unchanged. Seam yield thresholds and continuation conditions belong in authored predicates/scores, with each replacement traced to [CORE_INVENTORY](../v8/core/CORE_INVENTORY.md) and [CORE_DECISIONS](../v8/core/CORE_DECISIONS.md).

A semantic choice record retains its dictionary, selected node/band/score, up to three raw candidates from the winning band and actual fact witnesses. Hysteresis can retain a choice outside the raw top three. Fact witnesses can include rejected alternatives; presentation must not silently treat every witnessed input as the winning reason. Per-person history is bounded to the latest 16 choices. Causal links use actual recorded origins, not nearest-event guesses; successful meal consumption can bind subsequent hunger provenance to its initiating choice. UI reads records and never invokes policy evaluation.

## Alternatives considered

- Splitting the legacy cascade into a new chain of helpers would preserve its policy coupling and jeopardize shared mutation ordering; retain it for baseline instead.
- A generic scripting interpreter for effects would duplicate typed mechanics and weaken ownership; effects remain registered Rust code.
- A weighted score across all bands could let leisure outrank safety; use strict band precedence.
- Reusing the old chooser as a fact provider would hide policy in observations; prohibit it.

## Consequences

Source-level implementation does not prove shadow agreement, lifecycle equivalence, story yield, save compatibility, why coverage or performance. The goal requires shadow comparison before the one colony switch, frozen-policy held-outs afterward, baseline identity throughout and a full gate. See [authoring](../v8/core/MIND_AUTHORING.md), [goal](../v8/core/CORE_GOAL.md) and [ADR 0047](0047-baseline-isolation.md).

## Date

2026-10-26.

## Source

- [CORE_GOAL](../v8/core/CORE_GOAL.md), [authoring guide](../v8/core/MIND_AUTHORING.md).
- `rust/crates/sim-core/src/colony/mind/`, `rust/crates/sim-core/data/colony/mind.ron`.
- `rust/crates/sim-systems/src/runtime/mind/`, `rust/crates/sim-systems/src/runtime/agent_actions/`.

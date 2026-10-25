# ADR 0045 — Defend through player-respecting steward orders

Status: implemented for M-UX2; calibration, held-out acceptance and the full gate remain pending.

## Context

The M-UX playtest and evaluation exposed combat deaths while the steward never drafted defenders, unsafe outdoor flight and fuel hauling starved by equal-priority work. The owner authorized automatic defence with player precedence. This amends ADR 0036's no-drafting policy; the goal's reference to ADR 0035 is a numbering error (0035 concerns terrain). Existing ordinary command validation, deterministic ordering and physical inventories remain authoritative.

## Decision

Add a data-defined Defence action, currently rule 7 (`MUX_STEWARD_RULE_7`), independent of the unchanged All/Housing/Food scope codes. Its toggle defaults ON, but requires the overall steward to be enabled. The existing slow planner stays at priority 125 every 360 ticks. `StewardDefenceSystem` runs at priority 122 every tick, before civilian flight at 123.

A new approach response recruits at most four eligible weapon-capable adults, with HP at least 50 and no children. Available weapons are assigned by deterministic, capacity-aware matching; one stock weapon cannot support several candidates and manual claims remain fixed. Rank the complete proposed team by actual weapon draft rank (the authored spear-over-axe policy), health, camp distance and stable ID. Issue ordinary draft/attack commands against the nearest threatening animal. Content identities stay in registry data, not new content enums.

A valid explicit player draft, undraft or draft order protects that person's choice for one day, including a no-op reaffirmation and intent while the steward is OFF. Player ownership takes precedence; automatic release touches only steward-owned defenders. Recruit once per approach response, so a critical-need self-undraft can reach food without immediate refill. Existing defenders may update targets. A new pack, or a retired pack re-entering the camp radius, permits a new response. Release owned defenders immediately on retreat or disabled assistance; otherwise wait 60 consecutive ticks with no threatening wolf within 24 camp tiles. Chronicle kind 46, `StewardDefence`, records the actual newly accepted batch count; kind 45 retains its existing monotonic action sequence semantics.

Fast source envelopes export and replay with phase `before_flight`; ordinary steward actions retain `after_assignment`. Playback suppresses planning and applies recorded ordinary commands at the same phase. Saved defence ownership, manual expiry, response and quiet-window state affect subsequent policy and fingerprints. Save format 11 extends that state; `save/legacy_steward.rs` freezes the exact formats 7–10 steward wire layout, preserving earlier migrations and defaulting legacy defence state. Positional bincode layouts are not repaired by serde defaults.

Civilian flight compares actual reachable navigation paths and person/wolf travel speeds to choose the nearest roof it can reach first. Re-evaluate every 30 ticks; otherwise use a reachable hearth guarded by defenders, then flee away. A pending wolf bite must also check roof immunity when resolved, not only when its target was chosen.

Fuel-only hauling receives a round at each worker's own Haul priority, before other jobs at that same priority. Higher explicit priorities and disabled Haul remain authoritative; stockpile acceptance, space, reachability and reservations still apply. M1 assignment remains unchanged. This addresses equal-priority starvation without introducing free fuel, forced delivery or a global priority override.

Exact fatal-animal records (`FatalAnimalAttack`) are transient observer evidence: `SimResources.fatal_animal_attacks` is serde-skipped and cleared at tick start. It does not become saved policy or infer causation from proximity. The opt-in `mux2_native_timings` bridge method is a read-only performance diagnostic, outside the simulation policy API.

## Alternatives considered

- Keep the steward's no-draft rule: rejected because it contradicts the owner's defence decision.
- Refill all four defenders every tick: rejected because ordinary critical-need undrafting would immediately be undone.
- Rank available weapons independently for each candidate: rejected because it can reuse scarce stock or retain a worse final team despite a feasible better matching.
- Release every drafted person: rejected because manual player ownership must survive automatic policy changes.
- Move every hauling job above all priorities: rejected because it overrides explicit work preferences and unnecessarily changes M1.
- Append fields to historical steward decoders: rejected because positional legacy saves require their original wire layout.

## Consequences

Defence, flight and fuel scheduling intentionally change colony behavior. Scoped regression results do not establish survival, visibility, frame-time or full-gate acceptance. Calibration remains restricted to seeds 0–7 (at most four logged rounds); policy and provenance freeze before paired defence ON/OFF held-out seeds 1200–1231. The raw evidence, including failed attempts, stays outside git under `/Users/rexxa/work/mux2-data`. UI/renderer implementation and native timing instrumentation do not waive the original acceptance bars. Publication remains pending.

## Date

2026-10-26.

## Source

- [M-UX2 goal](../v8/mux2/MUX2_GOAL.md), [decisions D5–D7](../v8/mux2/MUX2_DECISIONS.md), [ADR 0036](0036-steward.md).
- `rust/crates/sim-systems/src/runtime/steward/{defence,arming,lifecycle}.rs`, `runtime/wildlife/{flee,fight}.rs`, `runtime/jobs/assign.rs`, `save/legacy_steward.rs`.
- `rust/crates/sim-test/tests/{steward_defence,steward_defence_fingerprint,steward_defence_migration,colony_flee_mux2,mux2_firewood}.rs`; `rust/crates/sim-test/src/probe/mux2.rs`.

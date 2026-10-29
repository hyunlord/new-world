# ADR 0054 — Preserve actual food-operation evidence across saves

## Date

2026-10-29. Status: accepted for M6a implementation; final verification pending.
Authority: M6A_GOAL §2.7 and the P8 historical-evidence decision in M6A_DECISIONS.
Extends ADR 0052's evidence model and supersedes its current-write format13 decision;
its physical ownership rules and historical migration boundaries remain intact.

## Context

A death-time snapshot of ripe plants does not explain why earlier acquisition
failed. First cooked intake and first preserved storage also need actual operation
parents rather than uncaused milestone labels. Existing UNKNOWN captures remain
honest historical evidence; new code cannot reconstruct their missing past.

## Decision

Record observed acquisition admission/query, completion and cancellation outcomes
at their real boundaries, without new food decisions or per-person plant scans.
Retain one active episode per affected person, its last timestamped receipt and
colony-wide food-flow counters. Successful intake or harvest retires the episode;
death copies the actual receipt into an immutable causal parent. The flow interval
is explicitly colony-wide, not proof that one person caused every missing unit.
Daily flow history retains at most four completed intervals.

Append FoodOperation53 after existing Chronicle kinds. Actual first cooked intake
and first preserved placement receive operation parents only after successful
physical changes. Historical owner identity remains valid after its stack or
container disappears. A merged stack's oldest recorded day is not individual
ingredient genealogy. Operation payloads use append-only Chronicle indices and
grow with emitted terminal events, not with every failed attempt. The owner feed
hides this internal kind through UI data; Why remains able to traverse it.

Append FoodEvidence after the twelve existing FoodState fields. Format14 writes
the new tail. An exact LegacyBodyV13/LegacyFoodStateV13 decoder moves all prior
physical state and system slots unchanged and initializes evidence empty. Format13
must not run <=12 abstract food conversion or insert another FoodDailySystem.
Preserve authentic old13 fixtures; never generate replacements to make migration
pass. Validate historical event-day freshness, identity consistency, chronology and
operation/payload coverage without demanding a matching live owner today.

The authored policy, food thresholds, selection cadence and physical transfer
rules are unchanged by this evidence pass. Chronicle insertion shifts ordinals,
causal references and story-yield counts; serialized evidence and full-state
fingerprints intentionally change. Do not claim byte equality with P7 or normalize
away these fields. Compare physical outcomes, food flows and meals separately;
audit old-event referents through an explicit ordinal mapping.

## Alternatives considered

Infer a past reason from current priorities or ripe counts: cannot establish what
happened then. Reuse format13 with a changed layout: silently breaks retained
fixtures. Remove evidence from fingerprints: hides state differences. Expose every
operation in the owner feed: adds internal records instead of a readable chronicle.

## Consequences

Persisted history adds save bytes and append-only Chronicle records. Old saves keep
missing history explicit; current facts cannot recover unrecorded past operations.

### Verification boundary

Uncommitted P8 source passed497 sim-systems tests (one ignored) and the separately
run authentic format13 fixture test. This is not a clean-head full gate. P8 actual
captures, fresh performance, calibration4, freeze, held-out and full gate remain
separate pending requirements. P7 performance PASS and calibration2/3 equality do
not certify the new evidence/schema source.

## Source

- `rust/crates/sim-core/src/colony/food_evidence.rs`
- `rust/crates/sim-systems/src/runtime/food/evidence.rs`
- `rust/crates/sim-systems/src/runtime/food/evidence_validation.rs`
- `rust/crates/sim-systems/src/save/legacy_food_v13.rs`
- `rust/crates/sim-systems/src/save/evidence_tests.rs`
- `docs/v8/m6a/M6A_DECISIONS.md`

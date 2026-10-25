# ADR 0039 — Preserve execution while separating existing collector and consumption domains

## Context

The M-UX review found snapshot.rs at1481 lines and decision/system.rs at3494 lines. Existing domain modules already separate targeting, routines and settlement reads. A wholesale decision framework would move broad shared state and ordering without a demonstrated behavioral benefit.

## Decision

Move the cohesive resource snapshot collector, marshallers and existing depletion tests to ffi/snapshot/resources.rs. The existing snapshot facade re-exports the four public symbols; the Godot marshaller retains ffi visibility. Resource order, sorting, clamping and the distinct depleted-source policies remain unchanged.

Extract only Food, Water and Sleep consume bodies to runtime/decision/consume.rs. Concrete borrowed inputs retain the current ownership boundary; the eight-input sleep helper has a narrow argument-count allowance instead of introducing a mutable context abstraction. The caller retains per-agent continue semantics, and helper returns represent the former strict-food and nighttime-sleep exits. Food version changes, finite/infinite handling, all tallies and exact need arithmetic remain unchanged. Keep the public-system tests in system.rs and protect unavailable urgent needs at night before moving code.

## Alternatives considered

- Split the entire decision cascade: rejected because GatherFood, Idle and routine replay share wider state and mutation order.
- Add a generic mutable context: rejected because the three actions already have concrete bounded inputs.
- Consolidate resource loops: rejected because depletion and registration policies differ by resource.
- Claim a performance improvement from fewer lines: rejected; this move has no such acceptance claim.

## Consequences

The facades are smaller but total source lines can increase from explicit module wiring and tests. Public API and save format remain unchanged. Every commit requires the original identity, nine literal OFF fixtures and frozen Stage A ON summary/command equivalence, in addition to compiler/lint and focused runtime checks. Existing large-module debt is reduced only by the stated extracted domains.

## Date

2026-10-25.

## Source

`docs/v8/mux/MUX_GOAL.md` B6; `docs/v8/mux/MUX_DECISIONS.md` D38; `docs/v8/review/CODE_REVIEW_2026-10.md`.

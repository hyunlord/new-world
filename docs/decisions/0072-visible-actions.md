# ADR 0072 — Project visible actions from the owners of physical state

## Context

The 3D view needs to distinguish travelling to a tool, working at a target,
carrying a load and interacting with another resident. Some transfers complete
within a single tick, before a snapshot could show an intermediate state.

## Decision

Project AgentId-ordered rows on demand from the completed tick. Project job
poses only when the FSM, reservation and live target still agree. Read the
physical inventory, dedicated interaction records and assigned bed for props,
partners and sleep anchors. Use the observed arrival and end ticks when the
owning action provides them.

Record a bounded recent sequence at successful item transfer boundaries. Clear
it on reset and load. Export the snapshot and cues as packed read APIs and use
SimReads invalidation for paused commands and world changes.

Resolve clip keys and fallbacks from a validated presentation catalog. Recipe
selectors use workshop tags, output categories and tool capabilities. The
projection and cues have no saved state and consume no RNG draws.

## Alternatives considered

The existing generic agent snapshot lacks job phases and physical targets.
A system that persists presentation state would add per-tick work and a
second owner for action lifecycles.

## Consequences

The view receives an explicit fallback for every action. If the consumer
falls behind the cue buffer, it can recover from the current sustained pose.
No saved fields or event IDs are added.

## Verification

The activity harnesses cover the stable job and routine vocabulary, live job
phases, partner and bed ownership, and command replay with reads enabled and
disabled. The real scene exercises the bridge and cache lifecycle.

## Date

2026-11-20

## Source

Owner decision for ACTION1 and the SYS3D audit, sections A1 and A2.

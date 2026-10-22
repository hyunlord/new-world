# ADR 0025 — Knowledge channels, forgetting and discovery

Status: proposed (M4 Part 0 draft, 2026-10-23; final in Part 6).

## Context
Knowledge must move between people at different fidelity and speed, fade when unused, and occasionally appear new.
VISION §4 lists six channels; M4 builds the first stage.

## Decision
- Oral (`OralTransmissionSystem`, once a day at the evening gathering): keyed pairs; a listener below threshold gains
  the knowledge at 350 ‰ with a keyed 50 % chance (+10 points with a storytelling teller), at most one per pair per
  evening; talk alone makes a practitioner only of knowledge with a threshold ≤ 300 ‰.
- Observation (`ObservationSystem`): working near a practitioner at gated work adds a little, capped below threshold.
- Apprenticeship (`OrderTeach`, paired Teach/Learn jobs): 60-tick sessions, ≤ 4 a day, +15 ‰ × fidelity.
- Forgetting (`KnowledgeDecaySystem`): after 6 days unused, a per-source rate per hour down to a per-source floor.
- Discovery (`DiscoverySystem`): curious adults (novelty seeking ≥ 600 ‰) holding every prerequisite may discover a
  node whose settlement conditions hold, with a keyed chance by difficulty.
- Every draw is keyed on `STREAM_KNOWLEDGE` (subject = lineage key; slot = knowledge id or pair index).

## Alternatives considered
- Continuous diffusion every tick: costly (C3) and hard to read; a daily evening matches the hearth gathering.
- No forgetting: then nothing can be lost except by death, and the at-risk state would be rare.

## Consequences
- Knowledge spreads mostly at the hearth, so the hall and the evening gathering matter.
- Every rate is a calibration candidate recorded in the DECISIONS file.

## Date
2026-10-23

## Source
docs/v8/m4/M4_GOAL.md §3.3; docs/VISION.md §4; docs/v8/m4/M4_DECISIONS.md

# ADR 0025 — Knowledge channels, forgetting and discovery

Status: accepted (M4, 2026-10-23).

## Context
Knowledge must move between people at different fidelity and speed, fade when unused, and occasionally appear new.
VISION §4 lists six channels; M4 builds the first stage: talk, watching, teaching, finding.

## Decision
- Oral (`OralTransmissionSystem` 160, once a day at 20:30 = the evening gathering): adults (≥ 10 years) within 16
  tiles of the camp hearth gather. Every pair rehearses what both hold (telling counts as use). For each listener, each
  node a teller practises and the listener lacks is a candidate, rolled in a keyed order with a keyed chance of 500 ‰
  (+100 with a storytelling teller); the first that takes is learned at 350 ‰, at most one per listener per evening.
  Talk alone makes a practitioner only of knowledge with a threshold ≤ 350 ‰ (DECISIONS D14).
- Observation (`ObservationSystem` 161, every 30 ticks): a person within 3 tiles of a practitioner at gated work gains
  5 ‰ of that node, capped below its threshold.
- Apprenticeship (`OrderTeach`; paired `Teach`/`Learn` jobs, `runtime/jobs/teach.rs`): 60-tick sessions, at most 4 a
  day, +15 ‰ × fidelity (950 ‰ when the teacher practises instruction, 700 ‰ without, 1,100 ‰ from a master at
  800 ‰). Either partner who is free walks to the meeting tile and waits up to 240 ticks for the other (D8 addendum);
  the order closes when the learner practises the node, or is cancelled when either partner dies or the command says.
- Forgetting (`KnowledgeDecaySystem` 162, every 60 ticks): after 6 days unused an entry loses ‰ per hour by source
  (seeded 1, discovered 1, oral 1, observed 2, apprenticed 1) down to a floor (seeded and discovered 100, others 0);
  an entry at 0 is removed.
- Discovery (`DiscoverySystem` 163, every 240 ticks): adults with novelty seeking ≥ 600 ‰, or among the most curious
  fifth (at least one; D19), holding every prerequisite, may discover a node whose settlement conditions hold, with a
  keyed chance of 1 in 40 / 90 / 200 by difficulty; the finder starts at 500 ‰.
- Every draw is keyed on `STREAM_KNOWLEDGE` (subject = lineage key; tick; slot = node id or a pair index), so a
  skipped draw changes nothing else.

## Alternatives considered
- Continuous diffusion every tick: costly (C3) and hard to read; a daily evening matches the hearth gathering.
- No forgetting: then nothing can be lost except by death, and the at-risk state would be rare.
- Two-sided lessons (both partners free in the same tick): measured up to 32 days per order in a busy colony.

## Consequences
- Knowledge spreads mostly at the hearth, so the gathering radius and the evening matter (calibration rounds 3–5).
- Every rate is a calibration value recorded in DECISIONS D14, D18, D19; the held-out evaluation (seeds 700–731)
  passed every channel bar (F2 31/32, F3 32/32, F4 32/32, F5 31/32, F6 32/32, F7 24/32).

## Date
2026-10-23

## Source
docs/v8/m4/M4_GOAL.md §3.3; docs/VISION.md §4; docs/v8/m4/M4_DECISIONS.md D8, D13, D14, D18, D19

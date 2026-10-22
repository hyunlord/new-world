# ADR 0026 — The settlement's technology aggregate and its era

Status: proposed (M4 Part 0 draft, 2026-10-23; final in Part 6).

## Context
The settlement level must observe what its people know, never declare it: VISION §4's era is a label computed from
coverage of marker technologies, and it can go down.

## Decision
- `TechAggregateSystem` (every 240 ticks) writes `SimResources.tech`: per node the practitioners (adults at threshold
  who used it in the last 6 days, or got it in the last 6 days), the holders (at threshold), a state (unknown, known —
  stable at ≥ 3 practitioners —, at risk, lost) and up to five practitioner names.
- Transitions log `KnowledgeAtRisk` and `KnowledgeLost` in the chronicle; the HUD shows the at-risk count.
- Era progress ‰ = Σ weight · min(1000, 1000 · practitioners / adults) / Σ weight over the Neolithic markers, the absent
  ones (farming, weaving) counted in the denominator; labels below 250 구석기, to 749 신석기 전환기, from 750 신석기.

## Alternatives considered
- A research tree with a button: contradicts VISION §4 (the era is observed, not chosen).
- Holders instead of practitioners for coverage: hides knowledge that nobody practises.

## Consequences
- Progress can fall when practitioners stop working or die; M4's ceiling is 64 % by construction.

## Date
2026-10-23

## Source
docs/v8/m4/M4_GOAL.md §3.4; docs/VISION.md §4; docs/v8/m4/M4_DECISIONS.md

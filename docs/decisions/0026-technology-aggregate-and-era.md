# ADR 0026 — The settlement's technology aggregate and its era

Status: accepted (M4, 2026-10-23).

## Context
The settlement level must observe what its people know, never declare it: VISION §4's era is a label computed from
coverage of marker technologies, and it can go down.

## Decision
- `TechAggregateSystem` (164, every 240 ticks) writes `ColonyState.tech`: per node the holders (anyone at threshold),
  the practitioners (adults at threshold who used it in the last 6 days), a state and up to five names. States: unknown
  (never held), at risk (exactly one holder; DECISIONS D7), known (two or more), stable (≥ 3 practitioners), lost (held
  once, no holder now).
- Transitions log `KnowledgeAtRisk` (known or stable → at risk) and `KnowledgeLost` (→ lost, with the last named
  holder) in the chronicle; the HUD shows the at-risk count, the T map shows every state.
- Era progress ‰ = Σ weight · min(1000, 1000 · practitioners / adults) / Σ weight over the Neolithic markers, the absent
  ones (farming 3, weaving 2) counted in the denominator of 14; labels below 250 구석기, to 749 신석기 전환기, from
  750 신석기. A fresh colony starts at 28 ‰ (two founders know timber walls; D12).

## Alternatives considered
- A research tree with a button: contradicts VISION §4 (the era is observed, not chosen).
- Holders instead of practitioners for coverage: hides knowledge that nobody practises.

## Consequences
- Progress can fall when practitioners stop working or die; M4's ceiling is (3 + 3 + 2 + 1) × 1000 / 14 = 642 ‰ by
  construction (held-out: 0 of 32 seeds above it).

## Date
2026-10-23

## Source
docs/v8/m4/M4_GOAL.md §3.4; docs/VISION.md §4; docs/v8/m4/M4_DECISIONS.md D7, D12

## Amendment M-SIMCHECK — the era steps down only after a full practice window (2026-11-15)

Progress still falls as above, but the era was a memoryless threshold on it, and seasonal work-mix
noise moved progress across a transition line again and again: in the M-SIMCHECK r1 runs (steward
ON, 48 days) 13/16 seeds logged era regressions, 47 down steps among 103 `era_changed` rows, some
within 240 ticks, with no death or knowledge loss behind them. VISION §4 and CONTENT_BIBLE §0.5
regress the era when a marker skill is lost, not when it pauses. The era now steps up on live
progress but steps down only as far as the same progress counted over two `practice_days` windows
allows, so a lapse must last a whole window before the era falls; a death or forgotten knowledge
(no holder) lowers both measures at once. No saved field or data tunable; nothing in the simulation
reads the era (chronicle, HUD and bridge only). Seed 0: 13 → 3 era rows. Commit e7c80ccf5; test
`spine_era_steps_down_only_after_a_full_practice_window_below`.

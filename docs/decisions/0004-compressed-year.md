# ADR 0004 — Compressed year: a 2-minute day, a 24-day year

## Context
The core fun (apprenticeship, the last bearer of a skill dying, children learning, eras moving with the spread of knowledge) needs generations to be visible within a 20–40-hour game, while a single day must still be watchable. RimWorld (day ≈ 16 min, 60-day year) and Dwarf Fortress (336-day year, 1–2 h per year) were measured.

## Decision
Option 3: a day is about 2 minutes at 1×; a year is 4 seasons × 6 days = 24 days (48 min per year at 1×, 16 min at 3×); a child becomes adult in about 15 years.

## Alternatives considered
Option 1 RimWorld-like (≈10-minute day, 60-day year: generations practically never turn over); option 2 DF-like (≈20-second day, 336-day year: routines become background).

**Tick definition (M1, 2026-10-20):** one tick stays one simulated minute (1,440 ticks per day). The live game feeds **12 ticks per real second at 1×**, so a day lasts 120 s; the speeds 1×/2×/3×/4× feed 12/24/36/48 ticks per second (a day in 120/60/40/30 s), at most 8 ticks per frame (a larger backlog is dropped). This changes no simulation state: only how much wall time the bridge feeds the fixed-step loop (`rust/crates/sim-bridge/src/ffi/node.rs`, `TICKS_PER_SECOND_AT_1X`, `MAX_TICKS_PER_FRAME`; `seconds_per_day`).

**Still open — the calendar half (milestone M1.5):** 6-day seasons, a 24-day year, ages counted in years, and aging and fertility recalibrated to years. Until M1.5 the sim keeps the V7 calendar (7-day seasons, a 28-day year, ages in days).

## Consequences
One apprenticeship lasts a season to a year; a person can train 3–5 apprentices in a lifetime. Seasons feel fast: one farming cycle is ~10 days (~20 minutes). With the M1 tick definition a V7-calendar year (28 days) lasts 56 min at 1×; the M1.5 calendar makes it 24 days (48 min). The identity world (`baseline`) keeps its tick-level behaviour: pacing never enters the simulation.

## Date
2026-10-20 (decided); 2026-10-20 (M1: tick definition)

## Source
docs/VISION.md §1, §10.1; docs/v8/m1/M1_GOAL.md 8.2; docs/v8/m1/M1_DECISIONS.md D22

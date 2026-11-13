# ADR 0016 — Measure wardrobe candidates against the shipped rig

## Context
B1 figures were thinner and paler than the shipped people and changed work
and carry poses. ART-BATCH round3 R1 requires explicit ±10% style comparisons.

## Decision
Edit matching shipped complete figure sheets, retain TEMPLATE_V3 structural
normalization and compare native-size shoulder/hip widths, clothing HSV and
outline thickness using reviewed reference annotations. All ten frames and
hash-bound visual review must pass before any sheet is promoted. Keep failures
and raw measurements outside the repository. Never import automatically.

## Alternatives considered
- Reuse B1 on its visual scores: rejected because in-game comparison disproved parity.
- Recolour or stretch B1 pixels: rejected because it cannot restore shipped poses.
- Adopt individual good panels: rejected because it breaks whole-sheet consistency.

## Consequences
Yield can be lower than the84look inventory. Explicit gaps are preferable to
claiming a style match from canvas size or visual opinion alone. Portrait and
import coverage must report these gaps rather than hide them behind fallbacks.

## Date
2026-11-14

## Source
`art/batch_r1/STYLE_CONTRACT.md`; external ART-BATCH BRIEF3 R1 and
`/Users/rexxa/work/art-import-data/REPORT.md` Part3.

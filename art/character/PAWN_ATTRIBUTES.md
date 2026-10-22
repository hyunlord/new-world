# Pawn attribute mapping (TRIAL4; no simulation wiring)

`PawnLook` is the immutable presentation record in `art/pipeline4/pawn_gen.py`.
The mapping below is a proposed M-ART2 adapter contract, not implemented bridge
reads or a claim that every cosmetic field already exists in the simulation.

| Pawn field | Values | Entity source / future adapter |
|---|---|---|
| sex | M, F | Identity sex |
| age | child, adult, elder | Age band derived from simulation age policy |
| skin_tone | 0–3 | Body/skin cosmetic index, persistent keyed assignment if absent |
| hair_style | none, short, long_tied, bun, braid, grey_thin | persistent cosmetic trait; grey_thin default candidate for elder |
| hair_color | 0–3 | persistent cosmetic index; grey palette for elder is a presentation choice |
| top | none, hide_wrap, fur_cloak, woven_tunic | equipped torso apparel category |
| top_palette | 0–1 | household/cloth dye cosmetic palette |
| item | none, stone_axe, log_bundle, basket | visible equipped tool or carried goods |
| facing | S, E, N | renderer-facing movement direction; W flips E |
| state | idle, walk_a, walk_b, work_a, work_b, carry, sleep | animation phase derived from activity and render clock |

Cosmetic choices must not consume simulation RNG stream state. Missing traits,
age thresholds, equipment priority and household dye ownership remain M-ART2
adapter decisions; this trial changes no simulation or bridge code.

All base colors select exact `rgb` entries from `art/style05/palette_05.json`:
skin21/14/4/6, hair20/3/13/16, primary cloth12/4, secondary17/11,
wood6, stone23. The specified outline is #141210. Shading and multiplicative
grain derive colors continuously; the source palette is never quantized.

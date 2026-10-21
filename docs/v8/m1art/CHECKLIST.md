# M1-ART checklist — the mock-up's elements, before → target → after

Target: `docs/images/v8_target_view_b.png` (drawn by `docs/v8/m1/mockup_gen_reference.py`).
"Before" is judged on the Part 0 captures at `bba3110e`/`a4a7eaf4` (`/Users/rexxa/work/m1art-data/before/`, the
`--m1-set` of V8-PLAY, seed 20261010). "After" is filled in Part 4 from the after captures.

Values: **있음** (drawn as the mock-up draws it), **임시** (a placeholder stands in), **없음** (not drawn),
**M2+** (content M1 does not have).

| # | element | before | target (this pass) | after |
|---|---|---|---|---|
| 1 | ground texture clusters | 임시 — dark patches from the elevation byte, chosen per tile by hash, blocky | value-noise clusters of two grass families (`g`/`G`, ~10 apart in value), 6 variants | |
| 2 | worn earth | 임시 — a square of dirt around the hearth with a hashed fringe | clearing plus a noise-eroded edge, grass-dither edge tiles | |
| 3 | river banks | 없음 — one `shallow` tile next to land | 8 bank tiles (4 sides, 4 inner corners) with a sand lip, 4-bit autotile, deep/shallow water | |
| 4 | hill rock | 임시 — a flat olive tile with two arcs | 3 hill variants with rock speckle; 3 cracked mountain variants with a lit top edge | |
| 5 | canopy | 임시 — one 16×16 flat circle per forest tile (green dots) | 3 species × 3 sizes at 24×24, overlapping into a mass, y-ordered, jittered | |
| 6 | tree shadow | 없음 | 20×8 translucent shadow 2 px down-right of each canopy | |
| 7 | stump | 없음 | a stump on a felled tree's tile for one game day | |
| 8 | hut | 임시 — a small log frame with an empty floor | 48×48 log walls (two courses, end grain), south doorway, earth floor, 4 bedrolls, hearth stone | |
| 9 | shelter | 임시 — a box icon | 16×20 lean-to: two poles, hide roof, dark opening | |
| 10 | hearth | 있음 — a static stone ring with fire | stone ring with 2 fire frames (0.4 s) | |
| 11 | stockpile floor | 임시 — a flat yellow tint with a light edge | tamped-earth floor set (corner/edge/interior) with corner stakes and a darker 1-px border | |
| 12 | log piles ×3 | 임시 — 2–5 thin bars in a small box; 34 logs read as four small squares | 3 sizes (1–8, 9–16, 17–25) of stacked round logs with end grain filling the tile | |
| 13 | blueprint ghost + label | 임시 — a light-grey ghost, label text without a backing box | blueprint-blue ghost (alpha 80, dashed outline), delivered logs as piles on the footprint, label in a backing box, 3-px progress bar under it | |
| 14 | person silhouette | 임시 — 16-px stub without outline weight, no walk frames | outlined figure (head 6×5, tunic 8×6 with belt, legs), 2 walk frames, west flip | |
| 15 | person shadow | 있음 — 16×5 ellipse | 10×4 ellipse, alpha 110, under the feet | |
| 16 | tunic colours | 있음 — 6 household colours | the 6 colours, darkest ≥ 25 % lighter than forest floor (unit test) | |
| 17 | carry bundle | 임시 — two logs on the head | 3 outlined logs across the shoulders | |
| 18 | tool | 있음 — axe/hammer icon in the cell's top-right | 5×7 axe/hammer held at the right hand | |
| 19 | child size | 있음 — 12×12 | 12×12 with the same layers and 2 walk frames | |
| 20 | elder hair | 있음 — white | grey/white, kept | |
| 21 | sleep pose | 없음 — sleepers stand | 16×8 lying frame, head left, on the bed tile | |
| 22 | night light | 임시 — hearth glow only (off-centre by half a tile) | glow kept; people near the hearth or in the hut get a warm modulate at night; light outline halo at night | |
| 23 | person on a forest tile | 없음 — hidden under the canopy | canopy below people; 1-px light halo on a forest tile | |
| 24 | chop mark | 임시 — a yellow X over the tree | 7×7 yellow axe glyph at the tile's top-right, outlined | |
| 25 | map text backing | 임시 — the blueprint label floats on the map | dark backing box (`art_ui.gd` style) | |
| 26 | farm plot | 없음 | — (no farming in M1) | M2+ |
| 27 | cave, rock rooms, mined floor | 없음 | — (no digging in M1) | M2+ |
| 28 | huts with beds, jars, planks inside villages | 없음 | — (one hut type in M1; its bedrolls are row 8) | M2+ |
| 29 | side panel content (skills, knowledge, thoughts) | 없음 | — (the inspector shows M1's fields) | M2+ |
| 30 | workshops, stone/flint/berry/hide items | 없음 | — (logs are M1's only item) | M2+ |
| 31 | animals | 없음 | — | M2+ |

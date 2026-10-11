# Part 6 baseline diagnosis (Art Pass 5 evidence)

Source: `docs/art/production5/ingame_qa.json` A18's `l_d24_1200_F2`, corroborated by `shots/l_d24_1200_F2.entities.json.gz`. The existing opaque-mask measurement found 3 of 37 visible villagers above 50% occlusion.

| Agent | Activity / pose | Tile | Drawn feet | Seat | Declump | Opaque coverage |
|---|---|---|---|---|---|---|
| 25 | walk / walk | 20,21 | 782,377 | no | +2,0 | 0.6439790576 |
| 8 | idle / idle | 20,21 | 776,366 | yes | 0,0 | 0.5300429185 |
| 7 | walk / walk | 22,19 | 817,344 | no | +8,0 | 0.6177884615 |

`CampSeats.declump` explicitly excludes seated villagers. Agent 8 was therefore skipped. Its front-neighbor group includes agents 3, 0, 25 and 9. Other seated actors around it were also excluded. The seat grid has 16px horizontal / 18px vertical spacing, while manifest canvases can be 21px wide and 31px tall.

Agents 25 and 7 were not skipped: both had `moving=false` at the captured instant despite `activity=walk`. Their 2px and 8px shifts already appear in metadata. Therefore the three failures cannot honestly be described as three skipped idle villagers. The moving flag reflects interpolation between rendering frames, and a walking villager can briefly report false after reaching a target. Part 6 must preserve walking actors by activity as well as instantaneous moving state. Relief should come from moving eligible seated/idle foreground neighbors, never relocating walkers.

The rectangle declumper uses a conservative 30% summed-canvas threshold, while A18 uses the union of actual opaque pixels at 50%. Those are different measures. A regression can verify offsets, water guards and unchanged walkers; only the final captures can establish A18.

## Merged v14 baseline (Part 0)

All115 natural baseline frames were independently measured using the existing opaque-mask `crowd()` function. `merged_baseline.json` records every frame and actor. Eleven frames fail the unchanged limit1:

- `a_d1_0830_F2`:4; `a_evening_06`:2; `a_evening_09`:2; `a_evening_10`:2; `a_strip_13`:2.
- `l_d24_1200_F1` and `l_d24_1200_F2`:4 each.
- `l_d27_0715_F2`:19; `l_d29_0630_F2`:32; `l_d29_0715_F2`:18.
- `l_d40_1200_F2`:2.

The merged d24 offenders differ from Pass5: seated idle/sit agents4,3,11 at feet(816,276),(800,276),(832,276), plus walk agent7 at(833,264). The three seated actors were skipped by the old seat exemption. The morning frames contain walking crowds leaving shelters; these cannot be fixed by displacing walkers without breaking the explicit scope constraint.

## Implemented bounded change

Seated stationary actors now participate in the existing deterministic front-to-back search. The search still tries horizontal1–8px shifts, rejects water/blocked feet, and retains zero if no candidate improves its current coverage. Walking/run/carry activities are excluded from seats, same-tile spreading and declump even when instantaneous interpolation reports `moving=false`. The renderer now passes semantic activity into this decision.

The two d24 regression fixtures reproduce original and merged positions and frame canvas rectangles. Their tests verify that stationary seats move (7 original /8 merged), walkers stay fixed, every shift is within8px and input order is irrelevant. Existing water/blocked-tile tests remain green. This is behavioral evidence, not an A18 pass; final screenshots must be measured separately. The morning walker crowd remains a known likely A18 failure under the unchanged contract.

# J6 remainder (Pass 13 Part 1)

Tuning seeds 0-31 x 120 days, Pass 12's final flags, `probe_p1`. Gap in Pass 6 Y9 `work_1500` (game_v16 - game_v18_nofission): 0.254 - 0.240 = **0.014**. `gap share`: the share of the gap the single reversion restores. `bisect_p13.py`.

| Run | work_1200 | work_1500 | idle_1200 | gap share | adults at 15:00 |
|---|---:|---:|---:|---:|---:|
| game_v16 | 0.358 | 0.254 | 0.621 | — | 29.000 |
| game_v18_nofission | 0.375 | 0.240 | 0.620 | — | 29.000 |
| abl18_v16_damage | 0.362 | 0.231 | 0.625 | -0.601 | 29.000 |
| abl18_no_hunger_weight | 0.375 | 0.240 | 0.620 | 0.000 | 29.000 |
| abl18_no_hardiness | 0.377 | 0.213 | 0.620 | -1.867 | 29.000 |
| abl18_v16_exposure | 0.368 | 0.233 | 0.620 | -0.485 | 29.000 |
| abl18_no_regional_wood | 0.375 | 0.240 | 0.620 | 0.000 | 29.000 |
| abl18_no_region_deadwood | 0.375 | 0.240 | 0.620 | 0.000 | 29.000 |
| abl18_no_region_bushes | 0.371 | 0.241 | 0.616 | 0.118 | 29.000 |
| abl18_no_regions | 0.371 | 0.241 | 0.616 | 0.118 | 29.000 |
| abl18_no_wet_burn | 0.375 | 0.240 | 0.620 | 0.000 | 29.000 |
| abl18_no_area_floor | 0.375 | 0.240 | 0.620 | 0.000 | 29.000 |

## Adults by activity at 15:00 (spring-autumn; pooled share over every 15:00 row)

| Run | forage | wood | build | haul | idle | eat | drink | rest | social | founding | walking |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| game_v16 | 0.178 | 0.031 | 0.000 | 0.060 | 0.592 | 0.005 | 0.119 | 0.000 | 0.016 | 0.000 | 0.336 |
| game_v18_nofission | 0.167 | 0.031 | 0.000 | 0.059 | 0.606 | 0.005 | 0.117 | 0.000 | 0.015 | 0.000 | 0.324 |
| abl18_v16_damage | 0.164 | 0.031 | 0.000 | 0.059 | 0.608 | 0.005 | 0.118 | 0.000 | 0.015 | 0.000 | 0.322 |
| abl18_no_hunger_weight | 0.167 | 0.031 | 0.000 | 0.059 | 0.606 | 0.005 | 0.117 | 0.000 | 0.015 | 0.000 | 0.324 |
| abl18_no_hardiness | 0.164 | 0.031 | 0.000 | 0.058 | 0.609 | 0.005 | 0.117 | 0.000 | 0.016 | 0.000 | 0.321 |
| abl18_v16_exposure | 0.167 | 0.032 | 0.000 | 0.059 | 0.605 | 0.004 | 0.118 | 0.000 | 0.015 | 0.000 | 0.325 |
| abl18_no_regional_wood | 0.167 | 0.031 | 0.000 | 0.059 | 0.606 | 0.005 | 0.117 | 0.000 | 0.015 | 0.000 | 0.324 |
| abl18_no_region_deadwood | 0.167 | 0.031 | 0.000 | 0.059 | 0.606 | 0.005 | 0.117 | 0.000 | 0.015 | 0.000 | 0.324 |
| abl18_no_region_bushes | 0.181 | 0.031 | 0.000 | 0.059 | 0.591 | 0.005 | 0.118 | 0.000 | 0.015 | 0.000 | 0.338 |
| abl18_no_regions | 0.181 | 0.031 | 0.000 | 0.059 | 0.591 | 0.005 | 0.118 | 0.000 | 0.015 | 0.000 | 0.338 |
| abl18_no_wet_burn | 0.167 | 0.031 | 0.000 | 0.059 | 0.606 | 0.005 | 0.117 | 0.000 | 0.015 | 0.000 | 0.324 |
| abl18_no_area_floor | 0.167 | 0.031 | 0.000 | 0.059 | 0.606 | 0.005 | 0.117 | 0.000 | 0.015 | 0.000 | 0.324 |

Shift v18_nofission − v16 of the pooled 15:00 shares: forage -0.011, wood +0.001, build -0.000, haul -0.001, idle +0.014, eat +0.000, drink -0.001, rest -0.000, social -0.001, founding +0.000, walking -0.012

## Reading (PASS13_GOAL.md 6.2)

- On the tuning seeds the remaining gap is small: game_v16 0.254 against game_v18_nofission 0.240 (0.014; Pass 12's held-out 0.241 vs 0.213).
- The missing afternoon share is **foraging**: adults foraging at 15:00 0.178 → 0.167 (−0.011), idle 0.592 → 0.606 (+0.014); wood, hauling and drinking are unchanged.
- **No remaining difference explains half of the gap.** Only the region bushes restore any of it (`abl18_no_region_bushes` and `abl18_no_regions` 0.118 of the gap; forage back to 0.181): the residual of Pass 12's finding after the local-food flag. Regional wood, region deadwood, wet burn and the area floor are exactly inert in a world without fission (0.000: no settlement uses a region; rain needs an intervention). The exposure reversions move `work_1500` the wrong way (−0.6 to −0.7 of the gap: −0.009 / −0.010), i.e. within the per-seed noise of a 0.014 gap.
- Decision: no J6 mechanism in Part 2 (6.2: none explains ≥ half). Hypothesis: the gap is the region bushes' incidental Gather arm outside the catchment (0.12) plus noise on these seeds.

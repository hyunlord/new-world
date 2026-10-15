# Pass 16 evaluation (tuning)

Worlds: 16100–16115 (16). Statistics: Spearman's rho (average ranks) over non-elder adults pooled across worlds; 95 % world-level bootstrap intervals (B = 2000, default_rng(0) per statistic).

## T1 / T2 — rho per scene

| measure | scene | rho | 95 % interval | n | valid draws |
|---|---|---|---|---|---|
| ns | v20 | 0.369 | [0.220, 0.504] | 1009 | 2000 |
| ns | inert | -0.024 | [-0.135, 0.103] | 1010 | 2000 |
| ha | v20 | -0.555 | [-0.677, -0.450] | 1008 | 2000 |
| ha | inert | 0.012 | [-0.061, 0.089] | 1010 | 2000 |
| rd | v20 | 0.800 | [0.768, 0.832] | 1009 | 2000 |
| rd | inert | -0.013 | [-0.091, 0.055] | 1010 | 2000 |
| p_food | v20 | 0.331 | [0.267, 0.399] | 953 | 2000 |
| p_food | inert | 0.074 | [-0.024, 0.158] | 962 | 2000 |
| p_wood | v20 | 0.346 | [0.264, 0.431] | 722 | 2000 |
| p_wood | inert | -0.017 | [-0.097, 0.062] | 713 | 2000 |

## Verdicts

- T1-ns: PASS
- T1-ha: PASS
- T1-rd: PASS
- T1-p_food: PASS
- T2-inert: PASS
- T2-inert (without the wood rho): PASS
- T3: PASS

## Axis quintile → behaviour mean

**ns**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) |
|---|---|---|
| 1 | 270 / 5.71 (201) | 268 / 6.28 (201) |
| 2 | 422 / 6.11 (202) | 420 / 6.19 (204) |
| 3 | 512 / 6.23 (205) | 510 / 6.23 (202) |
| 4 | 604 / 6.29 (201) | 597 / 6.21 (202) |
| 5 | 748 / 6.70 (200) | 744 / 6.29 (201) |

**ha**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) |
|---|---|---|
| 1 | 280 / 1349.57 (201) | 281 / 1327.48 (203) |
| 2 | 415 / 1335.60 (201) | 418 / 1327.89 (199) |
| 3 | 511 / 1326.51 (204) | 511 / 1326.63 (204) |
| 4 | 602 / 1322.33 (199) | 602 / 1328.50 (201) |
| 5 | 748 / 1307.21 (203) | 747 / 1327.89 (203) |

**rd**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) |
|---|---|---|
| 1 | 251 / 1.37 (202) | 250 / 1.76 (202) |
| 2 | 407 / 1.59 (204) | 405 / 1.72 (201) |
| 3 | 498 / 1.72 (201) | 496 / 1.75 (205) |
| 4 | 591 / 1.85 (199) | 588 / 1.76 (200) |
| 5 | 730 / 1.96 (203) | 730 / 1.73 (202) |

**p_food**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) |
|---|---|---|
| 1 | 265 / 2.80 (189) | 269 / 3.04 (192) |
| 2 | 409 / 3.07 (193) | 412 / 3.05 (195) |
| 3 | 500 / 3.00 (191) | 504 / 3.01 (190) |
| 4 | 589 / 3.07 (190) | 593 / 3.11 (192) |
| 5 | 724 / 3.46 (190) | 729 / 3.12 (193) |

**p_wood**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) |
|---|---|---|
| 1 | 263 / 1.52 (145) | 263 / 1.83 (144) |
| 2 | 412 / 1.85 (144) | 408 / 1.85 (141) |
| 3 | 501 / 1.85 (145) | 500 / 1.84 (143) |
| 4 | 589 / 1.84 (144) | 592 / 1.84 (143) |
| 5 | 725 / 2.11 (144) | 725 / 1.83 (142) |

## T3 — health (paired, v20 − reference)

- reference: inert
- alive at the last day: v20 46.56, reference 46.56, share 0.0 %, paired diff 0.00 [0.00, 0.00]
- non-old-age deaths per world: v20 0.56, reference 0.44, paired diff 0.12 [-0.12, 0.44]
- old-age deaths per world (reported): v20 28.44, reference 28.44, paired diff 0.00 [0.00, 0.00]
- fissions per world (reported): v20 0.25, reference 0.25, paired diff 0.00 [0.00, 0.00]
- first split day (reported, worlds where both split: 4; v20 4, reference 4): paired diff -0.25 [-0.75, 0.00]
- deaths by cause: v20 {"exposure": 9, "old_age": 455}; reference {"exposure": 7, "old_age": 455}
- T3 verdict: PASS

## Notes on the measures (review round 1)

- Bootstrap draws whose statistic is undefined (fewer than 3 pairs, no departer) are dropped; the valid-draw count is listed with each interval.
- T2 is judged on every T1 statistic including the wood rho (which has no verdict in T1); the reading without it is listed beside it.
- Idle distance is measured from the hearth of the first settlement listing the person. Departers stay on the parent's roster while walking to the new site (up to 2 days), so an Idle departer on the trek is sampled far from the old hearth; in v20 departers lean high-NS, so these few samples add to T1-NS through departure rather than the tether (the placebo is unaffected).
- A deposit's load is the amount carried on the tick before the drop (section 6.5); a pickup inside the deposit radius on the same tick as the drain is not included, so loads are slightly undercounted alike in every scene.
- Would-be (v19) temperaments use the AgentBorn parents alive at the end of the previous tick; in v20 the observer's computation is compared with the component (`component_mismatches`).

## Trace bookkeeping

- v20: {"born_event_misses": 0, "component_mismatches": 0, "fissions_cancelled": 0, "fissions_completed": 4, "fissions_pending": 0, "people": 1209, "traced_adults": 1152}
- inert: {"born_event_misses": 0, "component_mismatches": 0, "fissions_cancelled": 0, "fissions_completed": 4, "fissions_pending": 2, "people": 1207, "traced_adults": 1152}

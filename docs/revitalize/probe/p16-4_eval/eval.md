# Pass 16 evaluation (evaluation)

Worlds: 16000–16031 (32). Statistics: Spearman's rho (average ranks) over non-elder adults pooled across worlds; 95 % world-level bootstrap intervals (B = 2000, default_rng(0) per statistic).

## T1 / T2 — rho per scene

| measure | scene | rho | 95 % interval | n | valid draws |
|---|---|---|---|---|---|
| ns | v20 | 0.338 | [0.263, 0.428] | 4117 | 2000 |
| ns | inert | 0.044 | [-0.007, 0.097] | 4136 | 2000 |
| ns | v19 | 0.044 | [-0.007, 0.097] | 4136 | 2000 |
| ha | v20 | -0.578 | [-0.630, -0.532] | 4108 | 2000 |
| ha | inert | -0.058 | [-0.103, -0.007] | 4127 | 2000 |
| ha | v19 | -0.058 | [-0.103, -0.007] | 4127 | 2000 |
| rd | v20 | 0.782 | [0.739, 0.820] | 4117 | 2000 |
| rd | inert | 0.008 | [-0.043, 0.056] | 4136 | 2000 |
| rd | v19 | 0.008 | [-0.043, 0.056] | 4136 | 2000 |
| p_food | v20 | 0.217 | [0.147, 0.285] | 4051 | 2000 |
| p_food | inert | 0.031 | [-0.018, 0.077] | 4061 | 2000 |
| p_food | v19 | 0.031 | [-0.018, 0.077] | 4061 | 2000 |
| p_wood | v20 | 0.395 | [0.355, 0.435] | 3337 | 2000 |
| p_wood | inert | 0.010 | [-0.034, 0.056] | 3318 | 2000 |
| p_wood | v19 | 0.010 | [-0.034, 0.056] | 3318 | 2000 |

| departure | scene | NS departers − stayers | 95 % interval | departers / stayers | completed / cancelled fissions | valid draws |
|---|---|---|---|---|---|---|
| T1-dep | v20 | 115.8 | [94.7, 136.7] | 326 / 483 | 25 / 0 | 2000 |
| T1-dep | inert | -15.6 | [-37.9, 9.7] | 326 / 485 | 25 / 0 | 2000 |
| T1-dep | v19 | -15.6 | [-37.9, 9.7] | 326 / 485 | 25 / 0 | 2000 |

## Verdicts

- T1-ns: PASS
- T1-ha: PASS
- T1-rd: PASS
- T1-p_food: FAIL
- T1-dep: PASS
- T2-inert: PASS
- T2-inert (without the wood rho): PASS
- T2-v19: PASS
- T2-v19 (without the wood rho): PASS
- T3: PASS
- T4: PASS

## Axis quintile → behaviour mean

**ns**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) | v19 axis / behaviour (n) |
|---|---|---|---|
| 1 | 261 / 5.74 (822) | 257 / 6.24 (829) | 257 / 6.24 (829) |
| 2 | 405 / 6.18 (829) | 402 / 6.26 (825) | 402 / 6.26 (825) |
| 3 | 499 / 6.25 (819) | 494 / 6.24 (821) | 494 / 6.24 (821) |
| 4 | 586 / 6.31 (822) | 583 / 6.29 (835) | 583 / 6.29 (835) |
| 5 | 730 / 6.66 (825) | 728 / 6.29 (826) | 728 / 6.29 (826) |

**ha**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) | v19 axis / behaviour (n) |
|---|---|---|---|
| 1 | 265 / 1351.62 (823) | 263 / 1327.71 (825) | 263 / 1327.71 (825) |
| 2 | 410 / 1336.18 (815) | 407 / 1327.41 (826) | 407 / 1327.41 (826) |
| 3 | 502 / 1327.54 (824) | 501 / 1326.19 (825) | 501 / 1326.19 (825) |
| 4 | 598 / 1317.59 (823) | 597 / 1323.92 (826) | 597 / 1323.92 (826) |
| 5 | 739 / 1305.50 (823) | 736 / 1325.07 (825) | 736 / 1325.07 (825) |

**rd**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) | v19 axis / behaviour (n) |
|---|---|---|---|
| 1 | 254 / 1.39 (825) | 250 / 1.75 (831) | 250 / 1.75 (831) |
| 2 | 396 / 1.59 (821) | 396 / 1.74 (818) | 396 / 1.74 (818) |
| 3 | 488 / 1.72 (824) | 488 / 1.76 (836) | 488 / 1.76 (836) |
| 4 | 584 / 1.83 (825) | 584 / 1.75 (823) | 584 / 1.75 (823) |
| 5 | 730 / 2.00 (822) | 729 / 1.75 (828) | 729 / 1.75 (828) |

**p_food**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) | v19 axis / behaviour (n) |
|---|---|---|---|
| 1 | 266 / 2.73 (808) | 264 / 2.88 (816) | 264 / 2.88 (816) |
| 2 | 407 / 2.90 (809) | 406 / 2.90 (812) | 406 / 2.90 (812) |
| 3 | 502 / 2.93 (811) | 500 / 2.94 (811) | 500 / 2.94 (811) |
| 4 | 598 / 2.92 (816) | 595 / 2.94 (813) | 595 / 2.94 (813) |
| 5 | 744 / 3.16 (807) | 740 / 2.94 (809) | 740 / 2.94 (809) |

**p_wood**

| quintile | v20 axis / behaviour (n) | inert axis / behaviour (n) | v19 axis / behaviour (n) |
|---|---|---|---|
| 1 | 264 / 1.53 (670) | 261 / 1.82 (663) | 261 / 1.82 (663) |
| 2 | 406 / 1.82 (666) | 404 / 1.82 (665) | 404 / 1.82 (665) |
| 3 | 503 / 1.82 (666) | 498 / 1.82 (661) | 498 / 1.82 (661) |
| 4 | 597 / 1.83 (669) | 593 / 1.83 (666) | 593 / 1.83 (666) |
| 5 | 743 / 2.12 (666) | 741 / 1.82 (663) | 741 / 1.82 (663) |

## T3 — health (paired, v20 − reference)

- reference: v19
- alive at the last day: v20 62.53, reference 62.47, share 0.1 %, paired diff 0.06 [-0.34, 0.47]
- non-old-age deaths per world: v20 1.75, reference 1.44, paired diff 0.31 [-0.22, 0.91]
- old-age deaths per world (reported): v20 80.16, reference 80.09, paired diff 0.06 [-2.19, 2.59]
- fissions per world (reported): v20 0.78, reference 0.78, paired diff 0.00 [-0.09, 0.09]
- first split day (reported, worlds where both split: 18; v20 18, reference 18): paired diff 0.28 [-3.22, 3.56]
- deaths by cause: v20 {"exposure": 56, "old_age": 2565}; reference {"exposure": 46, "old_age": 2563}
- T3 verdict: PASS

## T4 — inheritance (v20)

Births with two known parents: 3845.

| axis | slope on mid-parent | 95 % interval | SD founders | SD alive at end | change |
|---|---|---|---|---|---|
| ns | 0.500 | [0.475, 0.524] | 171.3 | 168.8 | -1.5 % |
| ha | 0.485 | [0.441, 0.516] | 169.9 | 169.0 | -0.5 % |
| rd | 0.487 | [0.440, 0.529] | 168.5 | 166.6 | -1.1 % |
| p | 0.498 | [0.463, 0.532] | 177.4 | 168.6 | -4.9 % |
| pooled | 0.493 | [0.477, 0.508] | | | |

T4 verdict: PASS

## Founder temperament means vs outcomes (v20, descriptive — no verdict)

| seed | founders | NS | HA | RD | P | alive at end | first split day |
|---|---|---|---|---|---|---|---|
| 16000 | 24 | 479 | 459 | 499 | 463 | 136 | 28 |
| 16001 | 30 | 508 | 561 | 534 | 504 | 67 | 60 |
| 16002 | 21 | 460 | 517 | 513 | 452 | 104 | 30 |
| 16003 | 29 | 537 | 406 | 516 | 472 | 84 | 31 |
| 16004 | 19 | 543 | 470 | 476 | 526 | 35 | None |
| 16005 | 22 | 508 | 489 | 492 | 480 | 121 | 36 |
| 16006 | 25 | 412 | 482 | 502 | 529 | 81 | 61 |
| 16007 | 20 | 499 | 469 | 489 | 500 | 28 | None |
| 16008 | 29 | 509 | 507 | 508 | 455 | 79 | 19 |
| 16009 | 19 | 444 | 491 | 472 | 516 | 27 | None |
| 16010 | 18 | 574 | 504 | 499 | 535 | 68 | 90 |
| 16011 | 26 | 479 | 522 | 524 | 580 | 38 | None |
| 16012 | 21 | 519 | 481 | 490 | 486 | 70 | 64 |
| 16013 | 30 | 498 | 442 | 504 | 499 | 43 | None |
| 16014 | 29 | 506 | 544 | 519 | 518 | 33 | None |
| 16015 | 18 | 449 | 523 | 431 | 472 | 31 | None |
| 16016 | 20 | 554 | 507 | 524 | 488 | 69 | 31 |
| 16017 | 23 | 503 | 459 | 497 | 511 | 84 | 62 |
| 16018 | 30 | 491 | 476 | 469 | 525 | 25 | None |
| 16019 | 22 | 482 | 479 | 437 | 527 | 28 | None |
| 16020 | 20 | 542 | 552 | 427 | 465 | 27 | None |
| 16021 | 22 | 535 | 487 | 506 | 596 | 83 | 56 |
| 16022 | 26 | 543 | 467 | 456 | 522 | 72 | 30 |
| 16023 | 29 | 543 | 485 | 461 | 525 | 49 | 13 |
| 16024 | 28 | 533 | 438 | 462 | 531 | 32 | None |
| 16025 | 29 | 508 | 550 | 433 | 507 | 76 | 40 |
| 16026 | 30 | 510 | 458 | 509 | 527 | 115 | 12 |
| 16027 | 22 | 540 | 489 | 453 | 496 | 34 | None |
| 16028 | 30 | 452 | 489 | 492 | 492 | 64 | 59 |
| 16029 | 22 | 481 | 492 | 497 | 489 | 30 | None |
| 16030 | 26 | 450 | 452 | 471 | 563 | 44 | None |
| 16031 | 18 | 482 | 508 | 452 | 524 | 124 | 12 |

## Notes on the measures (review round 1)

- Bootstrap draws whose statistic is undefined (fewer than 3 pairs, no departer) are dropped; the valid-draw count is listed with each interval.
- T2 is judged on every T1 statistic including the wood rho (which has no verdict in T1); the reading without it is listed beside it.
- Idle distance is measured from the hearth of the first settlement listing the person. Departers stay on the parent's roster while walking to the new site (up to 2 days), so an Idle departer on the trek is sampled far from the old hearth; in v20 departers lean high-NS, so these few samples add to T1-NS through departure rather than the tether (the placebo is unaffected).
- A deposit's load is the amount carried on the tick before the drop (section 6.5); a pickup inside the deposit radius on the same tick as the drain is not included, so loads are slightly undercounted alike in every scene.
- Would-be (v19) temperaments use the AgentBorn parents alive at the end of the previous tick; in v20 the observer's computation is compared with the component (`component_mismatches`).

## Trace bookkeeping

- v20: {"born_event_misses": 0, "component_mismatches": 0, "fissions_cancelled": 0, "fissions_completed": 25, "fissions_pending": 0, "people": 4622, "traced_adults": 4444}
- inert: {"born_event_misses": 0, "component_mismatches": 0, "fissions_cancelled": 0, "fissions_completed": 25, "fissions_pending": 1, "people": 4608, "traced_adults": 4447}
- v19: {"born_event_misses": 0, "component_mismatches": 0, "fissions_cancelled": 0, "fissions_completed": 25, "fissions_pending": 1, "people": 4608, "traced_adults": 4447}

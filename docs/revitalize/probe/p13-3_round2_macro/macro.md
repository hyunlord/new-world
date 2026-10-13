# Pass 13 macro sensitivity (12.2, J11) — `game_v19`

Seeds 901100..901132, R = 2, 120 days, fork at 28800. Post-fork fission = the first fission whose group departed after the fork (schema-6 `depart_tick`); its day = the departure's display day.

| Comparison | n both split after fork | share post-fork day differs > 3 d | mean abs Δ day | signed shift mean | share shift < 0 / > 0 / ≠ 0 | mean abs Δ alive 118 | c-rule share |
|---|---:|---:|---:|---:|---|---:|---:|
| (a) factual r0 vs r1 (noise only) | 12 | 0.031 | 5.083 | — | — | 2.562 | 0.062 |
| (b) rot_wood vs factual, same replicate | 22 | 0.062 | 3.455 | 2.455 | 0.455 / 0.318 / 0.773 | 1.844 | 0.094 |
| (b) spoil_food vs factual, same replicate | 24 | 0.047 | 2.833 | 2.167 | 0.333 / 0.208 / 0.542 | 2.547 | 0.062 |
| (b) gift_food vs factual, same replicate | 22 | 0.031 | 0.364 | 0.273 | 0.045 / 0.273 / 0.318 | 1.219 | 0.047 |

J11 (a): {"share_day_differs_gt3": {"value": 0.03125, "passed": true}, "mean_abs_delta_day": {"value": 5.083333333333333, "n": 12, "passed": true}, "mean_abs_delta_alive_118": {"value": 2.5625, "passed": true}}
J11 (b): {"spoil_food_earlier": {"value": 0.3333333333333333, "n": 24, "passed": false}, "gift_food_later": {"value": 0.2727272727272727, "n": 22, "passed": false}}
J11 (c): {"noise_c": 0.0625, "variants_over": [], "passed": false}
J11: FAIL

# Pass 13 macro sensitivity (12.2, J11) — `game_v18`

Seeds 12000..12032, R = 2, 120 days, fork at 28800. Post-fork fission = the first fission whose group departed after the fork (schema-6 `depart_tick`); its day = the departure's display day.

| Comparison | n both split after fork | share post-fork day differs > 3 d | mean abs Δ day | signed shift mean | share shift < 0 / > 0 / ≠ 0 | mean abs Δ alive 118 | c-rule share |
|---|---:|---:|---:|---:|---|---:|---:|
| (a) factual r0 vs r1 (noise only) | 14 | 0.688 | 34.500 | — | — | 18.406 | 0.688 |
| (b) rot_wood vs factual, same replicate | 35 | 0.016 | 0.000 | 0.000 | 0.000 / 0.000 / 0.000 | 0.703 | 0.016 |
| (b) gift_wood vs factual, same replicate | 36 | 0.016 | 2.444 | 2.444 | 0.000 / 0.028 / 0.028 | 0.688 | 0.016 |
| (b) spoil_food vs factual, same replicate | 36 | 0.000 | 0.000 | 0.000 | 0.000 / 0.000 / 0.000 | 0.031 | 0.000 |
| (b) gift_food vs factual, same replicate | 36 | 0.016 | 2.444 | 2.444 | 0.000 / 0.028 / 0.028 | 0.672 | 0.016 |

J11 (a): {"share_day_differs_gt3": {"value": 0.6875, "passed": false}, "mean_abs_delta_day": {"value": 34.5, "n": 14, "passed": false}, "mean_abs_delta_alive_118": {"value": 18.40625, "passed": false}}
J11 (b): {"spoil_food_earlier": {"value": 0.0, "n": 36, "passed": false}, "gift_food_later": {"value": 0.027777777777777776, "n": 36, "passed": false}}
J11 (c): {"noise_c": 0.6875, "variants_over": [], "passed": false}
J11: FAIL

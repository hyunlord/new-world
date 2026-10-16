# Pass 13 macro sensitivity (12.2, J11) — `game_v19`

Seeds 12000..12032, R = 2, 120 days, fork at 28800. Post-fork fission = the first fission whose group departed after the fork (schema-6 `depart_tick`); its day = the departure's display day.

| Comparison | n both split after fork | share post-fork day differs > 3 d | mean abs Δ day | signed shift mean | share shift < 0 / > 0 / ≠ 0 | mean abs Δ alive 118 | c-rule share |
|---|---:|---:|---:|---:|---|---:|---:|
| (a) factual r0 vs r1 (noise only) | 20 | 0.094 | 3.550 | — | — | 2.406 | 0.094 |
| (b) rot_wood vs factual, same replicate | 37 | 0.109 | 3.000 | -1.595 | 0.486 / 0.108 / 0.595 | 2.688 | 0.109 |
| (b) gift_wood vs factual, same replicate | 37 | 0.156 | 4.405 | 0.892 | 0.162 / 0.405 / 0.568 | 1.984 | 0.188 |
| (b) spoil_food vs factual, same replicate | 39 | 0.078 | 3.103 | -0.692 | 0.513 / 0.128 / 0.641 | 2.156 | 0.094 |
| (b) gift_food vs factual, same replicate | 36 | 0.156 | 4.111 | 2.667 | 0.139 / 0.528 / 0.667 | 1.922 | 0.188 |

J11 (a): {"share_day_differs_gt3": {"value": 0.09375, "passed": true}, "mean_abs_delta_day": {"value": 3.55, "n": 20, "passed": true}, "mean_abs_delta_alive_118": {"value": 2.40625, "passed": true}}
J11 (b): {"spoil_food_earlier": {"value": 0.5128205128205128, "n": 39, "passed": false}, "gift_food_later": {"value": 0.5277777777777778, "n": 36, "passed": false}}
J11 (c): {"noise_c": 0.09375, "variants_over": [], "passed": false}
J11: FAIL

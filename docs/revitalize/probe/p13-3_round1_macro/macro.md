# Pass 13 macro sensitivity (12.2, J11) — `game_v19`

Seeds 901100..901132, R = 2, 120 days, fork at 28800. Post-fork fission = the first fission whose group departed after the fork (schema-6 `depart_tick`); its day = the departure's display day.

| Comparison | n both split after fork | share post-fork day differs > 3 d | mean abs Δ day | signed shift mean | share shift < 0 / > 0 / ≠ 0 | mean abs Δ alive 118 | c-rule share |
|---|---:|---:|---:|---:|---|---:|---:|
| (a) factual r0 vs r1 (noise only) | 16 | 0.000 | 0.562 | — | — | 0.562 | 0.031 |
| (b) rot_wood vs factual, same replicate | 31 | 0.109 | 2.065 | 1.419 | 0.290 / 0.290 / 0.581 | 1.312 | 0.172 |
| (b) spoil_food vs factual, same replicate | 32 | 0.047 | 0.719 | -0.406 | 0.344 / 0.156 / 0.500 | 0.906 | 0.078 |
| (b) gift_food vs factual, same replicate | 31 | 0.031 | 0.710 | 0.387 | 0.161 / 0.387 / 0.548 | 0.156 | 0.062 |

J11 (a): {"share_day_differs_gt3": {"value": 0.0, "passed": true}, "mean_abs_delta_day": {"value": 0.5625, "n": 16, "passed": true}, "mean_abs_delta_alive_118": {"value": 0.5625, "passed": true}}
J11 (b): {"spoil_food_earlier": {"value": 0.34375, "n": 32, "passed": false}, "gift_food_later": {"value": 0.3870967741935484, "n": 31, "passed": false}}
J11 (c): {"noise_c": 0.03125, "variants_over": ["rot_wood"], "passed": false}
J11: FAIL

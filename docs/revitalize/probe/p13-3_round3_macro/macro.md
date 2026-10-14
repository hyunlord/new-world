# Pass 13 macro sensitivity (12.2, J11) — `game_v19`

Seeds 901100..901132, R = 2, 120 days, fork at 28800. Post-fork fission = the first fission whose group departed after the fork (schema-6 `depart_tick`); its day = the departure's display day.

| Comparison | n both split after fork | share post-fork day differs > 3 d | mean abs Δ day | signed shift mean | share shift < 0 / > 0 / ≠ 0 | mean abs Δ alive 118 | c-rule share |
|---|---:|---:|---:|---:|---|---:|---:|
| (a) factual r0 vs r1 (noise only) | 16 | 0.000 | 0.625 | — | — | 0.438 | 0.031 |
| (b) rot_wood vs factual, same replicate | 31 | 0.109 | 2.387 | 1.355 | 0.323 / 0.323 / 0.645 | 1.484 | 0.156 |
| (b) spoil_food vs factual, same replicate | 32 | 0.047 | 0.750 | -0.375 | 0.312 / 0.188 / 0.500 | 0.938 | 0.062 |
| (b) gift_food vs factual, same replicate | 31 | 0.031 | 0.839 | 0.323 | 0.226 / 0.419 / 0.645 | 0.094 | 0.047 |

J11 (a): {"share_day_differs_gt3": {"value": 0.0, "passed": true}, "mean_abs_delta_day": {"value": 0.625, "n": 16, "passed": true}, "mean_abs_delta_alive_118": {"value": 0.4375, "passed": true}}
J11 (b): {"spoil_food_earlier": {"value": 0.3125, "n": 32, "passed": false}, "gift_food_later": {"value": 0.41935483870967744, "n": 31, "passed": false}}
J11 (c): {"noise_c": 0.03125, "variants_over": ["rot_wood"], "passed": false}
J11: FAIL

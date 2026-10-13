# Pass 13 macro sensitivity (12.2, J11) — game_v19 against game_v18

Frozen recorder `p13bin/wm_frozen` (commit `9f0d38ca`, sha256 `1844ebc3…`, schema 6), seeds 12000–12031, 120 days, `--every 60`, no `--agents`, noise fork at tick 28,800, R = 2; factual plus rot_wood, gift_wood, spoil_food 60, gift_food 60 at tick 29,400 on home (schedule files written explicitly into `p13/schedules/macro`, one per seed and arm: `[{"tick": 29400, "kind": "<kind>", "settlement": "home", "amount": 60}]`). Post-fork fission = the first fission whose group departed after the fork (schema-6 `depart_tick`), its day = the departure's display day; both scenes use this rule (v18 recorded fresh with the schema-6 recorder into `p13/macro_v18`).

## game_v19 (J11 judged here)

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

## game_v18 (fresh, same arms and rule)

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

## Commands

```bash
bash /Users/rexxa/work/p13bin/p13_macro.sh 9f0d38ca… game_v19 game_v18
python3 tools/revitalize/macro_sensitivity.py --root p13/macro --scene game_v19 --seeds 12000..12032 --variants rot_wood,gift_wood,spoil_food,gift_food --replicates 2 --j11 28800 --out docs/revitalize/probe/p13-6_macro/game_v19
python3 tools/revitalize/macro_sensitivity.py --root p13/macro_v18 --scene game_v18 --seeds 12000..12032 --variants rot_wood,gift_wood,spoil_food,gift_food --replicates 2 --j11 28800 --out docs/revitalize/probe/p13-6_macro/game_v18
```

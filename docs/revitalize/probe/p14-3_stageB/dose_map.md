# Pass 14 Stage B — dose–timing map (`game_v19`, seeds 14000..14064, R = 2, fork tick 14280)

Scored by `tools/revitalize/dose_map_p14.py` (rules in its module docs). Departure = the first departure of any group at a tick ≥ the cast tick (schema-6 `depart_tick` of founded daughters); Δday = arm − factual (same seed and replicate); noise = factual r1 − r0 on the arm's seeds after the arm's cast tick. Intervals: 90 % seed bootstrap (1,000×, `default_rng(0)`). ENR = mean |Δday| / max(0.5, noise mean |Δday|), NA if the noise has < 8 pairs where both depart.

Episode checks: 588 episodes read, 0 errors.
Factual r0 vs probe fission log (founded daughters' departure ticks): 64 / 64 seeds equal; probe departures without a founded daughter (not seen by the recorder) 2, cancelled 0.

## Direction and size of the shift of the first departure after the cast

| arm | kind | amount × doses | time | seeds | pairs | right-direction share [90 %] | earlier / later / tie | n both | mean \|Δday\| [90 %] | share \|Δ\| > 3 | mean signed Δday | noise n both / mean \|Δ\| / share > 3 | ENR [90 %] |
|---|---|---|---|---:|---:|---|---|---:|---|---:|---:|---|---|
| `sp240E` | spoil_food | 240 × 1 | E | 64 | 128 | 0.469 [0.375, 0.570] | 0.469 / 0.062 / 0.469 | 85 | 7.89 [5.11, 11.07] | 0.294 | -6.98 | 42 / 2.07 / 0.071 | 3.81 [2.13, 8.81] |
| `gf240E` | gift_food | 240 × 1 | E | 64 | 128 | 0.594 [0.500, 0.688] | 0.008 / 0.594 / 0.398 | 83 | 8.06 [5.71, 10.59] | 0.325 | 7.43 | 42 / 2.07 / 0.071 | 3.89 [2.15, 9.89] |
| `sp240L` | spoil_food | 240 × 1 | L | 51 | 102 | 0.794 [0.696, 0.873] | 0.794 / 0.000 / 0.206 | 79 | 26.04 [19.99, 32.50] | 0.861 | -26.04 | 39 / 2.77 / 0.103 | 9.40 [5.51, 22.61] |
| `gf240L` | gift_food | 240 × 1 | L | 51 | 102 | 0.382 [0.284, 0.490] | 0.078 / 0.382 / 0.539 | 74 | 5.39 [3.10, 7.86] | 0.230 | 5.12 | 39 / 2.77 / 0.103 | 1.95 [1.00, 4.57] |

## Shape of the world: settlements and people

| arm | settlement count differs d60 / d90 / d118 | noise d60 / d90 / d118 | d118 differs − noise | alive d118 Δ signed [90 %] | alive d118 \|Δ\| | noise \|Δ alive\| |
|---|---|---|---:|---|---:|---:|
| `sp240E` | 0.109 / 0.078 / 0.094 | 0.016 / 0.000 / 0.062 | 0.031 | 2.34 [0.72, 4.16] | 3.39 | 1.25 |
| `gf240E` | 0.141 / 0.039 / 0.070 | 0.016 / 0.000 / 0.062 | 0.008 | -2.65 [-4.46, -1.17] | 3.20 | 1.25 |
| `sp240L` | 0.324 / 0.255 / 0.206 | 0.020 / 0.000 / 0.078 | 0.127 | 7.44 [4.64, 10.39] | 7.83 | 1.57 |
| `gf240L` | 0.049 / 0.069 / 0.078 | 0.020 / 0.000 / 0.078 | 0.000 | -2.42 [-4.12, -1.00] | 2.50 | 1.57 |

## Deaths (mean signed Δ arm − factual; noise r1 − r0)

| arm | within 35 d of the cast: total [90 %] / \|Δ\| / noise signed / noise \|Δ\| / arm − noise [90 %] | by cause (starv., dehyd., combat, old age, exposure) | to day 120: total [90 %] / \|Δ\| / noise signed / noise \|Δ\| / arm − noise [90 %] | by cause |
|---|---|---|---|---|
| `sp240E` | 0.16 [-0.15, 0.66] / 0.35 / 0.00 / 0.16 / 0.16 [-0.19, 0.69] | 0.05, 0.06, 0.00, -0.09, 0.14 | 2.52 [1.24, 3.95] / 3.56 / 0.80 / 2.11 / 1.72 [0.34, 3.16] | 0.05, 0.05, 0.00, 2.39, 0.02 |
| `gf240E` | -0.12 [-0.24, -0.03] / 0.14 / 0.00 / 0.16 / -0.12 [-0.27, 0.01] | 0.00, 0.00, 0.00, 0.02, -0.15 | -3.91 [-5.68, -2.25] / 4.45 / 0.80 / 2.11 / -4.70 [-6.83, -2.68] | 0.00, 0.00, 0.00, -3.74, -0.16 |
| `sp240L` | 0.31 [0.03, 0.68] / 0.43 / -0.02 / 0.25 / 0.33 [-0.04, 0.76] | 0.00, 0.01, 0.00, -0.04, 0.34 | 10.51 [7.63, 13.45] / 11.08 / 0.96 / 2.45 / 9.55 [6.62, 12.51] | -0.01, 0.00, 0.00, 9.74, 0.78 |
| `gf240L` | -0.02 [-0.07, 0.02] / 0.06 / -0.02 / 0.25 / 0.00 [-0.18, 0.17] | 0.00, 0.00, 0.00, -0.01, -0.01 | -2.15 [-3.92, -0.57] / 2.75 / 0.96 / 2.45 / -3.11 [-5.43, -1.03] | -0.01, -0.01, 0.00, -2.05, -0.08 |

## Applied amounts

| arm | rows | requested | applied | rows applied < requested | min applied / requested |
|---|---:|---:|---:|---:|---:|
| `sp240E` | 128 | 30720 | 30624 | 3 | 0.779 |
| `gf240E` | 128 | 30720 | 30720 | 0 | 1.000 |
| `sp240L` | 102 | 24480 | 23584 | 23 | 0.533 |
| `gf240L` | 102 | 24480 | 24480 | 0 | 1.000 |

## Targets

- **K2**: {"sp240L": {"right_share": 0.7941176470588235, "right_ge_0.75": true, "enr": 9.402601969057665, "enr_ge_2": true, "settle118_differs": 0.20588235294117646, "noise_settle118": 0.0784313725490196, "settle_clause": false, "passed": false}, "gf240L": {"right_share": 0.38235294117647056, "right_ge_0.75": false, "enr": 1.9470720720720722, "enr_ge_2": false, "settle118_differs": 0.0784313725490196, "noise_settle118": 0.0784313725490196, "settle_clause": false, "passed": false}}
- **K3**: {"spoil_food": {"n_seeds": 51, "share_low": 0.7941176470588235, "share_eve": 0.5882352941176471, "diff": 0.2058823529411764, "ci90": [0.11764705882352944, 0.28480392156862705], "passed": true}, "gift_food": {"n_seeds": 51, "share_low": 0.38235294117647056, "share_eve": 0.7254901960784313, "diff": -0.3431372549019608, "ci90": [-0.46078431372549017, -0.22549019607843135], "passed": false}, "passed": false}

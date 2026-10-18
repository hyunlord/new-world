# Pass 21 comparison (7.5, reported only): `game_v26` on Pass 20's evaluation seeds 19400–19431

`game_v26` (final rule, G2) was run on 19400–19431, 120 days, with the evaluation flags (`bin_p2`, 00:49–01:56 KST, DETERMINISM PASS) and set beside Pass 20's evaluation records of `game_v25` and `game_v24` on the same seeds (`p20-data/p2/A`, Pass 20's build; `game_v25` and `game_v24` are bit-identical in this pass's build, I8/I9). The multi-village worlds are the same 19 in all three. Tool: `evaluate_pass21.py --no-u4` ([raw table](compare_raw.md), `compare.json`); Pass 20's numbers from `probe/p20-2_eval/eval.json`.

## U1's quantities (aid)

| quantity | `game_v24` | `game_v25` (Pass 20) | `game_v26` |
|---|---|---|---|
| multi-village worlds with ≥ 1 delivered trip | – | 9 of 19 (0.474) | **12 of 19 (0.632)** |
| trips delivered / porters | – | 38 / 71 | 113 / 221 |
| porters turned back | – | 0 | 0 |
| food / wood moved | – | 634 / 34 | 2,090 / 101 |
| departure causes | – | aid food 67, aid wood 4 | aid food 209, aid wood 12 |
| in-reach food-aid pair-days: open / between estranged villages | – | 38 / 176 | 120 / 69 |
| exchange trips | – | 0 | 0 |

## U3's quantities (conflict)

| quantity | `game_v25` (Pass 20) | `game_v26` |
|---|---|---|
| worlds with a dispute / a quarrel (of 19) | 13 / 13 | 13 / 13 |
| disputes (past the daily dispute cap) | 1,834 | 3,199 (1,762) |
| quarrels (disputes that met the quarrel cap) | 511 | 369 (1,726) |
| quarrels while estranged | 426 of 511 (0.834) | **36 of 369 (0.098)** |
| estrangements / reconciliations | 14 / **0** | 22 / **13 (0.591)** |
| estranged pairs at day 120 | 14 of 32 (0.438) | **9 of 32 (0.281)** |
| pair-days estranged | – | 254 |

## Daughters

Daughter non-old-age deaths over the 19 multi-village worlds: `game_v24` 6, `game_v25` 26, `game_v26` **12** (19414, Pass 20's outlier: 5 / 24 / 10); daughters alive at day 118: 24 / 24 / 24. Against `game_v25` the mean per multi-village world is −0.737 [−2.264, 0.105]; against `game_v24` +0.316 [0.000, 0.895].

## Reading

On Pass 20's own seeds `game_v26` crosses U1's 0.6 bar that `game_v25` missed (12 instead of 9 delivering worlds, three times the food moved), and reconciliation, absent in Pass 20, now ends 13 of 22 estrangements; quarrels between estranged villages fall from 83 % to 10 % of all quarrels. Estranged pairs at day 120 fall from 14 to 9 of 32, still above U3's 0.25. Disputes are more frequent (1.7×) because estranged pairs now reconcile and meet again, but each costs at most 3 × 5 + 1 × 40 a day. Exchange still never meets its conditions.

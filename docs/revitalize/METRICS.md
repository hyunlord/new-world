# Revitalize metrics

## Measurement contract

All measurements use the dedicated DGX clone, release probe, 1440 ticks/day,
120-tick regular samples, exact day 1/3/5/10 checkpoints and independently
constructed determinism twins. JSONL has 34 stable schema-version-1 fields.
Distributions use linear quantiles. T6 is cumulative sampled idle need
breaches divided by sampled living agents (including tick zero). Performance
is primary-engine tick wall time divided by exact pre-tick agent counts;
wall-clock run duration also includes output and twin verification.

## Part 1 legacy reference

Four identical repeats, seeds 0..4 (seed ignored by the fixture), 10 days:
DETERMINISM PASS. Day 10: 44 alive, 213 births, 233 dehydration deaths,
zero starvation/combat deaths. Final fingerprint `b9ce9cb344a98de5`.
The exact 2880-tick regression pin is frozen in
`harness_revitalize_legacy_pin.rs`; it must never be updated to accommodate
new behavior. Detailed distributions: `probe/p1_legacy/summary.md` and JSON.

## Game tuning

Part 2 reference, all three tuning rounds and final fresh verification follow below.

## Part 2 untuned world (16 seeds)

Before Part 3 behavior is connected, median day-10 alive=103.5, births=353,
dehydration deaths=272, starvation deaths=0, shelters completed=0, sampled
idle breach share=15.432967%, home distance=10.967236 tiles. This is a
reference, not a target result: unrestricted legacy births and wandering
are still active. Full distributions: `probe/p2_game_untuned/summary.*`.

## Part 3 round 0: requested defaults

64 seeds, 10 days, eight workers, twin validation PASS. Knobs: move6, pause0.6, tether6, residents6, cooldown720, food_cost6, bushes12/capacity6, builders4, progress240.

| Metric | Median | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| alive | 27.0 | 20.3 | 33.0 | 14.0 | 41.0 |
| births | 52.0 | 39.3 | 64.4 | 36.0 | 76.0 |
| deaths_dehydration | 47.5 | 42.0 | 57.7 | 40.0 | 66.0 |
| deaths_starvation | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| shelters_completed | 2.0 | 1.0 | 4.0 | 1.0 | 7.0 |
| idle_with_breach_share | 0.027761565840308533 | 0.011256159213734542 | 0.06573377361554701 | 0.0015384615384615385 | 0.10911045417898414 |
| home_distance_mean | 4.08743842364532 | 2.6772727272727272 | 5.3373793103448275 | 1.8275862068965518 | 7.357142857142857 |

New 3-day survival test fails at seed1: 11 dehydration deaths. All other six loop tests and unchanged legacy pin pass. Investigating fixed resource seeks before selecting allowed tuning knobs.

## Part 3 tuning round 1

Knobs: move4, pause0.8, tether4, residents6, cooldown1440, food_cost10, bushes20/capacity8, builders4, progress240. 64 twins PASS.

| Metric | Median | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| alive | 18.5 | 12.3 | 25.700000000000003 | 6.0 | 35.0 |
| births | 27.0 | 20.0 | 37.0 | 9.0 | 43.0 |
| deaths_dehydration | 34.0 | 28.3 | 39.0 | 19.0 | 46.0 |
| deaths_starvation | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| shelters_completed | 2.0 | 1.0 | 4.700000000000003 | 1.0 | 6.0 |
| idle_with_breach_share | 0.010022714715452146 | 0.0019441060784988332 | 0.03566609643773937 | 0.0 | 0.045166106756252336 |
| home_distance_mean | 3.9420168067226893 | 2.580267558528428 | 5.416776315789474 | 1.8181818181818181 | 6.526315789473684 |

## Part 3 tuning round 2

Knobs: move8, pause0.4, tether8, residents4, cooldown360, food_cost3, bushes8/capacity4, builders4, progress240. 64 twins PASS.

| Metric | Median | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| alive | 33.0 | 28.3 | 40.0 | 24.0 | 44.0 |
| births | 83.0 | 69.3 | 95.0 | 52.0 | 109.0 |
| deaths_dehydration | 75.0 | 63.3 | 83.7 | 47.0 | 90.0 |
| deaths_starvation | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| shelters_completed | 6.0 | 4.0 | 10.0 | 2.0 | 14.0 |
| idle_with_breach_share | 0.10707674211420556 | 0.048160227114587104 | 0.1730493381647773 | 0.016320084232692815 | 0.3320481927710843 |
| home_distance_mean | 3.7058823529411766 | 2.5188235294117645 | 5.409655172413793 | 1.967741935483871 | 6.642857142857143 |

## Part 3 tuning round 3 (cap reached)

Knobs: move4, pause0.6, tether4, residents4, cooldown360, food_cost6, bushes20/capacity8, builders4, progress240. 64 twins PASS.

| Metric | Median | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| alive | 49.0 | 36.3 | 59.0 | 26.0 | 84.0 |
| births | 120.0 | 91.3 | 154.0 | 76.0 | 174.0 |
| deaths_dehydration | 97.5 | 75.2 | 116.0 | 61.0 | 139.0 |
| deaths_starvation | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| shelters_completed | 11.0 | 7.0 | 15.0 | 3.0 | 19.0 |
| idle_with_breach_share | 0.01719722772041822 | 0.005555191831395521 | 0.04202239898767536 | 0.0015249068112504235 | 0.0751912568306011 |
| home_distance_mean | 2.8811541565778853 | 2.167738164815897 | 4.499671052631579 | 1.5869565217391304 | 5.947368421052632 |

Keep round2: both rounds2/3 pass higher-priority non-extinction, but round2 has fewer dehydration deaths (median75/max90 versus97.5/139). This sacrifices lower-priority idle-breach performance. None solves social seeking lock. Three tuning rounds exhausted; no further social or knob changes. p3_game_final copies round2, not a fresh run; final verification repeats it.

## Part 5 fresh verification

Source47ace745; release binary rebuilt. Legacy0..4 workers4 and game0..64 workers8, 10days, sample120, independent twins: PASS. Compare all JSONL fields except git_sha: legacy484 rows exactly match Part1; game7744 rows exactly match round2. Thus behavioral distributions above are reproduced, not assumed.

Timing ms/agent-tick median: legacy=0.011784043331, game=0.000944597051, ratio=0.080158993 (T9 PASS). Full quantiles and days1/3/5/10: `probe/p5_legacy/summary.*`, `probe/p5_game/summary.*`. T1 and T6 FAIL; T2/T3/T4/T5/T7/T8/T9 PASS. Three-day required harness still fails seed1 with dehydration11 vs0. No fourth tuning round or out-of-scope social modification.

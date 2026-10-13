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

## Pass 2 Part 1: unchanged P5 death census

64 seeds x10 days;64/64 deterministic twins PASS. All7,744 sample rows reproduce P5 v1 behavioral fields exactly, excluding schema_version/git_sha. Each schema2 row has38 keys. Measured source7cdc501c plus Part1 diff now committed as9d853102.

4,715 deaths, all dehydration: SeekingAgent partner alive4,280, dead207; Idle145; SeekingFood46; SeekingWater35; ConsumingAgent2. Never sought water4,664/4,715 (98.918%). Water distance0-1:1,790;2-5:1,740;6-15:1,156;16+:29.

Planner P2 census:6 seeds x5 days, earlier rules; social chasing86.3%, Idle13.5%, within1 tile17%. Current P5:95.16%,3.08%,37.96%. Different rules and horizons mean these are not controlled improvement percentages; both identify social seeking as dominant.

Social pairs per agent-day median 0.476660962409, p10 0.382924959553, p90 0.606505343448, min 0.295776604892, max 1.529827478695. Per-seed T11 baseline in summary.json. Instrumented timing median0.0012097822478452105 ms/agent-tick versus original P5 0.0009445970510943427. Observer capture duration excluded from tick timing; postprocessing is outside timing.

## Pass 2 Part 3: initial full census before tuning

Source `ade8b624`; 64 seeds × 10 days and 16 seeds × 30 days, independent twins enabled. Leader reports all 64/64 and 16/16 twins passed. Sources: [10-day summary](probe/p2-3_after_fix/summary.json), [30-day summary](probe/p2-3_30day/summary.json). This is the initial Part 3 configuration, not tuning round 1: birth cooldown 360, cost 3, residents/shelter 4, chase 240, retry 480, critical preemption 75. Earlier 16-seed trigger results do not substitute for this wider, changed-source run.

T8 twin checks PASS; legacy pin is a separate gate and is not established by these probe summaries. T1 FAIL (10-day dehydration maximum 55 > 1); T3 PASS (minimum alive 39 at both horizons); T2 FAIL (starvation median 33, p90 55.1); T4 PASS (median alive 52; median net births minus deaths 28). Net population change equals alive minus 24 founders, with births/deaths as the only population additions/removals. T10 FAIL: median 30-day dehydration is 0, but housing overflow exceeds 6. T5 PASS (median 12 shelters completed), T6a PASS (median actionable idle share 0.002381 ≤ 0.01), T7 PASS (median home distance 3.084354). Unchanged three-day survival/raw-idle harness remains a separate required gate; this report does not weaken it.

| Horizon | Metric | Median | p10 | p90 | Min | Max |
|---|---|---:|---:|---:|---:|---:|
| 10 days | alive | 52 | 47 | 59.7 | 39 | 77 |
| 10 days | births | 63 | 50 | 84.4 | 39 | 98 |
| 10 days | deaths_dehydration | 0 | 0 | 8.1 | 0 | 55 |
| 10 days | deaths_starvation | 33 | 9.2 | 55.1 | 1 | 64 |
| 10 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 10 days | shelters_completed | 12 | 9 | 15.7 | 4 | 19 |
| 10 days | housing_capacity | 60 | 48 | 74.8 | 28 | 88 |
| 10 days | idle_breach_actionable_share | 0.002381 | 0.001294 | 0.00372 | 0.000418 | 0.006348 |
| 10 days | idle_breach_unavailable_share | 0.26761 | 0.199959 | 0.331894 | 0.161672 | 0.366435 |
| 10 days | idle_with_breach_share | 0.26853 | 0.200688 | 0.333167 | 0.16239 | 0.368127 |
| 10 days | home_distance_mean | 3.084354 | 2.260714 | 3.877508 | 1.92 | 5.47619 |
| 10 days | stockpile_food | 5.5 | 0 | 73.1 | 0 | 220 |
| 10 days | food_tiles_total | 0 | 0 | 0 | 0 | 0 |
| 10 days | social_completed_per_agent_day | 0.515043 | 0.418346 | 0.641928 | 0.317283 | 0.700322 |
| 10 days | births minus deaths | 28 | 23 | 35.7 | 15 | 53 |
| 30 days | alive | 50.5 | 44.5 | 59.5 | 39 | 60 |
| 30 days | births | 170 | 119.5 | 236.5 | 99 | 295 |
| 30 days | deaths_dehydration | 0 | 0 | 22.5 | 0 | 184 |
| 30 days | deaths_starvation | 132.5 | 81 | 199 | 33 | 260 |
| 30 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 30 days | shelters_completed | 13 | 9.5 | 16 | 5 | 20 |
| 30 days | housing_capacity | 64 | 50 | 76 | 32 | 92 |
| 30 days | idle_breach_actionable_share | 0.002593 | 0.001175 | 0.003824 | 0.0002 | 0.004384 |
| 30 days | idle_breach_unavailable_share | 0.334038 | 0.241224 | 0.386453 | 0.205779 | 0.388322 |
| 30 days | idle_with_breach_share | 0.335634 | 0.242648 | 0.387789 | 0.205912 | 0.389604 |
| 30 days | home_distance_mean | 2.920238 | 2.331672 | 3.693296 | 2.083333 | 4.205128 |
| 30 days | stockpile_food | 16 | 0 | 253.5 | 0 | 552 |
| 30 days | food_tiles_total | 0 | 0 | 0 | 0 | 0 |
| 30 days | social_completed_per_agent_day | 0.509459 | 0.414324 | 0.606934 | 0.364661 | 0.680092 |
| 30 days | births minus deaths | 26.5 | 20.5 | 35.5 | 15 | 36 |

### T9 timing comparisons

Observer time is excluded by probe instrumentation. Ratios compare median ms per agent-tick on the same DGX, against both original P5 and the instrumented unchanged-P5 census. Different concurrency/load and measurement instrumentation mean this is observed timing, not an isolated causal speedup.

| Horizon | Median ms/agent-tick | p10 | p90 | Min | Max | / original P5 | / instrumented P5 | ≤1.5× both |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 10 | 0.00083 | 0.000772 | 0.001632 | 0.000715 | 0.001895 | 0.878642 | 0.686043 | PASS |
| 30 | 0.001119 | 0.000886 | 0.00184 | 0.000857 | 0.002109 | 1.184974 | 0.925227 | PASS |

### T11 per-seed social comparison and housing

Baseline is the same-seed 10-day instrumented P5 census (`p2-1_census_before`), which reproduces P5 behavior. Ratio ≥ 0.5 is the strict per-seed check; > 0 must hold in every seed. The 30-day ratios use that 10-day baseline rate and therefore compare different horizons.

| Horizon | Social ratio median | p10 | p90 | Min | Max | Seeds below 0.5 | Zero-social seeds | Overflow median | Overflow max | Seeds overflow >6 |
|---|---:|---:|---:|---:|---:|---|---|---:|---:|---:|
| 10 | 1.073952 | 0.806343 | 1.428681 | 0.379304 | 1.694778 | [8] | [] | 22 | 35 | 64/64 |
| 30 | 1.00323 | 0.74297 | 1.504474 | 0.271788 | 1.540145 | [8] | [] | 19 | 33 | 16/16 |

| Seed | P5 pairs/agent-day | 10-day rate | 10-day ratio | 30-day rate | 30-day ratio |
|---|---:|---:|---:|---:|---:|
| 0 | 0.528624 | 0.537715 | 1.017196 | 0.523301 | 0.98993 |
| 1 | 0.441577 | 0.660022 | 1.494694 | 0.680092 | 1.540145 |
| 2 | 0.43935 | 0.566699 | 1.289858 | 0.529188 | 1.204479 |
| 3 | 0.38717 | 0.498372 | 1.287219 | 0.452688 | 1.169225 |
| 4 | 0.555695 | 0.557785 | 1.003761 | 0.50613 | 0.910806 |
| 5 | 0.325048 | 0.465348 | 1.43163 | 0.484926 | 1.491862 |
| 6 | 0.422391 | 0.455036 | 1.077284 | 0.412859 | 0.977432 |
| 7 | 0.646594 | 0.505641 | 0.782008 | 0.540056 | 0.835232 |
| 8 | 1.529827 | 0.580269 | 0.379304 | 0.415789 | 0.271788 |
| 9 | 0.577654 | 0.583447 | 1.010027 | 0.514902 | 0.891368 |
| 10 | 0.421159 | 0.414022 | 0.983052 | 0.418408 | 0.993466 |
| 11 | 0.364036 | 0.517333 | 1.421102 | 0.507757 | 1.3948 |
| 12 | 0.504603 | 0.566653 | 1.122968 | 0.51116 | 1.012993 |
| 13 | 0.560406 | 0.319607 | 0.570314 | 0.364661 | 0.650708 |
| 14 | 0.370148 | 0.541108 | 1.461869 | 0.545958 | 1.474972 |
| 15 | 0.440259 | 0.660325 | 1.499858 | 0.66791 | 1.517086 |
| 16 | 0.613262 | 0.463619 | 0.755989 | — | — |
| 17 | 0.546652 | 0.524356 | 0.959213 | — | — |
| 18 | 0.478338 | 0.570836 | 1.193374 | — | — |
| 19 | 0.644813 | 0.481834 | 0.747245 | — | — |
| 20 | 0.442827 | 0.629611 | 1.4218 | — | — |
| 21 | 0.560635 | 0.46342 | 0.826598 | — | — |
| 22 | 0.575021 | 0.700322 | 1.217908 | — | — |
| 23 | 0.476475 | 0.512615 | 1.075849 | — | — |
| 24 | 0.645362 | 0.529866 | 0.821036 | — | — |
| 25 | 0.456581 | 0.472211 | 1.034231 | — | — |
| 26 | 0.554944 | 0.494066 | 0.8903 | — | — |
| 27 | 0.428194 | 0.55617 | 1.298876 | — | — |
| 28 | 0.415094 | 0.676324 | 1.629327 | — | — |
| 29 | 0.491367 | 0.508552 | 1.034974 | — | — |
| 30 | 0.496559 | 0.317283 | 0.638964 | — | — |
| 31 | 0.512546 | 0.549477 | 1.072056 | — | — |
| 32 | 0.53927 | 0.431441 | 0.800046 | — | — |
| 33 | 0.590741 | 0.603444 | 1.021504 | — | — |
| 34 | 0.468407 | 0.445344 | 0.950763 | — | — |
| 35 | 0.476847 | 0.431653 | 0.905224 | — | — |
| 36 | 0.487101 | 0.413151 | 0.848184 | — | — |
| 37 | 0.41802 | 0.449899 | 1.076261 | — | — |
| 38 | 0.466851 | 0.546304 | 1.17019 | — | — |
| 39 | 0.382468 | 0.458762 | 1.19948 | — | — |
| 40 | 0.645183 | 0.636014 | 0.985788 | — | — |
| 41 | 0.457465 | 0.697348 | 1.524374 | — | — |
| 42 | 0.555547 | 0.51925 | 0.934665 | — | — |
| 43 | 0.388323 | 0.514141 | 1.324005 | — | — |
| 44 | 0.48294 | 0.453188 | 0.938394 | — | — |
| 45 | 0.645944 | 0.644463 | 0.997707 | — | — |
| 46 | 0.445382 | 0.466588 | 1.047614 | — | — |
| 47 | 0.371438 | 0.419403 | 1.129136 | — | — |
| 48 | 0.437912 | 0.36928 | 0.843275 | — | — |
| 49 | 0.543516 | 0.616363 | 1.134029 | — | — |
| 50 | 0.474179 | 0.633037 | 1.335017 | — | — |
| 51 | 0.558987 | 0.471474 | 0.843443 | — | — |
| 52 | 0.383992 | 0.481074 | 1.252824 | — | — |
| 53 | 0.381422 | 0.484982 | 1.27151 | — | — |
| 54 | 0.295777 | 0.374942 | 1.267653 | — | — |
| 55 | 0.396512 | 0.671999 | 1.694778 | — | — |
| 56 | 0.448755 | 0.547803 | 1.220717 | — | — |
| 57 | 0.408259 | 0.515946 | 1.263771 | — | — |
| 58 | 0.490726 | 0.605321 | 1.23352 | — | — |
| 59 | 0.430923 | 0.526724 | 1.222314 | — | — |
| 60 | 0.551806 | 0.546412 | 0.990224 | — | — |
| 61 | 0.421888 | 0.471321 | 1.11717 | — | — |
| 62 | 0.492304 | 0.417893 | 0.848852 | — | — |
| 63 | 0.480685 | 0.464554 | 0.966441 | — | — |

T11 median-rate preservation passes and all seeds have positive social rates, but the stricter per-seed ≥50% comparison FAILS for seed 8 at both horizons (ratios 0.379304 and 0.271788). T9 PASS against both timing baselines at both horizons.

### 30-day population trajectory

| Day | Median alive | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| 1 | 33 | 29.5 | 34.5 | 28 | 35 |
| 2 | 42.5 | 37.5 | 45.5 | 35 | 47 |
| 3 | 51 | 44.5 | 54.5 | 41 | 56 |
| 4 | 57.5 | 50.5 | 61.5 | 45 | 62 |
| 5 | 58 | 52.5 | 62 | 42 | 65 |
| 6 | 58.5 | 50.5 | 61.5 | 45 | 65 |
| 7 | 55.5 | 49.5 | 64 | 39 | 66 |
| 8 | 55.5 | 47 | 61 | 40 | 66 |
| 9 | 52 | 47 | 62.5 | 40 | 65 |
| 10 | 50.5 | 47 | 58.5 | 42 | 60 |
| 11 | 51.5 | 45.5 | 57 | 44 | 62 |
| 12 | 51 | 46.5 | 56.5 | 45 | 57 |
| 13 | 50.5 | 47.5 | 56.5 | 43 | 60 |
| 14 | 51 | 45.5 | 56.5 | 42 | 65 |
| 15 | 52 | 45.5 | 56 | 40 | 67 |
| 16 | 52.5 | 44 | 54 | 39 | 60 |
| 17 | 51.5 | 45.5 | 55 | 39 | 55 |
| 18 | 52 | 42.5 | 55.5 | 39 | 60 |
| 19 | 50.5 | 44 | 54 | 39 | 58 |
| 20 | 51 | 45 | 54.5 | 40 | 57 |
| 21 | 49 | 44 | 55 | 39 | 56 |
| 22 | 49 | 42.5 | 56.5 | 37 | 58 |
| 23 | 51 | 44.5 | 56 | 39 | 59 |
| 24 | 53 | 45 | 56.5 | 41 | 57 |
| 25 | 52 | 43.5 | 55.5 | 40 | 60 |
| 26 | 51 | 45.5 | 54.5 | 39 | 56 |
| 27 | 50.5 | 46.5 | 53.5 | 42 | 56 |
| 28 | 51.5 | 46.5 | 55 | 43 | 59 |
| 29 | 52 | 44.5 | 57 | 40 | 64 |
| 30 | 50.5 | 44.5 | 59.5 | 39 | 60 |

Food context: eight renewable bushes add at most 96 food/day; founder and child hunger growth requires about 2.4 meals/resident/day. Thus ~40 residents is a food-backed steady-state benchmark before birth costs and delivery losses, not a hard runtime ceiling: legacy ground consumption can relieve hunger after another agent depleted its tile. Gathering only moves food. Initial three shelters × four residents supply capacity 12 for 24 founders, creating overflow 12 before growth. This readout motivates authorized tuning; it does not establish that any proposed configuration meets the targets.

## Pass 2 Part 3: original tuning round 1 (before critical-preemption correction)

Frozen original R1 measurements: [64-seed 10-day summary](probe/p2-3_round1/summary.json) and [16-seed 30-day summary](probe/p2-3_round1_30day/summary.json). Knobs: birth cooldown 1440, food cost 3, residents/shelter 6, social chase 240, retry 480, critical preemption 75. These results precede the upcoming critical-preemption implementation bug fix and must not be presented as final corrected-implementation measurements.

Both summaries enable independent determinism twins (64 and 16 completed runs). T8 probe determinism passes; legacy-pin verification remains separately required. Leader reports the unchanged `harness_population_game_world_survives_3_days` PASS for this original R1.

Target verdicts: T1 FAIL, T3 PASS, T2 PASS, T4 PASS, T10 FAIL, T5 PASS, T6a PASS, T7 PASS. See quantiles and social/timing comparisons below.

| Horizon | Metric | Median | p10 | p90 | Min | Max |
|---|---|---:|---:|---:|---:|---:|
| 10 days | alive | 49 | 41.3 | 55 | 37 | 58 |
| 10 days | births | 25 | 18 | 31 | 13 | 34 |
| 10 days | deaths_dehydration | 0 | 0 | 0 | 0 | 3 |
| 10 days | deaths_starvation | 0 | 0 | 0 | 0 | 8 |
| 10 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 10 days | shelters_completed | 3 | 1 | 5.7 | 1 | 9 |
| 10 days | housing_capacity | 36 | 24 | 52.2 | 24 | 72 |
| 10 days | idle_breach_actionable_share | 0.002611 | 0.001691 | 0.004296 | 0.001021 | 0.006324 |
| 10 days | idle_breach_unavailable_share | 0.071009 | 0.037072 | 0.128391 | 0.008852 | 0.153049 |
| 10 days | idle_with_breach_share | 0.071415 | 0.03723 | 0.129993 | 0.008852 | 0.155121 |
| 10 days | home_distance_mean | 3.854827 | 2.955841 | 4.99359 | 2.117647 | 6.706897 |
| 10 days | stockpile_food | 136.5 | 56.8 | 235.8 | 25 | 288 |
| 10 days | food_tiles_total | 0 | 0 | 1 | 0 | 3 |
| 10 days | social_completed_per_agent_day | 0.798386 | 0.717621 | 0.821387 | 0.637124 | 0.831723 |
| 10 days | births minus deaths | 25 | 17.3 | 31 | 13 | 34 |
| 30 days | alive | 50 | 46.5 | 60.5 | 41 | 66 |
| 30 days | births | 62 | 48.5 | 83.5 | 37 | 88 |
| 30 days | deaths_dehydration | 0 | 0 | 14 | 0 | 33 |
| 30 days | deaths_starvation | 34 | 4.5 | 54.5 | 2 | 57 |
| 30 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 30 days | shelters_completed | 6 | 4.5 | 9 | 2 | 9 |
| 30 days | housing_capacity | 54 | 45 | 72 | 30 | 72 |
| 30 days | idle_breach_actionable_share | 0.002941 | 0.002013 | 0.005098 | 0.001018 | 0.00872 |
| 30 days | idle_breach_unavailable_share | 0.193432 | 0.159085 | 0.234194 | 0.10285 | 0.256865 |
| 30 days | idle_with_breach_share | 0.194839 | 0.160341 | 0.235331 | 0.105004 | 0.258527 |
| 30 days | home_distance_mean | 3.535957 | 2.880256 | 4.998051 | 2.645833 | 5.196078 |
| 30 days | stockpile_food | 1 | 0 | 125 | 0 | 257 |
| 30 days | food_tiles_total | 0 | 0 | 0 | 0 | 0 |
| 30 days | social_completed_per_agent_day | 0.731019 | 0.696891 | 0.763534 | 0.657503 | 0.837708 |
| 30 days | births minus deaths | 26 | 22.5 | 36.5 | 17 | 42 |

Net births minus deaths uses the unchanged 24-founder population accounting identity; it does not subtract independently calculated medians.

### Original R1 timing, sociality and housing

Timing excludes the read-only census observer; measurements remain sensitive to DGX load.

| Horizon | Median ms/agent-tick | p10 | p90 | Min | Max | / original P5 | / instrumented P5 | T9 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 10 | 0.000794 | 0.000698 | 0.00173 | 0.00064 | 0.001852 | 0.84095 | 0.656613 | PASS |
| 30 | 0.000917 | 0.000757 | 0.00113 | 0.000708 | 0.001277 | 0.970354 | 0.757651 | PASS |

Same-seed social ratios use `p2-1_census_before` 10-day P5 rates. Thirty-day comparisons therefore span different horizons.

| Horizon | Ratio median | p10 | p90 | Min | Max | Seeds below 0.5 | Zero-social seeds | T11 per-seed | Overflow median | Overflow min | Overflow max | Seeds overflow >6 |
|---|---:|---:|---:|---:|---:|---|---|---|---:|---:|---:|---:|
| 10 | 1.659749 | 1.27678 | 2.055645 | 0.533029 | 2.46637 | [] | [] | PASS | 16.5 | 6 | 29 | 57/64 |
| 30 | 1.706465 | 1.187618 | 2.053121 | 0.497665 | 2.26317 | [8] | [] | FAIL | 14.5 | 6 | 33 | 15/16 |

### Original R1 30-day population trajectory

| Day | Median alive | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| 1 | 24 | 24 | 24 | 24 | 24 |
| 2 | 27 | 26 | 28 | 26 | 28 |
| 3 | 30 | 28 | 31 | 28 | 31 |
| 4 | 33 | 30 | 34 | 29 | 35 |
| 5 | 36 | 32 | 37.5 | 31 | 39 |
| 6 | 39 | 34 | 41 | 32 | 43 |
| 7 | 42 | 35.5 | 44 | 33 | 46 |
| 8 | 44.5 | 37.5 | 47 | 34 | 49 |
| 9 | 48 | 40 | 50 | 36 | 53 |
| 10 | 50 | 41.5 | 54 | 37 | 57 |
| 11 | 52 | 43.5 | 57.5 | 39 | 60 |
| 12 | 53.5 | 44.5 | 60 | 41 | 63 |
| 13 | 52.5 | 46 | 62.5 | 42 | 66 |
| 14 | 54 | 45.5 | 62 | 43 | 64 |
| 15 | 54.5 | 44 | 62.5 | 44 | 64 |
| 16 | 53 | 44.5 | 59 | 42 | 64 |
| 17 | 53.5 | 42 | 59 | 37 | 59 |
| 18 | 51.5 | 42.5 | 59.5 | 36 | 61 |
| 19 | 51.5 | 43.5 | 61 | 37 | 63 |
| 20 | 54 | 44 | 61 | 39 | 64 |
| 21 | 52 | 46 | 57.5 | 41 | 65 |
| 22 | 52 | 45 | 58.5 | 40 | 67 |
| 23 | 53 | 45 | 59 | 41 | 66 |
| 24 | 53 | 46 | 61 | 42 | 68 |
| 25 | 51.5 | 46 | 61 | 41 | 68 |
| 26 | 50 | 46.5 | 60.5 | 42 | 69 |
| 27 | 50.5 | 46 | 59 | 42 | 70 |
| 28 | 51 | 46.5 | 58.5 | 40 | 64 |
| 29 | 50 | 45.5 | 60 | 39 | 65 |
| 30 | 50 | 46.5 | 60.5 | 41 | 66 |

## Pass 2 Part 3: corrected tuning round 1

Sources: [64-seed 10-day summary](probe/p2-3_round1_corrected/summary.json), [16-seed 30-day summary](probe/p2-3_round1_corrected_30day/summary.json). Same R1 knobs: birth cooldown 1440, food cost 3, residents/shelter 6, social chase 240, retry 480, critical preemption 75. Implementation correction protects an agent already addressing its critical current need from being repeatedly interrupted. These measurements supersede original R1 for this corrected implementation; original measurements above remain historical evidence.

Leader reports 64/64 and 16/16 independent twins PASS. Legacy pin and unchanged three-day survival remain separate verification; original R1 survival PASS must not be assumed to establish this corrected source.

Target verdicts: T1 PASS, T3 PASS, T2 PASS, T4 PASS, T10 FAIL, T5 PASS, T6a PASS, T7 PASS.

| Horizon | Metric | Median | p10 | p90 | Min | Max |
|---|---|---:|---:|---:|---:|---:|
| 10 days | alive | 49 | 42 | 55 | 37 | 58 |
| 10 days | births | 25 | 18 | 31 | 13 | 34 |
| 10 days | deaths_dehydration | 0 | 0 | 0 | 0 | 0 |
| 10 days | deaths_starvation | 0 | 0 | 0.7 | 0 | 6 |
| 10 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 10 days | shelters_completed | 3 | 1 | 6 | 1 | 12 |
| 10 days | housing_capacity | 36 | 24 | 54 | 24 | 90 |
| 10 days | idle_breach_actionable_share | 0.002622 | 0.001646 | 0.004296 | 0.001021 | 0.006324 |
| 10 days | idle_breach_unavailable_share | 0.071009 | 0.037072 | 0.122651 | 0.008852 | 0.159343 |
| 10 days | idle_with_breach_share | 0.071415 | 0.03723 | 0.123426 | 0.008852 | 0.159806 |
| 10 days | home_distance_mean | 3.810727 | 2.925877 | 4.99359 | 2.117647 | 6.568966 |
| 10 days | stockpile_food | 133.5 | 61.3 | 245.6 | 25 | 288 |
| 10 days | food_tiles_total | 0 | 0 | 1 | 0 | 3 |
| 10 days | social_completed_per_agent_day | 0.798386 | 0.706332 | 0.821431 | 0.628495 | 0.831723 |
| 10 days | births minus deaths | 25 | 18 | 31 | 13 | 34 |
| 30 days | alive | 49.5 | 47 | 56.5 | 46 | 74 |
| 30 days | births | 64.5 | 47.5 | 78.5 | 37 | 89 |
| 30 days | deaths_dehydration | 0 | 0 | 0 | 0 | 0 |
| 30 days | deaths_starvation | 39 | 15.5 | 54.5 | 12 | 57 |
| 30 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 30 days | shelters_completed | 6 | 4.5 | 11.5 | 4 | 15 |
| 30 days | housing_capacity | 54 | 45 | 87 | 42 | 108 |
| 30 days | idle_breach_actionable_share | 0.002472 | 0.001823 | 0.004603 | 0.001382 | 0.00872 |
| 30 days | idle_breach_unavailable_share | 0.200912 | 0.164952 | 0.233457 | 0.108044 | 0.256865 |
| 30 days | idle_with_breach_share | 0.202395 | 0.165903 | 0.235711 | 0.110424 | 0.258527 |
| 30 days | home_distance_mean | 3.574348 | 2.872628 | 4.432245 | 2.645833 | 5.276596 |
| 30 days | stockpile_food | 0 | 0 | 99 | 0 | 127 |
| 30 days | food_tiles_total | 0 | 0 | 0 | 0 | 0 |
| 30 days | social_completed_per_agent_day | 0.730986 | 0.675932 | 0.760377 | 0.664938 | 0.837708 |
| 30 days | births minus deaths | 25.5 | 23 | 32.5 | 22 | 50 |

Net population change uses alive minus 24 founders.

### Corrected R1 timing, sociality and housing

| Horizon | Median ms/agent-tick | p10 | p90 | Min | Max | / original P5 | / instrumented P5 | T9 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 10 | 0.000852 | 0.000719 | 0.001281 | 0.000671 | 0.001908 | 0.901976 | 0.704262 | PASS |
| 30 | 0.001014 | 0.000819 | 0.001153 | 0.000726 | 0.001268 | 1.073213 | 0.837964 | PASS |

Timing excludes census observer time and remains host-load sensitive. Social baseline is same-seed `p2-1_census_before` 10-day P5; 30-day ratios compare different horizons.

| Horizon | Ratio median | p10 | p90 | Min | Max | Seeds below 0.5 | Zero-social seeds | T11 per-seed | Overflow median | Overflow min | Overflow max | Seeds overflow >6 |
|---|---:|---:|---:|---:|---:|---|---|---|---:|---:|---:|---:|
| 10 | 1.666403 | 1.279671 | 2.055645 | 0.533029 | 2.518003 | [] | [] | PASS | 16.5 | 6 | 29 | 56/64 |
| 30 | 1.691518 | 1.159918 | 2.004213 | 0.482465 | 2.26317 | [8] | [] | FAIL | 15.5 | 6 | 33 | 15/16 |

### Corrected R1 30-day population trajectory

| Day | Median alive | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| 1 | 24 | 24 | 24 | 24 | 24 |
| 2 | 27 | 26 | 28 | 26 | 28 |
| 3 | 30 | 28 | 31 | 28 | 31 |
| 4 | 33 | 30 | 34 | 29 | 35 |
| 5 | 36 | 32 | 37.5 | 31 | 39 |
| 6 | 39 | 34 | 41 | 32 | 43 |
| 7 | 42 | 35.5 | 44 | 33 | 46 |
| 8 | 44.5 | 37 | 47 | 34 | 49 |
| 9 | 47.5 | 39.5 | 50 | 36 | 53 |
| 10 | 49.5 | 41 | 54 | 37 | 57 |
| 11 | 51.5 | 43.5 | 57.5 | 39 | 59 |
| 12 | 53.5 | 44.5 | 60 | 41 | 61 |
| 13 | 54 | 45.5 | 62.5 | 42 | 64 |
| 14 | 55.5 | 45 | 62.5 | 43 | 64 |
| 15 | 55 | 44 | 61.5 | 44 | 65 |
| 16 | 54.5 | 45 | 63 | 44 | 65 |
| 17 | 54.5 | 45 | 61 | 40 | 67 |
| 18 | 53 | 45.5 | 61 | 41 | 66 |
| 19 | 53.5 | 45.5 | 62 | 42 | 70 |
| 20 | 53.5 | 45 | 63 | 44 | 70 |
| 21 | 52 | 48.5 | 61 | 44 | 71 |
| 22 | 52.5 | 47.5 | 60 | 43 | 66 |
| 23 | 52 | 47.5 | 59 | 44 | 63 |
| 24 | 52 | 47 | 59 | 46 | 63 |
| 25 | 50.5 | 46 | 61 | 45 | 65 |
| 26 | 50 | 46.5 | 62 | 46 | 66 |
| 27 | 49.5 | 46.5 | 61 | 46 | 69 |
| 28 | 51 | 48 | 56 | 45 | 70 |
| 29 | 49 | 45.5 | 56 | 45 | 73 |
| 30 | 49.5 | 47 | 56.5 | 46 | 74 |

## Pass 2 Part 3: tuning round 2

Sources: [64-seed 10-day summary](probe/p2-3_round2/summary.json), [16-seed 30-day summary](probe/p2-3_round2_30day/summary.json). R2 knobs: birth cooldown 1440, food cost 6, residents/shelter 8, social chase 240, retry 480, critical preemption 75. Retains the corrected critical-preemption implementation; this is the second authorized tuning round, not the final configuration selection.

Both probe logs end in DETERMINISM PASS (64/64 and 16/16 twins). The unchanged harness_population_game_world_survives_3_days PASSED: .harness/evidence/revitalize-pass2/logs/p3-round2-survival.log reports 1 passed, 0 failed, 0 ignored. Legacy pin remains a separate gate.

Target verdicts: T1 PASS, T3 PASS, T2 PASS, T4 PASS, T10 FAIL, T5 FAIL, T6a PASS, T7 PASS.

| Horizon | Metric | Median | p10 | p90 | Min | Max |
|---|---|---:|---:|---:|---:|---:|
| 10 days | alive | 43 | 37.6 | 49 | 33 | 51 |
| 10 days | births | 19.5 | 13.6 | 25 | 9 | 27 |
| 10 days | deaths_dehydration | 0 | 0 | 0 | 0 | 0 |
| 10 days | deaths_starvation | 0 | 0 | 0 | 0 | 5 |
| 10 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 10 days | shelters_completed | 1 | 1 | 2 | 1 | 3 |
| 10 days | housing_capacity | 32 | 32 | 40 | 32 | 48 |
| 10 days | idle_breach_actionable_share | 0.002805 | 0.00161 | 0.004194 | 0.000274 | 0.006621 |
| 10 days | idle_breach_unavailable_share | 0.068032 | 0.038726 | 0.119606 | 0.011824 | 0.151243 |
| 10 days | idle_with_breach_share | 0.069522 | 0.039148 | 0.119993 | 0.012327 | 0.151953 |
| 10 days | home_distance_mean | 4 | 3.184436 | 4.844879 | 2.294118 | 6.976744 |
| 10 days | stockpile_food | 139 | 31.3 | 240.6 | 4 | 321 |
| 10 days | food_tiles_total | 0 | 0 | 0.7 | 0 | 2 |
| 10 days | social_completed_per_agent_day | 0.801254 | 0.713196 | 0.82803 | 0.588903 | 0.835644 |
| 10 days | births minus deaths | 19 | 13.6 | 25 | 9 | 27 |
| 30 days | alive | 47.5 | 45 | 57 | 40 | 62 |
| 30 days | births | 41 | 31 | 50 | 30 | 57 |
| 30 days | deaths_dehydration | 0 | 0 | 0 | 0 | 1 |
| 30 days | deaths_starvation | 18 | 0 | 27.5 | 0 | 33 |
| 30 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 30 days | shelters_completed | 2.5 | 1 | 4 | 1 | 7 |
| 30 days | housing_capacity | 44 | 32 | 56 | 32 | 80 |
| 30 days | idle_breach_actionable_share | 0.002258 | 0.00153 | 0.004582 | 0.001158 | 0.009036 |
| 30 days | idle_breach_unavailable_share | 0.16445 | 0.104025 | 0.192313 | 0.087889 | 0.20054 |
| 30 days | idle_with_breach_share | 0.165886 | 0.104255 | 0.19317 | 0.091355 | 0.201106 |
| 30 days | home_distance_mean | 3.914943 | 3.381019 | 5.545 | 3.193548 | 7.106383 |
| 30 days | stockpile_food | 3 | 0 | 163 | 0 | 241 |
| 30 days | food_tiles_total | 0 | 0 | 0 | 0 | 2 |
| 30 days | social_completed_per_agent_day | 0.78577 | 0.716227 | 0.858284 | 0.631166 | 0.881809 |
| 30 days | births minus deaths | 23.5 | 21 | 33 | 16 | 38 |

Net population change uses alive minus 24 founders.

### R2 timing, sociality and housing

| Horizon | Median ms/agent-tick | p10 | p90 | Min | Max | / original P5 | / instrumented P5 | T9 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 10 | 0.000863 | 0.000717 | 0.001371 | 0.000658 | 0.001582 | 0.91318 | 0.71301 | PASS |
| 30 | 0.000978 | 0.000909 | 0.001327 | 0.000749 | 0.00162 | 1.034902 | 0.808051 | PASS |

Timing excludes census observer time and remains host-load sensitive. Social baseline is same-seed `p2-1_census_before` 10-day P5; 30-day ratios compare different horizons.

| Horizon | Ratio median | p10 | p90 | Min | Max | Seeds below 0.5 | Zero-social seeds | T11 per-seed | Overflow median | Overflow min | Overflow max | Seeds overflow >6 |
|---|---:|---:|---:|---:|---:|---|---|---|---:|---:|---:|---:|
| 10 | 1.684004 | 1.239503 | 2.016268 | 0.542235 | 2.481729 | [] | [] | PASS | 11 | 0 | 19 | 48/64 |
| 30 | 1.793039 | 1.178265 | 2.151938 | 0.556675 | 2.382314 | [] | [] | PASS | 13.5 | 4 | 26 | 15/16 |

### R2 30-day population trajectory

| Day | Median alive | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| 1 | 24 | 24 | 24 | 24 | 24 |
| 2 | 27 | 26 | 27 | 25 | 27 |
| 3 | 29.5 | 27 | 30 | 27 | 30 |
| 4 | 32 | 28 | 33 | 28 | 33 |
| 5 | 34 | 29.5 | 36 | 29 | 36 |
| 6 | 36.5 | 31 | 39 | 30 | 39 |
| 7 | 38.5 | 32.5 | 42 | 31 | 42 |
| 8 | 40.5 | 34 | 45 | 32 | 45 |
| 9 | 42.5 | 35 | 47.5 | 33 | 48 |
| 10 | 44 | 36 | 49.5 | 34 | 50 |
| 11 | 45 | 37.5 | 50.5 | 35 | 52 |
| 12 | 46.5 | 38.5 | 52.5 | 36 | 54 |
| 13 | 48 | 39.5 | 54.5 | 37 | 55 |
| 14 | 48 | 38.5 | 55.5 | 37 | 56 |
| 15 | 46 | 39.5 | 56 | 38 | 57 |
| 16 | 47 | 40.5 | 55.5 | 39 | 58 |
| 17 | 46.5 | 41.5 | 55 | 40 | 58 |
| 18 | 46.5 | 42.5 | 52.5 | 41 | 59 |
| 19 | 47.5 | 43.5 | 54 | 40 | 58 |
| 20 | 48 | 44.5 | 55 | 39 | 58 |
| 21 | 48.5 | 45 | 55 | 40 | 57 |
| 22 | 49 | 46 | 55.5 | 41 | 58 |
| 23 | 48.5 | 46 | 55.5 | 42 | 58 |
| 24 | 48.5 | 47 | 55.5 | 41 | 58 |
| 25 | 49 | 45.5 | 54.5 | 42 | 59 |
| 26 | 48.5 | 44 | 56 | 42 | 60 |
| 27 | 48 | 45.5 | 55.5 | 42 | 61 |
| 28 | 48 | 45.5 | 56 | 42 | 61 |
| 29 | 47.5 | 46 | 56.5 | 41 | 61 |
| 30 | 47.5 | 45 | 57 | 40 | 62 |

## Pass 2 Part 3: tuning round 3

Sources: [64-seed 10-day summary](probe/p2-3_round3/summary.json), [16-seed 30-day summary](probe/p2-3_round3_30day/summary.json). R3 knobs: birth cooldown 1440, food cost 10, residents/shelter 8, social chase 240, retry 480, critical preemption 75. Retains the corrected critical-preemption implementation; this is the third and final authorized tuning round, not the final configuration selection.

Both probe logs end in DETERMINISM PASS (64/64 and 16/16 twins). The unchanged harness_population_game_world_survives_3_days PASSED: .harness/evidence/revitalize-pass2/logs/p3-round3-survival.log reports 1 passed, 0 failed, 0 ignored. Legacy pin remains a separate gate.

Target verdicts: T1 PASS, T3 PASS, T2 PASS, T4 PASS, T10 FAIL, T5 FAIL, T6a PASS, T7 PASS.

| Horizon | Metric | Median | p10 | p90 | Min | Max |
|---|---|---:|---:|---:|---:|---:|
| 10 days | alive | 42 | 35 | 45 | 33 | 47 |
| 10 days | births | 18 | 11 | 21 | 9 | 23 |
| 10 days | deaths_dehydration | 0 | 0 | 0 | 0 | 0 |
| 10 days | deaths_starvation | 0 | 0 | 0 | 0 | 4 |
| 10 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 10 days | shelters_completed | 1 | 1 | 2 | 1 | 3 |
| 10 days | housing_capacity | 32 | 32 | 40 | 32 | 48 |
| 10 days | idle_breach_actionable_share | 0.003001 | 0.001642 | 0.004384 | 0.000274 | 0.005689 |
| 10 days | idle_breach_unavailable_share | 0.065959 | 0.031619 | 0.104971 | 0.011824 | 0.1361 |
| 10 days | idle_with_breach_share | 0.066205 | 0.03169 | 0.105301 | 0.012327 | 0.138272 |
| 10 days | home_distance_mean | 3.905869 | 3.238038 | 4.95122 | 2.590909 | 7.05 |
| 10 days | stockpile_food | 95.5 | 16.6 | 213.8 | 0 | 285 |
| 10 days | food_tiles_total | 0 | 0 | 1 | 0 | 2 |
| 10 days | social_completed_per_agent_day | 0.801917 | 0.740513 | 0.831295 | 0.666354 | 0.84263 |
| 10 days | births minus deaths | 18 | 11 | 21 | 9 | 23 |
| 30 days | alive | 48 | 43 | 53.5 | 40 | 56 |
| 30 days | births | 29.5 | 26.5 | 38 | 25 | 49 |
| 30 days | deaths_dehydration | 0 | 0 | 0 | 0 | 1 |
| 30 days | deaths_starvation | 6.5 | 0 | 19 | 0 | 23 |
| 30 days | deaths_combat | 0 | 0 | 0 | 0 | 0 |
| 30 days | shelters_completed | 2 | 1 | 3.5 | 1 | 6 |
| 30 days | housing_capacity | 40 | 32 | 52 | 32 | 72 |
| 30 days | idle_breach_actionable_share | 0.002697 | 0.00169 | 0.004721 | 0.001245 | 0.008759 |
| 30 days | idle_breach_unavailable_share | 0.160663 | 0.083277 | 0.183354 | 0.065249 | 0.186642 |
| 30 days | idle_with_breach_share | 0.162058 | 0.085354 | 0.185462 | 0.065526 | 0.19088 |
| 30 days | home_distance_mean | 4.089184 | 3.230839 | 4.902123 | 2.603774 | 6.413043 |
| 30 days | stockpile_food | 5.5 | 0 | 19.5 | 0 | 153 |
| 30 days | food_tiles_total | 0 | 0 | 0 | 0 | 3 |
| 30 days | social_completed_per_agent_day | 0.808724 | 0.773398 | 0.855221 | 0.644889 | 0.85734 |
| 30 days | births minus deaths | 24 | 19 | 29.5 | 16 | 32 |

Net population change uses alive minus 24 founders.

### R3 timing, sociality and housing

| Horizon | Median ms/agent-tick | p10 | p90 | Min | Max | / original P5 | / instrumented P5 | T9 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 10 | 0.000802 | 0.000695 | 0.001255 | 0.000658 | 0.003827 | 0.848777 | 0.662725 | PASS |
| 30 | 0.000937 | 0.000799 | 0.001549 | 0.00079 | 0.001734 | 0.992288 | 0.774778 | PASS |

Timing excludes census observer time and remains host-load sensitive. Social baseline is same-seed `p2-1_census_before` 10-day P5; 30-day ratios compare different horizons.

| Horizon | Ratio median | p10 | p90 | Min | Max | Seeds below 0.5 | Zero-social seeds | T11 per-seed | Overflow median | Overflow min | Overflow max | Seeds overflow >6 |
|---|---:|---:|---:|---:|---:|---|---|---|---:|---:|---:|---:|
| 10 | 1.673252 | 1.298525 | 2.063828 | 0.549126 | 2.439453 | [] | [] | PASS | 9 | 0 | 15 | 44/64 |
| 30 | 1.819895 | 1.206261 | 2.212209 | 0.560362 | 2.403954 | [] | [] | PASS | 13.5 | 1 | 25 | 14/16 |

### R3 30-day population trajectory

| Day | Median alive | p10 | p90 | Min | Max |
|---|---:|---:|---:|---:|---:|
| 1 | 24 | 24 | 24 | 24 | 24 |
| 2 | 26.5 | 25 | 27 | 25 | 27 |
| 3 | 28.5 | 26 | 30 | 26 | 30 |
| 4 | 30.5 | 27 | 33 | 27 | 33 |
| 5 | 32.5 | 28.5 | 36 | 28 | 36 |
| 6 | 34.5 | 29.5 | 39 | 29 | 39 |
| 7 | 36.5 | 30.5 | 41 | 30 | 42 |
| 8 | 38.5 | 32 | 43 | 31 | 45 |
| 9 | 41 | 33 | 44.5 | 32 | 46 |
| 10 | 42.5 | 34.5 | 45 | 33 | 46 |
| 11 | 43.5 | 36 | 46 | 34 | 46 |
| 12 | 44.5 | 37 | 47.5 | 35 | 49 |
| 13 | 44.5 | 38 | 48 | 36 | 50 |
| 14 | 46 | 39 | 48.5 | 37 | 50 |
| 15 | 46 | 39.5 | 48.5 | 38 | 50 |
| 16 | 46.5 | 40.5 | 48.5 | 39 | 50 |
| 17 | 46.5 | 41.5 | 49.5 | 40 | 51 |
| 18 | 47 | 42 | 50.5 | 41 | 51 |
| 19 | 47 | 42.5 | 51.5 | 42 | 52 |
| 20 | 46 | 43 | 51.5 | 41 | 53 |
| 21 | 46 | 44 | 52.5 | 39 | 53 |
| 22 | 46.5 | 44 | 52.5 | 38 | 55 |
| 23 | 47.5 | 43.5 | 52 | 38 | 55 |
| 24 | 47.5 | 44 | 51 | 38 | 55 |
| 25 | 47.5 | 44.5 | 51 | 39 | 56 |
| 26 | 47 | 45 | 51 | 38 | 57 |
| 27 | 47 | 44.5 | 52 | 38 | 57 |
| 28 | 47.5 | 43 | 53 | 38 | 58 |
| 29 | 47 | 43 | 53.5 | 39 | 58 |
| 30 | 48 | 43 | 53.5 | 40 | 56 |

### Part 3 retained configuration and boundary

Retain R3: cooldown 1440, food cost 10, residents/shelter 8, social chase 240, retry 480, critical preemption 75, corrected current-critical-need protection. This is a priority-ordered choice, not selection by total passed-target count. All three corrected configurations pass the higher-priority T8 probe twins, T1, T3, T2 and T4. R3 has the smallest 10-day starvation tail and, among the configurations that still fail T10, the smallest observed housing-overflow maxima. These higher-priority improvements take precedence over corrected R1 passing lower-priority T5. Legacy pin remains a separately executed gate.

| Configuration | 10-day starvation median | p90 | Max | 30-day starvation median | Housing overflow max 10d / 30d | 10-day shelters completed median | T5 |
|---|---:|---:|---:|---:|---:|---:|---|
| Corrected R1 | 0.0 | 0.7 | 6.0 | 39.0 | 29 / 33 | 3.0 | PASS |
| R2 | 0.0 | 0 | 5.0 | 18.0 | 19 / 26 | 1.0 | FAIL |
| R3 retained | 0.0 | 0 | 4.0 | 6.5 | 15 / 25 | 1.0 | FAIL |

Remaining target failures: T10 housing overflow exceeds +6, with zero dehydration deaths across all 64 retained-R3 10-day seeds and one total dehydration death across the 16 retained-R3 30-day seeds (median 0, max 1); T5 median shelters completed is 1, below 2. T11 strict per-seed comparisons and T9 timing results remain recorded separately above; passing aggregate social rates must not conceal a failing individual seed.

Retained R3 10-day strict T11: PASS; below-half-baseline seeds []; zero-social seeds [].
Retained R3 30-day strict T11: PASS; below-half-baseline seeds []; zero-social seeds [].

The unchanged three-day survival harness passes R3 (1 passed, 0 failed, 0 ignored), verified from `.harness/evidence/revitalize-pass2/logs/p3-round3-survival.log`; both R3 probe logs end in `DETERMINISM PASS`. No fourth tuning round. Night-rest stretch is skipped because not every required target passes.

Part 3 readout boundary recorded 2026-10-10 08:16:04 UTC / 17:16:04 KST, elapsed 26 minutes 18 seconds from Pass 2 start 16:49:46 KST. The eight-hour deadline is 2026-10-11 00:49:46 KST; not reached.


## Pass 2 Part 6 — final code verification

Code commit: `0ec143c027a2570ac97f7b5f78356345f892f0cb`. Recorded at 2026-10-10T09:11:34+00:00 UTC / 2026-10-10T18:11:34+09:00 KST. p6-full PASS: debug/release each 1806 passed, zero failed/ignored across 114 groups; both Clippy commands, forbidden math (zero), bridge, import, strict (17 files), FFI (14 calls), smoke 900, and diff check all exited 0. Evidence: `.harness/evidence/revitalize-pass2/logs/p6-full/status.tsv`.

Final twins: legacy 4/4 and game 64/64 PASS. `p6-probe-comparison.json`: legacy 484 old behavior rows = P5; game 7744 behavior rows plus 11 death rows = R3. Behavior distributions above are unchanged; 30-day results remain the prior 16-seed R3 cohort, not a fresh 30-day run.

Timing ms/agent-tick: median / p10 / p90 / min / max.

| Dataset | Distribution |
|---|---|
| p2-6_final_legacy | 0.009940629997 / 0.007255788611 / 0.011748330428 / 0.006829124056 / 0.011799077431 |
| p2-6_final_game | 0.000809877367 / 0.000724790290 / 0.000863759264 / 0.000662099180 / 0.001204102904 |

Game median / original P5 = 0.857378674; / instrumented P5 = 0.669440611. T9 passes. Observer time is excluded; host load still varies. T5 and T10 remain FAIL; mutation raw kill rate remains 593/597 (99.329983%). Remote WIP and green both verified at `0ec143c027a2570ac97f7b5f78356345f892f0cb` after the full gate. This is the code verification boundary before report publication; the leader separately verifies exact publication HEAD and fresh raw probes.

## Pass 3

### Part 0 — frozen v8 reproduction

Base `0941e1b76ee250a017295fe923f7c0841dafaaa7`, DGX release, seeds0..64 x10days, workers8, independent twins: PASS. All simulation metrics in summary.json are identical to p2-6_final_game, excluding only timing/commit metadata. Tick1440 fingerprints seeds0..4: f718457fcd90aa27,23b14e7b27051671,e968d90d40d0568e,a8aa877059579761.

Same-run v8 performance ms/agent-tick (median/p10/p90/min/max): 0.0007874892241498525 / 0.0006824113740941615 / 0.0008385900837547184 / 0.0006611407105494327 / 0.0008617722934239015. This is the U11 baseline; observer costs are outside the engine timer and host-load variation remains.

### Part 1 — schema 3 and 60-day v8 baseline

Producer: Part 1 worktree based on b5844620; release binary SHA and exact diff retained in ignored logs. v8 64x10 twins PASS; all original summary simulation metrics and 7744 samples/11 deaths identical to Part0 after projecting schema3 onto prior keys. 32x60 twins with clock samples 02:00,12:00,19:30 PASS.

Each cell median / p10 / p90 / min / max. Existing capacity overflow and new owned overflow are distinct: v8 never owns housing, so owned capacity zero is not a housing regression.

| Day | Alive | Stock | Starvation | Settlements |
|---|---|---|---|---|
| 1 | 24.000 / 24.000 / 24.000 / 24.000 / 24.000 | 66.000 / 53.000 / 70.900 / 19.000 / 78.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 5.000 / 3.100 / 6.000 / 2.000 / 7.000 |
| 2 | 26.000 / 25.000 / 27.000 / 25.000 / 28.000 | 80.000 / 64.400 / 94.600 / 28.000 / 101.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 5.500 / 4.000 / 6.900 / 2.000 / 7.000 |
| 3 | 29.000 / 26.000 / 30.000 / 26.000 / 30.000 | 90.500 / 72.200 / 120.600 / 59.000 / 127.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 4.000 / 6.900 / 2.000 / 7.000 |
| 4 | 31.000 / 27.000 / 33.000 / 27.000 / 33.000 | 99.500 / 75.000 / 142.700 / 53.000 / 156.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 4.100 / 6.900 / 3.000 / 7.000 |
| 5 | 33.000 / 28.000 / 35.000 / 28.000 / 36.000 | 106.000 / 65.100 / 170.600 / 52.000 / 180.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 4.100 / 6.900 / 3.000 / 7.000 |
| 6 | 34.500 / 29.000 / 38.000 / 29.000 / 39.000 | 110.000 / 61.200 / 195.700 / 39.000 / 206.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 4.100 / 6.900 / 3.000 / 7.000 |
| 7 | 36.500 / 30.000 / 40.000 / 30.000 / 42.000 | 110.000 / 58.100 / 211.000 / 16.000 / 225.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 5.000 / 6.900 / 3.000 / 8.000 |
| 8 | 38.500 / 31.100 / 42.000 / 31.000 / 45.000 | 112.500 / 48.200 / 230.000 / 1.000 / 244.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 5.000 / 6.900 / 3.000 / 8.000 |
| 9 | 40.500 / 32.100 / 43.900 / 32.000 / 46.000 | 107.000 / 42.200 / 242.700 / 6.000 / 264.000 | 0.000 / 0.000 / 0.000 / 0.000 / 0.000 | 6.000 / 5.000 / 7.000 / 3.000 / 8.000 |
| 10 | 42.000 / 33.100 / 44.900 / 33.000 / 47.000 | 105.000 / 22.600 / 252.000 / 3.000 / 285.000 | 0.000 / 0.000 / 0.000 / 0.000 / 1.000 | 6.000 / 5.000 / 7.000 / 3.000 / 8.000 |
| 11 | 43.000 / 34.200 / 46.000 / 34.000 / 49.000 | 105.500 / 11.300 / 259.400 / 3.000 / 298.000 | 0.000 / 0.000 / 0.000 / 0.000 / 1.000 | 6.000 / 5.000 / 7.000 / 3.000 / 8.000 |
| 12 | 44.000 / 35.200 / 47.000 / 35.000 / 49.000 | 95.500 / 5.000 / 272.100 / 0.000 / 319.000 | 0.000 / 0.000 / 1.000 / 0.000 / 2.000 | 6.000 / 5.000 / 7.000 / 3.000 / 8.000 |
| 13 | 44.500 / 36.200 / 48.000 / 36.000 / 50.000 | 100.500 / 2.000 / 278.700 / 0.000 / 327.000 | 0.000 / 0.000 / 2.900 / 0.000 / 6.000 | 6.000 / 5.000 / 7.000 / 4.000 / 8.000 |
| 14 | 46.000 / 38.100 / 48.900 / 37.000 / 50.000 | 90.000 / 0.000 / 273.900 / 0.000 / 341.000 | 0.000 / 0.000 / 3.900 / 0.000 / 6.000 | 6.000 / 5.000 / 7.000 / 4.000 / 8.000 |
| 15 | 46.000 / 39.100 / 48.900 / 38.000 / 50.000 | 83.500 / 0.100 / 266.500 / 0.000 / 348.000 | 0.000 / 0.000 / 3.900 / 0.000 / 7.000 | 6.000 / 5.000 / 7.000 / 4.000 / 8.000 |
| 16 | 46.000 / 40.000 / 49.000 / 39.000 / 50.000 | 64.500 / 1.100 / 255.500 / 0.000 / 357.000 | 0.000 / 0.000 / 4.000 / 0.000 / 7.000 | 6.000 / 5.000 / 7.000 / 4.000 / 8.000 |
| 17 | 46.500 / 40.100 / 50.000 / 40.000 / 51.000 | 38.000 / 0.000 / 232.100 / 0.000 / 352.000 | 0.000 / 0.000 / 4.900 / 0.000 / 7.000 | 6.000 / 5.000 / 7.000 / 4.000 / 8.000 |
| 18 | 47.000 / 41.000 / 50.900 / 39.000 / 52.000 | 18.500 / 1.000 / 216.800 / 0.000 / 361.000 | 0.000 / 0.000 / 4.900 / 0.000 / 8.000 | 6.000 / 5.000 / 8.000 / 4.000 / 9.000 |
| 19 | 47.000 / 42.000 / 51.000 / 40.000 / 52.000 | 8.500 / 0.000 / 223.900 / 0.000 / 361.000 | 0.000 / 0.000 / 5.900 / 0.000 / 8.000 | 6.000 / 5.000 / 8.000 / 4.000 / 10.000 |
| 20 | 47.500 / 43.000 / 52.000 / 41.000 / 53.000 | 4.500 / 0.000 / 230.100 / 0.000 / 356.000 | 0.500 / 0.000 / 6.900 / 0.000 / 10.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 21 | 47.000 / 43.100 / 53.000 / 39.000 / 54.000 | 8.500 / 0.100 / 228.600 / 0.000 / 357.000 | 2.000 / 0.000 / 7.000 / 0.000 / 13.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 22 | 47.000 / 43.000 / 53.000 / 38.000 / 55.000 | 6.500 / 0.000 / 221.900 / 0.000 / 349.000 | 2.000 / 0.000 / 9.000 / 0.000 / 15.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 23 | 47.000 / 43.000 / 52.900 / 38.000 / 56.000 | 9.500 / 0.000 / 205.900 / 0.000 / 333.000 | 2.500 / 0.000 / 9.000 / 0.000 / 16.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 24 | 47.000 / 43.100 / 52.900 / 38.000 / 56.000 | 7.000 / 0.000 / 197.700 / 0.000 / 324.000 | 3.000 / 0.000 / 10.900 / 0.000 / 17.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 25 | 47.000 / 44.000 / 53.800 / 38.000 / 56.000 | 4.000 / 0.000 / 205.400 / 0.000 / 310.000 | 4.500 / 0.000 / 13.900 / 0.000 / 18.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 26 | 46.000 / 43.100 / 51.000 / 38.000 / 57.000 | 5.000 / 0.000 / 202.700 / 0.000 / 308.000 | 5.000 / 0.000 / 14.900 / 0.000 / 22.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 27 | 46.500 / 43.000 / 52.000 / 38.000 / 57.000 | 6.000 / 0.000 / 207.900 / 0.000 / 294.000 | 5.500 / 0.000 / 15.000 / 0.000 / 23.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 28 | 46.500 / 43.000 / 53.000 / 38.000 / 58.000 | 3.000 / 0.100 / 202.400 / 0.000 / 304.000 | 6.500 / 0.000 / 16.900 / 0.000 / 23.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 29 | 47.000 / 43.000 / 53.900 / 38.000 / 58.000 | 3.000 / 0.000 / 218.400 / 0.000 / 313.000 | 6.500 / 0.000 / 17.900 / 0.000 / 23.000 | 6.000 / 5.000 / 8.000 / 4.000 / 11.000 |
| 30 | 46.500 / 43.000 / 53.900 / 39.000 / 56.000 | 5.500 / 0.000 / 222.300 / 0.000 / 310.000 | 6.500 / 0.000 / 18.000 / 0.000 / 23.000 | 6.000 / 5.000 / 8.900 / 4.000 / 11.000 |
| 31 | 46.000 / 43.000 / 54.000 / 40.000 / 56.000 | 3.000 / 0.000 / 206.100 / 0.000 / 319.000 | 7.000 / 0.100 / 18.900 / 0.000 / 24.000 | 6.000 / 5.000 / 8.900 / 4.000 / 11.000 |
| 32 | 46.000 / 43.000 / 53.900 / 40.000 / 57.000 | 3.000 / 0.000 / 195.500 / 0.000 / 331.000 | 7.000 / 1.000 / 19.900 / 0.000 / 24.000 | 6.000 / 5.000 / 8.900 / 4.000 / 13.000 |
| 33 | 46.000 / 43.100 / 53.900 / 40.000 / 57.000 | 3.000 / 0.000 / 160.600 / 0.000 / 343.000 | 7.000 / 1.000 / 20.900 / 0.000 / 27.000 | 6.000 / 5.000 / 8.900 / 4.000 / 13.000 |
| 34 | 46.500 / 44.000 / 53.900 / 39.000 / 59.000 | 3.500 / 0.000 / 167.100 / 0.000 / 354.000 | 7.500 / 1.100 / 22.800 / 0.000 / 27.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 35 | 46.000 / 43.000 / 53.900 / 37.000 / 59.000 | 2.000 / 0.000 / 172.200 / 0.000 / 368.000 | 8.000 / 1.100 / 22.900 / 0.000 / 29.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 36 | 46.500 / 43.000 / 53.800 / 37.000 / 60.000 | 2.000 / 0.000 / 176.300 / 0.000 / 374.000 | 8.000 / 1.100 / 23.900 / 0.000 / 30.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 37 | 46.000 / 43.000 / 53.600 / 37.000 / 60.000 | 3.000 / 0.000 / 175.300 / 0.000 / 387.000 | 9.000 / 1.100 / 24.000 / 0.000 / 31.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 38 | 45.000 / 43.000 / 53.600 / 38.000 / 61.000 | 3.000 / 0.000 / 178.900 / 0.000 / 400.000 | 9.000 / 2.000 / 26.900 / 0.000 / 31.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 39 | 45.500 / 43.100 / 53.600 / 39.000 / 61.000 | 2.000 / 0.000 / 183.100 / 0.000 / 395.000 | 9.000 / 2.000 / 28.000 / 0.000 / 31.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 40 | 45.000 / 43.000 / 53.600 / 40.000 / 60.000 | 4.000 / 0.000 / 189.800 / 0.000 / 403.000 | 9.000 / 2.000 / 28.900 / 0.000 / 34.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 41 | 46.000 / 43.000 / 54.600 / 38.000 / 60.000 | 3.000 / 0.000 / 186.000 / 0.000 / 417.000 | 9.500 / 2.000 / 29.000 / 0.000 / 34.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 42 | 46.000 / 43.100 / 54.600 / 39.000 / 60.000 | 2.000 / 0.000 / 189.300 / 0.000 / 427.000 | 9.500 / 2.000 / 30.800 / 0.000 / 37.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 43 | 46.000 / 43.000 / 54.600 / 40.000 / 60.000 | 4.500 / 0.100 / 192.100 / 0.000 / 435.000 | 10.000 / 2.000 / 30.900 / 0.000 / 42.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 44 | 46.000 / 43.000 / 54.600 / 40.000 / 60.000 | 2.000 / 0.000 / 190.400 / 0.000 / 440.000 | 10.000 / 2.000 / 32.000 / 0.000 / 42.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 45 | 46.000 / 43.000 / 54.600 / 36.000 / 60.000 | 3.500 / 0.000 / 176.300 / 0.000 / 450.000 | 10.000 / 2.000 / 34.900 / 0.000 / 42.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 46 | 46.000 / 43.000 / 54.600 / 37.000 / 60.000 | 2.000 / 0.000 / 170.500 / 0.000 / 456.000 | 10.000 / 2.000 / 34.900 / 0.000 / 42.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 47 | 46.000 / 43.000 / 54.600 / 38.000 / 60.000 | 4.000 / 0.000 / 152.100 / 0.000 / 468.000 | 11.500 / 2.100 / 36.900 / 0.000 / 43.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 48 | 46.000 / 43.000 / 54.700 / 39.000 / 60.000 | 3.000 / 0.000 / 158.800 / 0.000 / 478.000 | 11.500 / 2.100 / 37.000 / 0.000 / 44.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 49 | 46.500 / 43.100 / 54.700 / 40.000 / 60.000 | 2.000 / 0.000 / 135.800 / 0.000 / 489.000 | 12.000 / 2.100 / 37.000 / 0.000 / 44.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 50 | 46.000 / 43.100 / 53.800 / 41.000 / 61.000 | 2.000 / 0.000 / 139.300 / 0.000 / 499.000 | 12.000 / 2.100 / 38.800 / 0.000 / 44.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 51 | 46.000 / 43.100 / 52.900 / 40.000 / 61.000 | 3.000 / 0.000 / 128.400 / 0.000 / 505.000 | 12.000 / 2.100 / 39.900 / 0.000 / 46.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 52 | 45.500 / 43.100 / 52.900 / 37.000 / 61.000 | 3.000 / 0.000 / 123.600 / 0.000 / 515.000 | 13.000 / 2.100 / 42.700 / 1.000 / 51.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 53 | 45.500 / 43.000 / 52.900 / 37.000 / 62.000 | 3.500 / 0.000 / 130.700 / 0.000 / 529.000 | 13.000 / 3.000 / 43.700 / 1.000 / 51.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 54 | 46.000 / 43.000 / 52.000 / 38.000 / 62.000 | 2.500 / 0.000 / 120.500 / 0.000 / 542.000 | 13.000 / 3.000 / 43.800 / 1.000 / 51.000 | 6.000 / 5.000 / 9.000 / 4.000 / 13.000 |
| 55 | 46.000 / 43.000 / 51.800 / 39.000 / 61.000 | 3.000 / 0.000 / 114.300 / 0.000 / 545.000 | 13.000 / 3.000 / 43.800 / 1.000 / 52.000 | 6.000 / 5.000 / 9.000 / 5.000 / 13.000 |
| 56 | 46.000 / 43.000 / 51.700 / 39.000 / 61.000 | 2.000 / 0.000 / 121.000 / 0.000 / 559.000 | 13.500 / 3.000 / 44.700 / 1.000 / 58.000 | 6.000 / 5.000 / 9.000 / 5.000 / 13.000 |
| 57 | 46.000 / 43.100 / 51.700 / 39.000 / 61.000 | 3.500 / 0.000 / 121.100 / 0.000 / 567.000 | 13.500 / 3.000 / 45.900 / 1.000 / 59.000 | 6.000 / 5.000 / 9.000 / 5.000 / 13.000 |
| 58 | 46.000 / 43.000 / 51.700 / 39.000 / 61.000 | 2.500 / 0.000 / 118.300 / 0.000 / 578.000 | 13.500 / 3.000 / 46.900 / 1.000 / 61.000 | 6.000 / 5.000 / 9.000 / 5.000 / 13.000 |
| 59 | 46.000 / 43.000 / 50.900 / 38.000 / 61.000 | 5.000 / 0.000 / 120.400 / 0.000 / 585.000 | 13.500 / 3.100 / 48.700 / 1.000 / 62.000 | 6.000 / 5.000 / 9.000 / 5.000 / 13.000 |
| 60 | 46.000 / 44.000 / 51.000 / 39.000 / 59.000 | 3.000 / 0.000 / 128.000 / 0.000 / 597.000 | 13.500 / 3.100 / 48.900 / 1.000 / 63.000 | 6.000 / 5.000 / 9.000 / 5.000 / 13.000 |

Death census totals: `{"dehydration": 2, "starvation": 656}`.

Day60 free-meal tally distribution: `{"max": 2146.0, "median": 762.0, "min": 313.0, "p10": 437.1, "p90": 1401.7000000000003}`.

Rhythm: `{"evening_near_hearth_share": {"max": 0.25, "median": 0.12771739130434784, "min": 0.0, "p10": 0.03614285714285714, "p90": 0.2210526315789474}, "night_in_shelter_share": {"max": 0.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 0.0}, "noon_idle_share": {"max": 0.6382978723404256, "median": 0.5051020408163265, "min": 0.4, "p10": 0.43505434782608693, "p90": 0.5813953488372093}}`.

### Part5 round1 — initial v9 rules (2025adc1)

Knobs: bushes10, utilization0.85, reserve0.5days, cooldown720, headroom2, night1260..360/recovery0.12, child5760, eveningradius3. Both cohorts deterministic; v8/legacy all baseline-existing summary and raw fields reproduce exactly. No tuning change yet.

| Target | Verdict |
|---|---|
| U1 | PASS |
| U2 | FAIL |
| U3 | PASS |
| U4 | PASS |
| U5 | PASS |
| U6 | FAIL |
| U7 | PASS |
| U8 | PASS |
| U9 | PASS |
| U10 | PASS |
| U11 | PASS |

All median/p10/p90/min/max target clauses: [round1 targets](probe/p3-5_round1_targets/u_targets.md); source receipts and producer revision are in the adjacent JSON. U2 dehydration max14 in both runs; U6 all-sample population minimum18. Ten-day deaths57 and sixty-day deaths24 were all SeekingSleep, never returning to water after the observed thirst breach. No starvation; meals_without_food=0 under v9 (v8 sixty-day median762).

| Day | Alive median [p10,p90;min,max] | Food stock median [p10,p90;min,max] | Starvation median [p10,p90;min,max] |
|---|---|---|---|
| 1 | 25 [24.1,25;24,25] | 29.5 [17.2,45.6;1,47] | 0 [0,0;0,0] |
| 5 | 33 [27,33;24,33] | 47 [13.2,129.2;0,153] | 0 [0,0;0,0] |
| 10 | 40.5 [29.2,42;24,42] | 42.5 [13.2,157.4;3,212] | 0 [0,0;0,0] |
| 15 | 42 [32.3,42;24,42] | 49 [11.1,206.7;0,288] | 0 [0,0;0,0] |
| 20 | 42 [33.2,42;24,42] | 62 [1.7,260.8;0,359] | 0 [0,0;0,0] |
| 30 | 42 [35.1,42;24,42] | 62.5 [5.3,337.7;0,498] | 0 [0,0;0,0] |
| 45 | 42 [36.1,42;25,42] | 80 [1.1,479.1;0,692] | 0 [0,0;0,0] |
| 60 | 42 [42,42;42,42] | 116.5 [38.9,641.6;12,891] | 0 [0,0;0,0] |

Round1 requires a correctness repair before parameter tuning. Observed seed33 agent12 reached old bed(24,25) while bed_of pointed to(25,24); position==SeekTarget!=bed_of, thirst100. This confirms the assigned-bed target became stale after reassignment. Retain initial tuning while repairing that synchronization; do not expand critical-preemption behavior without evidence.

### Part5 round2 — retained configuration, bed synchronization repaired (3450163d)

All U1–U11 PASS. This is the second complete measurement round and first correctness repair; no knob changed and no third round is needed. Initial v9 values are retained. Fresh v8/legacy reproduction and the named legacy pin pass on the same producer revision.

| Target | Verdict |
|---|---|
| U1 | PASS |
| U2 | PASS |
| U3 | PASS |
| U4 | PASS |
| U5 | PASS |
| U6 | PASS |
| U7 | PASS |
| U8 | PASS |
| U9 | PASS |
| U10 | PASS |
| U11 | PASS |

Every clause with median/p10/p90/min/max: [round2 targets](probe/p3-5_round2_targets/u_targets.md). Original cohorts now have zero deaths of any cause in64x10 and32x60; all60-day seeds end at42. Same-tick owned overflow0; new shelters10daymedian3. Maturation576observed/572matured/4right-censored/0late. Night97.619% median, evening61.607% median. These median criteria do not guarantee every seed has a half-day reserve or50% evening gathering: stock min12, evening-share min13.043%.

| Day | Alive median [p10,p90;min,max] | Food stock median [p10,p90;min,max] | Starvation median [p10,p90;min,max] |
|---|---|---|---|
| 1 | 25 [24.1,25;24,25] | 29.5 [17.2,45.6;1,47] | 0 [0,0;0,0] |
| 2 | 27 [25,27;24,27] | 43.5 [16.3,67;0,79] | 0 [0,0;0,0] |
| 3 | 29 [26,29;24,29] | 45 [15.1,87.8;0,108] | 0 [0,0;0,0] |
| 4 | 31 [27,31;24,31] | 42.5 [12,106.1;0,128] | 0 [0,0;0,0] |
| 5 | 33 [27.1,33;24,33] | 45 [13.2,129.2;0,145] | 0 [0,0;0,0] |
| 6 | 35 [28.1,35;24,35] | 51 [24.2,143.3;8,158] | 0 [0,0;0,0] |
| 7 | 37 [28.2,37;24,37] | 52 [19.5,154.9;0,174] | 0 [0,0;0,0] |
| 8 | 39 [29.2,39;24,39] | 51.5 [19.7,150.4;0,190] | 0 [0,0;0,0] |
| 9 | 40.5 [30.2,41;24,41] | 47 [14,150.4;0,189] | 0 [0,0;0,0] |
| 10 | 42 [31.2,42;24,42] | 42.5 [13.2,157.4;3,204] | 0 [0,0;0,0] |
| 11 | 42 [31.2,42;24,42] | 47 [22.1,172.6;5,217] | 0 [0,0;0,0] |
| 12 | 42 [32.2,42;24,42] | 48 [11.2,182.9;7,229] | 0 [0,0;0,0] |
| 13 | 42 [32.2,42;24,42] | 49.5 [9.2,191.4;3,248] | 0 [0,0;0,0] |
| 14 | 42 [32.2,42;24,42] | 46 [9,199.7;2,257] | 0 [0,0;0,0] |
| 15 | 42 [32.3,42;24,42] | 49 [11.1,204.1;0,271] | 0 [0,0;0,0] |
| 16 | 42 [32.3,42;24,42] | 53 [13.1,211.8;0,284] | 0 [0,0;0,0] |
| 17 | 42 [32.3,42;24,42] | 53.5 [1.7,225.9;0,298] | 0 [0,0;0,0] |
| 18 | 42 [32.3,42;24,42] | 57.5 [7.1,233;1,316] | 0 [0,0;0,0] |
| 19 | 42 [32.3,42;24,42] | 59 [0.3,244.4;0,326] | 0 [0,0;0,0] |
| 20 | 42 [33.2,42;24,42] | 62 [1.7,259.6;0,345] | 0 [0,0;0,0] |
| 21 | 42 [33.2,42;24,42] | 60 [0,265.8;0,356] | 0 [0,0;0,0] |
| 22 | 42 [33.2,42;24,42] | 59.5 [5.1,272.6;0,365] | 0 [0,0;0,0] |
| 23 | 42 [34.1,42;24,42] | 59.5 [8.2,275.6;0,387] | 0 [0,0;0,0] |
| 24 | 42 [34.1,42;24,42] | 54 [2.2,279.5;0,396] | 0 [0,0;0,0] |
| 25 | 42 [34.1,42;24,42] | 64.5 [0.5,296.8;0,412] | 0 [0,0;0,0] |
| 26 | 42 [34.1,42;24,42] | 63.5 [5.2,300.2;0,426] | 0 [0,0;0,0] |
| 27 | 42 [34.2,42;24,42] | 60.5 [0.1,310;0,438] | 0 [0,0;0,0] |
| 28 | 42 [34.2,42;24,42] | 59 [3.9,318.7;0,452] | 0 [0,0;0,0] |
| 29 | 42 [34.2,42;24,42] | 54.5 [1.2,329.1;0,465] | 0 [0,0;0,0] |
| 30 | 42 [35.1,42;24,42] | 62.5 [5.3,335.5;0,481] | 0 [0,0;0,0] |
| 31 | 42 [36,42;24,42] | 62 [10,344.1;3,493] | 0 [0,0;0,0] |
| 32 | 42 [36.1,42;24,42] | 62 [3.1,353.1;0,508] | 0 [0,0;0,0] |
| 33 | 42 [36.1,42;25,42] | 64.5 [5.1,361.3;0,526] | 0 [0,0;0,0] |
| 34 | 42 [36.1,42;25,42] | 63.5 [2.3,367.6;0,532] | 0 [0,0;0,0] |
| 35 | 42 [36.1,42;25,42] | 73.5 [6,378.7;0,548] | 0 [0,0;0,0] |
| 36 | 42 [36.1,42;25,42] | 75 [0.6,394;0,563] | 0 [0,0;0,0] |
| 37 | 42 [36.1,42;25,42] | 76 [2.3,402.5;0,578] | 0 [0,0;0,0] |
| 38 | 42 [36.1,42;25,42] | 79.5 [0.1,412.7;0,597] | 0 [0,0;0,0] |
| 39 | 42 [36.1,42;25,42] | 70.5 [3,421;0,604] | 0 [0,0;0,0] |
| 40 | 42 [36.1,42;25,42] | 70 [0,429.9;0,620] | 0 [0,0;0,0] |
| 41 | 42 [36.1,42;25,42] | 71.5 [0.1,440.5;0,633] | 0 [0,0;0,0] |
| 42 | 42 [36.1,42;25,42] | 72 [1.3,449.8;0,645] | 0 [0,0;0,0] |
| 43 | 42 [36.1,42;25,42] | 79 [1.3,460.8;0,663] | 0 [0,0;0,0] |
| 44 | 42 [36.1,42;25,42] | 80.5 [3.1,469.9;0,674] | 0 [0,0;0,0] |
| 45 | 42 [36.1,42;25,42] | 80 [1.1,476.8;0,692] | 0 [0,0;0,0] |
| 46 | 42 [36.1,42;25,42] | 87 [8.1,484.5;0,710] | 0 [0,0;0,0] |
| 47 | 42 [36.2,42;25,42] | 99 [11,497.6;0,720] | 0 [0,0;0,0] |
| 48 | 42 [37.2,42;26,42] | 97 [16.6,501;9,744] | 0 [0,0;0,0] |
| 49 | 42 [38.2,42;28,42] | 95 [19.1,517.8;8,757] | 0 [0,0;0,0] |
| 50 | 42 [39.1,42;30,42] | 94.5 [17.6,529.2;9,763] | 0 [0,0;0,0] |
| 51 | 42 [40,42;32,42] | 97 [23.5,540.8;14,779] | 0 [0,0;0,0] |
| 52 | 42 [40.1,42;34,42] | 102 [29.1,550.2;15,786] | 0 [0,0;0,0] |
| 53 | 42 [40.2,42;36,42] | 98 [28.3,561.9;20,797] | 0 [0,0;0,0] |
| 54 | 42 [40.2,42;38,42] | 109.5 [28.7,572.3;19,798] | 0 [0,0;0,0] |
| 55 | 42 [41.1,42;40,42] | 103.5 [29.4,583.7;21,814] | 0 [0,0;0,0] |
| 56 | 42 [41.1,42;41,42] | 107 [31.1,599.1;20,831] | 0 [0,0;0,0] |
| 57 | 42 [42,42;42,42] | 114 [29.4,612.2;20,847] | 0 [0,0;0,0] |
| 58 | 42 [42,42;42,42] | 119.5 [34.1,627.2;18,868] | 0 [0,0;0,0] |
| 59 | 42 [42,42;42,42] | 115.5 [40.2,635.6;22,879] | 0 [0,0;0,0] |
| 60 | 42 [42,42;42,42] | 116.5 [38.9,641.1;12,887] | 0 [0,0;0,0] |

Timing ms/agent-tick median10day0.00074145 and60day0.00086803, both below1.5×same-host frozen baseline0.001181233836. This is run-level timing, not a graphics FPS/device-performance claim.

### Part7 final cohorts

Producer `6db6db6a315841b0e40d52e7c66811f0009a13e2`; all four fresh cohorts exit0/twins PASS. [Full final U1–U11 clauses and distributions](probe/p3-7_final_targets/u_targets.md), [machine evidence](probe/p3-7_final_targets/u_targets.json), [receipts](probe/p3-7_final_targets/receipts.json). All U targets PASS. Final v9 simulation values equal Round2 at every sample kind:10day10240rows/526285scalar checks,60day30720rows/1582779checks; only timing/git metadata differ. Therefore the complete60-day Round2 trajectories above also describe final results.

Legacy4seeds exactly reproduce Pass1 and Pass2 (484rawrows;16056/35200scalar comparisons); v8game64x10 exactly reproduces Pass2 (7744rows/281568comparisons), with zero differences. Current producer legacy pin1/1PASS. [v8 comparison](probe/p3-7_final_v8/comparison.json); [legacy comparison](probe/p3-7_final_legacy/comparison-p1_legacy.json). Final v9 zero deaths in both cohorts; day60everyseed42residents, same-tickownedoverflow0, finalstockmedian116.5/min12.

Final performance ms/agent-tick (median,p10,p90,min,max):10day (0.0007298523,0.0006297925,0.0011856014,0.0006074260,0.0017314190);60day (0.0008718874,0.0007079225,0.0012058925,0.0006674926,0.0013499535). U11 applies to medians, limit0.0011812338; some individual timing observations exceed it and are retained. This is simulation timing, not renderer FPS.

## Pass 4 baseline

Produced on the Mac (macOS aarch64) from a clean worktree at `e136ba8a` (Pass 3 simulation code, `a8611e07` plus tooling only).

- `p4-0_v9_repro`: `--scene game_v9 --seeds 0..64 --days 10 --check-determinism --clock-samples 02:00,12:00,19:30` → DETERMINISM PASS; `tools/revitalize/compare_summaries.py` against `p3-7_final_10day`: 4,046 shared simulation values, 0 differences (timing excluded). The first run without `--clock-samples` matched every non-rhythm value too; Pass 3's 10-day run had used clock samples, so the stored label is the clock-sampled rerun.
- `p4-0_v8_repro`: `--scene game --seeds 0..64 --days 10 --check-determinism` (the stored `p3-7_final_v8` used the same seeds and days) → DETERMINISM PASS; 4,081 shared values, 0 differences.
- Pass 3 summaries were produced on the DGX (Linux aarch64); the Mac reproduces them bit for bit. No cross-platform determinism defect.
- `p4-0_v9_60day_seeds7-9`: `--scene game_v9 --seeds 7..10 --days 60 --clock-samples 02:00,12:00,19:30` (half-open 7..10 adds seed 9 to the requested 7 and 8).

Seed 7 and seed 8 trajectories (elapsed-day checkpoints; harvest and meals are per-day differences; stock is the stockpile; carried is food in members' inventories; food tiles is the food on the ground):

| Day | s7 alive | s7 births | s7 stock | s7 harvest/d | s7 meals/d | s7 carried | s7 food tiles | s8 alive | s8 births | s8 stock | s8 harvest/d | s8 meals/d | s8 carried | s8 food tiles |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 25 | 1 | 33 | 93 | 46 | 4 | 36 | 24 | 0 | 1 | 56 | 48 | 7 | 28 |
| 2 | 27 | 3 | 54 | 108 | 59 | 12 | 28 | 24 | 0 | 16 | 71 | 59 | 4 | 28 |
| 3 | 29 | 5 | 86 | 111 | 60 | 11 | 27 | 24 | 0 | 0 | 35 | 45 | 10 | 28 |
| 4 | 31 | 7 | 107 | 104 | 66 | 8 | 32 | 24 | 0 | 19 | 82 | 73 | 0 | 28 |
| 5 | 33 | 9 | 130 | 112 | 66 | 11 | 28 | 24 | 0 | 4 | 34 | 49 | 0 | 28 |
| 6 | 35 | 11 | 145 | 108 | 74 | 10 | 30 | 24 | 0 | 24 | 83 | 63 | 0 | 28 |
| 7 | 37 | 13 | 157 | 107 | 77 | 8 | 31 | 24 | 0 | 0 | 35 | 57 | 2 | 28 |
| 8 | 39 | 15 | 169 | 113 | 82 | 7 | 28 | 24 | 0 | 28 | 89 | 56 | 7 | 28 |
| 9 | 41 | 17 | 168 | 109 | 86 | 11 | 28 | 24 | 0 | 6 | 34 | 63 | 0 | 28 |
| 10 | 42 | 18 | 178 | 110 | 93 | 8 | 28 | 24 | 0 | 13 | 64 | 53 | 4 | 28 |
| 11 | 42 | 18 | 188 | 107 | 94 | 11 | 27 | 24 | 0 | 5 | 45 | 54 | 3 | 28 |
| 12 | 42 | 18 | 202 | 108 | 97 | 8 | 32 | 24 | 0 | 7 | 59 | 60 | 0 | 28 |
| 13 | 42 | 18 | 222 | 118 | 95 | 11 | 26 | 24 | 0 | 3 | 48 | 52 | 0 | 28 |
| 14 | 42 | 18 | 228 | 111 | 105 | 11 | 28 | 24 | 0 | 26 | 96 | 69 | 4 | 28 |
| 15 | 42 | 18 | 240 | 109 | 98 | 10 | 28 | 24 | 0 | 12 | 35 | 53 | 0 | 28 |
| 16 | 42 | 18 | 245 | 108 | 104 | 9 | 28 | 24 | 0 | 1 | 38 | 49 | 0 | 28 |
| 17 | 42 | 18 | 253 | 109 | 99 | 11 | 28 | 24 | 0 | 10 | 81 | 68 | 4 | 28 |
| 18 | 42 | 18 | 259 | 105 | 99 | 11 | 28 | 24 | 0 | 3 | 36 | 47 | 0 | 28 |
| 19 | 42 | 18 | 262 | 100 | 104 | 4 | 35 | 24 | 0 | 0 | 67 | 63 | 7 | 28 |
| 20 | 42 | 18 | 272 | 110 | 99 | 5 | 32 | 24 | 0 | 0 | 52 | 50 | 9 | 28 |
| 21 | 42 | 18 | 275 | 111 | 102 | 11 | 28 | 24 | 0 | 0 | 56 | 58 | 7 | 28 |
| 22 | 42 | 18 | 289 | 114 | 100 | 11 | 26 | 24 | 0 | 6 | 67 | 68 | 0 | 28 |
| 23 | 42 | 18 | 299 | 109 | 99 | 11 | 28 | 24 | 0 | 10 | 61 | 57 | 0 | 28 |
| 24 | 42 | 18 | 310 | 114 | 103 | 11 | 27 | 24 | 0 | 4 | 51 | 57 | 0 | 28 |
| 25 | 42 | 18 | 323 | 107 | 99 | 6 | 32 | 24 | 0 | 0 | 57 | 57 | 4 | 28 |
| 26 | 42 | 18 | 328 | 113 | 102 | 12 | 28 | 24 | 0 | 17 | 82 | 61 | 8 | 28 |
| 27 | 42 | 18 | 335 | 108 | 102 | 11 | 28 | 24 | 0 | 0 | 35 | 58 | 2 | 28 |
| 28 | 42 | 18 | 352 | 113 | 97 | 10 | 28 | 24 | 0 | 22 | 75 | 55 | 0 | 28 |
| 29 | 42 | 18 | 354 | 108 | 105 | 11 | 28 | 24 | 0 | 0 | 35 | 57 | 0 | 28 |
| 30 | 42 | 18 | 364 | 108 | 98 | 11 | 28 | 24 | 0 | 5 | 51 | 46 | 0 | 28 |
| 31 | 42 | 18 | 377 | 116 | 103 | 11 | 25 | 24 | 0 | 15 | 82 | 72 | 0 | 28 |
| 32 | 42 | 18 | 386 | 105 | 101 | 6 | 32 | 24 | 0 | 4 | 35 | 46 | 0 | 28 |
| 33 | 42 | 18 | 396 | 112 | 101 | 7 | 28 | 25 | 1 | 19 | 100 | 67 | 8 | 28 |
| 34 | 42 | 18 | 401 | 109 | 100 | 11 | 28 | 25 | 1 | 0 | 34 | 55 | 6 | 28 |
| 35 | 42 | 18 | 415 | 113 | 99 | 11 | 28 | 25 | 1 | 13 | 74 | 63 | 4 | 28 |
| 36 | 42 | 18 | 424 | 112 | 103 | 11 | 28 | 25 | 1 | 0 | 36 | 50 | 3 | 28 |
| 37 | 42 | 18 | 433 | 110 | 100 | 12 | 28 | 25 | 1 | 14 | 81 | 70 | 0 | 28 |
| 38 | 42 | 18 | 448 | 112 | 99 | 10 | 28 | 25 | 1 | 0 | 44 | 43 | 15 | 20 |
| 39 | 42 | 18 | 456 | 113 | 104 | 11 | 27 | 25 | 1 | 18 | 91 | 80 | 8 | 28 |
| 40 | 42 | 18 | 469 | 110 | 97 | 11 | 28 | 25 | 1 | 0 | 35 | 54 | 7 | 28 |
| 41 | 42 | 18 | 477 | 112 | 103 | 12 | 28 | 25 | 1 | 1 | 64 | 63 | 7 | 28 |
| 42 | 42 | 18 | 482 | 106 | 102 | 11 | 28 | 25 | 1 | 5 | 65 | 60 | 8 | 28 |
| 43 | 42 | 18 | 495 | 112 | 99 | 11 | 28 | 25 | 1 | 5 | 39 | 47 | 0 | 28 |
| 44 | 42 | 18 | 502 | 103 | 103 | 4 | 35 | 25 | 1 | 8 | 77 | 74 | 0 | 28 |
| 45 | 42 | 18 | 515 | 117 | 98 | 10 | 28 | 25 | 1 | 0 | 53 | 57 | 4 | 28 |
| 46 | 42 | 18 | 522 | 111 | 103 | 11 | 28 | 25 | 1 | 9 | 75 | 62 | 8 | 28 |
| 47 | 42 | 18 | 534 | 112 | 101 | 10 | 28 | 25 | 1 | 11 | 51 | 57 | 0 | 28 |
| 48 | 42 | 18 | 549 | 114 | 98 | 11 | 27 | 26 | 2 | 23 | 80 | 58 | 0 | 28 |
| 49 | 42 | 18 | 560 | 113 | 102 | 11 | 28 | 28 | 4 | 33 | 96 | 66 | 0 | 28 |
| 50 | 42 | 18 | 569 | 104 | 101 | 5 | 32 | 30 | 6 | 35 | 93 | 61 | 10 | 27 |
| 51 | 42 | 18 | 574 | 114 | 102 | 12 | 28 | 32 | 8 | 48 | 90 | 67 | 0 | 27 |
| 52 | 42 | 18 | 585 | 110 | 101 | 10 | 28 | 34 | 10 | 51 | 97 | 71 | 3 | 28 |
| 53 | 42 | 18 | 601 | 113 | 96 | 11 | 28 | 36 | 12 | 55 | 90 | 69 | 0 | 28 |
| 54 | 42 | 18 | 603 | 106 | 104 | 11 | 28 | 38 | 14 | 51 | 99 | 83 | 0 | 28 |
| 55 | 42 | 18 | 613 | 110 | 100 | 11 | 28 | 40 | 16 | 37 | 96 | 82 | 8 | 28 |
| 56 | 42 | 18 | 623 | 113 | 105 | 9 | 28 | 41 | 17 | 49 | 100 | 86 | 0 | 28 |
| 57 | 42 | 18 | 630 | 107 | 99 | 10 | 28 | 42 | 18 | 44 | 100 | 95 | 0 | 28 |
| 58 | 42 | 18 | 645 | 110 | 100 | 5 | 30 | 42 | 18 | 49 | 97 | 92 | 0 | 28 |
| 59 | 42 | 18 | 653 | 113 | 102 | 8 | 28 | 42 | 18 | 42 | 99 | 103 | 3 | 28 |
| 60 | 42 | 18 | 659 | 108 | 102 | 8 | 28 | 42 | 18 | 47 | 98 | 96 | 0 | 28 |

Seed 8 stays at the 24 founders with zero births until day 45 while its bushes keep about 28 of 40 units on the ground; seed 7 reaches 42 by day 24 and its stock climbs to 659 by day 60. This reproduces section 3.2.

## Pass 4 Part 1 — v9 over 120 days (instrument baseline)

`p4-1_v9_120day`: `--scene game_v9 --seeds 0..32 --days 120 --check-determinism --clock-samples 02:00,12:00,19:30`, schema 4, DETERMINISM PASS. Days 1–60 equal `p3-7_final_60day` on all 15,300 shared day values; the 4,500 extra values are the 15 new schema-4 keys.

- Population: median 42 from day 10 to day 120, births 18, zero deaths of any cause, children 0 after day 44 — the frozen village of section 3.2.
- Store: stockpile median 29.5 on day 1, 452.5 on day 118, 463.5 on day 120 (hoarding grows without bound).
- Regrowth lost at the 4-unit cap: 32 units on day 1, about 13–15 per day later (median per seed-day 15 over all 120 days).
- Harvest median 104 per day against a nominal supply of 120 and a need of 100.8.

## Pass 4 Part 2 — forage, haul home, store target (no aging, no seasons)

`p4-2_v10_10day` (64 × 10, determinism PASS) and `p4-2_v10_120day` (32 × 120, clock-sampled, determinism PASS) on `game_v10` = v9 + 12 bushes + utilization 0.705 + forage (06:00–18:00, radius 16, haul 4, 1 forager per bush) + stock target (1.5 days, hoard cap 2.0).

| Metric | Pass 3 v9 | Part 2 v10 |
|---|---|---|
| births by day 4 (min over seeds) | 0 (seed 8 stagnant 45 days) | 7 |
| alive day 10 (min / median) | 24 / 42 | 42 / 42 |
| deaths of any cause, 120 days | 0 | 0 |
| stockpile day 120 (median, min–max) | 463.5 | 274.5 (259–288) |
| stock target day 120 | — | 151.2 (hoard cap 302.4) |
| home distance day 10 (median, max) | 5.80 / 11.28 | 5.76 / 10.95 |
| night in shelter (median) | 0.968 | 1.0 |
| evening near hearth (median, min) | 0.69 / 0.11 | 0.80 / 0.70 |
| noon idle (median) | 0.596 | 0.738 |
| ms per agent-tick (10-day median) | 0.000778 | 0.000944 |

Forage trips concentrate in the first two to four days (median total 81) while the store fills; once `stock_in_hand ≥ target` the Forage arm is silent and opportunistic gathering continues up to the hoard cap, after which most adults idle at noon. Population stays frozen at 42 (aging is Part 3).

## Pass 4 Part 3 — aging, elders, old-age replacement (no seasons)

`p4-3_v10_10day` (64 × 10) and `p4-3_v10_120day` (32 × 120, clock-sampled), both DETERMINISM PASS. v10 now adds the aging rule (elder 46 d, lifespan 58 ± 8 d, founders aged 6–40 d with ≥ 3 d left, elder tether 4).

| Day (120-day run) | alive median (min–max) | births | old-age deaths | children | elders | mean age (d) | stockpile |
|---|---|---|---|---|---|---|---|
| 10 | 42 (42–42) | 18 | 0 | 7 | 3 (0–7) | 21.4 | 252 |
| 30 | 42 (40–42) | 27 | 9 (5–13) | 3 | 8.5 | 29.0 | 258 |
| 60 | 42 (38–42) | 47 | 29 | 4.5 | 13 | 31.3 | 253 |
| 90 | 42 (40–42) | 71 | 53 | 3 | 8 | 28.8 | 265 |
| 120 | 42 (39–42) | 91 | 73 (69–78) | 4 | 11 | 31.0 | 251.5 |

Starvation and dehydration remain zero in every seed. Rhythm (120 days): night in shelter median 1.0, evening near hearth median 0.807 (min 0.676), noon idle median 0.81 — worse than Part 2 because elders rest at the camp and foraging still stops once the store meets its target. Day-10 home distance median 5.35, max 10.95 (one seed).

## Pass 4 Part 4 — seasons (v10 complete before tuning)

`p4-4_v10_10day` (64 × 10) and `p4-4_v10_120day` (32 × 120, clock-sampled), both DETERMINISM PASS. v10 adds `SeasonRule {7, [750, 1250, 1000, 500]}` and utilization 0.81.

Seasons (120-day run, medians over seed-days, closing-checkpoint attribution):

| Season | Supply/day | Harvest/day | Need/day | Stock | Regrowth lost at cap/day | Forage trips/day |
|---|---:|---:|---:|---:|---:|---:|
| spring | 108 | 106 | 96.96 | 156 | 0 | 0 |
| summer | 180 | 137 | 96.96 | 266 | 39 | 20.5 |
| autumn | 144 | 124 | 98.88 | 474 | 22 | 0 |
| winter | 72 | 67 | 98.88 | 284.5 | 18 | 0 |

W targets (`p4-4_targets/w_targets.md`; W11 against same-host v9 of the same duration: 10-day `p4-0_v9_repro` 0.000778, 120-day `p4-1_v9_120day` 0.000883):

| Target | Result | Key observation |
|---|---|---|
| W1–W4 | PASS | zero starvation/dehydration in every seed; one settlement every day |
| W5 | PASS | births by day 4 = 7 in every seed; day-10 alive 42; longest stagnation 1 day |
| W6 | PASS | old-age deaths by day 30 min 5; births − deaths (30–120) min −3; alive from day 30 in [37, 42]; child share 0.095, elder share 0.262 |
| W7 | PASS | autumn-closing stock median 586.5 vs required 150.5; max stock − 3 × max target ≤ −352 |
| W8 | FAIL | noon idle median 0.809; forage trips/day median 0 in summer and autumn |
| W9 | FAIL | night in shelter 1.0; evening near hearth median 0.571 but min 0.273 |
| W10 | FAIL | day-10 home distance median 7.012 (> 7.0), max 11.02; T6a and social pass |
| W11 | PASS | 10-day 0.00100 (limit 0.00117); 120-day 0.00121 (limit 0.00132) |

The store fills through opportunistic gathering up to the 2 × target hoard cap, so the Forage arm is silent most days (trips only early in summer). Elders (share 0.26) and children are idle at noon by design, and haulers walking home are in the `Idle` state, so noon idle counts them. The probes ran while the gate's builds competed for CPU, so timings are pessimistic.

## Pass 4 Part 5 — tuning

Round 0 is the Part 4 configuration (`p4-4_*`): forage 06:00–18:00, haul 4, 1 forager per bush, base 1.5 days, hoard cap 2.0, elder 46 days; 12 bushes, capacity 4, utilization 0.81, reserve 0.5, seasons 7 days [750, 1250, 1000, 500], lifespan 58 ± 8 days.

### Round 1 (`p4-5_round1_*`)

Changes: forage window start 360 → 480 (08:00, so the 08:00 day checkpoint is not taken mid-forage), haul threshold 4 → 6, max foragers per bush 1 → 2, base days 1.5 → 2.0, hoard cap 2.0 → 1.5, elder age 46 → 50 days. Everything else as round 0.

| Target | Round 0 | Round 1 |
|---|---|---|
| W5 stagnation (max run, days) | 1 | 6 (seed 22, spring after winter: stockpile 6–21) |
| W6 elder share day 120 (median) | 0.262 | 0.238 |
| W7 autumn-closing stock (median / required) | 586.5 / 150.5 | 572 / 150.5 |
| W8 noon idle (median) | 0.809 | 0.821 |
| W8 forage trips/day summer+autumn (median) | 0 | 0 (summer 101, autumn 0) |
| W9 evening near hearth (median / min) | 0.571 / 0.273 | 0.530 / 0.257 |
| W10 day-10 home distance (median / max) | 7.012 / 11.02 | 5.25 / 10.88 (seed 7) |
| W11 10-day / 120-day ms per agent-tick | 0.00100 / 0.00121 | 0.00100 / 0.00130 |

W1–W4 and W6–W7 pass in both rounds. Diagnosis from the raw summer samples (all 32 seeds): food on the ground is 47 units at 06:00 after the night, 32 at 08:00 and 0 from 10:00 to 20:00 — every unit of daytime regrowth is taken as soon as it appears, so after the 08:00–10:00 rush 34–35 of 42 residents are idle with nothing to forage while stock in hand is still below target (noon summer: in hand 310.5 vs target 368.6). Noon idleness is bounded by supply, not by labour. Autumn forage trips are zero because the summer surplus already lifts the store past the autumn target. The low-evening seeds (15, 28) are summer haulers still walking home from distant bushes at 19:30 after an 18:00 window end.

### Round 2 (`p4-5_round2_*`)

Changes from round 1: forage window end 1080 → 960 (16:00), base days 2.0 → 2.5. Hypothesis: summer haulers walking home after 18:00 lower the 19:30 hearth share, and a bigger store carries seed 22 through winter.

| Target | Round 1 | Round 2 |
|---|---|---|
| W5 stagnation (max run) | 6 | 6 |
| W6 births − deaths over days 30–120 (min) | −3 | −6 (FAIL); alive band min 34 |
| W7 autumn-closing stock (median / min) | 572 / 248 | 634.5 / 229 |
| W8 noon idle / forage trips summer+autumn | 0.821 / 0 | 0.833 / 0 (summer 80) |
| W9 evening near hearth (median / min) | 0.530 / 0.257 | 0.564 / 0.257 |
| W10 home distance day 10 (median / max) | 5.25 / 10.88 | 5.27 / 11.12 |
| W11 10-day / 120-day | 0.00100 / 0.00130 | 0.00100 / 0.00120 |

The evening hypothesis was wrong: in the lowest seeds (15, 28) at 19:30 the whole membership's mean home distance is 6.2–7.4 tiles with only 3–4 haulers, so the low share is not haulers. Best remaining hypothesis: with seasons, bushes are empty most of the day; every newly regrown unit makes every hungry resident (global `NeedAvailability`) walk toward it, so in seeds with distant bushes hungry residents are strung out across the map in the evening while the store, used only as the critical-hunger relief at home, stays full. No permitted knob changes where residents eat. Round 2 is worse than round 0 on W5 and W6, so round 3 returns to round 0 and changes one knob.

### Round 3 (`p4-5_round3_*`) — last round

Changes from round 0 only: forage window start 360 → 480 (all other knobs back to round 0: haul 4, 1 forager per bush, base 1.5, hoard cap 2.0, elder 46 days, end 1080).

| Target | Round 0 | Round 1 | Round 2 | Round 3 |
|---|---|---|---|---|
| W1–W4 | PASS | PASS | PASS | PASS |
| W5 stagnation max run (days) | 1 PASS | 6 FAIL | 6 FAIL | 8 FAIL (seed 22) |
| W6 replacement min / alive band min | −3 / 37 PASS | −3 / 37 PASS | −6 / 34 FAIL | −3 / 35 PASS |
| W6 child / elder share day 120 | 0.095 / 0.262 | 0.095 / 0.238 | 0.095 / 0.238 | 0.095 / 0.262 |
| W7 autumn stock median vs required | 586.5 vs 150.5 PASS | 572 PASS | 634.5 PASS | 562 PASS |
| W8 noon idle median | 0.809 FAIL | 0.821 FAIL | 0.833 FAIL | 0.813 FAIL |
| W8 forage trips/day summer+autumn median | 0 FAIL | 0 FAIL | 0 FAIL | 0 FAIL |
| W9 evening near hearth median / min | 0.571 / 0.273 FAIL | 0.530 / 0.257 FAIL | 0.564 / 0.257 FAIL | 0.571 / 0.281 FAIL |
| W10 home distance day 10 median / max | 7.012 / 11.02 FAIL | 5.25 / 10.88 FAIL | 5.27 / 11.12 FAIL | 5.06 / 10.93 FAIL |
| W10 T6a 10 / 120, social 10 / 120 | PASS | PASS | PASS | PASS |
| W11 10-day / 120-day | PASS | PASS | PASS | PASS |

Seed 22's stockpile in the spring after the first winter (days 28–36): round 0 `46, 38, 18, 31, 14, 23, 16, 8, 24`; round 3 `38, 23, 7, 0, 2, 1, 0, 5, 31`. The seed is marginal in every configuration; round 0 keeps births often enough to stay inside the 4-day limit.

**Kept configuration: round 0 (the Part 4 `game_v10()`, commit `e72d7e28`).** In table priority order it is the only configuration that passes W1–W7; it fails W8, W9 and W10 (median 7.012 just over 7.0 and max 11.02), all three after three rounds (three-strikes rule).

## Pass 4 final (Part 7, producer `0dc9a38d`, after the review fixes)

All seven runs DETERMINISM PASS, run back to back on the Mac with no other job competing:

| Label | Command | Comparison |
|---|---|---|
| `p4-7_final_legacy` | legacy 0..4 × 10 | equal to `p3-7_final_legacy` on all 2,696 shared values (and to Pass 1 `p1_legacy` on all 569 shared values); only the summary `schema_version` differs (3/1 → 4) |
| `p4-7_final_v8` | game 0..64 × 10 | equal to `p3-7_final_v8` on 4,081 shared values (schema_version only) |
| `p4-7_final_v9_10day` | game_v9 0..64 × 10, clocks | equal to `p3-7_final_10day` on 4,046 shared values (schema_version only) |
| `p4-7_final_v9_60day` | game_v9 0..32 × 60, clocks | equal to `p3-7_final_60day` on 16,060 shared values (schema_version only) |
| `p4-7_final_v9_120day` | game_v9 0..32 × 120, clocks | same-host W11 baseline (0.000890 ms) |
| `p4-7_final_v10_10day` | game_v10 0..64 × 10 | W targets |
| `p4-7_final_v10_120day` | game_v10 0..32 × 120, clocks | W targets |

W targets (`p4-7_final_targets/w_targets.md`; v9 evaluated the same way in `p4-7_v9_reference_targets/`). Values are median / p10 / p90 / min / max across seeds unless stated.

| # | v9 (Pass 3 unchanged) | v10 final | v10 |
|---|---|---|---|
| W1 | determinism 64/64, 32/32; legacy/v8/v9 identical | same | PASS |
| W2 dehydration day 10 / day 120 | 0/0/0/0/0 · 0/0/0/0/1 | 0/0/0/0/0 · 0/0/0/0/0 | PASS |
| W3 starvation day 10 / day 120 | 0 · 0 | 0 · 0 | PASS |
| W4 settlements at every day sample | 1 | 1 | PASS |
| W5 births by day 4 | 7 / 3 / 7 / 0 / 7 | 7 / 7 / 7 / 7 / 7 | PASS |
| W5 alive day 10 | 42 / 31 / 42 / 24 / 42 | 42 / 42 / 42 / 42 / 42 | PASS |
| W5 longest stagnation (days) | 0 / 0 / 15.8 / 0 / 39 | 0 / 0 / 0 / 0 / 2 | PASS |
| W6 old-age deaths by day 30 | 0 | 9 / 6 / 12 / 5 / 13 | PASS |
| W6 births − deaths days 30–120 | 0 / 0 / 7.8 / 0 / 18 | 0 / −1 / 0 / −3 / 1 | PASS |
| W6 alive at day samples ≥ 30 | 42 / 42 / 42 / 24 / 42 | 42 / 41 / 42 / 37 / 42 | PASS |
| W6 child share day 120 | 0 | 0.095 / 0.048 / 0.170 / 0.024 / 0.195 | PASS |
| W6 elder share day 120 | 0 | 0.262 / 0.214 / 0.333 / 0.171 / 0.381 | PASS |
| W7 autumn-closing stock vs required | no seasons | 581 vs 150.5 (paired margin min 172.5) | PASS |
| W7 max stock − 3 × max target | +486 (no target) | −455 / −544 / −374 / −650 / −359 | PASS |
| W8 noon idle, spring–autumn (v9: all days) | 0.476 / 0.333 / 0.702 / 0.310 / 0.738 | 0.810 / 0.739 / 0.902 / 0.691 / 0.929 | FAIL |
| W8 forage trips/day summer+autumn | 0 | 0 / 0 / 74 / 0 / 91 (summer 19, autumn 0) | FAIL |
| W9 night in shelter | 0.976 / 0.929 / 1.0 / 0.905 / 1.0 | 1.0 / 0.955 / 1.0 / 0.929 / 1.0 | PASS |
| W9 evening near hearth | 0.586 / 0.318 / 0.792 / 0.130 / 0.900 | 0.554 / 0.348 / 0.766 / 0.281 / 0.813 | FAIL (min) |
| W10 T6a 10 / 120 (median) | 0.0005 / 0.0008 | 0.0010 / 0.0088 | PASS |
| W10 home distance day 10 | 5.36 / 3.63 / 7.94 / 2.76 / 10.52 | 7.06 / 5.61 / 8.31 / 5.14 / 11.05 | FAIL |
| W10 social / agent-day 10 / 120 | 0.645 / 0.841 | 0.650 / 0.835 (min 0.592 / 0.804) | PASS |
| W11 ms per agent-tick 10 / 120 | 0.000780 / 0.000890 | 0.000989 / 0.001210 (limits 0.001170 / 0.001335) | PASS |

120-day demography (v10 medians): day 10 alive 42, births 18, children 7, elders 3; day 30: births 27, old-age deaths 9, elders 8.5; day 60: 47 births, 29 deaths, elders 13; day 90: 71 / 53; day 120: 91 births, 73 old-age deaths, children 4, elders 11, mean age 31.0 days; alive median 42 throughout, minimum 37–42 from day 30. Stockpile swings with the seasons: 157 (spring median), 309.5 (summer), 491.5 (autumn), 254 (winter).

Seasons (v10 final, opening-checkpoint attribution): supply/harvest/need per day spring 108/107/97.0, summer 180/138/97.0, autumn 144/121/98.9, winter 72/66/98.9; regrowth lost at the cap 2 / 39 / 20 / 10 per day; forage trips 0 / 19 / 0 / 0 per day.

Seeds 7 and 8, v9 → v10 (alive, births, old-age deaths, stockpile at days 4 / 10 / 60 / 120):
- seed 7: v9 31/7/0/107 · 42/18/0/178 · 42/18/0/659 · 42/18/0/1259 → v10 31/7/0/125 · 42/18/0/296 · 41/47/30/212 · 42/91/73/268 (no more unbounded hoard).
- seed 8: v9 24/0/0/19 · 24/0/0/13 · 42/18/0/47 · 42/18/0/0 → v10 31/7/0/131 · 42/18/0/289 · 42/45/27/230 · 42/92/74/276 (no more stagnation).

## Pass 5 baseline

Part 0 (2026-10-11, Mac `hyunlordui-MacBookPro.local`, Darwin 27.0.0 arm64), base `5cbcbc4f` + goal commit `175c2576`. All runs release, `--workers 4`, from a frozen copy of the base probe binary built before any Pass 5 edit.

| Label | Command | Comparison |
|---|---|---|
| `p5-0_v10_10day` | game_v10 0..64 × 10, `--check-determinism` | DETERMINISM PASS; equal to `p4-7_final_v10_10day` on all 4,796 shared values (0 differences) |
| (not kept) | the same with `--clock-samples 02:00,12:00,19:30` | DETERMINISM PASS; 397 differences, all under `rhythm` / `runs[*].rhythm`, because the Pass 4 reference ran without clock samples; every other value identical |
| `p5-0_v9_10day` | game_v9 0..64 × 10, clocks 02:00,12:00,19:30 | DETERMINISM PASS; equal to `p4-7_final_v9_10day` on all 4,796 shared values |
| `p5-0_v10_120day` | game_v10 0..32 × 120, clocks 02:00,12:00,19:30 | DETERMINISM PASS; equal to `p4-7_final_v10_120day` on all 40,855 shared values |
| `p5-0_v10_30day_seeds7-9` | game_v10 7..10 × 30, clocks 06:00,08:00,10:00,12:00,15:00,18:00,19:30,02:00 | rhythm: noon idle median 0.800 (seeds 0.781–0.829), evening near hearth 0.806, night in shelter 1.0 |

Timing on this host, Part 0 (median ms per agent-tick, runs not isolated from each other): v10 10-day 0.000994, v9 10-day 0.000856, v10 120-day 0.001182, v10 30-day 0.001085. X12 uses a same-host v10 measurement taken in Part 5/7 next to the v11 runs.

Fingerprints of `game_v10_scene(seed)` after 1,440 ticks (`sim_test::probe::fingerprint`), the constants of the Pass 5 `*_v10_unchanged` tests: seed 0 `0e38264bd3abe6ba` (forage trips 64, haul deposits 22), seed 1 `3b0590ec71558a58` (51, 27), seed 2 `d265695444b8b314` (71, 21), seed 3 `48120c365d543729` (49, 24). The Pass 4 v9 constants stay in `harness_forage_v9_unchanged`.

### Noon-idle mechanism (game_v10 seeds 7..9, 30 days)

Share of all residents in each state at each clock sample, mean over display days 1–29 of the three seeds (n = rows), and the total food left on the twelve bushes (capacity 48). Measured with a throwaway integration test (not committed) that classifies each agent: Idle split into hauling (`haul_due`) and plain idle; Seeking/Consuming of GatherFood = foraging; of Food = eating at a bush; of Agent = social; of Sleep = sleeping; Water and ConstructionSite = other.

| Clock | n | Idle | Hauling | Foraging | Eating at a bush | Social | Sleeping | Other | Bush food total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 06:00 | 90 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.938 | 0.062 | 39.8 |
| 08:00 | 90 | 0.360 | 0.032 | 0.173 | 0.012 | 0.356 | 0.000 | 0.067 | 32.2 |
| 10:00 | 87 | 0.621 | 0.052 | 0.016 | 0.010 | 0.182 | 0.000 | 0.120 | 8.0 |
| 12:00 | 87 | 0.760 | 0.025 | 0.055 | 0.029 | 0.014 | 0.000 | 0.117 | 8.0 |
| 15:00 | 87 | 0.696 | 0.028 | 0.143 | 0.028 | 0.010 | 0.000 | 0.094 | 10.7 |
| 18:00 | 87 | 0.856 | 0.007 | 0.014 | 0.004 | 0.007 | 0.000 | 0.111 | 5.5 |
| 19:30 | 87 | 0.663 | 0.062 | 0.136 | 0.042 | 0.027 | 0.000 | 0.070 | 9.0 |
| 02:00 | 90 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.999 | 0.001 | 22.0 |

Reading: everybody wakes at 06:00 to bushes refilled overnight (39.8 of 48). By 08:00 the morning rush is under way (foraging 17 %, social 36 %); by 10:00 the bushes hold 8 units in total and stay there through noon, so neither the Forage arm (store above target) nor the opportunistic Gather arm (nothing within 8 tiles) has anything to do. Noon is 76 % plain idle, 12 % water trips and only 5.5 % foraging. Ground eating is small (1–4 %) at every hour because hungry residents find bushes near empty; the store feeds them only when no bush holds food. The evening uptick (13.6 % foraging at 19:30) is opportunistic gathering of the afternoon regrowth inside the hoard cap.

## Pass 5 Part 1 — v10 baseline with the schema-5 instruments

`p5-1_v10_120day`: game_v10 0..32 × 120, `--check-determinism --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, built from the Part 1 code (`2fed42d9` + the two display-only meal tallies). DETERMINISM PASS. Against `p5-0_v10_120day` (schema 4): all 40,855 shared values identical except `schema_version` (4 → 5); the schema-5 keys are additions.

Meals (v10): store meals 86 / 85 / 88 / 85.5 per day (spring / summer / autumn / winter medians over seed-days), ground meals 7 / 10 / 10 / 11, store share from day 3 0.919 / 0.895 / 0.896 / 0.887. **v10 already takes about nine meals in ten from the store**, contrary to the planner's reading in section 3.2: the relief pass (`StockpileConsumeSystem`, hunger ≥ 50 within 8 tiles, every 5 ticks) skips only members on the ground-eat path, so sleepers in the shelters (hunger crosses 50 during the 21:00–06:00 rest) and members busy at camp (social, water, building, hauling) are fed from the store at any hour. A throwaway hour-by-hour count over seeds 7–9 × 30 days: v10 store meals are spread over all 24 hours (130–512 per hour, 3,200 of 7,333 inside 21:00–06:00) with 695 ground meals; the Part 2 v11 rule moves them into the two meals (06:00–07:00: 3,141, 18:00–19:00: 2,899 of 8,240) with 122 ground meals. X4 therefore does not discriminate v10 from v11; the meal rule's visible effect is the timing (breakfast and dinner at the hearth) and the camp wait.

Work by hour (v10, all days ≥ 2; per-seed medians, then median across seeds): adult work share 0 / 0 / 0.258 / 0.090 / 0.068 / 0.222 / 0 / 0.194 at 02:00 / 06:00 / 08:00 / 10:00 / 12:00 / 15:00 / 18:00 / 19:30; idle share (Idle and not hauling) 0 / 0 / 0.357 / 0.595 / 0.786 / 0.667 / 0.869 / 0.690. Spring–autumn: adult work 0.083 at 12:00 and 0.242 at 15:00; idle 0.774 at 12:00 (X8 asks ≥ 0.35, ≥ 0.25 and ≤ 0.60). Pass 4's `noon_idle_share` (haulers counted idle) is 0.829 median over all days in this run (Pass 4 final, spring–autumn: 0.810).

Fire and wood: all zero in v10 (no firewood rule).

## Pass 5 Part 5 — tuning

Rounds use `game_v11` 0..64 × 10 (`--check-determinism`, clocks 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30) and 0..32 × 120 (same flags), evaluated by `evaluate_pass5.py` against the same-host v10 timing of Parts 0–1 (`p5-0_v10_10day`, `p5-1_v10_120day`). X1's reproduction receipts are only given in Part 7, so X1 shows FAIL on them in every round. Values: median / p10 / p90 / min / max across seeds (or seed-days where the target says so).

### Round 1 (`p5-5_round1_*`, code `5182fa86`: the section 7–9 defaults)

Configuration: meals 06:00–08:00 and 18:00–20:00, `meal_hunger` 25, `camp_radius` 8; firewood 10:00–17:00, radius 18, haul 2, 1 per spot, regen 500‰, `base_days` 1.5, burn [2, 1, 2, 3] 18:00–07:00, cold 22:00–06:00 ×1.5; deadwood 20 × 6 at 5–18; forage and stock target as v10 (`base_days` 1.5, `hoard_cap_factor` 2.0, forage 06:00–18:00, haul 4). Both runs DETERMINISM PASS.

| Target | Clause | Verdict | Observed |
|---|---|---|---|
| X1 | legacy_pin | FAIL | false |
| X1 | twins_10 | PASS | 64 |
| X1 | twins_120 | PASS | 32 |
| X1 | v10_repro | FAIL | false |
| X1 | v8_repro | FAIL | false |
| X1 | v9_repro | FAIL | false |
| X10 | home_distance | PASS | 6.89 / 5.58 / 8.32 / 5.31 / 10.7 |
| X10 | social_10 | PASS | 0.628 / 0.585 / 0.658 / 0.554 / 0.697 |
| X10 | social_120 | PASS | 0.824 / 0.802 / 0.844 / 0.79 / 0.86 |
| X10 | t6a_10 | PASS | 0.00124 / 0.000248 / 0.00265 / 0 / 0.00347 |
| X10 | t6a_120 | PASS | 0.00859 / 0.00668 / 0.0118 / 0.00337 / 0.0138 |
| X11 | day10 | PASS | 0 |
| X11 | day120 | PASS | 0 |
| X12 | day10 | PASS | 0.00114 / 0.00107 / 0.00126 / 0.00103 / 0.00144 |
| X12 | day120 | PASS | 0.00131 / 0.00124 / 0.00161 / 0.00121 / 0.00166 |
| X2 | deaths_dehydration_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_dehydration_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_120 | PASS | 0 / 0 / 0 / 0 / 2 |
| X3 | alive_band | PASS | 42 / 41 / 42 / 33 / 42 |
| X3 | alive_day10 | PASS | 42 / 42 / 42 / 41 / 42 |
| X3 | birth_by_day4 | PASS | 7 / 7 / 7 / 7 / 7 |
| X3 | child_share | PASS | 0.0952 / 0.0238 / 0.17 / 0 / 0.195 |
| X3 | elder_share | PASS | 0.262 / 0.214 / 0.366 / 0.171 / 0.429 |
| X3 | old_age_by_day30 | PASS | 9 / 6 / 12 / 5 / 13 |
| X3 | one_settlement | PASS | 1 / 1 / 1 / 1 / 1 |
| X3 | replacement | PASS | 0 / -1 / 0 / -3 / 1 |
| X3 | stagnation | FAIL | 0 / 0 / 0 / 0 / 9 |
| X4 | every_seed | PASS | 0.99 / 0.977 / 0.995 / 0.858 / 0.997 |
| X4 | store_share | PASS | 0.99 / 0.96 / 1 / 0.149 / 1 |
| X4 | winter | PASS | 1 / 0.99 / 1 / 0.149 / 1 |
| X5 | autumn | FAIL | 0 / 0 / 28 / 0 / 82 |
| X5 | spring | PASS | 41 / 12.9 / 60 / 0 / 73 |
| X5 | summer | PASS | 64 / 16 / 77 / 0 / 87 |
| X5 | winter | FAIL | 0 / 0 / 31.5 / 0 / 58 |
| X6 | autumn_wood | PASS | {"required": 218.40000000000003, "wood_stock": {"max": 347.0, "median": 334.5, "min": 302.0, "n": 128, "p10": 312.0, "p9 |
| X6 | lit_at_0200 | PASS | 1 / 1 / 1 / 1 / 1 |
| X7 | winter_cold | PASS | 0 / 0 / 0 / 0 / 0 |
| X8 | idle_1200 | PASS | 0.524 / 0.431 / 0.633 / 0.343 / 0.857 |
| X8 | work_1200 | PASS | 0.458 / 0.316 / 0.576 / 0.125 / 0.727 |
| X8 | work_1500 | PASS | 0.348 / 0.22 / 0.444 / 0.118 / 0.645 |
| X9 | evening_near_hearth | FAIL | 0.555 / 0.355 / 0.775 / 0.269 / 0.842 |
| X9 | night_in_shelter | PASS | 1 / 0.976 / 1 / 0.946 / 1 |

Seasons (round 1 120-day, medians over seed-days): store meals 98 / 96 / 100 / 101 per day (spring/summer/autumn/winter) with 0–2 ground meals; forage trips 41 / 64 / 0 / 0; stock 86 / 210.5 / 384.5 / 180 (v10: 157 / 309.5 / 491.5 / 254); wood stock 30 / 287 / 312 / 158 against targets 39 / 292.5 / 312 / 58.5; burn 26 / 13 / 26 / 39; fire lit at every 02:00 sample from day 3; cold member-hours 0; wood trips 14 / 21 / 23 / 0 per day. Work by hour (spring–autumn, seed medians): adult work 0.33 at 08:00, 0.13 at 10:00, 0.70 at 12:00, 0.42 at 15:00; idle 0.27 / 0.59 / 0.33 / 0.53.

Diagnosis. (1) X3 stagnation (one seed 9 days, another 4): the store is eaten from at both meals; serving from hunger 25 wastes up to 5 of each unit's 30 points, so the village eats about 98–103 units a day from the store against v10's 96 in total, the autumn reserve peaks lower and a few seeds reach the end of winter with an empty store; the birth gate pays food from the store, so spring births stall for a week (seed 22: store 0 at every 08:00 check on days 27–35, members 42 → 33). (2) X5: autumn and winter trips are 0 because the store sits above its target there — summer's surplus and opportunistic gathering up to 2 × target fill it past the autumn target (which carries the same winter-prep term), and the winter reserve keeps it above the winter target. (3) X9 minimum: on far-water maps adults are still walking back from the river at 19:30 (seed 55 in the 10-day set: idle adults 10–12 tiles from the hearth).

Screening before round 2 (a throwaway integration test, not committed: 8–32 seeds × 35–120 days per variant, measuring forage trips per season, evening share at 19:30, fire, deaths and a stagnation proxy — members ≤ 39 and no birth that day). Findings: `hoard_cap_factor` 1.5 restores winter trips (≈ 12/day) and helps the evening (≈ 0.57); `meal_hunger` 15 gives trips in every season (autumn ≈ 70, winter ≈ 40) but starves people over 63–120 days (7–21 starvation and 1–11 dehydration deaths per 16–32 seeds) — rejected; `meal_hunger` 30–35 removes the waste; `base_days` 2.5 with `meal_hunger` 30 and hoard 1.5 was the only family with the stagnation proxy ≤ 4 in all 32 seeds over 120 days (autumn and winter trips 0, evening ≈ 0.556 / min ≈ 0.30); forage end 17:00 or a firewood end 16:00 raises the evening median but brought back a 5–8-day stagnation seed; `camp_radius` 6/10 and a later dinner did not help.

### Round 2 (`p5-5_round2_*`, code `6184ed67`) — kept

Configuration: round 1 plus `meal_hunger` 30, `base_days` 2.5 and `hoard_cap_factor` 1.5 (stock target now set in `game_v11()`). Both runs DETERMINISM PASS.

| Target | Clause | Verdict | Observed |
|---|---|---|---|
| X1 | legacy_pin | FAIL | false |
| X1 | twins_10 | PASS | 64 |
| X1 | twins_120 | PASS | 32 |
| X1 | v10_repro | FAIL | false |
| X1 | v8_repro | FAIL | false |
| X1 | v9_repro | FAIL | false |
| X10 | home_distance | PASS | 7.1 / 6.21 / 8.97 / 4.88 / 10.5 |
| X10 | social_10 | PASS | 0.619 / 0.585 / 0.658 / 0.548 / 0.694 |
| X10 | social_120 | PASS | 0.826 / 0.802 / 0.843 / 0.783 / 0.858 |
| X10 | t6a_10 | PASS | 0.00124 / 0.000248 / 0.0024 / 0 / 0.00322 |
| X10 | t6a_120 | PASS | 0.0085 / 0.00673 / 0.0122 / 0.00588 / 0.0148 |
| X11 | day10 | PASS | 0 |
| X11 | day120 | PASS | 0 |
| X12 | day10 | PASS | 0.00108 / 0.00105 / 0.00112 / 0.00103 / 0.00115 |
| X12 | day120 | PASS | 0.00128 / 0.00124 / 0.0015 / 0.00123 / 0.00154 |
| X2 | deaths_dehydration_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_dehydration_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X3 | alive_band | PASS | 42 / 41 / 42 / 35 / 42 |
| X3 | alive_day10 | PASS | 42 / 42 / 42 / 42 / 42 |
| X3 | birth_by_day4 | PASS | 7 / 7 / 7 / 7 / 7 |
| X3 | child_share | PASS | 0.107 / 0.0476 / 0.17 / 0 / 0.195 |
| X3 | elder_share | PASS | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 |
| X3 | old_age_by_day30 | PASS | 9 / 6 / 12 / 5 / 13 |
| X3 | one_settlement | PASS | 1 / 1 / 1 / 1 / 1 |
| X3 | replacement | PASS | 0 / -1 / 0 / -3 / 1 |
| X3 | stagnation | PASS | 0 / 0 / 0 / 0 / 4 |
| X4 | every_seed | PASS | 0.985 / 0.969 / 0.991 / 0.912 / 0.995 |
| X4 | store_share | PASS | 0.99 / 0.948 / 1 / 0.559 / 1 |
| X4 | winter | PASS | 1 / 0.98 / 1 / 0.769 / 1 |
| X5 | autumn | FAIL | 0 / 0 / 14.5 / 0 / 83 |
| X5 | spring | PASS | 34 / 12 / 61 / 0 / 77 |
| X5 | summer | PASS | 64 / 0 / 76.3 / 0 / 94 |
| X5 | winter | FAIL | 0 / 0 / 28 / 0 / 57 |
| X6 | autumn_wood | PASS | stock 336 / 313 / 341 / 305 / 348; required 218.4 |
| X6 | lit_at_0200 | PASS | 1 / 1 / 1 / 1 / 1 |
| X7 | winter_cold | PASS | 0 / 0 / 0 / 0 / 0 |
| X8 | idle_1200 | PASS | 0.524 / 0.429 / 0.664 / 0.333 / 0.805 |
| X8 | work_1200 | PASS | 0.475 / 0.305 / 0.631 / 0.167 / 0.72 |
| X8 | work_1500 | PASS | 0.35 / 0.215 / 0.459 / 0.133 / 0.545 |
| X9 | evening_near_hearth | FAIL | 0.556 / 0.354 / 0.783 / 0.303 / 0.815 |
| X9 | night_in_shelter | PASS | 1 / 0.976 / 1 / 0.951 / 1 |

Seasons (120-day medians over seed-days, spring / summer / autumn / winter): store meals 95 / 92 / 96 / 98 per day, ground meals 0 / 3 / 2 / 0 (the waste is gone: store meals now sit at or below v10's total); stock 193 / 326 / 504 / 282 (v10: 157 / 309.5 / 491.5 / 254); forage trips 34 / 64 / 0 / 0 per day; wood stock 30 / 287 / 313 / 159 against targets 39 / 292.5 / 312 / 58.5; burn 26 / 13 / 26 / 39; fire lit at every 02:00 sample from day 3 in every seed; cold member-hours 0; wood trips 14 / 21 / 23 / 0 per day (wood gathered 28 / 42 / 46 / 0). Work by hour (seed medians, all days ≥ 2 → spring–autumn): adult work 0.286 → 0.294 at 08:00, 0.090 → 0.109 at 10:00, 0.220 → 0.475 at 12:00, 0.307 → 0.350 at 15:00, 0.222 → 0.231 at 19:30; idle 0.310 → 0.291, 0.615 → 0.595, 0.738 → 0.524, 0.667 → 0.619, 0.726 → 0.702. Winter has no firewood or forage trips, which is why the all-days noon values are lower.

No round 3. Twelve more variants around round 2 were screened (32 seeds × 120 days each; the screen reproduces round 2 exactly: evening 0.556 / min 0.303, stagnation max 4): forage end 17:00, forage haul 2 or 6, firewood 10:00–16:00, 09:00–16:00 or 12:00–17:00, forage start 07:00, dinner 17:00–19:00, `camp_radius` 10, `meal_hunger` 35 (with and without firewood end 16:00). None raised the evening minimum above 0.31 and most brought back a 5–10-day stagnation seed (forage haul 2 also caused 5 starvation deaths). A third formal round would have reproduced round 2, so round 2 is kept.

Three strikes — kept evidence and hypotheses:
- X5 (autumn and winter forage trips 0): purposeful trips run only while stock in hand is below the store target. Summer's surplus plus opportunistic gathering (up to 1.5 × target) carries the store past the autumn target (the same winter-prep term applies in summer and autumn), and the winter-prep reserve keeps the store above the winter target. The only lever that produced trips in every season (`meal_hunger` 15, which wastes half of each served unit) starved residents over 63–120 days. Making trips happen in those seasons needs a change to the target rule or the opportunistic arm, which this pass does not allow.
- X9 evening minimum (0.303, median 0.556 passes): the low seeds are far-water maps where adults are still walking back from the river at 19:30 (11–19 % of idle adults are hauling or returning), and elders (≈ 25 % of idle adults by then) are tethered at radius 4 while "near the hearth" counts radius 3. Neither the elder tether nor water placement is a Pass 5 knob. v10 had the same weakness (Pass 4 W9 minimum 0.281).

## Pass 5 Part 7 — final verification

### Reproduction of the frozen versions (code head `a27589ad`, run back to back with nothing else competing)

| Label | Command | Comparison with Pass 4 final |
|---|---|---|
| `p5-7_final_legacy` | legacy 0..4 × 10 | DETERMINISM PASS; all 3,446 shared values equal except `schema_version` (4 → 5) |
| `p5-7_final_v8` | game 0..64 × 10 | DETERMINISM PASS; all 4,831 shared values equal except `schema_version` |
| `p5-7_final_v9_10day` | game_v9 0..64 × 10, clocks 02:00,12:00,19:30 | DETERMINISM PASS; all 4,796 shared values equal except `schema_version` |
| `p5-7_final_v9_120day` | game_v9 0..32 × 120, same clocks | DETERMINISM PASS; all 40,368 shared values equal except `schema_version` |
| `p5-7_final_v10_10day` | game_v10 0..64 × 10 | DETERMINISM PASS; all 4,796 shared values equal except `schema_version`; same-host X12 baseline (median 0.000986 ms per agent-tick) |
| `p5-7_final_v10_120day` | game_v10 0..32 × 120, same clocks | DETERMINISM PASS; all 40,855 shared values equal except `schema_version`; X12 baseline for 120 days (median 0.001189 ms) |

### v11 after the review fixes, round 2 configuration (`p5-7_review_head_*`)

The first final v11 pair (64 × 10 and 32 × 120, eight clocks, DETERMINISM PASS) ran on `a27589ad`. The review fix shifted trajectories enough that seed 22 now stalls for 5 days after each winter (X3 limit 4): its spring harvest is 85–95 units a day against v10's 100–108, its store is 0 at every 08:00 check on days 29–35 and the food-paid birth gate stays shut. Everything else matched round 2.

| Target | Clause | Verdict | Observed |
|---|---|---|---|
| X1 | legacy_pin | PASS | true |
| X1 | twins_10 | PASS | 64 |
| X1 | twins_120 | PASS | 32 |
| X1 | v10_repro | PASS | true |
| X1 | v8_repro | PASS | true |
| X1 | v9_repro | PASS | true |
| X10 | home_distance | PASS | 6.93 / 6.14 / 8.97 / 5.36 / 10.4 |
| X10 | social_10 | PASS | 0.622 / 0.59 / 0.659 / 0.527 / 0.694 |
| X10 | social_120 | PASS | 0.828 / 0.802 / 0.842 / 0.792 / 0.859 |
| X10 | t6a_10 | PASS | 0.00124 / 0.000322 / 0.00265 / 0 / 0.00322 |
| X10 | t6a_120 | PASS | 0.00851 / 0.00656 / 0.0124 / 0.00632 / 0.0144 |
| X11 | day10 | PASS | 0 |
| X11 | day120 | PASS | 0 |
| X12 | day10 | PASS | 0.00108 / 0.00105 / 0.00147 / 0.00103 / 0.00209 |
| X12 | day120 | PASS | 0.00126 / 0.00123 / 0.0013 / 0.00121 / 0.00133 |
| X2 | deaths_dehydration_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_dehydration_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X3 | alive_band | PASS | 42 / 41 / 42 / 35 / 42 |
| X3 | alive_day10 | PASS | 42 / 42 / 42 / 42 / 42 |
| X3 | birth_by_day4 | PASS | 7 / 7 / 7 / 7 / 7 |
| X3 | child_share | PASS | 0.107 / 0.0476 / 0.17 / 0 / 0.195 |
| X3 | elder_share | PASS | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 |
| X3 | old_age_by_day30 | PASS | 9 / 6 / 12 / 5 / 13 |
| X3 | one_settlement | PASS | 1 / 1 / 1 / 1 / 1 |
| X3 | replacement | PASS | 0 / -1 / 0 / -3 / 1 |
| X3 | stagnation | FAIL | 0 / 0 / 0 / 0 / 5 |
| X4 | every_seed | PASS | 0.985 / 0.969 / 0.991 / 0.916 / 0.994 |
| X4 | store_share | PASS | 0.99 / 0.948 / 1 / 0.552 / 1 |
| X4 | winter | PASS | 1 / 0.98 / 1 / 0.552 / 1 |
| X5 | autumn | FAIL | 0 / 0 / 15 / 0 / 85 |
| X5 | spring | PASS | 33 / 11 / 60 / 0 / 70 |
| X5 | summer | PASS | 64 / 0 / 77 / 0 / 94 |
| X5 | winter | FAIL | 0 / 0 / 23 / 0 / 57 |
| X6 | autumn_wood | PASS | stock 336 / 315 / 341 / 300 / 348; required 218.4 |
| X6 | lit_at_0200 | PASS | 1 / 1 / 1 / 1 / 1 |
| X7 | winter_cold | PASS | 0 / 0 / 0 / 0 / 0 |
| X8 | idle_1200 | PASS | 0.506 / 0.429 / 0.624 / 0.325 / 0.812 |
| X8 | work_1200 | PASS | 0.47 / 0.295 / 0.612 / 0.167 / 0.739 |
| X8 | work_1500 | PASS | 0.354 / 0.206 / 0.433 / 0.118 / 0.514 |
| X9 | evening_near_hearth | FAIL | 0.569 / 0.376 / 0.763 / 0.286 / 0.806 |
| X9 | night_in_shelter | PASS | 1 / 0.976 / 1 / 0.952 / 1 |

Screening `max_foragers_per_bush` 2 (the Pass 4 range is 1–2) on the throwaway screen (which reproduces this run exactly: stagnation max 5, evening 0.569 / 0.286) gave stagnation max 3 and evening 0.590 / min 0.333 with no deaths; adding forage haul 6 gave stagnation 2 but a worse evening, and `meal_hunger` 35 brought the 5-day seed back. Round 3 therefore takes `max_foragers_per_bush` 2.

### Round 3 = final v11 (`p5-5_round3_*`, code `c77ee27c`) — kept

Configuration: round 2 plus `max_foragers_per_bush` 2. Runs on the final code (review fixes included), back to back with nothing competing; both DETERMINISM PASS. Evaluated against the same-host v10 timing of `p5-7_final_v10_*` and with the reproduction receipts of the final probes (legacy, v8, v9, v10 identical to Pass 4).

| Target | Clause | Verdict | Observed |
|---|---|---|---|
| X1 | legacy_pin | PASS | true |
| X1 | twins_10 | PASS | 64 |
| X1 | twins_120 | PASS | 32 |
| X1 | v10_repro | PASS | true |
| X1 | v8_repro | PASS | true |
| X1 | v9_repro | PASS | true |
| X10 | home_distance | PASS | 7.48 / 5.82 / 9.06 / 4.26 / 10.7 |
| X10 | social_10 | PASS | 0.608 / 0.559 / 0.651 / 0.53 / 0.679 |
| X10 | social_120 | PASS | 0.813 / 0.79 / 0.836 / 0.766 / 0.851 |
| X10 | t6a_10 | PASS | 0.000991 / 0.000248 / 0.00273 / 0 / 0.00421 |
| X10 | t6a_120 | PASS | 0.0084 / 0.00677 / 0.0115 / 0.00634 / 0.0139 |
| X11 | day10 | PASS | 0 |
| X11 | day120 | PASS | 0 |
| X12 | day10 | PASS | 0.00109 / 0.00105 / 0.00111 / 0.00101 / 0.00114 |
| X12 | day120 | PASS | 0.0013 / 0.00124 / 0.00187 / 0.00123 / 0.00204 |
| X2 | deaths_dehydration_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_dehydration_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| X2 | deaths_starvation_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| X3 | alive_band | PASS | 42 / 41 / 42 / 36 / 42 |
| X3 | alive_day10 | PASS | 42 / 42 / 42 / 42 / 42 |
| X3 | birth_by_day4 | PASS | 7 / 7 / 7 / 7 / 7 |
| X3 | child_share | PASS | 0.107 / 0.0476 / 0.17 / 0 / 0.195 |
| X3 | elder_share | PASS | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 |
| X3 | old_age_by_day30 | PASS | 9 / 6 / 12 / 5 / 13 |
| X3 | one_settlement | PASS | 1 / 1 / 1 / 1 / 1 |
| X3 | replacement | PASS | 0 / -1 / 0 / -3 / 1 |
| X3 | stagnation | PASS | 0 / 0 / 0 / 0 / 3 |
| X4 | every_seed | PASS | 0.986 / 0.968 / 0.992 / 0.937 / 0.995 |
| X4 | store_share | PASS | 0.99 / 0.949 / 1 / 0.674 / 1 |
| X4 | winter | PASS | 1 / 0.98 / 1 / 0.924 / 1 |
| X5 | autumn | FAIL | 0 / 0 / 0 / 0 / 139 |
| X5 | spring | PASS | 32.5 / 11 / 81.1 / 0 / 113 |
| X5 | summer | PASS | 83 / 0 / 120 / 0 / 144 |
| X5 | winter | FAIL | 0 / 0 / 21 / 0 / 85 |
| X6 | autumn_wood | PASS | stock 335 / 308 / 341 / 301 / 347; required 218.4 |
| X6 | lit_at_0200 | PASS | 1 / 1 / 1 / 1 / 1 |
| X7 | winter_cold | PASS | 0 / 0 / 0 / 0 / 0 |
| X8 | idle_1200 | PASS | 0.512 / 0.392 / 0.575 / 0.3 / 0.767 |
| X8 | work_1200 | PASS | 0.5 / 0.36 / 0.665 / 0.125 / 0.75 |
| X8 | work_1500 | PASS | 0.416 / 0.26 / 0.516 / 0.188 / 0.654 |
| X9 | evening_near_hearth | FAIL | 0.59 / 0.385 / 0.764 / 0.333 / 0.8 |
| X9 | night_in_shelter | PASS | 1 / 0.955 / 1 / 0.929 / 1 |

Seasons (120-day medians over seed-days, spring / summer / autumn / winter): store meals 95 / 93 / 96 / 98 per day, ground meals 0 / 3 / 2 / 0, store share 1.000 / 0.969 / 0.980 / 1.000; stock 202 / 340 / 528 / 289.5 (v10 157 / 309.5 / 491.5 / 254); harvest 108 / 140 / 120 / 71; forage trips 32.5 / 83 / 0 / 0 per day; wood stock 28 / 287 / 311 / 158 against targets 39 / 292.5 / 312 / 58.5; burn 26 / 13 / 26 / 39 per day; fire lit at every 02:00 sample from day 3 in every seed; cold member-hours 0; wood trips 14 / 22 / 21 / 0 per day (gathered 28 / 44 / 42 / 0). Day-120 totals (median, min–max): store meals 11,285.5 (10,522–11,389), ground meals 158 (60–707), forage trips 3,455.5, wood trips 1,719 (1,611–1,793), wood gathered 3,218, gathering preempted 1 (0–15), cold member-hours 0. Demography (medians): day 10 alive 42, births 18; day 30 births 27, old-age deaths 9, elders 8.5; day 60 46.5 / 29; day 90 71 / 53; day 120 91 / 73, children 4.5, elders 11.

Work by hour (seed medians; all days ≥ 2 → spring–autumn): adult work share 0.344 → 0.359 at 08:00, 0.097 → 0.118 at 10:00, 0.285 → 0.500 at 12:00, 0.382 → 0.416 at 15:00, 0 at 18:00, 0.260 → 0.261 at 19:30; idle share 0.262 → 0.238, 0.595 → 0.571, 0.667 → 0.512, 0.595 → 0.571, 0.857 → 0.855, 0.690 → 0.667. v10 (`p5-1_v10_120day`, spring–autumn): adult work 0.083 at 12:00 and 0.242 at 15:00, idle 0.774 at 12:00. Pass 4's `noon_idle_share` (haulers counted idle, all days including winter): v10 0.829 → v11 0.738.

v10 evaluated by the same script (`p5-7_v10_reference_targets`: 10-day from `p5-7_final_v10_10day`, 120-day from `p5-1_v10_120day`, which has all eight clocks and the same simulation values): X1–X4, X7, X10–X12 PASS; X5 spring, autumn and winter FAIL (0 / 19 / 0 / 0 trips per day); X6 FAIL (no fire); X8 FAIL (work 0.083 / 0.242, idle 0.774); X9 FAIL (0.554 / min 0.281).

## Pass 6 baseline

Part 0 (2026-10-11, Mac `hyunlordui-MacBookPro.local`, Darwin 27.0.0 arm64), base `682b4426` + goal commit. All runs release, `--workers 4`, from a frozen copy of the base probe binary built before any Pass 6 edit (the art run was running `cargo test` on the same Mac; Part 0 needs no timing).

| Label | Command | Comparison |
|---|---|---|
| `p6-0_v11_10day` | game_v11 0..64 × 10, `--check-determinism --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30` | DETERMINISM PASS; equal to `p5-5_round3_10day` on all 5,546 shared values (0 differences) |
| `p6-0_v11_120day` | game_v11 0..32 × 120, same flags | DETERMINISM PASS; equal to `p5-5_round3_120day` on all 49,773 shared values |
| `p6-0_v10_10day` | game_v10 0..64 × 10, same flags | DETERMINISM PASS; 397 differences, all under `rhythm` / `runs[*].rhythm`, because `p5-7_final_v10_10day` ran without clock samples (the same note as Pass 5 Part 0); every other value identical |
| `p6-0_v10_10day_noclock` | game_v10 0..64 × 10, `--check-determinism` (no clocks, as the reference) | DETERMINISM PASS; equal to `p5-7_final_v10_10day` on all 5,546 shared values |

Fingerprints of `game_v11_scene(seed)` after 1,440 ticks, the constants of `harness_v11_fingerprints_unchanged` (`harness_revitalize_pass6_frozen.rs`): seed 0 `270000e767fea504` (forage trips 87, wood trips 33), seed 1 `f1d0871987035e90` (57, 32), seed 2 `f9c54edccd2f5d82` (85, 25), seed 3 `5a22e3cd53b90a43` (57, 28). The Pass 5 v10 constants stay in `harness_firewood_v10_unchanged` and its siblings.

## Pass 6 Part 1 — v11 baseline with the schema-6 instruments

`p6-1_v11_10day` (game_v11 0..64 × 10) and `p6-1_v11_120day` (0..32 × 120), both `--check-determinism --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, built from the Part 1 code. Both DETERMINISM PASS. Against Part 0: all 5,546 / 49,773 shared values identical except `schema_version` (5 → 6); the schema-6 keys are additions (300 / 3,600 summary paths). Workspace debug tests 1,923 passed (Pass 5: 1,912), both Clippy variants clean.

`evaluate_pass6.py` on these runs (`p6-1_v11_targets`, the v11 value of every Y): Y2, Y3, Y4, Y9, Y11, Y12 PASS; Y1 lacks the legacy/v8/v9/v10 receipts (Part 7); Y5 autumn wood 42 vs summer 44 gathered per day (needs ≥ 1.5×); Y6 0 trip-days and 0 trips per winter (median); Y7 winter nights lit 1.000 in every seed (needs ≤ 0.97); Y8 cold 0; Y10 evening 0.590 / min 0.333; Y13 needs the same-host v11 timing. The two v11 measures Pass 6 compares with: **autumn adult work share at 12:00 median 0.458** (Y5 floor 0.408) and **winter food store at each winter's last periodic sample median 253.5** (min 42, 128 seed-winters; Y8 floor 202.8).

Wood by season (v11, medians over seed-days; stock, target and food at 08:00 checkpoints): wood stock 28 / 280 / 302 / 197 against targets 39 / 292.5 / 312 / 58.5; gathered 28 / 44 / 42 / 0 per day; wood trips 14 / 22 / 21 / 0; 02:00 lit 1.000 in every season (nights booked to their evening); cold member-hours 0; food store 203 / 293 / 510 / 322. Work by season (seed medians): adult work at 12:00 0.487 / 0.609 / 0.458 / 0.024 (spring / summer / autumn / winter), at 15:00 0.409 / 0.548 / 0.296 / 0; idle at 12:00 0.551 / 0.429 / 0.509 / 0.881. Winters (128 seed-winters): wood in hand at the first winter day 314 (280–326), nights out 0, trip-days 0 (max 1), trips 0 (max 24), food at the end 253.5 (42–307).

## Pass 6 Part 2 — autumn for wood, a lean winter

Gate (`gate_pass6.sh`) on code head `0a40b57c`: green — debug 1,931 / release 1,931 passed, 0 failed; both Clippy variants; forbidden math 0; bridge build; Godot import; strict check; smoke without `SCRIPT ERROR`/`ERROR:`; Python unit tests (`verification/p6-2/gate.txt`).

4.5 v11 guard (`p6-2_v11_guard_10day`, `p6-2_v11_guard_120day`, Part 2 binary, Part 0 flags): DETERMINISM PASS; identical to Part 1's v11 runs on all 5,846 / 53,373 shared values.

Informative v12 run at the Part 2 state (`p6-2_v12_120day`, game_v12 0..32 × 120, same flags, DETERMINISM PASS; urgent mode and the elder tether not yet in; evaluated with the v11 10-day guard as the short round, so only the long-round values are meaningful): Y2 0 deaths; Y3 PASS (stagnation max 4); Y4 PASS; Y5 autumn wood 48 vs summer 0 gathered per day (median), autumn noon work 0.616 (floor 0.408); Y6 7 trip-days and 108 wood trips per winter (median; 57 units gathered); Y7 winter lit 0.714 (min 0.679), other seasons 1.000; Y8 cold 0.231 (max 0.251), winter food at the end 229 (floor 202.8, min 20); Y10 evening 0.562 / min 0.313 (FAIL). Every one of the 128 seed-winters had a dark night, 2 in all but a few (max 3), first on winter night 6 (5–6): autumn's stock (142 at the first winter day, 130–173) plus the ground carries five nights. Wood by season (medians): stock 25 / 22 / 143 / 58 against targets 39 / 19.5 / 156 / 156; gathered 30 / 0 / 48 / 6 per day; wood trips 15 / 0 / 24 / 12.

## Pass 6 Part 3 — urgent mornings

4.5 v11 guard (`p6-3_v11_guard_10day`, `p6-3_v11_guard_120day`, Part 3 binary at `4cdc7125`): DETERMINISM PASS; identical to Part 1's v11 runs on all 5,846 / 53,373 shared values. The new tests (`harness_revitalize_pass6_urgent.rs`, 4) pass with the Pass 5 firewood, meal and preemption harnesses and the Part 0/2 tests; both Clippy variants clean.

Informative v12 at the Part 3 state (`p6-3_v12_120day`, code `4cdc7125`, DETERMINISM PASS; long-round values only): urgent trips 50 per winter (median; 3 a day in spring), 124 wood trips and 7 trip-days per winter, winter lit 0.714 (min 0.643), cold 0.236 (max 0.268), winter food at the end 227 (floor 202.8). Two regressions against the Part 2 state: Y3 stagnation max 6 (one seed) and Y9 noon work 0.305 / idle 0.645 (Part 2: 0.397 / 0.574). Work by season at 12:00: spring 0.409 (Part 2 0.553), summer 0.051 (v11 0.609), autumn 0.596. Summer noon work fell by design (summer gathers only for its own nights); urgent mornings move spring's wood work from noon to 07:00–10:00.

## Pass 6 Part 4 — elders at the evening fire

4.5 v11 guard (`p6-4_v11_guard_10day`, `p6-4_v11_guard_120day`, Part 4 binary at `c1e407e7`): DETERMINISM PASS; identical to Part 1's v11 runs on all 5,846 / 53,373 shared values. New tests `harness_revitalize_pass6_elders.rs` (2) pass.

## Pass 6 Part 5 — tuning

Rounds use `game_v12` 0..64 × 10 and 0..32 × 120, both `--check-determinism --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30` (the Part 0 flags), evaluated by `evaluate_pass6.py` with the Part 1 v11 measures (`p6-1_v11_targets`: autumn noon work 0.458, winter food at the end 253.5) and `--winter-burn-per-day 39 --cold-window 1320,360` (from `game_v12()`). Y1's reproduction receipts and Y13's same-host v11 timing are only given in Part 7, so Y1 and Y13 show FAIL in every round. Screens are throwaway runs (32 × 120, no twin, same clocks) from a probe built with `game_v12()` edited and restored; each is listed with its numbers.

### Round 1 (`p6-5_round1_*`, code `c1e407e7`: the section 7–9 start values)

Configuration: `winter_prep_days` 3.0, `winter_prep_in_summer` false, `winter_base_days` 4.0, `winter_regen_units_per_day` 6, `winter_deadwood_cap` 1, `night_season_from_evening` true, `urgent_start_minute` 420, `cold_hunger_permille` 1500, `elder_evening_tether` true; every Pass 5 value as in `game_v11()`. Both runs DETERMINISM PASS.

| Target | Clause | Verdict | Observed (median / p10 / p90 / min / max) |
|---|---|---|---|
| Y1 | legacy_pin | FAIL | false |
| Y1 | twins_10 | PASS | 64 |
| Y1 | twins_120 | PASS | 32 |
| Y1 | v10_repro | FAIL | false |
| Y1 | v11_repro | FAIL | false |
| Y1 | v8_repro | FAIL | false |
| Y1 | v9_repro | FAIL | false |
| Y2 | deaths_dehydration_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y2 | deaths_dehydration_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y2 | deaths_starvation_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y2 | deaths_starvation_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y3 | alive_band | PASS | 42 / 41 / 42 / 35 / 42 |
| Y3 | alive_day10 | PASS | 42 / 42 / 42 / 42 / 42 |
| Y3 | birth_by_day4 | PASS | 7 / 7 / 7 / 7 / 7 |
| Y3 | child_share | PASS | 0.107 / 0.0476 / 0.17 / 0 / 0.195 |
| Y3 | elder_share | PASS | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 |
| Y3 | old_age_by_day30 | PASS | 9 / 6 / 12 / 5 / 13 |
| Y3 | one_settlement | PASS | 1 / 1 / 1 / 1 / 1 |
| Y3 | replacement | FAIL | 0 / -1 / 0 / -4 / 1 |
| Y3 | stagnation | PASS | 0 / 0 / 0 / 0 / 4 |
| Y4 | every_seed | PASS | 0.987 / 0.966 / 0.993 / 0.928 / 0.996 |
| Y4 | store_share | PASS | 0.99 / 0.949 / 1 / 0.602 / 1 |
| Y4 | winter | PASS | 1 / 0.98 / 1 / 0.919 / 1 |
| Y5 | autumn_wood | PASS | autumn 48 / 0 / 129 / 0 / 148; summer 0 / 0 / 36 / 0 / 52 |
| Y5 | autumn_work | PASS | 0.604 / 0.434 / 0.729 / 0.244 / 0.766; floor 0.408 |
| Y6 | trip_days | PASS | 7 / 6 / 7 / 6 / 7 |
| Y6 | trips | PASS | 123 / 112 / 130 / 103 / 139 |
| Y7 | other_lit | PASS | 1 / 1 / 1 / 1 / 1 |
| Y7 | winter_lit | PASS | 0.714 / 0.679 / 0.714 / 0.643 / 0.75 |
| Y8 | winter_cold | PASS | 0.228 / 0.215 / 0.249 / 0.196 / 0.268 |
| Y8 | winter_food | PASS | 224 / 177 / 259 / 42 / 279; floor 203 |
| Y9 | idle_1200 | FAIL | 0.667 / 0.502 / 0.785 / 0.341 / 0.857 |
| Y9 | work_1200 | FAIL | 0.291 / 0.116 / 0.434 / 0.04 / 0.704 |
| Y9 | work_1500 | PASS | 0.422 / 0.204 / 0.518 / 0.133 / 0.667 |
| Y10 | evening_near_hearth | FAIL | 0.629 / 0.407 / 0.792 / 0.344 / 0.852 |
| Y10 | night_in_shelter | PASS | 1 / 0.955 / 1 / 0.931 / 1 |
| Y11 | home_distance | PASS | 7.5 / 6.03 / 8.89 / 4.88 / 11 |
| Y11 | social_10 | PASS | 0.636 / 0.592 / 0.668 / 0.553 / 0.7 |
| Y11 | social_120 | PASS | 0.814 / 0.794 / 0.843 / 0.768 / 0.856 |
| Y11 | t6a_10 | PASS | 0.00112 / 0.000495 / 0.00241 / 0 / 0.00421 |
| Y11 | t6a_120 | PASS | 0.00842 / 0.00673 / 0.0123 / 0.0052 / 0.0135 |
| Y12 | day10 | PASS | 0 |
| Y12 | day120 | PASS | 0 |
| Y13 | day10 | FAIL | 0.00112 / 0.00108 / 0.00189 / 0.00102 / 0.00259; limit – |
| Y13 | day120 | FAIL | 0.00139 / 0.00137 / 0.00199 / 0.00134 / 0.00211; limit – |

Seasons (round 1, 120-day medians over seed-days; spring / summer / autumn / winter): wood stock 25 / 21 / 143 / 57 against targets 39 / 19.5 / 156 / 156; gathered 30 / 0 / 48 / 6 per day; wood trips 15 / 3 / 24 / 14; urgent trips 4 / 0 / 0 / 7; 02:00 lit 1.000 / 1.000 / 1.000 / 0.708; cold member-hours 0 except winter; food store 159 / 263 / 487 / 317. Adult work at 12:00 (seed medians) 0.372 / 0.063 / 0.604 / 0.130; at 15:00 0.465 / 0.422 / 0.374 / 0.220. Winters (128): wood at the first winter day 141 (131–175), nights out 2 (1–3), first on night 6, trip-days 7, wood trips 123, urgent trips 49, cold member-hours 756, food at the end 224 (42–279); every seed-winter has a dark night.

Diagnosis. (1) Y7/Y8: autumn's 156 plus the ground carries about five winter nights, so the fire goes out twice a winter (lit 0.714 is at the bottom of the band, cold 0.228). One more night of preparation (`winter_prep_days` 4.0, +39 units) should leave about one. (2) Y9 (0.291 / idle 0.667): summer no longer gathers for winter, so summer noon work is 0.06 (v11 0.609, which was nearly all winter-prep wood); a seed's spring–autumn median over 21 days is its 11th value, so with seven summer days near zero it lands among the lower spring days. Urgent mornings (07:00) move spring's wood work before noon (spring noon 0.372 against 0.553 at the Part 2 state). (3) Y3 replacement (min −4, one seed) and Y10 minimum (0.344) are single seeds at the threshold.

Screens after round 1 (32 × 120, no twin; the screen reproduces the formal 120-day run of the same configuration exactly):

| Screen | Change from round 1 | Y3 stagnation max / replacement min | Y7 winter lit median / min | Y8 cold median / max; food median | Y9 work 12:00 / idle 12:00 | Y10 evening median / min | Dark nights per winter (mean) | Urgent trips per winter |
|---|---|---|---|---|---|---|---|---|
| A | `winter_prep_days` 4.0 | 4 / −4 | 0.857 / 0.786 | 0.107 / 0.143; 248 | 0.319 / 0.614 | 0.646 / 0.353 | 1.03 | 22 |
| B | A + `urgent_start_minute` 540 | 4 / −3 | 0.857 / 0.821 | 0.098 / 0.134; 246.5 | 0.317 / 0.619 | 0.640 / 0.368 | 1.01 | 18.5 |
| C | A + `urgent_start_minute` 480 | 5 / −4 | 0.857 / 0.786 | 0.107 / 0.143; 245.5 | 0.315 / 0.631 | 0.659 / 0.346 | 1.05 | 23.5 |

Spring noon work rises to 0.47–0.50 in A–C but the seed medians stay at 0.32: the seven summer days decide them. B is the only screen with every target but Y9 (and the Part 7 receipts) passing; round 2 takes it.

### Round 2 (`p6-5_round2_*`, code `3cf26767`) — kept

Configuration: round 1 plus `winter_prep_days` 4.0 and `urgent_start_minute` 540. Both runs DETERMINISM PASS; the 120-day run equals screen B on every value.

| Target | Clause | Verdict | Observed (median / p10 / p90 / min / max) |
|---|---|---|---|
| Y1 | legacy_pin | FAIL | false |
| Y1 | twins_10 | PASS | 64 |
| Y1 | twins_120 | PASS | 32 |
| Y1 | v10_repro | FAIL | false |
| Y1 | v11_repro | FAIL | false |
| Y1 | v8_repro | FAIL | false |
| Y1 | v9_repro | FAIL | false |
| Y2 | deaths_dehydration_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y2 | deaths_dehydration_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y2 | deaths_starvation_10 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y2 | deaths_starvation_120 | PASS | 0 / 0 / 0 / 0 / 0 |
| Y3 | alive_band | PASS | 42 / 41 / 42 / 35 / 42 |
| Y3 | alive_day10 | PASS | 42 / 42 / 42 / 42 / 42 |
| Y3 | birth_by_day4 | PASS | 7 / 7 / 7 / 7 / 7 |
| Y3 | child_share | PASS | 0.107 / 0.0476 / 0.17 / 0.0238 / 0.195 |
| Y3 | elder_share | PASS | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 |
| Y3 | old_age_by_day30 | PASS | 9 / 6 / 12 / 5 / 13 |
| Y3 | one_settlement | PASS | 1 / 1 / 1 / 1 / 1 |
| Y3 | replacement | PASS | 0 / -1 / 0 / -3 / 1 |
| Y3 | stagnation | PASS | 0 / 0 / 0 / 0 / 4 |
| Y4 | every_seed | PASS | 0.985 / 0.967 / 0.992 / 0.926 / 0.995 |
| Y4 | store_share | PASS | 0.99 / 0.948 / 1 / 0.677 / 1 |
| Y4 | winter | PASS | 1 / 0.98 / 1 / 0.945 / 1 |
| Y5 | autumn_wood | PASS | autumn 50 / 0 / 130 / 0 / 148; summer 0 / 0 / 36.3 / 0 / 54 |
| Y5 | autumn_work | PASS | 0.635 / 0.466 / 0.749 / 0.262 / 0.785; floor 0.408 |
| Y6 | trip_days | PASS | 5 / 5 / 6 / 5 / 6 |
| Y6 | trips | PASS | 96 / 89 / 109 / 84 / 117 |
| Y7 | other_lit | PASS | 1 / 1 / 1 / 1 / 1 |
| Y7 | winter_lit | PASS | 0.857 / 0.821 / 0.889 / 0.821 / 0.893 |
| Y8 | winter_cold | PASS | 0.0984 / 0.0804 / 0.125 / 0.0623 / 0.134 |
| Y8 | winter_food | PASS | 246 / 200 / 280 / 44 / 300; floor 203 |
| Y9 | idle_1200 | FAIL | 0.619 / 0.548 / 0.692 / 0.333 / 0.905 |
| Y9 | work_1200 | FAIL | 0.317 / 0.18 / 0.456 / 0.0417 / 0.68 |
| Y9 | work_1500 | PASS | 0.431 / 0.288 / 0.519 / 0.125 / 0.694 |
| Y10 | evening_near_hearth | PASS | 0.64 / 0.433 / 0.807 / 0.368 / 0.864 |
| Y10 | night_in_shelter | PASS | 1 / 0.955 / 1 / 0.929 / 1 |
| Y11 | home_distance | PASS | 7.68 / 5.91 / 8.8 / 4.79 / 11.3 |
| Y11 | social_10 | PASS | 0.624 / 0.587 / 0.661 / 0.563 / 0.685 |
| Y11 | social_120 | PASS | 0.812 / 0.788 / 0.843 / 0.769 / 0.855 |
| Y11 | t6a_10 | PASS | 0.000991 / 0.000248 / 0.00248 / 0 / 0.00371 |
| Y11 | t6a_120 | PASS | 0.00843 / 0.00676 / 0.0126 / 0.00487 / 0.0133 |
| Y12 | day10 | PASS | 0 |
| Y12 | day120 | PASS | 0 |
| Y13 | day10 | FAIL | 0.00109 / 0.00107 / 0.00113 / 0.00102 / 0.00128; limit – |
| Y13 | day120 | FAIL | 0.00139 / 0.00136 / 0.00146 / 0.00133 / 0.00154; limit – |

Seasons (120-day medians over seed-days; spring / summer / autumn / winter): wood stock 26 / 21 / 182 / 106 against targets 39 / 19.5 / 195 / 156; gathered 30 / 0 / 50 / 6 per day; wood trips 15 / 0 / 25 / 12; urgent trips 2 / 0 / 0 / 0 (p90 21 / 10 / 0 / 17); 02:00 lit 1.000 / 1.000 / 1.000 / 0.856; food store at 08:00 checkpoints in `y_targets.md`. Adult work at 12:00 (seed medians) 0.499 / 0.074 / 0.635 / 0.136; at 15:00 0.432 / 0.415 / 0.389 / 0.171; idle at 12:00 0.548 / 0.783 / 0.393 / 0.786. Winters (128): wood at the first winter day 202 (172–216), nights out 1 (0–2; 124 of 128 seed-winters have one), the dark night is the last winter night (night 7), trip-days 5 (5–6), wood trips 96, urgent trips 18.5, gathered 45, cold member-hours 336, food at the end 246 (44–300).

No round 3. Y9 is the only failing target and none of the six knobs reaches it: summer gathers only for its own nights (target 19.5), so summer noon work is 0.07 against v11's 0.609, which was almost entirely winter-prep wood gathered in summer; each seed's spring–autumn median over 21 days is its 11th value, so seven summer days near zero pull it to the lower spring days (0.32) even with spring at 0.50 and autumn at 0.64. Rounds 1 and 2 and screens A–C (urgent start 07:00, 08:00, 09:00) all fail Y9 between 0.29 and 0.32. A third round would reproduce round 2. Tuning rounds used: 2 of 3.

Gate on the round 2 head `d919dd0f` (code `3cf26767` + docs): green — debug 1,937 / release 1,937 passed, 0 failed; both Clippy variants; forbidden math 0; bridge build; Godot import; strict check; smoke clean; Python unit tests (`verification/p6-5/gate.txt`).

## Pass 6 Part 7 — final verification

### Reproduction of the frozen versions (code `3cf26767`, Part 7 probe binary)

| Label | Command | Comparison |
|---|---|---|
| `p6-7_final_legacy` | legacy 0..4 × 10, `--check-determinism` | DETERMINISM PASS; all 4,196 shared values equal `p5-7_final_legacy` except `schema_version` (5 → 6) |
| `p6-7_final_v8` | game 0..64 × 10 | DETERMINISM PASS; all 5,581 shared values equal `p5-7_final_v8` except `schema_version` |
| `p6-7_final_v9_10day` | game_v9 0..64 × 10, clocks 02:00,12:00,19:30 | DETERMINISM PASS; all 5,546 equal `p5-7_final_v9_10day` except `schema_version` |
| `p6-7_final_v9_120day` | game_v9 0..32 × 120, same clocks | DETERMINISM PASS; all 49,368 equal `p5-7_final_v9_120day` except `schema_version` |
| `p6-7_final_v10_10day` | game_v10 0..64 × 10 (no clocks, as the reference) | DETERMINISM PASS; all 5,546 equal `p5-7_final_v10_10day` except `schema_version` |
| `p6-7_final_v10_120day` | game_v10 0..32 × 120, clocks 02:00,12:00,19:30 | DETERMINISM PASS; all 49,855 equal `p5-7_final_v10_120day` except `schema_version` |
| `p6-7_final_v11_repro_10day` | game_v11 0..64 × 10, the eight Part 0 clocks | DETERMINISM PASS; all 5,546 equal `p5-5_round3_10day` except `schema_version` |
| `p6-7_final_v11_repro_120day` | game_v11 0..32 × 120, same clocks | DETERMINISM PASS; all 49,773 equal `p5-5_round3_120day` except `schema_version` |

The schema-6 keys are additions (300 / 3,600 summary paths). The legacy pin test `harness_population_legacy_scene_pinned` passes in every gate.

### Correction after the independent review (Y8 cold share)

Rows are written after a completed tick, so the 22:00 sample shows the 21:00 fire update (outside the cold window, nobody marked) and the 06:00 sample the 05:00 one. `evaluate_pass6.py` now books the cold window and the night to the system tick (`system_minute`, `system_night_season`); the X7/Pass 5 definition (row minute) understated the share. Re-evaluated with the same raw runs: round 1 winter cold 0.330 / max 0.375 (**FAIL**; the round 1 table above shows the old 0.228), round 2 **0.152 / max 0.196 (PASS)** (old 0.098 / 0.134). The v11 baseline is 0 either way (`p6-1_v11_targets` unchanged). Screens A–C were measured with the old definition; their cold values above are low by the same bias. `p6-5_round1_targets` and `p6-5_round2_targets` hold the corrected evaluations; nothing else in them changed.

### Final game_v12 and Y13 timing (back to back, nothing else running)

Run 06:53–07:02 KST on the Mac with no other Godot, cargo or probe process (checked with `pgrep` first), in this order: `p6-7_final_v11_10day`, `p6-7_final_v12_10day`, `p6-7_final_v11_120day`, `p6-7_final_v12_120day` (all `--check-determinism`, the eight clocks, code `3cf26767` = the production code of the final head `01fd135d`). All DETERMINISM PASS. The v12 pair equals round 2 on all 5,846 / 53,385 shared values; the v11 pair equals Part 1's v11 runs on all 5,846 / 53,373. Median ms per agent-tick: v11 0.001073 → v12 0.001068 (10 days, 1.00×), v11 0.001273 → v12 0.001421 (120 days, 1.12×); Y13 limit 1.2×.

Final evaluation (`p6-7_final_targets`, receipts from the reproduction table above and the gate's legacy pin test, same-host v11 timing):

| Target | Clause | v11 (Part 1) | v12 final | v12 verdict |
|---|---|---|---|---|
| Y1 | legacy_pin | false | true | PASS |
| Y1 | twins_10 | 64 | 64 | PASS |
| Y1 | twins_120 | 32 | 32 | PASS |
| Y1 | v10_repro | false | true | PASS |
| Y1 | v11_repro | true | true | PASS |
| Y1 | v8_repro | false | true | PASS |
| Y1 | v9_repro | false | true | PASS |
| Y2 | deaths_dehydration_10 | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 | PASS |
| Y2 | deaths_dehydration_120 | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 | PASS |
| Y2 | deaths_starvation_10 | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 | PASS |
| Y2 | deaths_starvation_120 | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 | PASS |
| Y3 | alive_band | 42 / 41 / 42 / 36 / 42 | 42 / 41 / 42 / 35 / 42 | PASS |
| Y3 | alive_day10 | 42 / 42 / 42 / 42 / 42 | 42 / 42 / 42 / 42 / 42 | PASS |
| Y3 | birth_by_day4 | 7 / 7 / 7 / 7 / 7 | 7 / 7 / 7 / 7 / 7 | PASS |
| Y3 | child_share | 0.107 / 0.0476 / 0.17 / 0 / 0.195 | 0.107 / 0.0476 / 0.17 / 0.0238 / 0.195 | PASS |
| Y3 | elder_share | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 | 0.262 / 0.214 / 0.341 / 0.171 / 0.381 | PASS |
| Y3 | old_age_by_day30 | 9 / 6 / 12 / 5 / 13 | 9 / 6 / 12 / 5 / 13 | PASS |
| Y3 | one_settlement | 1 / 1 / 1 / 1 / 1 | 1 / 1 / 1 / 1 / 1 | PASS |
| Y3 | replacement | 0 / -1 / 0 / -3 / 1 | 0 / -1 / 0 / -3 / 1 | PASS |
| Y3 | stagnation | 0 / 0 / 0 / 0 / 3 | 0 / 0 / 0 / 0 / 4 | PASS |
| Y4 | every_seed | 0.986 / 0.968 / 0.992 / 0.937 / 0.995 | 0.985 / 0.967 / 0.992 / 0.926 / 0.995 | PASS |
| Y4 | store_share | 0.99 / 0.949 / 1 / 0.674 / 1 | 0.99 / 0.948 / 1 / 0.677 / 1 | PASS |
| Y4 | winter | 1 / 0.98 / 1 / 0.924 / 1 | 1 / 0.98 / 1 / 0.945 / 1 | PASS |
| Y5 | autumn_wood | autumn 42 / 0 / 58 / 0 / 76; summer 44 / 0 / 113 / 0 / 139 | autumn 50 / 0 / 130 / 0 / 148; summer 0 / 0 / 36.3 / 0 / 54 | PASS |
| Y5 | autumn_work | 0.458 / 0.283 / 0.639 / 0.231 / 0.722 | 0.635 / 0.466 / 0.749 / 0.262 / 0.785 | PASS |
| Y6 | trip_days | 0 / 0 / 1 / 0 / 1 | 5 / 5 / 6 / 5 / 6 | PASS |
| Y6 | trips | 0 / 0 / 19.3 / 0 / 24 | 96 / 89 / 109 / 84 / 117 | PASS |
| Y7 | other_lit | 1 / 1 / 1 / 1 / 1 | 1 / 1 / 1 / 1 / 1 | PASS |
| Y7 | winter_lit | 1 / 1 / 1 / 1 / 1 | 0.857 / 0.821 / 0.889 / 0.821 / 0.893 | PASS |
| Y8 | winter_cold | 0 / 0 / 0 / 0 / 0 | 0.152 / 0.116 / 0.178 / 0.0982 / 0.196 | PASS |
| Y8 | winter_food | 254 / 196 / 282 / 42 / 307 | 246 / 200 / 280 / 44 / 300 | PASS |
| Y9 | idle_1200 | 0.512 / 0.392 / 0.575 / 0.3 / 0.767 | 0.619 / 0.548 / 0.692 / 0.333 / 0.905 | FAIL |
| Y9 | work_1200 | 0.5 / 0.36 / 0.665 / 0.125 / 0.75 | 0.317 / 0.18 / 0.456 / 0.0417 / 0.68 | FAIL |
| Y9 | work_1500 | 0.416 / 0.26 / 0.516 / 0.188 / 0.654 | 0.431 / 0.288 / 0.519 / 0.125 / 0.694 | PASS |
| Y10 | evening_near_hearth | 0.59 / 0.385 / 0.764 / 0.333 / 0.8 | 0.64 / 0.433 / 0.807 / 0.368 / 0.864 | PASS |
| Y10 | night_in_shelter | 1 / 0.955 / 1 / 0.929 / 1 | 1 / 0.955 / 1 / 0.929 / 1 | PASS |
| Y11 | home_distance | 7.48 / 5.82 / 9.06 / 4.26 / 10.7 | 7.68 / 5.91 / 8.8 / 4.79 / 11.3 | PASS |
| Y11 | social_10 | 0.608 / 0.559 / 0.651 / 0.53 / 0.679 | 0.624 / 0.587 / 0.661 / 0.563 / 0.685 | PASS |
| Y11 | social_120 | 0.813 / 0.79 / 0.836 / 0.766 / 0.851 | 0.812 / 0.788 / 0.843 / 0.769 / 0.855 | PASS |
| Y11 | t6a_10 | 0.000991 / 0.000248 / 0.00273 / 0 / 0.00421 | 0.000991 / 0.000248 / 0.00248 / 0 / 0.00371 | PASS |
| Y11 | t6a_120 | 0.0084 / 0.00677 / 0.0115 / 0.00634 / 0.0139 | 0.00843 / 0.00676 / 0.0126 / 0.00487 / 0.0133 | PASS |
| Y12 | day10 | 0 | 0 | PASS |
| Y12 | day120 | 0 | 0 | PASS |
| Y13 | day10 | 0.00114 / 0.00106 / 0.00144 / 0.00103 / 0.00171 | 0.00107 / 0.00104 / 0.0011 / 0.00101 / 0.00111 | PASS |
| Y13 | day120 | 0.00133 / 0.00128 / 0.00161 / 0.00125 / 0.00165 | 0.00142 / 0.00137 / 0.00172 / 0.00133 / 0.00175 | PASS |

Every Y target passes except Y9 (three-strike stop, round 2 notes). Full gate on the final code head `01fd135d`: green — debug 1,941 / release 1,941 passed (Pass 5: 1,912 each; +29 Pass 6 tests), 0 failed, 0 ignored; both Clippy variants; forbidden math 0; bridge build; Godot import; strict check; smoke without `SCRIPT ERROR`/`ERROR:`; Python unit tests (`verification/p6-7/gate.txt`).

## Pass 7 baseline

Base `fe60de21` (code head `01fd135d`), release probe built from the base and frozen as a copy (`probe_base`), `--workers 4`, run 13:48–13:55 KST. Flags exactly those of Pass 6's final runs (`## Pass 6 Part 7`): the eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30` for v11 and v12, none for legacy.

| Label | Command | Comparison |
|---|---|---|
| `p7-0_v12_10day` | game_v12 0..64 × 10, `--check-determinism`, the eight clocks | DETERMINISM PASS; all 5,846 shared values equal `p6-7_final_v12_10day` (0 differences, no one-sided paths) |
| `p7-0_v12_120day` | game_v12 0..32 × 120, same flags | DETERMINISM PASS; all 53,385 equal `p6-7_final_v12_120day` |
| `p7-0_v11_10day` | game_v11 0..64 × 10, same flags | DETERMINISM PASS; all 5,846 equal `p6-7_final_v11_10day` |
| `p7-0_legacy` | legacy 0..4 × 10, `--check-determinism` | DETERMINISM PASS; all 4,496 equal `p6-7_final_legacy` |

Fingerprints of `game_v12_scene(seed)` after 1,440 ticks (seeds 0–3), pinned in `harness_v12_fingerprints_unchanged`: `519530c0aaaae844` (forage 83, wood 20), `97f3bb5d07eb719c` (68, 32), `b44a71909b8615cc` (84, 28), `c16d5b002bea6cc5` (55, 37).

## Pass 7 Part 1 — v13 skeleton, schema 7, the v12 baseline

`p7-1_v12_10day` (game_v12 0..64 × 10) and `p7-1_v12_120day` (0..32 × 120), both `--check-determinism` with the eight clocks, release probe built from the Part 1 code (frozen copy `probe_p1`). Both DETERMINISM PASS. Against Part 0: all 5,846 / 53,385 shared values identical except `schema_version` (6 → 7); the schema-7 keys are additions (250 / 3,000 summary paths: 5 keys × 5 statistics × days). Workspace debug tests 1,952 passed, 0 failed (Pass 6: 1,941).

`p7-1_v12_targets` (`evaluate_pass7.py`, `--v11-targets p6-1_v11_targets`): every Z has a v12 value. Z2 and Z5 PASS (Pass 6's Y2–Y8, Y10–Y12 unchanged). Winters (128 seed-winters): wood gathered 45 / 45 / 51 / 45 / 51; re-targets 49 / 45 / 57 / 39 / 63 a winter; **re-target share 0.514** / 0.495 / 0.531 / 0.446 / 0.565 (the brief's 0.51); first dark night on the seventh in 119 of 124, the sixth in 5; no dark night in 4 (share 0.031), two or more in 5 (0.039). Severity −1 everywhere (no weights in v12). Z3 and Z4 therefore FAIL for v12, as expected; Z1/Z6/Z7/Z8 receipts are not given for the baseline.

## Pass 7 Part 2 — winters of drawn severity, bounded claims

Code `a3669b66`; release probe `probe_p2`.

**4.5 v12 guard:** `p7-2_v12_guard_10day` / `p7-2_v12_guard_120day` (game_v12 0..64 × 10 and 0..32 × 120, Part 0 flags) — DETERMINISM PASS; all 6,096 / 56,385 shared values equal `p7-1_v12_*` (0 differences, no one-sided paths).

**Gate** (`gate_pass7.sh`, run in a separate worktree of `a3669b66`, 14:10–14:25): green — debug 1,962 passed, release green, both Clippy variants clean, forbidden math 0, bridge build, Godot import, strict check, smoke without errors, Python tests; the worldmodel step was a recorded skip (no tests yet). `codex/revitalize-2026-10` fast-forwarded to `a3669b66`.

**First look at v13 (starting values, not a recorded round):** `p7-2_v13_10day` / `p7-2_v13_120day`, evaluated into a scratch directory. Z3, Z4 and Z5 PASS: no dark night in 0.320 of the 128 seed-winters, two or more in 0.273, first dark night on nights 5 / 6 / 7 = 14 / 21 / 52 (0.598 on the seventh), severity shares mild 0.320 / normal 0.422 / harsh 0.258; re-target share median 0.056 (v12 0.514), normal-severity winter wood gathered 45 (floor 40.5). **Z2 FAILS on Y3 stagnation (max 6 > 4)**, the shape section 7.4 predicted. By severity (medians): mild 0 nights out, 38 wood gathered; normal 1 night (the seventh), 45; harsh 2 nights (first the sixth), 45; food at winter's end 244 / 241 / 229.

## Pass 7 Part 3 — the god's hand

Code `dc6760b0`; release probe `probe_p3`. **4.5 v12 guard:** `p7-3_v12_guard_10day` / `p7-3_v12_guard_120day` — DETERMINISM PASS; all 6,096 / 56,385 shared values equal `p7-1_v12_*` (0 differences). The eight tests of `harness_revitalize_pass7_interventions.rs` pass (prefix identity on seed 4 for every kind applied at the first evening; three-day null identity of all seven kinds at amount 0).

## Pass 7 Part 4 — recorded worlds

Recorder `worldsim_wm` (frozen copy `wm_p4`). A 10-day `game_v13` episode takes 0.55 s; a 30-day one 2.7 s (release, one thread). Sample: `docs/worldmodel/sample/` (seeds 0 and 1, 10 days, `--agents`), 1.8 MB. 9.4 byte identity (two runs, `--workers 1` against 4, with and without `--agents`, with a schedule directory) passes in `test_export_determinism.py`.

**Z6** (`make_dataset.py z6`, `check_z6.py` → `docs/revitalize/probe/p7-z6/`): seeds 0–7 × 7 kinds, one intervention at tick 31,920 (day 23 12:00), 30 days, factual included — **PASS**: 52 pairs equal the factual `row_digest` at every row ≤ 31,920 and differ from the next row (tick 31,980); 4 applied 0 and are listed, not failed: windfall in seeds 1, 3, 4 and 7, mild winters where all 20 deadwood spots already hold the winter cap (deadwood total 20 at that row).

## Pass 7 Part 5 — tuning game_v13

Rounds use `game_v13` 0..64 × 10 and 0..32 × 120, both `--check-determinism` with the eight Pass 6 clocks, evaluated by `evaluate_pass7.py` (`--v11-targets p6-1_v11_targets`, `--v12-targets p7-1_v12_targets`). Z1, Z6, Z7 and Z8 are receipts given only in Part 7, so they FAIL in every round. Screens are throwaway 32 × 120 runs (no twin, same clocks) from a probe built with `game_v13()` edited and restored (`build_variant.sh`), evaluated with round 1's 10-day run (days 1–10 hold no winter, so the knobs cannot change them). Knob notation: weights mild/normal/harsh in thousandths; burn per hour mild/normal/harsh; fall per day mild/normal/harsh.

| Run | Weights | Burn | Fall | Z2 (stagnation max / replacement min) | No dark night | ≥ 2 dark | First dark night 5 / 6 / 7 (7th share) | Severity shares m / n / h | Re-target share | Winter lit med / min | Verdict |
|---|---|---|---|---|---:|---:|---|---|---:|---|---|
| **round 1** (start) | 300/400/300 | 2/3/4 | 8/6/4 | 6 / −3 **FAIL** | 0.320 | 0.273 | 14 / 21 / 52 (0.598) | 0.320 / 0.422 / 0.258 | 0.056 | 0.857 / 0.679 | Z2 FAIL (Y3 stagnation, seed 22) |
| screen A | 300/400/300 | 2/3/**3** | 8/6/4 | 6 / −3 FAIL | 0.320 | **0.094** | 0 / 12 / 75 (**0.862**) | same | 0.051 | 0.893 / 0.786 | Z2, Z3 FAIL |
| screen B | 300/**500/200** | 2/3/4 | 8/6/4 | 5 / **−4** FAIL | 0.320 | 0.180 | 8 / 15 / 64 (**0.736**) | 0.320 / 0.516 / 0.164 | 0.054 | 0.857 / 0.679 | Z2, Z3 FAIL |
| screen C | 300/400/300 | 2/3/4 | 8/6/**5** | 6 / −4 FAIL | 0.320 | 0.289 | 14 / 23 / 50 (0.575) | 0.320 / 0.422 / 0.258 | 0.052 | 0.857 / 0.643 | Z2 FAIL |
| screen D | **400/400/200** | 2/3/4 | 8/6/4 | 5 / −4 FAIL | 0.422 | 0.172 | 9 / 13 / 52 (**0.703**) | 0.422 / 0.414 / 0.164 | 0.043 | 0.893 / 0.714 | Z2, Z3 FAIL |
| screen E | **550/250/200** | 2/3/4 | 8/6/4 | 4 / −3 PASS | **0.602** | 0.180 | 10 / 13 / 28 (0.549) | 0.602 / 0.234 / 0.164 | 0.051 | 0.911 / 0.714 | Z3 FAIL (no dark night 0.602 > 0.60) |
| screen F | 550/250/200 | 2/3/**5** | 8/6/**3** | 4 / −3 PASS | **0.602** | 0.180 | 4th 13, 8 / 2 / 28 (0.549) | same as E | 0.052 | 0.893 / 0.607 | Z3 FAIL |
| screen G | **600/200/200** | 2/3/4 | 8/6/4 | 4 / −3 PASS | **0.656** | 0.180 | 11 / 12 / 21 (0.477) | 0.656 / 0.180 / 0.164 | 0.052 | 0.929 / 0.714 | Z3, Z5 (Y6 trip days) FAIL |
| screen H | **520/280/200** | 2/3/4 | 8/6/4 | 4 / −3 PASS | 0.547 | 0.172 | 9 / 13 / 36 (0.621) | 0.547 / 0.289 / 0.164 | 0.050 | 0.893 / 0.714 | Z2–Z5 PASS |

Diagnosis: the Y3 failure is always seed 22 (and in B–D replacement −4 there too). In B and D all four of seed 22's winters are normal (Pass 6's winter), and it still fails: the binding cause is the claims fix of section 7.3 (7.4 foresaw "a 6-day stagnation with the claims fix"), which no Part 5 knob reaches. Softening harsh (A–D) does not help. Seed 22's severity draws are u = 752, 423, 785, 508 (years 0–3); with the mild weight above 508 two of its winters become mild and its spring births recover (E–H). The mild weight is then bounded by Z3's no-dark-night share (≤ 0.60): 520 gives 0.547. Screen H becomes round 2. This choice is fitted to the 32 probe seeds through one seed's draws; the report says so.

**Round 2** (`p7-5_round2_10day` / `p7-5_round2_120day`, code `0c46f69c`, weights 520/280/200, burns 2/3/4, falls 8/6/4): both DETERMINISM PASS; the 120-day run equals screen H on all 56,374 shared values (only `determinism_checked` differs), the 10-day run equals round 1 (days 1–10 hold no winter). `p7-5_round2_targets`: **Z2, Z3, Z4 and Z5 PASS**; Y9 FAIL (reported only: noon work 0.323, idle 0.655). Highlights (median / p10 / p90 / min / max): Y3 stagnation max 4, replacement min −3; no dark night 0.547, ≥ 2 dark nights 0.172, first dark night on nights 5 / 6 / 7 = 9 / 13 / 36 (0.621 on the seventh); severity shares 0.547 / 0.289 / 0.164 against 0.52 / 0.28 / 0.20; re-target share 0.050 / 0 / 0.103 / 0 / 0.185 (v12 0.514); normal-severity winter wood gathered 45 (floor 40.5); winter wood trips 46 / 38 / 52 / 31 / 56 (v12 96: the wasted trips are gone, the wood gathered is not); winter lit 0.893, winter cold 0.103, winter food at the end 242. By severity (medians): mild (70) 0 nights out, 38 gathered, 0 cold member-hours, food 243.5; normal (37) 1 night (the seventh), 45, 336, 246; harsh (21) 2 nights (first the sixth), 45, 840, 236. Tuning stops at round 2: every tunable target passes (two of three allowed rounds used).
- Part 5 gate (`gate_pass7.sh` on `0c46f69c`, worktree, 14:38–14:50): green — debug 1,986 / release 1,986 passed, both Clippy variants, forbidden math 0, bridge build, Godot import, strict check, smoke clean, both Python unittest discoveries. `codex/revitalize-2026-10` fast-forwarded to `0c46f69c`.

## Pass 7 Part 6 — dataset v0 and WM-Bench v0

Dataset v0 (`docs/worldmodel/DATASET_v0.md`, `dataset_v0_manifest.json`): 400 episodes, 1,368.3 MiB, recorded 14:39–14:51 from `0c46f69c` with `wm_r2` — A_obs 128 (1,078.3 MiB, 44 with agents), B_act 64 (221.1), C_cf 160 (30.6), OOD_v12 32 (19.3), OOD_long 16 (19.0). A 120-day episode takes 10–12 s with agents (one thread, four episodes in parallel), a 240-day one 21 s. Copied to the DGX and verified there (4,660 hashes, `sha256sum -c`).

WM-Bench v0, **first run (superseded by the review round 1 re-run below; T1/T2 values changed, T3/T4/sanity identical)** (`docs/worldmodel/BENCH_v0.md`, `bench_v0.json`; `bench_v0.py`, 10 s): λ = 100 for both ridges (the top of the grid). Mean nMAE over V — T1: A test persistence 0.157 / daily naive 0.235 / ridge 0.099 / action ridge 0.100; B test 0.161 / 0.241 / 0.102 / 0.102; OOD_v12 0.159 / 0.249 / 0.106 / 0.107; OOD_long 0.161 / 0.234 / 0.099 / 0.100. T2 24 h (A test): persistence 0.136, ridge 0.143; 168 h: persistence 0.544, ridge 0.554, action ridge 0.537. T3 (448 A-test winter evenings, 54 dark): rule Brier 0.033 / AUC 0.981 / accuracy 0.967; logistic 0.001 / 1.000 / 0.998. T4 (24 test seeds): mean effects gift_wood −0.75 dark nights / −223 cold member-hours / +9.9 food; rot_wood +0.79 / +277 / −12.4; rain +0.29 / +119 / −6.4; cold_snap +0.33 / +115 / −5.3; no variant changes deaths. Sanity checks: ridge beats persistence on T1 food and wood stock in all four test splits; T3 logistic AUC ≥ 0.70; every variant changes a target in ≥ 25 % of test seeds (0.625 / 0.833 / 0.958 / 0.625).

## Pass 7 Part 7 — final verification

### Reproduction of the frozen versions (probe `probe_r2`, code `0c46f69c` = the Rust of every later head)

| Label | Command | Comparison |
|---|---|---|
| `p7-7_final_legacy` | legacy 0..4 × 10, `--check-determinism` | DETERMINISM PASS; all 4,496 shared values equal `p6-7_final_legacy` except `schema_version` (6 → 7) |
| `p7-7_final_v8` | game 0..64 × 10 | DETERMINISM PASS; all 5,881 equal `p6-7_final_v8` except `schema_version` |
| `p7-7_final_v9_10day` | game_v9 0..64 × 10, clocks 02:00,12:00,19:30 | DETERMINISM PASS; all 5,846 equal `p6-7_final_v9_10day` except `schema_version` |
| `p7-7_final_v9_120day` | game_v9 0..32 × 120, same clocks | DETERMINISM PASS; all 52,968 equal `p6-7_final_v9_120day` except `schema_version` |
| `p7-7_final_v10_10day` | game_v10 0..64 × 10 (no clocks, as the reference) | DETERMINISM PASS; all 5,846 equal `p6-7_final_v10_10day` except `schema_version` |
| `p7-7_final_v10_120day` | game_v10 0..32 × 120, clocks 02:00,12:00,19:30 | DETERMINISM PASS; all 53,455 equal `p6-7_final_v10_120day` except `schema_version` |
| `p7-7_final_v11_10day` | game_v11 0..64 × 10, the eight clocks | DETERMINISM PASS; all 5,846 equal `p6-7_final_v11_10day` except `schema_version` |
| `p7-7_final_v11_120day` | game_v11 0..32 × 120, same clocks | DETERMINISM PASS; all 53,373 equal `p6-7_final_v11_120day` except `schema_version` |
| `p7-7_final_v12_10day` | game_v12 0..64 × 10, the eight clocks | DETERMINISM PASS; all 5,846 equal `p6-7_final_v12_10day` except `schema_version` |
| `p7-7_final_v12_120day` | game_v12 0..32 × 120, same clocks | DETERMINISM PASS; all 53,385 equal `p6-7_final_v12_120day` except `schema_version` |

The schema-7 keys are additions (250 / 3,000 summary paths). **Z6 again** with the final recorder (`wm_r2`, `0c46f69c`): `docs/revitalize/probe/p7-7_z6/` — PASS, 50 pairs equal up to tick 31,920 and different from tick 31,980; 6 applied 0 and are listed: windfall in seeds 0, 1, 3, 4, 5, 7 (all 20 spots at the winter cap; with weights 52/28/20 more of those winters are mild).

### Z8 — timing (back to back, recorder off)

Run 15:17:38–15:26:29 KST on the Mac with the release probe `probe_r2`, in this order: `p7-7_timing_v12_10day`, `p7-7_timing_v13_10day`, `p7-7_timing_v12_120day`, `p7-7_timing_v13_120day` (all `--check-determinism`, the eight clocks, `--workers 4`; all DETERMINISM PASS). Before starting, the art run (`new-world-art-prod`) was running a `cargo test`; the start waited until it went quiet (15:17:30). During the window a monitor sampled the CPU of art-run, Godot and rustc processes every 15 s: one art-run test binary used about one core (≈ 100 %) throughout, and from 15:19:08 to 15:19:38 it peaked at 295–580 % (during the 10-day pair). Median ms per agent-tick: v12 0.0010628 → v13 0.0010711 (10 days, **1.01×**), v12 0.0014576 → v13 0.0013519 (120 days, **0.93×**); limit 1.2×. Z8 PASS, with the art-run load recorded above.

Recorder wall time per 120-day episode (dataset v0, `timing.json`, one thread per episode, four episodes in parallel while the Part 5 gate and probes also ran): without agents median 10.7 s (A_obs, 9.4–11.8), 10.2 s (B_act), 10.3 s (OOD_v12); with agents 12.6 s (A_obs, 9.8–17.8); 240 days 20.8 s; 35 days 2.5 s. The simulation alone takes about 9.3 s for a 120-day run (0.00135 ms × ~40 agents × 172,800 ticks), so the recorder adds roughly 15–35 %.

### WM-Bench v0 re-run after review round 1 (L4 requested amounts, L5 scored-row denominator) — the published numbers

λ = 100 for both ridges. Mean nMAE over V — T1 (persistence / daily naive / ridge / action ridge): A test 0.158 / 0.237 / 0.100 / 0.101; B test 0.161 / 0.243 / 0.103 / 0.103; OOD_v12 0.159 / 0.251 / 0.106 / 0.107; OOD_long 0.161 / 0.237 / 0.100 / 0.101. T2 24 h (persistence / ridge / action ridge): A test 0.139 / 0.145 / 0.146; B test 0.137 / 0.148 / 0.149; OOD_v12 0.137 / 0.209 / 0.205; OOD_long 0.135 / 0.147 / 0.148. T2 168 h: A test 0.554 / 0.557 / 0.541; B test 0.549 / 0.575 / 0.555; OOD_v12 0.553 / 0.453 / 0.419; OOD_long 0.536 / 0.543 / 0.528. T3, T4 and the sanity checks are identical to the first run (T3 logistic Brier 0.0015, AUC 1.000).

## Pass 8 baseline

Base `ce7acb25` (code head `6df3a7e0`), release probe built from the base and frozen as a copy (`/Users/rexxa/work/p8bin/probe_p0`), `--workers 4`, run 17:57–18:03 KST. Flags exactly those of Pass 7's recorded runs: the eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30` and `--check-determinism` for v12 and v13, `--check-determinism` only for legacy.

| Label | Command | Comparison |
|---|---|---|
| `p8-0_v13_10day` | game_v13 0..64 × 10 | DETERMINISM PASS; all 6,096 shared values equal `p7-5_round2_10day` (0 differences, no one-sided paths) |
| `p8-0_v13_120day` | game_v13 0..32 × 120 | DETERMINISM PASS; all 56,374 equal `p7-5_round2_120day` |
| `p8-0_v12_10day` | game_v12 0..64 × 10 | DETERMINISM PASS; all 6,096 equal `p7-7_final_v12_10day` |
| `p8-0_v12_120day` | game_v12 0..32 × 120 | DETERMINISM PASS; all 56,385 equal `p7-7_final_v12_120day` |
| `p8-0_legacy` | legacy 0..4 × 10 | DETERMINISM PASS; all 4,746 equal `p7-7_final_legacy` |

Fingerprints of `game_v13_scene(seed)` after 1,440 ticks (seeds 0–3), pinned in `harness_v13_fingerprints_unchanged`: `519530c0aaaae844` (forage 83, wood 20), `24d191164e3c4132` (69, 32), `b44a71909b8615cc` (84, 28), `c16d5b002bea6cc5` (55, 37).

v0 sample rebuilt with the Part 0 recorder (`wm_p0`, `ce7acb25`) into `.harness/evidence/worldmodel/p8-0_sample/` (`--scene game_v13 --seeds 0..2 --days 10 --every 60 --agents`, `WORLDSIM_GIT_SHA=dc6760b0…`), `timing.json` removed: `diff -r` against `docs/worldmodel/sample/` finds no difference (byte-identical).

## Pass 8 Part 1 — v14 skeleton, schema 8, recorder schema 2, the reference runs

Release probe `probe_p1` built from `d6e34ea3` (verified byte-identical to a rebuild of that commit). Tuning runs `p8-1_v12_tuning_{10,120}day` (game_v12 0..64 × 10, 0..32 × 120) and `p8-1_v13_tuning_{10,120}day`, all `--check-determinism` with the eight clocks: DETERMINISM PASS; against Part 0 every shared value is identical (6,096 / 56,385 / 6,096 / 56,374) except `schema_version` (7 → 8); the new paths are the two schema-8 keys (5 statistics × days) and the summary `rules` block. Workspace debug tests 1,995 passed, 0 failed (Pass 7: 1,987); both Clippy variants clean; Python 55 + 21 tests OK.

`p8-1_v12_tuning_targets` and `p8-1_v13_tuning_targets` (`evaluate_pass8.py`; v13 in comparison mode against v12): v12 has no failing seed in any per-seed clause on the tuning seeds. v13: X2 PASS (no failing seed either); X3 FAIL on (a) R² **0.943**, (b) normal winters 36 of 37 with one dark night (mode share 0.973), (e) first dark night on 3 distinct nights (5 / 6 / 7 = 9 / 13 / 36, 0.621 on the seventh); (c) no dark night 0.547, (d) ≥ 2 dark 0.172 and (f) shares 0.547 / 0.289 / 0.164 pass. X4 PASS (re-target share 0.050, normal gathered 45). These are section 3.3's v13 row.

## Pass 8 Part 2 — winters that vary night by night

Code `4ab0778a`; release probe `probe_p2`. Workspace debug tests 2,005 passed, 0 failed (+ the 9 tests of `harness_revitalize_pass8_winter.rs` and the rules unit tests); both Clippy variants clean. **4.5 v13 guard:** `p8-2_v13_guard_10day` / `p8-2_v13_guard_120day` (game_v13 0..64 × 10 and 0..32 × 120, Part 0 flags) — DETERMINISM PASS; every shared value equals Part 0 (only `schema_version` and the new keys differ) and all 57,588 values equal `p8-1_v13_tuning_120day`.

## Pass 8 Part 3 — interventions and the recorder under v14

**Z6 for v14** (`make_dataset.py z6 --scene game_v14`, recorder `wm_p2` = `4ab0778a`, `check_z6.py` → `docs/revitalize/probe/p8-3_z6/`): seeds 0–7 × 7 kinds at tick 31,920 (day 23 12:00), 30 days — **PASS, 56 of 56**: every pair equals the factual `row_digest` at every row ≤ 31,920 and differs from tick 31,980; **windfall applied 10 in all 8 seeds** (Pass 7: 2 of 8), because it now lands on the snow. Recording took 41 s (64 episodes, four workers).

Recorder checks (Part 2 binaries): `test_hidden_columns.py` (game_v14 seeds 0–1 × 30 days, recorder `--every 60`, probe `--sample-every 60`): `night_burn_per_hour`, `winter_fall_today` and the summed `wood_target` equal the probe keys at every common tick; `winter_severity` differs only on 07:00 rows of a season's first day (the documented completed-tick vs system-tick booking). `test_export_determinism.py` gains a game_v14 case (30 days, with agents and with a schedule): two runs at four workers and one at one worker are byte-identical; schema 2. Part 3 changes no Rust: the Part 3 probe and recorder are the Part 2 binaries, so the 4.5 guard of Part 2 stands for Part 3.

### Part 1 held-out references (seeds 64–127 × 10, 32–95 × 120)

`p8-1_v12_heldout_{10,120}day`, `p8-1_v13_heldout_{10,120}day` (`probe_p1`, the eight clocks, `--check-determinism`): all four DETERMINISM PASS. Evaluations `p8-1_v12_heldout_targets` (absolute) and `p8-1_v13_heldout_targets` (comparison mode against v12 held-out). Section 3.2 reproduces exactly:

| held-out | v12 | v13 |
|---|---|---|
| Y3 replacement failing seeds | 1 (80: −4) | 2 (68: −4, 80: −4) |
| Y10 evening near hearth failing seeds | 2 (55: 0.267, 75: 0.344) | 2 (55: 0.281, 76: 0.344) |
| Y3 longest stagnation (max) | 4 | 2 |
| no-dark-night share | 0.023 | 0.594 |
| severity-only R² | — (no severity) | 0.943 (tuning 0.943) |
| normal winters 0 / 1 / 2 / 3 dark | — | 0 / 62 / 3 / 0 |
| first dark night 5 / 6 / 7 | 0 / 8 / 242 | 20 / 22 / 62 |

Every other per-seed clause has no failing seed for either ruleset on the held-out seeds. v13 in comparison mode passes X2 (2 ≤ 1 + 1 and 2 ≤ 2 + 1).

## Pass 8 Part 4 — tuning game_v14 on the tuning seeds

Round 1 = section 7.1's start values (variant C of 3.3: severity weights 350/400/250, burns 2/3/4, falls 8/6/4; night weights 250/500/250, step 2; fall spread 2; windfall over snow). Runs `p8-4_round1_10day` (game_v14 0..64 × 10) and `p8-4_round1_120day` (0..32 × 120), probe `probe_p2` (`4ab0778a`), both `--check-determinism` with the eight clocks: DETERMINISM PASS. Evaluation `p8-4_round1_targets` (`evaluate_pass8.py --compare-to p8-1_v12_tuning_targets`). No screens were run.

| Round | Weights | Night (−/0/+) × step | Fall spread | Burns / falls | X2 | X3 | X4 | X5 | R² | No dark | ≥ 2 dark | First dark 4/5/6/7 (7th share) | Shares m/n/h |
|---|---|---|---|---|---|---|---|---|---:|---:|---:|---|---|
| **round 1** | 350/400/250 | 250/500/250 × 2 | 2 | 2/3/4, 8/6/4 | PASS | PASS | PASS | PASS | **0.530** | 0.484 | 0.281 | 5 / 18 / 15 / 28 (0.424) | 0.406 / 0.391 / 0.203 |

Round 1 per target (median / min / max where useful):
- **X2** (comparison mode against v12's tuning evaluation, which has no failing seed): no failing seed in any per-seed clause — Y3 stagnation max 3, replacement min −3 (seed 22 now −3, passing), alive band, birth by day 4 (7 in every seed), alive at day 10 (42), old age by day 30 (min 5); Y7 winter lit every seed ≥ 0.607; Y8 winter cold every seed ≤ 0.383; Y10 evening near hearth every seed ≥ 0.379; Y11 home distance ≤ 11.3, social > 0; Y12 0. Y2: zero starvation and dehydration deaths in every seed (10 and 120 days). Child share median 0.107, elder share 0.262.
- **X3**: (a) R² 0.530; (b) normal winters 0/1/2/3/4 dark = 17/20/9/3/1 (mode share 0.40), harsh 1/4/4/13/4 (0.50); (c) 0.484; (d) 0.281; (e) first dark night on 4 distinct nights, 0.424 on the seventh; (f) shares within 0.06 of the weights.
- **X4**: re-target share median 0.055; normal-severity winter wood gathered 48 (floor 40.5).
- **X5**: Y4 store share 0.99 (every seed ≥ 0.924, winter 1.0); Y5 autumn wood 50/day vs summer 0, autumn noon work 0.614; Y6 trip days 5, trips 48; Y7 winter lit 0.857, other lit 1.0; Y8 winter cold 0.139, winter food 237.5; Y10 night in shelter 1.0, evening near hearth 0.648; Y11 home distance 7.62, social 0.63 / 0.82, t6a 0.001 / 0.009.
- Y9 (reported only): noon work 0.308, noon idle 0.667, 15:00 work 0.416.
- By severity: mild (52) dark 0/1/2 = 44/6/2, night burn 2.29 (the floor of 1 lifts mild's −2 nights), fall 8.14, food at winter's end 246 (min 184); normal (50) 17/20/9/3/1, food 232.5 (min 41); harsh (26) 1/4/4/13/4, food 228.5 (min 48).

Tuning stops at round 1 (1 of 3 rounds): every tunable target passes; `game_v14()` keeps the start values.

## Pass 8 Part 5 — the held-out look (once)

`p8-5_heldout_v14_10day` (game_v14 64..128 × 10) and `p8-5_heldout_v14_120day` (32..96 × 120), `probe_p2` (`4ab0778a`, `game_v14()` = round 1), `--check-determinism`, the eight clocks, run 18:44–18:52 KST: both DETERMINISM PASS. Evaluation `p8-5_heldout_v14_targets` in comparison mode against `p8-1_v12_heldout_targets`. **X2, X3, X4 and X5 pass on the held-out seeds**; no corrective round (10.3) is needed, so the fresh seeds are not used. X1's determinism clauses pass (64/64 and 64/64; see DECISIONS for the evaluator fix of the 120-day seed count); its repro receipts, X6, X7 and X8 are Part 7's.

### X targets, tuning and held-out, v12 / v13 / v14

v12 is evaluated in absolute mode (it is the reference; its held-out X2 'FAIL' is its own failing seeds), v13 and v14 in comparison mode against v12 on the same seeds. v12 has no severity, so its X3 values describe the Pass 6 calendar.

| Measure | tuning v12 | tuning v13 | tuning v14 | held-out v12 | held-out v13 | held-out v14 |
|---|---|---|---|---|---|---|
| X2 verdict | PASS | PASS | PASS | FAIL | PASS | PASS |
| X3 verdict | FAIL | FAIL | PASS | FAIL | FAIL | PASS |
| X3a severity-only R² | — (no severity) | 0.943 | 0.530 | — (no severity) | 0.943 | 0.602 |
| X3b normal mode share | — | 0.973 | 0.400 | — | 0.954 | 0.444 |
| X3b harsh mode share | — | 0.571 | 0.500 | — | 0.513 | 0.543 |
| X3c no-dark-night share | 0.031 | 0.547 | 0.484 | 0.023 | 0.594 | 0.574 |
| X3d ≥ 2 dark share | 0.039 | 0.172 | 0.281 | 0.031 | 0.164 | 0.281 |
| X3e first dark night distinct / 7th share | 2 / 0.960 | 3 / 0.621 | 4 / 0.424 | 2 / 0.968 | 3 / 0.596 | 4 / 0.339 |
| X3f shares m/n/h | 0.000/0.000/0.000 | 0.547/0.289/0.164 | 0.406/0.391/0.203 | 0.000/0.000/0.000 | 0.594/0.254/0.152 | 0.398/0.422/0.180 |
| X4 verdict | FAIL | PASS | PASS | FAIL | PASS | PASS |
| X4 re-target share median | 0.514 | 0.050 | 0.055 | 0.516 | 0.051 | 0.051 |
| X4 normal gathered median | 0.0 | 45.0 | 48.0 | 0.0 | 45.0 | 48.0 |
| X5 verdict | PASS | PASS | PASS | PASS | PASS | PASS |
| Y7 winter lit median | 0.857 | 0.893 | 0.857 | 0.857 | 0.929 | 0.893 |
| Y8 winter cold median | 0.152 | 0.103 | 0.139 | 0.143 | 0.089 | 0.125 |
| Y8 winter food median | 246.5 | 242.5 | 237.5 | 242.5 | 238.0 | 234.5 |
| Y6 trips median | 96 | 46 | 48 | 96 | 46 | 48 |
| Y2 starvation+dehydration (120 d max) | 0 | 0 | 0 | 0 | 0 | 0 |

Per-seed clauses — failing seeds (value); every clause not listed has no failing seed in any column:

| Clause | tuning v12 | tuning v13 | tuning v14 | held-out v12 | held-out v13 | held-out v14 |
|---|---|---|---|---|---|---|
| Y10_evening_near_hearth_every_seed | none | none | none | 55 (0.267), 75 (0.344) | 55 (0.281), 76 (0.344) | 55 (0.281), 79 (0.344) |
| Y3_replacement | none | none | none | 80 (-4.0) | 68 (-4.0), 80 (-4.0) | 80 (-4.0) |

Distributions (median / p10 / p90 / min / max) of the per-seed values:

| Clause | tuning v12 | tuning v13 | tuning v14 | held-out v12 | held-out v13 | held-out v14 |
|---|---|---|---|---|---|---|
| Y3_stagnation | 0 / 0 / 0 / 0 / 4 | 0 / 0 / 0 / 0 / 4 | 0 / 0 / 0 / 0 / 3 | 0 / 0 / 0 / 0 / 4 | 0 / 0 / 0 / 0 / 2 | 0 / 0 / 0 / 0 / 4 |
| Y3_replacement | 0 / -1 / 0 / -3 / 1 | 0 / -1 / 0 / -3 / 1 | 0 / -1 / 0 / -3 / 1 | 0 / -1 / 0 / -4 / 1 | 0 / -1 / 0 / -4 / 1 | 0 / -1 / 0 / -4 / 1 |
| Y7_winter_lit_every_seed | 0.857 / 0.821 / 0.889 / 0.821 / 0.893 | 0.893 / 0.825 / 0.964 / 0.714 / 1 | 0.857 / 0.786 / 0.929 / 0.607 / 0.964 | 0.857 / 0.821 / 0.857 / 0.821 / 0.929 | 0.929 / 0.821 / 0.964 / 0.75 / 1 | 0.893 / 0.75 / 0.964 / 0.571 / 1 |
| Y8_winter_cold_every_seed | 0.152 / 0.116 / 0.178 / 0.0982 / 0.196 | 0.103 / 0.0358 / 0.187 / 0 / 0.301 | 0.139 / 0.0813 / 0.221 / 0.0268 / 0.383 | 0.143 / 0.125 / 0.185 / 0.0894 / 0.188 | 0.0891 / 0.0268 / 0.197 / 0 / 0.241 | 0.125 / 0.0268 / 0.263 / 0 / 0.438 |
| Y10_evening_near_hearth_every_seed | 0.64 / 0.433 / 0.807 / 0.368 / 0.864 | 0.643 / 0.416 / 0.791 / 0.353 / 0.852 | 0.648 / 0.421 / 0.793 / 0.379 / 0.885 | 0.655 / 0.387 / 0.825 / 0.267 / 0.926 | 0.662 / 0.384 / 0.826 / 0.281 / 0.944 | 0.658 / 0.38 / 0.827 / 0.281 / 0.931 |
| Y11_home_distance_every_seed | 7.68 / 5.91 / 8.8 / 4.79 / 11.3 | 7.62 / 5.82 / 9.07 / 4.79 / 11.3 | 7.62 / 5.82 / 9.07 / 4.79 / 11.3 | 6.85 / 5.45 / 8.5 / 3.93 / 10.1 | 6.92 / 5.41 / 8.53 / 4.05 / 10.1 | 6.92 / 5.41 / 8.53 / 4.05 / 10.1 |

### Dark nights by severity (seed-winters: tuning 128, held-out 256)

| Set | Ruleset | mild: dark 0/1/2/3/4 | normal | harsh | first dark night 4/5/6/7 | night burn by severity (median) | fall by severity (median) |
|---|---|---|---|---|---|---|---|
| tuning | v13 | 70/0/0/0/0 | 0/36/1/0/0 | 0/0/12/9/0 | 0/9/13/36 | 2.00/3.00/4.00 | 8.00/6.00/4.00 |
| tuning | v14 | 44/6/2/0/0 | 17/20/9/3/1 | 1/4/4/13/4 | 5/18/15/28 | 2.29/3.00/4.29 | 8.14/6.00/4.21 |
| held-out | v13 | 152/0/0/0/0 | 0/62/3/0/0 | 0/0/19/20/0 | 0/20/22/62 | 2.00/3.00/4.00 | 8.00/6.00/4.00 |
| held-out | v14 | 98/3/1/0/0 | 48/31/19/10/0 | 1/3/11/25/6 | 6/36/30/37 | 2.14/3.00/4.00 | 8.14/6.00/4.00 |

Held-out v14 against tuning v14: R² 0.602 (tuning 0.530), no dark night 0.574 (0.484), ≥ 2 dark 0.281 (0.281), first dark night on 4 nights with 0.339 on the seventh (0.424); severity shares 0.398 / 0.422 / 0.180 against weights 0.35 / 0.40 / 0.25 (within 0.07). Per-seed clauses: v14 fails Y3 replacement in seed 80 (−4; v12 the same seed, v13 seeds 68 and 80) and Y10 evening near hearth in seeds 55 (0.281) and 79 (0.344) (v12: 55 and 75; v13: 55 and 76) — counts 1 ≤ 1 + 1 and 2 ≤ 2 + 1, so X2 passes in comparison mode. No starvation or dehydration death in any seed. Seed 80's replacement margin is −4 under all three rulesets: a weak village independent of the winter rule, as the planner found.

## Pass 8 Part 6 — dataset v1 and WM-Bench v1

Dataset v1 (`docs/worldmodel/DATASET_v1.md`, `docs/worldmodel/dataset_v1_manifest.json`): 432 episodes, 1,383.8 MiB, recorded 18:55–19:08 KST from `4ab0778a` with `wm_p2` — A_obs 128 (44 with agents), B_act 64, C_cf 192 (32 seeds × factual + 5 variants), OOD_v13 32, OOD_long 16; recorder schema 2 throughout. B_act: 512 interventions, 35 windfalls applied 0 (outside winter, spots full); C_cf windfall applied 20 in 32 of 32 seeds. Copied to the DGX and all 5,012 hashes verified there (`sha256sum -c`), 19:10.

WM-Bench v1 (`docs/worldmodel/BENCH_v1.md`, `bench_v1.json`; `bench_v1.py`, 7 s): λ — ridge 100, action ridge 100, severity-oracle ridge 100 (see the json). Mean nMAE over V, best non-oracle baseline:

| Task | A test | B test | OOD_v13 | OOD_long |
|---|---:|---:|---:|---:|
| T1 next hour (ridge / action ridge) | 0.101 | 0.107 | 0.102 | 0.101 |
| T1 persistence | 0.157 | 0.161 | 0.157 | 0.161 |
| T2 24 h, every-hour starts | 0.188 | 0.196 | 0.196 | 0.191 |
| T2 24 h, 06:00 starts (v0 protocol) | 0.134 | 0.167 | 0.136 | 0.133 |
| T2 168 h, every-hour starts | 0.267 | 0.265 | 0.264 | 0.271 |
| T2 168 h, 06:00 starts | 0.278 | 0.297 | 0.264 | 0.277 |

T3 (448 A-test winter evenings, 64 dark): observable logistic Brier 0.020 / AUC 0.993 / accuracy 0.975; references: severity rule 0.036 / 0.979 / 0.964, severity logistic 0.021 / 0.995 / 0.973. **T3b** (64 A-test winters, 384 train): train mean MAE 0.995 / exact 0.203; ridge **0.600 / 0.516**; severity oracle (reference) 0.609 / 0.547. T4 (24 C_cf test seeds, ranked targets, best baseline MAE): gift_wood dark 0.521, cold 167.2, food 11.5; rot_wood 0.531, 147.8, 10.3; rain 0.708, 214.1, 9.75; cold_snap 0.292, 116.8, 5.48; windfall 0.375, 69.3, 5.60 (nMAE in BENCH_v1.md). T5 (8 test agent episodes, 940,878 agent-hours): persistence state tag 0.744, target kind 0.718, Chebyshev 2.47, hunger/thirst/fatigue nMAE 0.445 / 0.595 / 0.255; 24 hours earlier 0.757, 0.738, 4.75, 1.219 / 0.688 / 0.172. Sanity checks all hold: ridge beats persistence on T1 food and wood in all four test splits; the T3b ridge beats the train mean; every C_cf variant changes a ranked target in ≥ 25 % of test seeds; the severity oracle's T3b MAE is 0.609 (not 0).

### WM-Bench v1 re-run after review round 1 (M1, L1) — the published numbers

`bench_v1.py` re-run after the fixes (T1, T3, T3b, T4 and the sanity checks unchanged): the T2 severity oracle now reads each rollout row's true severity — 168 h A test 0.275 (first run, start row's severity held: 0.691, worse than persistence 0.639); ridge 0.271 and action ridge 0.267 unchanged. T5 scores both baselines on one pair set (alive at t+1, t and t−23; 924,554 agent-hours): persistence state tag 0.741, target kind 0.714, Chebyshev 2.49, hunger/thirst/fatigue nMAE 0.450 / 0.607 / 0.256; 24 hours earlier 0.757, 0.738, 4.75, 1.223 / 0.696 / 0.172. The first-run T5 line above is superseded.

## Pass 8 Part 7 — final verification

### Reproduction of the frozen versions (probe `probe_p2`, code `4ab0778a` = the Rust of every later head)

Run 19:11–19:31 KST with Pass 7's exact flags (`final_probes.sh`). All twelve DETERMINISM PASS; every shared value equals Pass 7's final summary except `schema_version` (7 → 8); the one-sided paths are the schema-8 keys and the `rules` block.

| Label | Command | Comparison |
|---|---|---|
| `p8-7_final_legacy` | legacy 0..4 × 10 | all 4,746 shared values equal `p7-7_final_legacy` except `schema_version` |
| `p8-7_final_v8` | game 0..64 × 10 | all 6,131 equal `p7-7_final_v8` except `schema_version` |
| `p8-7_final_v9_10day` | game_v9 0..64 × 10, clocks 02:00,12:00,19:30 | all 6,096 equal `p7-7_final_v9_10day` except `schema_version` |
| `p8-7_final_v9_120day` | game_v9 0..32 × 120, same clocks | all 55,968 equal except `schema_version` |
| `p8-7_final_v10_10day` | game_v10 0..64 × 10 | all 6,096 equal except `schema_version` |
| `p8-7_final_v10_120day` | game_v10 0..32 × 120, clocks 02:00,12:00,19:30 | all 56,455 equal except `schema_version` |
| `p8-7_final_v11_10day` | game_v11 0..64 × 10, the eight clocks | all 6,096 equal except `schema_version` |
| `p8-7_final_v11_120day` | game_v11 0..32 × 120, same clocks | all 56,373 equal except `schema_version` |
| `p8-7_final_v12_10day` | game_v12 0..64 × 10, the eight clocks | all 6,096 equal except `schema_version` |
| `p8-7_final_v12_120day` | game_v12 0..32 × 120, same clocks | all 56,385 equal except `schema_version` |
| `p8-7_final_v13_10day` | game_v13 0..64 × 10, the eight clocks | all 6,096 equal `p7-5_round2_10day` except `schema_version` |
| `p8-7_final_v13_120day` | game_v13 0..32 × 120, same clocks | all 56,374 equal `p7-5_round2_120day` except `schema_version` |

The final `game_v14` runs are Part 4's tuning runs (`p8-4_round1_*`) and Part 5's held-out runs (`p8-5_heldout_v14_*`), both from `probe_p2` with the eight clocks and the death census, `--check-determinism`: no Rust changed after them, so they are the final v14 probes. **X6 again** (`make_dataset.py z6 --scene game_v14` into a scratch root, `check_z6.py` → `docs/revitalize/probe/p8-7_z6/`): PASS 56 of 56, windfall applied in 8 of 8, every `.npy` byte-identical to Part 3's Z6 episodes.

### X8 — timing (back to back, recorder off)

Run 19:33:10–19:41:47 KST with `probe_p2`, in this order: `p8-7_timing_v13_10day`, `p8-7_timing_v14_10day`, `p8-7_timing_v13_120day`, `p8-7_timing_v14_120day` (all `--check-determinism`, the eight clocks, `--workers 4`; all DETERMINISM PASS). No Godot, cargo, probe, recorder or Python job of this or another WorldSim run was active (`pgrep`); a 15-second CPU monitor saw other users' processes: a `clawdbot` process at 83–110 % (one core) in 4 samples, a "Sow and Siege" app, `node`, `osascript` and Chrome at a median of 29 % (max 227 %) in 63 samples. Median ms per agent-tick: v13 0.0010639 → v14 0.0010696 (10 days, **1.005×**), v13 0.0013615 → v14 0.0013542 (120 days, **0.995×**); limit 1.2×. **X8 PASS.**

## Pass 9 — keyed randomness, lineage keys, noise forks (`game_v15`)

All probe runs use Pass 8's flags (the eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, `--sample-every` 120, `--workers 4`, `--check-determinism`; legacy and v8–v10 with their own Pass 8 flags). Binaries frozen in `/Users/rexxa/work/p9bin/`: `probe_p0` (`4c35e295`, byte-identical to Pass 8's `probe_p2`), `probe_p2` / `wm_p2` (`874dbf54`, the Part 2 Rust; every later Rust change is a test or the recorder's fork-tick validation), `wm_p6` (`ba278357`).

### Part 0 — baseline reproduction (`probe_p0`)

| run | reference | shared values | differences |
|---|---|---:|---:|
| `p9-0_v14_10day` | `p8-4_round1_10day` | 6,212 | 0 |
| `p9-0_v14_120day` | `p8-4_round1_120day` | 57,589 | 0 |
| `p9-0_v13_10day` | `p8-7_final_v13_10day` | 6,210 | 0 |
| `p9-0_v13_120day` | `p8-7_final_v13_120day` | 57,588 | 0 |
| `p9-0_legacy` | `p8-7_final_legacy` | 4,847 | 0 |

### Part 2 — v14 guard (4.5, `probe_p2`)

`p9-2_v14_guard_10day` / `_120day` vs Part 0: 6,212 / 57,589 shared values, the only difference `schema_version` 8 → 9 (plus the two new rules keys).

### Part 3 — fork experiment (`wm_p2`; re-recorded with `wm_p6` in Part 6, see below)

`game_v15`, seeds 12000–12031, 35 days, `--every 60`, `--agents`, R = 8 replicates forked at tick 28,800; 2,520 recorded episodes (1,792 family members, 448 placebo members, 280 unforked runs), 31 min, 8.8 GiB. Full tables: `docs/revitalize/probe/p9-3_forks/fork_metrics.md`.

G0a: placebos 448 / 448 identical, replicate 0 = unforked run 224 / 224. **PASS**.

G0b (VRF = V_unpaired / V_paired, 95 % cluster bootstrap over seeds):

| variant | dark nights VRF [95 %] | cold member-hours VRF [95 %] | food store VRF (reported) | mean effect: dark nights / cold hours / food |
|---|---|---|---|---|
| gift_wood 60 | 1.00 [1.00, 1.00] | 1.54 [0.94, 3.07] | 5.89 | −0.82 / −282 / +13.2 |
| rot_wood 60 | 1.19 [1.00, 1.70] | 1.81 [1.16, 2.38] | 3.55 | +1.35 / +434 / −19.3 |
| rain 13 h | 1.00 [1.00, 1.00] | 1.69 [1.09, 3.51] | 16.5 | +0.23 / +65 / −2.0 |
| cold_snap 2 | 1.00 [1.00, 1.00] | 1.63 [1.14, 2.36] | 24.9 | +0.48 / +155 / −6.9 |
| windfall 20 | 1.00 [1.00, 1.00] | 2.17 [1.40, 3.36] | 3.86 | −0.31 / −125 / +6.9 |
| gift_food 1 | ∞ | 23.3 [11.6, ∞] | 5.65 | 0 / +0.33 / +0.34 |

Seeds whose 8 factual replicates are not all equal: dark nights 3 / 32, cold member-hours 12 / 32, food store 32 / 32; deaths and births at day 29 06:00 never differ (0 / 32). G0b: **FAIL** (dark nights of every variant; cold hours of gift_wood).

Divergence (replicate 0, medians over seeds): first differing agent position 60 (windfall) to 2,550 (cold_snap) ticks after the intervention, 4,440 for the one-unit gift; rain changes the fire after 60 ticks; deaths never differ within 35 days. Newborn key matches across arms (Y6): 98.4–100 %.

v14 single forks (8.3): mean over seeds |Δ_v14 − mean_k Δ_v15| 0.25–0.43 dark nights, 58–100 cold member-hours, 3.8–7.2 food, against v15's median within-seed spread 0 / 0–15 / 0.7–3.3.

### Part 4 — v15 health and shape (`probe_p2`)

| set | X2 | X3 | X4 | X5 | R² (v14 → v15) | v14→v15 intervals excluding 0 |
|---|---|---|---|---|---|---|
| tuning 0–63 × 10, 0–31 × 120 | PASS | PASS | PASS | PASS | 0.530 → 0.594 | 0 of 29 |
| held-out 64–127 × 10, 32–95 × 120 | PASS | PASS | PASS | PASS | 0.602 → 0.564 | 0 of 29 |

(29 = 10 X3 statistics + 19 median clauses; two of the median clauses read the pooled summary and have no per-seed interval.) Comparison mode against `p8-1_v12_tuning_targets` / `p8-1_v12_heldout_targets`. Determinism twins pass in all four runs.

Iteration-order ablations (tuning, day-10 / day-120 median forage trips): v14 905 / 4,773; v14 + AgentId order 878.5 / 4,685; v15 keyed in hecs order 910.5 / 4,744.5; v15 888 / 4,556.5. Their only interval excluding 0 is R² (+0.073, +0.065).

Y8 timing (back to back, 12:11–12:21 KST): ms per agent-tick v14 0.0011744 / v15 0.0011563 (10 days, 0.985×), v14 0.0014555 / v15 0.0015884 (120 days, 1.091×).

### Part 6 — final reproduction (`probe_p2`)

Legacy, v8, v9–v13 (10 and 120 days) and v14 (10 and 120 days) reproduce Pass 8's final summaries (`p8-7_final_*`, `p8-4_round1_*`) on every shared value; the only differences are `schema_version` 8 → 9 and the two new rules keys. All 14 determinism twins pass.

Fork experiment re-recorded with the final recorder (`wm_p6`, `ba278357`, 12:58–13:27): 42,840 files byte-identical to the Part 3 recording (`meta.json` up to the `git_sha` label); `fork_metrics` unchanged apart from the new "(N quiet)" annotations.

### Part 6 — final gate

`gate_pass9.sh` on `8d3ad643` (13:27–13:46 KST): every step exit 0; cargo test debug 2,029 / release 2,029 passed, 0 failed (Pass 8: 2,005 each); clippy both; forbidden math 0; strict check; bridge build; smoke without errors; Python 77 (`tools/revitalize`) + 37 (`tools/worldmodel`).

## Pass 10 — exposure mortality, hidden world variety, measured seed diversity (`game_v16_neutral`, `game_v16`)

All probe runs use Pass 9's flags (the eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, `--sample-every` 120, `--workers 4`, `--check-determinism`; legacy with its own flags). Binaries frozen in `/Users/rexxa/work/p10bin/`.

### Part 0 — baseline reproduction (`probe_p0`, = Pass 9's `probe_p6`)

| run | reference | shared values | differences |
|---|---|---:|---:|
| `p10-0_v15_10day` | `p9-2_v15_10day` | 6,214 | 0 |
| `p10-0_v15_120day` | `p9-2_v15_120day` | 57,597 | 0 |
| `p10-0_v15_heldout_10day` | `p9-4_v15_heldout_10day` | 6,214 | 0 |
| `p10-0_v15_heldout_120day` | `p9-4_v15_heldout_120day` | 58,731 | 0 |
| `p10-0_v14_10day` | `p9-6_final_v14_10day` | 6,214 | 0 |
| `p10-0_v14_120day` | `p9-6_final_v14_120day` | 57,591 | 0 |
| `p10-0_v13_10day` | `p9-6_final_v13_10day` | 6,212 | 0 |
| `p10-0_v13_120day` | `p9-6_final_v13_120day` | 57,590 | 0 |
| `p10-0_legacy` | `p9-6_final_legacy` | 4,849 | 0 |

### Part 1 and 2 — 4.6 guards and first looks

`p10-1_v15_guard_*` (`probe_p1`) and `p10-2_v15_guard_*` (`probe_p2`) equal `p10-0_v15_*` on every shared value (6,214 / 57,597); only `schema_version` 9 → 10 differs. First looks at the start knobs (exposure [9, 6, 9] / 2 h; 7.1's ranges): `game_v16_neutral` (`p10-1_v16n_*`) passes H2, H4, H5 and fails H3's maximum (seed-winter deaths bimodal: 0 or 7–21, max share 0.51); `game_v16` (`p10-2_v16_*`) fails H2 in seed 24 and H3's maximum (0.69), passes H4, H5, H6.

### Part 3 — tuning (tuning seeds 0–63 × 10, 0–31 × 120; v15 reference = the guard run of the same binary)

Exploration (not a judged round): 13 scratch configurations on seeds 5000–5031 × 120 days (seeds of no judged or pilot set), run by a probe built from a copy of the tree outside the clone that reads the knobs from environment variables (never committed). 11 ran before round 1 and 2 before round 2; the judged rounds below use the committed code on the tuning seeds only.

| Run | Time | Scene | Exposure (child, adult, elder / block h) | Utilization | Supply weighting | Seed-winters with a death | Max share of members | Starvation + dehydration (max per seed, seeds) |
|---|---|---|---|---|---|---:|---:|---|
| A | 16:35 | v16 | 5,4,6 / 2 | – | – | 0.008 | 0.350 | 11 (1 seeds) |
| B | 16:36 | v16 | 6,4,7 / 2 | – | – | 0.039 | 0.525 | 7 (1 seeds) |
| C | 16:39 | v16 | 5,3,5 / 1 | – | – | 0.008 | 0.125 | 9 (1 seeds) |
| D | 16:40 | v16 | 7,5,8 / 2 | – | – | 0.156 | 0.550 | 9 (1 seeds) |
| E | 16:42 | v16 | 10,4,5 / 2 | – | – | 0.258 | 0.200 | 8 (1 seeds) |
| F | 16:42 | v16 | 12,4,5 / 2 | – | – | 0.305 | 0.200 | 8 (1 seeds) |
| G | 16:42 | v16 | 9,4,6 / 1 | – | – | 0.250 | 0.350 | 7 (1 seeds) |
| H | 16:42 | v16 | 10,4,5 / 2 | 0.78 | – | 0.266 | 0.154 | 0 (0 seeds) |
| I | 16:45 | v16 | 10,4,5 / 2 | – | min(cap,4)/4 | 0.266 | 0.150 | 0 (0 seeds) |
| J | 16:45 | v16 | 10,4,5 / 2 | – | cap/4 | 0.273 | 0.381 | 42 (6 seeds) |
| K | 16:45 | v16_neutral | 10,4,5 / 2 | – | – | 0.273 | 0.143 | 0 (0 seeds) |
| L | 16:57 | v16 | 10,4,5 / 2 | 0.78 | min(cap,4)/4 | 0.273 | 0.154 | 0 (0 seeds) |
| M | 16:57 | v16_neutral | 10,4,5 / 2 | 0.78 | min(cap,4)/4 (no effect) | 0.250 | 0.150 | 0 (0 seeds) |

| Round | Knobs | Scene | H1 | H2 | H3 | H4 | H5 | H6 |
|---|---|---|---|---|---|---|---|---|
| start | [9, 6, 9] / 2 h, ranges of 7.1 | v16_neutral | – | PASS | FAIL (max) | PASS | PASS | – |
| start | same | v16 | – | FAIL (seed 24) | FAIL (max) | PASS | PASS | PASS |
| 1 | [10, 4, 5] / 2 h; v16: capacity-weighted supply | v16_neutral | PASS | PASS | PASS | PASS | PASS | – |
| 1 | same | v16 | PASS | PASS | FAIL: max_share_of_members | PASS | PASS | PASS |

Round 1 H3 detail — v16_neutral: share 0.195, median 0, max share 0.119, means mild/normal/harsh 0.06/0.28/1.62, starvation+dehydration day-120 median/max 0/0; v16: share 0.250, median 0, max share 0.371, means mild/normal/harsh 0.06/0.38/2.44, starvation+dehydration day-120 median/max 0/4. The one failing seed-winter is seed 24's last winter (9 bushes, 28 founders, 35 residents, 4 dark nights of a harsh winter): 13 exposure deaths (4 children, 9 non-elder adults weakened by hunger; 4 starvation deaths in the same seed) = 0.371 of the members. Every other seed-winter loses at most 5 (0.11), mostly children, spread over hours and nights. Evaluations: `docs/revitalize/probe/p10-3_round1_*_targets/`.

| Round | Knobs | Scene | H1 | H2 | H3 | H4 | H5 | H6 |
|---|---|---|---|---|---|---|---|---|
| 2 | round 1 + `SustainRule::utilization` 0.81 → 0.78 in both v16 rulesets | v16_neutral | PASS | PASS | PASS | PASS | PASS | – |
| 2 | same | v16 | PASS | PASS | PASS | PASS | PASS | PASS |

Round 2 H3 detail — v16_neutral: share 0.203, median 0, max share 0.100, means mild/normal/harsh 0.06/0.30/1.54, starvation+dehydration day-120 median/max 0/0; v16: share 0.242, median 0, max share 0.182, means mild/normal/harsh 0.06/0.32/2.24, starvation+dehydration day-120 median/max 0/0. v15 guard of the round-2 binary (`p10-3_round2_v15_guard_*`) equals Part 0 on every shared value (only `schema_version`). H1's receipts: v13–v15 and legacy were reproduced in Part 0 and by the guards, v8–v12 were pinned by tests; the probe reproduction of legacy and v8–v14 against Pass 9's finals is Part 7's (`p10-7_final_*`, every run reproduces). All `h_targets.json` were regenerated in Part 7 after review round 1 with these receipts and the stricter H5 rule (the verdicts did not change).

**Frozen knobs (round 2):** exposure `damage_per_cold_hour` [child 10, adult 4, elder 5], `heal_block_hours` 2; `SustainRule::utilization` 0.78 in `game_v16_neutral` and `game_v16`; `SustainRule::capacity_weighted` in `game_v16`; variety ranges and `CLIMATE_PERMILLE` as in 7.1 (unchanged). Strikes used: 2 of 3 judged rounds, after 13 disclosed scratch configurations on seeds 5000–5031 (table above). The capacity weighting feeds the supply estimate behind both the birth gate and the summer/autumn winter-prep store target, so in capacity-3 worlds it also raises that target (review round 1, M1).

**Frozen commit:** `58119582` (`581195820e8e6e79924f0583339da077bfc8d9f7`, Part 3, 17:06 KST). Parts 4–6 run `probe_frozen` / `wm_frozen` built from it (`probe_frozen` is byte-identical to the round-2 probe `probe_p3r2`); recordings carry `WORLDSIM_GIT_SHA` = that commit.

### Part 4 — held-out health (frozen commit; seeds 64–127 × 10, 32–95 × 120)

`p10-4_heldout_v15_*` (H5's reference, `probe_frozen`) equals Part 0's v15 held-out runs on every shared value (6,214 / 58,731; only `schema_version`). Determinism twins pass in all six runs.

| Scene | H1 | H2 | H3 | H4 | H5 | H6 (tuning + held-out, 128 seeds) |
|---|---|---|---|---|---|---|
| v16_neutral | PASS | PASS | PASS | PASS | PASS¹ | – |
| v16 | PASS | PASS | PASS | PASS | FAIL: Y9_work_1500 (reported: H5 is judged on v16_neutral) | PASS |

¹ Population clauses that pass by the "no new failure" rule (both scenes fail the bound, v16 no worse than v15 on the bounded statistic; from Part 7 the statistic and direction come from an explicit per-clause table, review rounds 1 L1 and 2 M1). Every escape in every run is a Pass 6 Y9 median: round 1 v16_neutral `idle_1200` 0.619 vs v15 0.634 and `work_1200` 0.3077 vs 0.3077 (an exact tie); round 1 v16 `idle_1200` 0.631 vs 0.634 and `work_1200` 0.333 vs 0.308; round 2 v16_neutral `idle_1200` 0.620 vs 0.634; round 2 v16 `idle_1200` 0.621 vs 0.634; held-out v16_neutral `work_1200` 0.344 vs 0.308; held-out v16 `idle_1200` 0.617 vs 0.619; the timing runs as round 2.

H3 detail — v16_neutral: share 0.207, median 0, max share 0.154, means mild/normal/harsh 0.00/0.32/2.09 (n 102/108/46), starvation+dehydration day-120 median/max 0/0; v16: share 0.266, median 0, max share 0.176, means mild/normal/harsh 0.00/0.37/2.07 (n 89/107/60), starvation+dehydration day-120 median/max 0/0.

H5 for v16 (reported only; section 10 judges v16_neutral): every clause passes, or escapes as no worse than v15 (Y9 `idle_1200`, footnote ¹), except Pass 6 Y9 `work_1500` (adult work share at 15:00, median 0.241 vs ≥ 0.25; v15 on the same seeds 0.417; tuning-seed round 2 gave 0.254). v16_neutral's is 0.388. The drop comes with the world variety, not with exposure.

H6 (128 seeds of game_v16): parameter means berry_bushes 12.21 (centre 12), berry_capacity 4.11 (centre 4), deadwood_capacity 6.12 (centre 6), deadwood_spots 19.77 (centre 20), food_regen_permille 1004.35 (centre 1000), forest_permille 294.24 (centre 300), starting_agents 23.95 (centre 24); climate shares harsh 0.367, mild 0.375, normal 0.258 (weights 0.333 / 0.334 / 0.333); deadwood shortfalls 0 seeds; berry-repair swaps 0 seeds.

Worldgen counts (H6 "counted and explained"): none of the 128 judged game_v16 seeds and none of the 128 game_v15 seeds needed a berry-repair swap or placed fewer deadwood spots than asked. A wider count on one-day worlds of seeds 1000–2999 (`probe_frozen`, measurement only): berry-repair swaps in 4 / 2,000 game_v15 seeds (1–3 swaps) and 16 / 2,000 game_v16 seeds (1–8 swaps; more where the forest share is low and up to 15 bushes need forest edges), deadwood shortfalls in 0 / 2,000 of either — the 5–18 band holds far more than 26 tiles two apart.

### Part 5 — the diversity pilot (frozen commit; seeds 20000–20095 fit, 20096–20111 scored)

11.2 implementation check: `diversity_p10.py check-v1` reproduces WM-1.1's dataset-v1 A_test values within 0.002 (η² food 0.1946, wood 0.0489, fire 0.0387, cold 0.5982, alive 0.0613, deadwood 0.0270, bushes 0.1360, work 0.2837; R²_abs food 0.7676, wood 0.9470, fire 0.9589, alive 0.9320; `p10-5_pilot/check_v1.json`).

| Variable | η² / R²_abs game_v15 | game_v16_neutral | game_v16 |
|---|---|---|---|
| food_stock | 0.153 / 0.845 | 0.112 / 0.885 | 0.500 / 0.474 |
| wood_stock | 0.055 / 0.942 | 0.056 / 0.941 | 0.063 / 0.934 |
| fire_lit | 0.041 / 0.956 | 0.042 / 0.955 | 0.040 / 0.957 |
| cold_members | 0.586 / 0.359 | 0.603 / 0.361 | 0.582 / 0.354 |
| alive | 0.021 / 0.978 | 0.030 / 0.967 | 0.978 / -0.043 |
| deadwood_total | 0.030 / 0.968 | 0.030 / 0.968 | 0.168 / 0.827 |
| food_on_bushes_total | 0.124 / 0.872 | 0.154 / 0.839 | 0.503 / 0.468 |
| adults_working | 0.304 / 0.676 | 0.327 / 0.653 | 0.484 / 0.474 |

**H7 (judged once, game_v16): FAIL** — food η² 0.500 / R²_abs 0.474 (pass), alive 0.978 / -0.043 (pass), wood 0.063 / 0.934 (fail). game_v15 and game_v16_neutral fail every clause.

T2 nMAE, 24 h → 168 h (every-hour starts, test split std) for game_v16: food_stock: persistence 0.142 → 0.823, apfit_abs 0.090 → 0.327, CalAbs 0.584, an5 0.216; wood_stock: persistence 0.276 → 1.174, apfit_abs 0.175 → 0.200, CalAbs 0.202, an5 0.178; alive: persistence 0.024 → 0.080, apfit_abs 0.026 → 0.089, CalAbs 0.861, an5 0.146.

Attribution (Δη²): exposure (v16_neutral − v15; includes round 2's tighter birth gate) food_stock -0.041, wood_stock +0.002, fire_lit +0.001, cold_members +0.018, alive +0.010, deadwood_total +0.000, food_on_bushes_total +0.030, adults_working +0.023; variety (v16 − v16_neutral) food_stock +0.388, wood_stock +0.007, fire_lit -0.002, cold_members -0.021, alive +0.947, deadwood_total +0.138, food_on_bushes_total +0.349, adults_working +0.157.

Deaths per episode by day 120 (mean of 112): game_v15 old age 72.87, exposure 0.00, starvation + dehydration 0.00; game_v16_neutral old age 70.74, exposure 2.00, starvation + dehydration 0.00; game_v16 old age 64.67, exposure 2.32, starvation + dehydration 0.00.

11.5 (declared adjustment): not used. Scratch recordings of game_v16 on seeds 30000–30111 (outside every judged and pilot set; a scratch recorder outside the clone reading the ranges from an environment variable) with 7.1's ranges, with the wood ranges widened by the allowed step (deadwood spots 12–28, capacity 3–9) and with every range widened by its step plus climate weights [383, 234, 383] (this third set exceeds the climate step, −100 against ±50: an over-wide check, not an allowed set) gave wood η² 0.073 / 0.094 / 0.111 (food 0.331 / 0.327 / 0.524, alive 0.960 / 0.960 / 0.976). None of the three tested sets, the widest beyond the allowed step, came near 0.30, so a second pilot was judged unable to change H7's verdict and was not recorded (see DECISIONS).

### Part 4 — H10 timing (back to back, 17:58–18:12 KST, nothing else heavy; `probe_frozen`, tuning sets with twins)

| Duration | game_v15 ms / agent-tick | game_v16_neutral | ratio | game_v16 | ratio |
|---|---:|---:|---:|---:|---:|
| 10day | 0.0011412 | 0.0011721 | 1.027 | 0.0012091 | 1.060 |
| 120day | 0.0014388 | 0.0016967 | 1.179 | 0.0016874 | 1.173 |

H10: **PASS** (≤ 1.25 for both scenes and durations). The timing runs reproduce the round-2 summaries (v16 scenes) and Part 0's (v15) on every simulation value.

### Part 6 — consequential interventions (fork experiment, game_v16, frozen recorder)

Seeds 12000–12031, 35 days, R = 4 forked at 28,800, the five C_cf variants and the one-unit gift plus the factual; placebos on 12000–12007; 1,400 recorded episodes (17:41–17:57). G0a: placebos 224 / 224 identical, replicate 0 = unforked 224 / 224: **PASS**. G0b (reported, Pass 9's gate): FAIL as in Pass 9 (dark nights VRF 1.00 for every variant; cold member-hours passes for gift_wood, rot_wood, rain, windfall, gift1, fails for cold_snap's lower bound 0.997).

| variant | seeds with deaths effect > 0 | mean deaths effect | dark-night effect differs from v15 (seeds) | mean dark-night effect v16 / v15 |
|---|---:|---:|---|---|
| cold_snap | 0.062 | +0.156 | 0.562 (18 / 32) | +0.328 / +0.480 |
| gift1 | 0.000 | -0.008 | 0.031 (1 / 32) | +0.008 / +0.000 |
| gift_wood | 0.000 | -0.625 | 0.562 (18 / 32) | -0.734 / -0.820 |
| rain | 0.000 | -0.594 | 0.594 (19 / 32) | +0.266 / +0.230 |
| rot_wood | 0.562 | +1.180 | 0.562 (18 / 32) | +1.117 / +1.352 |
| windfall | 0.000 | -0.164 | 0.531 (17 / 32) | -0.367 / -0.309 |

Deaths effect = replicate-mean (variant − factual) deaths by the day-29 06:00 row, exposure deaths included (`--deaths-with-exposure`). By cause the whole effect is exposure: rain −0.59 exposure deaths (births effect −0.39), cold_snap +0.16 (births effect +0.12), rot_wood +1.18 (+0.57), gift_wood −0.63 (−0.41); old age and starvation effects 0 for every variant. The "differs from v15" column compares different worlds (game_v16 seed 12000 is not game_v15 seed 12000 under world variety) and replicate means over R = 4 against R = 8, so "differs" is nearly automatic; it is reported as asked, not as evidence of a changed intervention.

**H8: FAIL** — of {rain, cold_snap, rot_wood} only rot_wood has deaths effect > 0 in ≥ 0.25 of seeds (0.56) with mean ≥ 0.25 (+1.18); rain's mean is −0.59 (0 seeds positive) and cold_snap's +0.16 (0.06 of seeds). gift_wood's mean deaths effect −0.63 ≤ 0 holds. Rain keeps the fire out for its 13 hours but burns no wood, so the saved wood keeps a later night lit and breaks the streak that would have killed (its dark-night effect is still +0.27); a two-night cold snap mostly costs hp that a fed village heals.

VRF / mean effect / sign stability / divergence tables: `docs/revitalize/probe/p10-6_forks/fork_metrics.md`.

Note (review round 2, L6): the probe's reported `food_supply_per_day` (`probe/long_run.rs`) counts bushes unweighted; the game_v16 birth gate and store target use the capacity-weighted estimate (`min(cap, 4) / 4` per bush). The difference is reporting only.

### Part 7 — final reproduction and final gate

Final reproduction (`probe_frozen` = the final Rust, Pass 9's flags, 18:14–18:25): `p10-7_final_legacy`, `_v8`, `_v9_10day` / `_120day`, `_v10_*`, `_v11_*`, `_v12_*`, `_v13_*`, `_v14_*` equal `p9-6_final_*` on every shared value (4,849 to 57,658 per run); the only difference is `schema_version` 9 → 10 (plus the new keys); all 14 determinism twins pass. v15: Part 0 and the three guards.

Final gate (`gate_pass10.sh` on `14372ea2`, 20:29–20:53 KST, `verification/p10-7/gate.txt`): every step exit 0; cargo test debug 2,054 / release 2,054 passed, 0 failed (Pass 9: 2,029 each); clippy both; forbidden math 0; strict check; bridge build; smoke without `SCRIPT ERROR`/`ERROR:`; Python 86 (`tools/revitalize`) + 42 (`tools/worldmodel`). `codex/revitalize-2026-10` fast-forwarded to `14372ea2` at 20:53 (earlier to `58119582` at 17:33).

| H | Verdict | Where |
|---|---|---|
| H1 | PASS | round 2 + held-out twins, legacy pin test, Part 0 / guards / Part 7 reproductions |
| H2 | PASS | `p10-3_round2_*_targets`, `p10-4_heldout_*_targets` |
| H3 | PASS | same (game_v16) |
| H4 | PASS | same + `harness_exposure_only_from_cold` |
| H5 | PASS | v16_neutral vs v15, tuning and held-out |
| H6 | PASS | `p10-4_heldout_v16_targets` (128 seeds) |
| H7 | FAIL (wood) | `p10-5_pilot/diversity.md` |
| H8 | FAIL (only rot_wood) | `p10-6_forks/fork_metrics.md` |
| H9 | PASS | recorder byte identity (v14/v15/v16 families), schemas 1–3 load, v16 fork tests |
| H10 | PASS | `p10-4_timing_*_targets` |

## Pass 11 — villages that split: wild regions, fission, exposure v2, soaking rain (`game_v17_nofission`, `game_v17`)

All probe runs use Pass 10's flags (the eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, `--sample-every` 120, `--workers 4`, `--check-determinism`; legacy with its own flags). Binaries frozen in `/Users/rexxa/work/p11bin/`.

### Part 0 — baseline reproduction (`probe_p0`, byte-identical to Pass 10's `probe_frozen`; 21:21–22:04 KST)

| run | reference | shared values | differences |
|---|---|---:|---:|
| `p11-0_v16_10day` | `p10-3_round2_v16_10day` | 11484 | 0 |
| `p11-0_v16_120day` | `p10-3_round2_v16_120day` | 84211 | 0 |
| `p11-0_v16n_10day` | `p10-3_round2_v16n_10day` | 11036 | 0 |
| `p11-0_v16n_120day` | `p10-3_round2_v16n_120day` | 83968 | 0 |
| `p11-0_v16_heldout_10day` | `p10-4_heldout_v16_10day` | 11484 | 0 |
| `p11-0_v16_heldout_120day` | `p10-4_heldout_v16_120day` | 110181 | 0 |
| `p11-0_v16n_heldout_10day` | `p10-4_heldout_v16n_10day` | 11036 | 0 |
| `p11-0_v16n_heldout_120day` | `p10-4_heldout_v16n_120day` | 109702 | 0 |
| `p11-0_v15_10day` | `p10-0_v15_10day` | 6214 | 1 |
| `p11-0_v15_120day` | `p10-0_v15_120day` | 57597 | 1 |
| `p11-0_v14_10day` | `p10-7_final_v14_10day` | 10844 | 0 |
| `p11-0_v14_120day` | `p10-7_final_v14_120day` | 83775 | 0 |
| `p11-0_legacy` | `p10-7_final_legacy` | 5115 | 0 |

The two v15 differences are `schema_version` only (Pass 9's binary wrote 9, Pass 10's 10); every determinism twin passes. Frozen pins `harness_revitalize_pass11_frozen.rs` (v16 digests seeds 0–3 at 1,440 and 40,320 ticks, `game_v16_neutral()` and `WorldVariety::draw` literals) pass on the base.

### Part 1–2 — 4.6 guard (commit `600cb93d`, `probe_p12`, 22:32–22:46 KST)

`p11-2_guard_v16_*` and `p11-2_guard_v16n_*` (tuning sets, 64 × 10 and 32 × 120) equal Part 0's `p11-0_v16_*` / `p11-0_v16n_*` on every shared value (11,484 / 84,211 / 11,036 / 83,968); the only difference is `schema_version` 10 → 11. All determinism twins pass. (Parts 1 and 2 share one commit; see DECISIONS.)

### Part 3 — tuning

Exploration (not judged rounds; disclosed as 4.4 asks): scratch seeds 900000–900031 × 120 days, `game_v17` unless noted, run by a scratch binary outside the clone (`/Users/rexxa/work/p11bin/scratch`, path dependencies on the clone's crates, knobs set on the `WorldRules` value in process; never committed). Quick statistics computed in process (seed-winter = display days 21–27 + 28 k; stage attribution approximate). Config D–G include the shared-bush supply change (below).

| Run | Knobs (exposure child/adult/elder, hunger weight; fission) | Seeds with fission | First-fission day median / IQR | ≥ 2 alive day 120 | Seed-winters with a death / median / max | Stage child/adult/elder (A–G: counted old-age deaths of the same tick, unreliable) | Starvation+dehydration median / max |
|---|---|---:|---|---:|---|---|---|
| A | start: 8/6/8, 0.5; 28, 0.95 | 0.97 | 28 / 0.5 | 0.94 | 0.59 / 4 / 86 | 0.02 / 0.30 / 0.68 | 3.5 / 51 |
| B | start, `game_v17_nofission` | – | – | – | 0.42 / 0 / 26 | 0.03 / 0.07 / 0.90 | 0 / 0 |
| C | 7/5/5, 0.5; 30, 1.0 | 0.91 | 28 / 0 | 0.91 | 0.45 / 0 / 49 | 0.01 / 0.24 / 0.75 | 5 / 42 |
| D | C + shared-bush supply | 0.91 | 28 / 0 | 0.91 | 0.38 / 0 / 23 | 0.01 / 0.10 / 0.89 | 0 / 18 |
| E | 6/7/5, 1.0; 32, 1.0 | 0.81 | 28 / 1.75 | 0.78 | 0.55 / 3 / 72 | 0.01 / 0.36 / 0.63 | 0 / 2 |
| F | 5/5/5, 1.0; 32, 1.0 | 0.81 | 28 / 0 | 0.81 | 0.29 / 0 / 22 | 0.00 / 0.11 / 0.89 | 0 / 6 |
| G | 5/6/5, 1.0; 32, 1.0 | 0.81 | 28 / 0.75 | 0.81 | 0.43 / 0 / 33 | 0.00 / 0.23 / 0.77 | 0 / 6 |
| H | 9/4/6, 0.5; 32, 1.0 (first fixed stage attribution) | 0.81 | 28 / 0 | 0.81 | 0.40 / 0 / 20 | (scratch shares unreliable) | 0 / 7 |
| I | 8/5/6, 0.5; 32, 1.0 | 0.81 | 28 / 0 | 0.81 | 0.39 / 0 / 30 | – | 0 / 5 |
| J | H + utilization 0.76 | 0.75 | 28 / 1 | 0.75 | 0.45 / 0 / 21 | – | 0 / 8 |
| K | J + `min_members` 30 | 0.91 | 28 / 0 | 0.91 | 0.48 / 0 / 21 | – | 0 / 19 |
| L | 9/5/6, 0.5; region bushes 7–11 | 0.81 | 28 / 0 | 0.81 | 0.42 / 0 / 33 | – | 0 / 5 |
| M | L + region deadwood 8–14 | 0.81 | 28 / 0 | 0.81 | 0.43 / 0 / 33 | – | 0 / 6 |
| N | L + region deadwood 8–14 + cooldown 42 (after round 2) | 0.81 | 28 / 0 | 0.75 | 0.39 / 0 / 19 | – | 0 / 4 |
| N2 | N + region resources beyond the camp's reach (after round 2) | 0.81 | 28 / 0 | 0.75 | 0.40 / 0 / 25 | – | 0 / 4 |

Findings that shaped round 1. (1) With the start values both villages' birth gates counted the same bushes (centres 22–28 apart, forage radius 16): famines with empty stores in both villages (262 starvation/dehydration deaths in 32 seeds, half in the original village, by the probe's death census on the same seeds). Under `FissionRule` a bush inside the catchments of n settlements with members now counts 1/n in the capacity-weighted supply (`food_economy::weighted_source_count`; v8–v16 untouched). (2) The first fission is pinned to display day 28 (the first spring day after the 14-day minimum age): the birth gate holds every village at its capacity from about day 14 (members/capacity at day 28 06:00 were equal in 27 of 32 seeds and within 2 in the rest), so with `crowding` ≤ 1.0 the daily check fires at the first opportunity whenever capacity ≥ `min_members`, and never otherwise. Within the allowed knobs the first-fission day is a threshold on a static world parameter, not a state that varies. (3) The scratch stage shares of A–G were wrong (see the header); the probe of round 1 showed adults taking 0.81 of exposure deaths at [5, 6, 5] with hunger weight 1.0.
A scratch probe with H's exposure knobs (built from the uncommitted tree, seeds 900000–900031) read stages from the death census: exposure deaths elder 0.56 / child 0.27 / adult 0.17, starvation + dehydration 35 (all in daughters, late winter, empty stores). A second scratch experiment (after round 2; seeds 900000–900031 × 120 days, the eight clocks) measured the median adult work share at 15:00 (12:00): `game_v16` 0.154 (0.272), `game_v17_nofission` round 2 0.068 (0.305), the same build without wild regions 0.160 (0.269) — the regions, whose bushes (16 tiles) and deadwood (16–18 tiles) fell inside the camp's forage and wood radii, made the original village richer and idle in the afternoon.

| Round (commit) | Knobs | Scene | J1 | J2 | J3 | J4 | J5 | J6 |
|---|---|---|---|---|---|---|---|---|
| 1 (`867fa6e2`) | exposure [5, 6, 5], hunger weight 1.0; fission 32 / 1.0; shared-bush supply | v17 | PASS | PASS | FAIL: first-fission median, IQR | PASS | FAIL: adult share, max share of members, starvation | FAIL: Y9 work_1500 |
| 1 | same | v17_nofission | PASS | PASS | – | – | (adult share 0.99, reported) | FAIL: Y9 work_1200, work_1500, X4 normal gathered |
| 2 (`0bdd99e5`) | exposure [9, 5, 6], 0.5; region bushes 7–11; regional fall at the camp's per-spot rate | v17 | PASS | PASS | FAIL: first-fission median, IQR | PASS | FAIL: max share of members, starvation | FAIL: Y10 evening every seed, Y9 work_1200, work_1500 |
| 2 | same | v17_nofission | PASS | PASS | – | – | (adult share 0.13, reported) | FAIL: Y9 work_1200, work_1500, idle_1200 |

Round details (game_v17, tuning seeds): round 1 — fissions in 0.81 of seeds, first-fission day median 28 / IQR 0, ≥ 2 settlements alive on day 120 0.81, cancellations 0 / 61, daughters alive 28 days after founding 0.75 (39 / 52; 0.30 of 61 daughters dissolved or merged); exposure stages child / adult / elder 0.00 / 0.81 / 0.19 (506 deaths in 48 seed-winters), seed-winters with a death 0.375, median 0, max share of members 0.70, mild / normal / harsh means 0.72 / 2.40 / 13.72, most deaths in one settlement-hour median 2 (max 7); starvation + dehydration day 120 median 0, max 17. Round 2 — fissions 0.81, first day median 28 / IQR 0, ≥ 2 alive 0.75, daughters alive 28 days 0.78 (40 / 51; 0.27 dissolved or merged); stages 0.18 / 0.43 / 0.39 (415 deaths, 49 seed-winters), share 0.38, median 0, max share 0.50, mild / normal / harsh 0.62 / 2.08 / 10.96, settlement-hour median 1 (max 6); starvation max 22 (in daughters founded at regions with 0–3 bushes, seeds 19, 11, 9, 28). game_v17_nofission rounds 1 / 2: seed-winters with a death 0.16 / 0.18, max share 0.34 / 0.26, starvation 0 / 0.
| 3 (`c5b200d7`) | round 2 + fission `cooldown_days` 42, region deadwood 8–14, region resources beyond the camp's forage and wood radii | v17 | PASS | PASS | FAIL: first-fission median, IQR | PASS | FAIL: max share of members, starvation | FAIL: Y9 work_1200, work_1500 |
| 3 | same | v17_nofission | PASS | PASS | – | – | (adult share 0.147, reported) | FAIL: Y9 work_1200, work_1500, idle_1200 |

Round 3 detail (game_v17): fissions 0.81 of seeds, first day median 28 / IQR 0 (26 seeds; max 34), ≥ 2 alive on day 120 0.72, cancellations 0 / 44, daughters alive 28 days 0.79 (33 / 42; 0.23 of 44 dissolved or merged); exposure stages child / adult / elder 0.22 / 0.42 / 0.37 (283 deaths, 43 seed-winters), settlement-hour median 1 (max 5), max share of members 0.46 (seeds 24 and 27, day-106 harsh winters of a 2- and a 3-settlement world); starvation + dehydration median 0, max 9. game_v17_nofission: adult share 0.147 (75 deaths; below the 0.15 bound, reported only), starvation 0.

**Frozen knobs (round 3, 3 of 3 judged rounds used, after 15 disclosed scratch configurations A–N2):** exposure `damage_per_cold_hour` [child 9, adult 5, elder 6], `hunger_weight` 0.5, `heal_block_hours` 2, hardiness 700–1300; `rain_soak_permille` 300; `SustainRule::utilization` 0.78 (v16's); `FissionRule` `min_members` 32, `crowding` 1.0, `group_share` 0.35, `group_min` 6, `group_max` 14, `min_adults` 4, `cooldown_days` 42, `min_age_days` 14, `site_clearance` 16, `stock_share` true, `max_settlements` 4, `settle_timeout_days` 2, `remnant_members` 2; wild regions 1–3, region bushes 7–11, region deadwood 8–14.

**Frozen commit:** `c5b200d7` (`c5b200d7ee618b465d6aa0fdf054fb82d820eed6`, Part 3, 00:01 KST on 2026-10-13). Parts 4–6 run `probe_frozen` / `wm_frozen` built from it (sha256 `1aa47e82…` / `47f70a9e…`; `probe_frozen` = `probe_p3r3`); recordings carry `WORLDSIM_GIT_SHA` = that commit.

Unmet after three rounds (hypotheses, evidence above): J3's first-fission timing (the check fires at the first spring day after the minimum age whenever the static capacity reaches `min_members`; with `crowding` ≤ 1.0 no allowed knob makes it a state that varies by seed); J5's starvation and max-share clauses (daughters founded at regions with few bushes; and, review round 2 M1, every winter over the 0.35 bound is a harsh one with 4–5 dark nights whose deaths cluster in a daughter on 2–3 consecutive late nights — seed 24: 35 deaths, 27 in a daughter of 50 members; held-out seeds 38 and 47 the same — a daughter has 8–14 deadwood spots against the camp's 14–26 and runs out of wood in a harsh winter); J6's Y9 work shares (the 15:00 adult work share of `game_v17_nofission` stays below v16's; a scratch experiment tied most of the drop to the wild regions, and moving region resources beyond the camp's reach in round 3 did not restore it on the tuning seeds — the remaining cause was not isolated within the rounds).

### Part 3 — 4.6 guard on the frozen commit (`probe_frozen`, 00:02–00:28 KST)

`p11-3_guard_v16_*` and `p11-3_guard_v16n_*` (tuning sets) equal Part 0's runs on every shared value (11,484 / 84,211 / 11,036 / 83,968); only `schema_version` 10 → 11 differs; all twins pass. (Review round 1, M2: this guard finished after the round-3 targets were written with `--v8-v16-repro PASS`; the receipt is backed by this guard and by Part 7's reproduction.)

### Part 4 — held-out health (frozen commit; seeds 64–127 × 10, 32–95 × 120; 00:02–00:28 KST)

| Scene | J1 | J2 | J3 | J4 | J5 | J6 | J9 (tuning + held-out 10-day, 128 seeds) |
|---|---|---|---|---|---|---|---|
| v17 | PASS | PASS | FAIL: first-fission median 28, IQR 1 | PASS (0.85) | FAIL: max share of members 0.48, starvation max 9 | (reported) FAIL: Y9 work_1200 0.335 vs v16 0.370, work_1500 0.174 vs 0.241 | PASS |
| v17_nofission | PASS | PASS | – | – | (reported) PASS | FAIL: Y9 work_1500 0.116 vs v16 0.241 (both fail; worse), idle_1200 0.623 vs 0.617 (both fail; worse) | PASS |

v17 detail: fissions in 0.72 of the 64 seeds (75 attempts, 0 cancelled), first-fission day median 28 (p90 29.5, max 32; these are zero-based day indices of the probe, i.e. one-based display day 29 — review round 2, L5), ≥ 2 settlements alive on day 120 0.64, never above 4; daughters alive 28 days after founding 0.85 (64 / 75), 0.20 dissolved or merged by day 120; founders median 13 (11–14). Exposure (J5 stage clauses pooled over tuning round 3 and held-out, 131 seed-winters with a death, 827 deaths): child 0.23, adult 0.41, elder 0.36 (all seed-winters had elders at their start); most deaths in one settlement-hour: median 1, max 9 (288 settlement-nights). H3 on held-out: seed-winters with a death 0.34, median 0, max 34 deaths, max share of members 0.48 (seeds 38 and 47, day-78 winters), mild / normal / harsh means 0.30 / 1.66 / 5.65 (increasing); starvation + dehydration 0 at day 10 in every seed, day 120 median 0, max 9. J2: alive ≥ max(8, 0.4 × founders) at every sample, day-120 retention min 0.80, no settlement lingering with 1–2 members over 48 h. v17_nofission: exposure stages child 0.59 / adult 0.15 / elder 0.26 (233 deaths, 86 seed-winters), seed-winters with a death 0.24, max share 0.27, mild / normal / harsh 0.01 / 0.36 / 1.98, starvation 0.

J9 (128 seeds, 10-day rounds of game_v17; judged, as the goal states it, on the drawn parameters with placement failures counted): every drawn mean within ±10 % of its range centre — founders 23.95 (24), bushes 12.21 (12), capacity 4.11 (4), deadwood spots 19.77 (20), capacity 6.12 (6), forest 294.2 (300), food regen 1004.4 (1000), wild regions 1.98 (2), region bushes 9.03 (9), region deadwood 10.83 (11); climate shares within ±0.10. Region placement failures 0 (every requested region placed). Shortfalls counted: region bushes 746 short of the requested total in 93 of 128 seeds (the camp berries' filter — Forest beside Plain, 2–6 from the centre, beyond the camp's 18-tile reach — finds few tiles near the map edge, where many centres lie), region deadwood 39 short; distance from a region centre to water median 16 (1–30). The bush shortfall is the main reason daughters at some sites starve (J5). Review round 2, M2: realized region bushes are 1,546 of 2,292 requested (0.67, ≈ 6.1 per placed region against the drawn mean 9.0) — outside ±10 % of the centre if J9 were judged on what exists; the PASS is on the drawn parameters only.

### Part 5 — the diversity pilot (frozen commit; seeds 22000–22095 fit, 22096–22111 scored; recording 00:02–00:23 KST)

Every pilot `meta.json` carries the frozen commit and schema 5 (336 episodes). 11.2 implementation check: `diversity_p11.py check-v1` reproduces WM-1.1's 12 dataset-v1 A_test numbers within 0.002 (`p11-5_pilot/check_v1.json`).

| Variable | η² / dyn-η² / R²_abs game_v15 (P10 pilot) | game_v16 (P10 pilot) | game_v16 | game_v17_nofission | game_v17 |
|---|---|---|---|---|---|
| food_stock | 0.153 / 0.130 / 0.845 | 0.500 / 0.320 / 0.474 | 0.411 / 0.298 / 0.573 | 0.437 / 0.434 / 0.548 | 0.672 / 0.694 / 0.309 |
| wood_stock | 0.055 / 0.057 / 0.942 | 0.063 / 0.065 / 0.934 | 0.068 / 0.069 / 0.929 | 0.068 / 0.069 / 0.929 | 0.269 / 0.276 / 0.709 |
| fire_lit | 0.041 / 0.044 / 0.956 | 0.040 / 0.043 / 0.957 | 0.044 / 0.048 / 0.952 | 0.045 / 0.048 / 0.951 | 0.045 / 0.048 / 0.951 |
| cold_members | 0.586 / 0.586 / 0.359 | 0.582 / 0.582 / 0.354 | 0.598 / 0.599 / 0.358 | 0.598 / 0.598 / 0.361 | 0.615 / 0.616 / 0.338 |
| alive | 0.021 / 0.790 / 0.978 | 0.978 / 0.999 / −0.043 | 0.953 / 0.998 / 0.004 | 0.953 / 0.998 / 0.003 | 0.908 / 0.933 / 0.079 |
| deadwood_total | 0.030 / 0.030 / 0.968 | 0.168 / 0.087 / 0.827 | 0.187 / 0.097 / 0.809 | 0.318 / 0.116 / 0.681 | 0.340 / 0.124 / 0.658 |
| food_on_bushes_total | 0.124 / 0.133 / 0.872 | 0.503 / 0.492 / 0.468 | 0.375 / 0.442 / 0.606 | 0.826 / 0.467 / 0.164 | 0.802 / 0.576 / 0.169 |
| adults_working | 0.304 / 0.299 / 0.676 | 0.484 / 0.490 / 0.474 | 0.474 / 0.475 / 0.491 | 0.490 / 0.493 / 0.472 | 0.492 / 0.491 / 0.478 |

**J7 (judged once, game_v17): FAIL** — food_stock 0.672 / R²_abs 0.309 (pass), alive 0.908 / 0.079 (pass), wood_stock 0.269 / 0.709 (fail, just below 0.30 and just above 0.70); dyn-η² alive 0.933 and food_stock 0.694 (pass, ≥ 0.25).

Reading dyn-η²: by its definition (subtract each episode's own mean of display days 3–10, η² over days ≥ 11) `alive` scores 0.79–0.999 in every scene since game_v15, because a world's later population level (its capacity, reached after day 10) differs from its founders' level; the statistic removes the founding offset but not the capacity offset, so for `alive` it is not a test of diverging histories. For food it separates: game_v16 0.30, game_v17_nofission 0.43, game_v17 0.69.

Attribution (Δη² / Δdyn-η²): v16 → v17_nofission (exposure v2 + rain + regions) food +0.026 / +0.136, wood 0.000 / 0.000, alive 0.000 / 0.000, deadwood +0.131 / +0.019, bushes +0.451 / +0.025, work +0.016 / +0.018; v17_nofission → v17 (fission) food +0.235 / +0.260, wood +0.201 / +0.207, alive −0.045 / −0.065, cold +0.017 / +0.017, deadwood +0.022 / +0.008, bushes −0.024 / +0.109, work +0.002 / −0.002.

Settlements alive (game_v17, 16 test seeds) at days 30 / 60 / 90 / 118: means 1.19 / 1.50 / 1.69 / 1.69, variances 0.15 / 0.25 / 0.72 / 0.72; day 118: 9 seeds one village, 3 two, 4 three (every earlier scene: one in every seed).

T2 nMAE 24 h → 168 h (test std, every-hour starts), game_v17: food persistence 0.114 → 0.651, apfit_abs 0.077 → 0.345, cal_abs 0.630, an5 0.228; wood persistence 0.221 → 0.974, apfit_abs 0.179 → 0.295, cal_abs 0.375, an5 0.214; alive persistence 0.024 → 0.117, apfit_abs 0.026 → 0.133, cal_abs 0.767, an5 0.199 (game_v16 on the same seeds: alive persistence 0.027 → 0.103).

Deaths per episode by day 120 (mean of 112): game_v16 old age 66.07, exposure 2.46; game_v17_nofission 65.43 / 2.89; game_v17 old age 73.59, exposure 8.19, starvation 0.18, dehydration 0.23.

Part 2 scratch (disclosed here as 4.4 asks; review round 1, L7): seeds 900000–900011 × 120 days of game_v17 at the start values, with the existing membership rule and with sticky membership — `resolve_exclusivity` moves per seed 0–2,667 (median ≈ 600) versus 0–5.

### Part 6 — consequential interventions (fork experiment, game_v17, frozen recorder, 00:28–00:45 KST)

Seeds 12000–12031, 35 days, R = 4 forked at 28,800, the five C_cf variants and the one-unit gift plus the factual; placebos on 12000–12007; 1,120 distinct forked and 280 unforked episodes on disk (1,152 and 288 listed runs: the placebo pass re-records the factual of seeds 12000–12007 into the same directories, byte-identical by determinism). Dark nights read from the home settlement's slot (`--home-slot-fire`), exposure deaths counted (`--deaths-with-exposure`). G0a: placebos 224 / 224 identical, replicate 0 = unforked 224 / 224: **PASS**. G0b (reported): FAIL as in Passes 9–10 (dark-night VRF 1 for every variant; cold member-hours passes for every variant).

| variant | seeds with deaths effect > 0 | mean deaths effect | mean dark-night effect | deaths effect by stage adult / child / elder |
|---|---:|---:|---:|---|
| cold_snap | 0.312 | +0.297 | +0.297 | +0.055 / +0.172 / +0.070 |
| gift1 | 0.062 | −0.023 | 0.000 | −0.016 / −0.008 / 0.000 |
| gift_wood | 0.000 | −0.727 | −0.797 | −0.156 / −0.359 / −0.211 |
| rain | 0.062 | −0.359 | +0.695 | −0.078 / −0.133 / −0.148 |
| rot_wood | 0.594 | +1.617 | +1.055 | +0.422 / +0.805 / +0.391 |
| windfall | 0.031 | −0.273 | −0.367 | −0.109 / −0.094 / −0.070 |

(Stage effects: all causes, agents gone by the day-29 06:00 row by their last life stage, replicate mean, mean over seeds.)

**J8: FAIL** — placebo identity 100 % (pass); rain: dark-night effect +0.70 (pass) but mean deaths effect −0.36 (fail: rain still saves lives on average); cold_snap (0.31 of seeds positive) and rot_wood (0.59) have positive deaths effects in ≥ 0.25 of seeds (pass, two of three); gift_wood mean deaths effect −0.73 ≤ 0 (pass). Hypothesis: a 13-hour rain spares 13 hours of burn (3–4 wood an hour in winter, ≈ 40–50 wood) while the soak at 300 ‰ removes 30 % of a store that is usually 30–60 wood (≈ 10–18): the rain still leaves more wood for the following nights. A soak large enough to remove more than the spared burn (≈ 700 ‰ at those stores, above the allowed 150–500) or a rain that also burns would be needed; J8 was not measured in Part 3, so no round could tune it.

### Part 6.2 — macro sensitivity (game_v17, 120 days, frozen recorder, 00:45–00:58 KST; J11 reported)

Seeds 12000–12031, `--every 60`, no `--agents`, schedules = the `rot_wood` and `gift_wood` files plus the factual, forked at 28,800 with R = 2 (192 episodes, 739 s). Shape = settlements alive at the 08:00 rows of days 60 / 90 / 118 and the first-fission day.

| Comparison | Share differing | Mean abs Δ alive day 118 |
|---|---:|---:|
| (a) factual replicate 0 vs 1 (noise only), per seed | 0.094 | 1.56 |
| (b) rot_wood vs factual of the same replicate, per seed-replicate pair | 0.266 | 1.95 |
| (b) gift_wood vs factual of the same replicate, per pair | 0.172 | 0.50 |

Seeds differing in at least one / in every replicate: rot_wood 0.31 / 0.22, gift_wood 0.22 / 0.13. A small act (60 wood rotten or given on day 21) changes the number of villages later in the year or the first-fission day in about a quarter (rot) or a sixth (gift) of the worlds, against about a tenth from agent noise alone; the effect on the population at day 118 (1.95 agents for rot_wood, 0.50 for gift_wood) is not distinguishable from agent noise (1.56); only the shape share separates (0.27 against 0.09).

### Part 7 — final reproduction (`probe_final` built from `ff7e0fd9`, the final Rust; Pass 10's flags; 00:45–01:43 KST)

| run | reference | shared values | differences | note |
|---|---|---:|---:|---|
| `p11-7_final_legacy` | `p10-7_final_legacy` | 5115 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v8` | `p10-7_final_v8` | 8240 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v9_10day` | `p10-7_final_v9_10day` | 8205 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v9_120day` | `p10-7_final_v9_120day` | 67963 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v10_10day` | `p10-7_final_v10_10day` | 8397 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v10_120day` | `p10-7_final_v10_120day` | 68546 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v11_10day` | `p10-7_final_v11_10day` | 10712 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v11_120day` | `p10-7_final_v11_120day` | 83707 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v12_10day` | `p10-7_final_v12_10day` | 10712 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v12_120day` | `p10-7_final_v12_120day` | 83719 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v13_10day` | `p10-7_final_v13_10day` | 10842 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v13_120day` | `p10-7_final_v13_120day` | 83774 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v14_10day` | `p10-7_final_v14_10day` | 10844 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v14_120day` | `p10-7_final_v14_120day` | 83775 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v15_10day` | `p11-0_v15_10day` | 10844 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v15_120day` | `p11-0_v15_120day` | 83781 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v16_10day` | `p11-0_v16_10day` | 11484 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v16_120day` | `p11-0_v16_120day` | 84211 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v16n_10day` | `p11-0_v16n_10day` | 11036 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v16n_120day` | `p11-0_v16n_120day` | 83968 | 1 | `schema_version` 10 → 11 only |
| `p11-7_final_v17_10day` | `p11-3_round3_v17_10day` | 16226 | 0 | identical |
| `p11-7_final_v17_120day` | `p11-3_round3_v17_120day` | 97206 | 0 | identical |
| `p11-7_final_v17n_10day` | `p11-3_round3_v17n_10day` | 15394 | 0 | identical |
| `p11-7_final_v17n_120day` | `p11-3_round3_v17n_120day` | 95097 | 0 | identical |
| `p11-7_final_heldout_v17_10day` | `p11-4_heldout_v17_10day` | 16231 | 0 | identical |
| `p11-7_final_heldout_v17_120day` | `p11-4_heldout_v17_120day` | 130812 | 0 | identical |
| `p11-7_final_heldout_v17n_10day` | `p11-4_heldout_v17n_10day` | 15399 | 0 | identical |
| `p11-7_final_heldout_v17n_120day` | `p11-4_heldout_v17n_120day` | 127126 | 0 | identical |

Every legacy, v8–v16 run reproduces Pass 10's final (or Part 0's) summaries on every shared value; every determinism twin passes. Every `game_v17` and `game_v17_nofission` run of the final binary (tuning and held-out) is identical to the frozen-commit run of the same set, so the post-freeze review fixes (M1 timeout retarget, L2, L4) changed no probe result.

### Part 7 — J12 timing (back to back, 01:47–02:05 KST, nothing else heavy; `probe_final`, tuning sets with twins)

| Duration | game_v16 ms / agent-tick | game_v17_nofission | ratio | game_v17 | ratio |
|---|---:|---:|---:|---:|---:|
| 10day | 0.0012756 | 0.0012756 | 1.000 | 0.0013254 | 1.039 |
| 120day | 0.0017171 | 0.0017480 | 1.018 | 0.0018300 | 1.066 |

J12: **PASS** (≤ 1.3 for both scenes and durations; `p11-7_timing_*_targets`). The timing runs reproduce the round-3 summaries (v17 scenes) and Part 0's (v16) on every simulation value. All J-target files were regenerated with the final evaluator (report-only targets marked; receipts backed by the Part 0/2/3 guards and this Part's reproduction).

### Part 7 — held-out v16 / v16_neutral with the final binary, J10, and the post-review verification re-recordings (02:07–02:48 KST)

- Held-out (review round 2, L6): `p11-7_final_heldout_v16_*` and `_v16n_*` (`probe_final`, seeds 64–127 × 10, 32–95 × 120) equal Pass 10's `p10-4_heldout_v16*` on every shared value (11,484 / 110,181 / 11,036 / 109,702); only `schema_version` 10 → 11; twins pass.
- **J10: PASS.** `wm_final`, game_v17 with agents, seeds 900000–900006 × 30 days (5 fissions founded on day 28 or 29, so the founding, parent-slot and founded-tick columns are exercised), recorded at workers 4, 4 and 1: all 102 data files byte-identical; only the 7 `timing.json` (wall seconds, excluded by definition as in `test_export_determinism.py`) differ. The automated tests `test_v14/v15/v16/v17_fork_families_are_byte_identical` (schema 5, workers 4 / 4 / 1) pass; schemas 1–4 load (`test_wm_data.py`); the v15/v16 fork tests (`harness_revitalize_pass9_forks`, `pass10_forks`) pass.
- Verification re-recordings with `wm_final` (post-review code, labelled with the frozen commit's sha so `meta.json` compares): pilot `game_v17_nofission` 112 / 112 and `game_v17` 112 / 112, forks 1,120 / 1,120, macro 192 / 192 episodes identical to the frozen-commit recordings (every array's sha256 and `interventions.json`). Together with the probe reproduction above: the post-freeze review fixes changed no result of Parts 3–6.

### Part 7 — final gate and J verdicts

Final gate (`gate_pass11.sh` on `e6dedd34`, the last commit that changes code or tests; 02:23–02:48 KST in a detached worktree; `verification/p11-7/gate.txt`): every step exit 0; cargo test debug 2,078 / release 2,078 passed, 0 failed (Pass 10: 2,054 each); clippy both; forbidden math 0; strict check; bridge build; smoke without `SCRIPT ERROR`/`ERROR:`; Python 95 (`tools/revitalize`) + 45 (`tools/worldmodel`). Later commits change docs only. An earlier gate on `600cb93d` (Parts 1–2) was green at 23:02 (2,075 each); one started on `1295cfbc` was stopped when review round 2 changed tests.

| J | Verdict | Where |
|---|---|---|
| J1 | PASS | twins of every v17 round (tuning, held-out, final, timing); legacy pin test; legacy and v8–v16 reproductions (Part 0, guards, Part 7) |
| J2 | PASS | `p11-3_round3_v17*_targets`, `p11-4_heldout_v17*_targets` |
| J3 | FAIL (first-fission timing) | same; share 0.72–0.81, ≥ 2 alive 0.64–0.72, cancellations 0 pass |
| J4 | PASS | 0.79 (tuning), 0.85 (held-out) |
| J5 | FAIL (max share 0.48, starvation max 9) | stage clauses pass (adult 0.41, child 0.23, hour median 1) |
| J6 | FAIL (Y9 work_1500, idle_1200 on game_v17_nofission) | `p11-4_heldout_v17n_targets`, round 3 |
| J7 | FAIL (wood 0.269 / 0.709) | `p11-5_pilot/diversity.md` |
| J8 | FAIL (rain mean deaths effect −0.36) | `p11-6_forks/fork_metrics.md` |
| J9 | PASS (drawn parameters; realized region bushes 0.67) | `p11-4_heldout_v17_targets` |
| J10 | PASS | byte identity, schema loads, fork tests |
| J11 | reported | `p11-6_macro/macro.md` |
| J12 | PASS (1.000–1.066) | `p11-7_timing_*_targets` |

## Pass 12 — a split under pressure, daughters that can live, rain that hurts, a diversity measure for histories (`game_v18_nofission`, `game_v18`)

All probe runs use Pass 11's final flags (the eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, `--sample-every` 120, `--workers 4`, `--check-determinism`; legacy with its own flags). Binaries frozen in `/Users/rexxa/work/p12bin/`.

### Part 0 — baseline reproduction (`probe_p0`, byte-identical to Pass 11's `probe_final`; 11:27–11:52 KST)

| run | reference | shared values | differences |
|---|---|---:|---:|
| `p12-0_v17_10day` | `p11-7_final_v17_10day` | 16226 | 0 |
| `p12-0_v17_120day` | `p11-7_final_v17_120day` | 97206 | 0 |
| `p12-0_v17n_10day` | `p11-7_final_v17n_10day` | 15394 | 0 |
| `p12-0_v17n_120day` | `p11-7_final_v17n_120day` | 95097 | 0 |
| `p12-0_v16_10day` | `p11-7_final_v16_10day` | 13868 | 0 |
| `p12-0_v16_120day` | `p11-7_final_v16_120day` | 94339 | 0 |
| `p12-0_v15_10day` | `p11-7_final_v15_10day` | — | 0 |
| `p12-0_v15_120day` | `p11-7_final_v15_120day` | — | 0 |
| `p12-0_legacy` | `p11-7_final_legacy` | — | 0 |

Every determinism twin passes. Frozen pins `harness_revitalize_pass12_frozen.rs` (v17 and v17_nofission digests seeds 0–3 at 1,440 and 40,320 ticks with births and fissions; literals of `WorldRules::game_v17(0)` and `RANGES`) pass on the base.

### Part 0 — UBS on old data (5.2, `diversity_p12.py`, read-only; `docs/revitalize/probe/p12-0_ubs/`)

Mean over days 60/90/118, all 112 episodes; R²_static / UBS (old dyn-η² on the test split for continuity).

| Variable | p10 `game_v15` | p10 `game_v16` | p11 `game_v16` | p11 `game_v17_nofission` | p11 `game_v17` |
|---|---|---|---|---|---|
| food_stock | see `p10_pilot/ubs.md` | | 0.547 / 0.299 (0.298) | 0.639 / 0.267 (0.434) | 0.275 / 0.516 (0.694) |
| wood_stock | | | 0.073 / 0.084 (0.069) | 0.081 / 0.081 (0.069) | 0.078 / 0.207 (0.276) |
| alive | | | 0.907 / 0.309 (0.998) | 0.901 / 0.319 (0.998) | 0.371 / 0.881 (0.933) |
| deadwood_total | | | 0.959 / 0.103 (0.097) | 0.994 / 0.053 (0.116) | 0.987 / 0.076 (0.124) |
| food_on_bushes_total | | | 0.379 / 0.221 (0.442) | 0.967 / 0.163 (0.467) | 0.696 / 0.434 (0.576) |
| adults_working | | | 0.431 / 0.176 (0.475) | 0.369 / 0.196 (0.493) | 0.308 / 0.244 (0.491) |

p10 pilot UBS (mean 60/90/118): `game_v15` food 0.246, wood 0.088, alive 0.171; `game_v16` food 0.299, wood 0.088, alive 0.240 (bootstrap Δ v16 − v15: food [0.022, 0.112], alive [0.026, 0.143]). Synthetic checks (`test_diversity_p12.py`): static linear worlds UBS < 0.05; an independent random walk after day 10 UBS > 0.3; a static effect quadratic in a hidden parameter that appears only after day 10 gives UBS(alive) **0.970** — the linear ridge counts a nonlinear static effect as unexplained. `game_v17`'s alive UBS 0.881 must be read with that in mind: whether a v17 world splits is a threshold of its day-29 population (planner's reading), i.e. a nonlinear static effect, not history.

### Part 1 — the J6 bisection (`probe_p1`, `probe_p1b`–`p1d`; 11:52–12:57 KST)

Tuning seeds 0–31 × 120 days, every twin passed. Gap in Pass 6 Y9 `work_1500` on these seeds: game_v16 0.254 − game_v17_nofission 0.119 = 0.135 (held-out per Pass 11: 0.241 − 0.116). Decisive: the wild regions' bushes (`abl_no_regions` restores 1.000 of the gap, `fwd_regions` 1.107, `fwd_regions_bushes_only` 1.107, `fwd_regions_wood_only` 0.000); every exposure, rain and regional-wood difference within noise. Activity that took the afternoon: foraging (adults at 15:00 0.178 → 0.128, idle 0.592 → 0.656). Mechanism: region-bush food reaches the store through the incidental Gather arm, the store holds its target in 93 % of 15:00 rows (v16 82.5 %), the forage arm is off. The v18 flag `local_food_availability` (availability, meal target and Gather tile inside the catchment) restores 0.707 of the gap on v17_nofission (`abl_local_food3`). Full table and versions: `docs/revitalize/probe/p12-1_bisect/BISECTION.md`.

### Part 2 — 4.6 guard (commit `40dda8f2`, `probe_p2`, 12:55–13:15 KST)

`p12-2_guard_v17_*`, `p12-2_guard_v17n_*`, `p12-2_guard_v16_*` (tuning sets, 64 × 10 and 32 × 120) equal Part 0's runs on every shared value (16,226 / 97,206 / 15,394 / 95,097 / 13,868 / 94,339); the only difference is `schema_version` 11 → 12 (plus the new schema-12 keys). All determinism twins pass.

### Part 3 — tuning (tuning seeds; `evaluate_pass12.py`, game_v16 reference `p12-0_v16_*`)

Scratch exploration before round 1 (disclosed as 4.4 asks): `p12-scratch_A` = the Part 2 start values (`scratch_probe_a`, = `probe_p2`'s rules before the final J6-fix version; game_v18 and game_v18_nofission, seeds 900000–900031 × 120 days, no twin): fission share 0.906, first-fission display day median 33 / IQR 51 / peak share 0.10, ≥ 2 alive at day 120 0.906, daughters alive 28 d 1.00, settlement-winters over 0.35: 2 of 231 (max 0.53), starvation + dehydration max 0.

| Round | Commit | Knobs (changes) | J3 | J4 | J5 | J6 (v18_nofission) |
|---|---|---|---|---|---|---|
| 1 | `40dda8f2` | 7.1/7.3 start values (hazard 20/40/850/1050/2/500/500/120, min_members 22, min_age 10, cooldown 28, seasons T,T,T,F; floor 600; water 12; region bushes 8–12, deadwood 12–18) | FAIL: fission share 0.969, ≥ 2 alive 0.969 (timing PASS: median 35, IQR 23.5, peak 0.097; cancellations 0/49) | PASS 1.00 (41/41) | FAIL: settlement-winter max 0.417 (3 of 243 over: seed 5 parent 8/20 and daughter 5/12 after an autumn split at day 103, seed 24 daughter 18/50 with 7 dark nights); stage shares child 0.52 / adult 0.25 / elder 0.23; S+D max 1 | FAIL: Y9 work_1500 0.230 (v16 0.254), idle_1200 0.627 (v16 0.621, both > 0.60); work_1200 0.373 PASS |
| 2 | `e94f7404` | min_members 22 → 32, hazard size_lo/size_hi 20/40 → 28/48, area_fall_floor 600 → 1000 | **PASS**: fission share 0.75, first-fission display day median 39.5, IQR 34, peak 0.125 (day 39), ≥ 2 alive 0.75, cancellations 0/30 | PASS (25 of 25 judged... share 1.00 of judged) | FAIL: settlement-winter max 0.38 (2 of 198 over, both seed 24's harsh winter from day 106 with 6 world dark nights: parent 9/25, daughter 19/50); stage shares child 0.54 / adult 0.17 / elder 0.29; S+D max 2 | FAIL (unchanged by fission knobs): work_1500 0.230, idle_1200 0.627 |

Scratch exploration between rounds 2 and 3 (seeds 900000–900031 × 120 days, no twin; `scratch_probe_b` built from round 2's tree with two knob edits and then restored — the restored build is byte-identical to `probe_r2`): B = round 2 + region bushes 5–9 + exposure [8, 4, 5]; C = round 2 (game_v18_nofission, game_v16). v18_nofission work_1500: C 0.174, B 0.175 (game_v16 on these seeds 0.239) — the region-bush range does not move the afternoon, dropped. game_v18 under B: settlement-winters over 0.35: 0 of 202, but exposure stage shares child 0.95 / adult 0.05 (J5 adult bound ≥ 0.15) — adult damage 4 is too low, dropped. Round 3 keeps the adult rate and lowers child and elder by one.
| 3 | `8142cc3a` | exposure [9, 5, 6] → [8, 5, 5] (v18 scenes only) | **PASS** (unchanged: share 0.75, median 39.5, IQR 34, peak 0.125, ≥ 2 alive 0.75) | **PASS** 1.00 (25 of 25 judged; 30 daughters, none dissolved) | FAIL only `max_share_settlement_winter`: max 0.36 (1 of 198: seed 24's parent, 9 of 25, harsh winter from day 106); stage shares child 0.63 / adult 0.31 / elder 0.06; share of seed-winters with a death 0.164; S+D max 2 | FAIL only Y9 work_1500 0.240 (v16 0.254, bound 0.25); work_1200 0.375 PASS, idle_1200 0.620 (v16 0.621, escape) PASS |

Round 2's J4 cell: 25 of 25 daughters judged 28 days after founding were alive (30 daughters, 5 founded too late to judge).

**Freeze** (after three of three judged rounds): commit `8142cc3a` (round 3, the best state: J1–J4 and J9 pass; J5 misses by one settlement-winter at 0.36; J6 misses Y9 work_1500 by 0.010). Binaries `p12bin/probe_frozen` (sha256 `5d92f398…`), `p12bin/wm_frozen` (sha256 `4d321052…`). Strikes (0.3 rule 7): J5 settlement-winter max — three rounds (0.417 → 0.38 → 0.36); hypothesis: seed 24's harsh winter (6–7 world dark nights) runs both villages out of wood whatever the floor; the remaining knob (adult damage) trades directly against J5's adult share (scratch B: adult 4 → adult share 0.05). J6 Y9 work_1500 on v18_nofission — three rounds (0.230, 0.230, 0.240); hypothesis: the 0.29 of Part 1's gap that the local-food flag does not restore (BISECTION.md), a mechanism question for follow-up work, not a knob.

### Part 3 — 4.6 guard on the frozen commit (`probe_frozen`, 13:46–14:05 KST)

`p12-3_guard_*` (v17, v17_nofission, v16 tuning sets) equal Part 2's guard runs on every value (0 differences in all six), hence Part 0's on every shared value.

### Part 4 — held-out health (frozen `8142cc3a`, `probe_frozen`; held-out seeds 64–127 × 10 and 32–95 × 120; 13:46–14:05 KST, concurrently with the guard and the start of the pilot recording; no knob changed)

J6 reference: game_v16 on the same held-out seeds (`p11-7_final_heldout_v16_*`, Pass 11's final binary, schema 11). J5 for game_v18 pools the stage counts and settlement-winters of tuning round 3 and held-out (`--extra-round120`); its H3 clauses are the held-out set's. J9 over the 128 seeds of both 10-day sets.

| Target | game_v18 | game_v18_nofission |
|---|---|---|
| J1 | PASS (twins 64/64, 32/32... held-out 64 × 10 and 64 × 120) | PASS |
| J2 | PASS | PASS |
| J3 | **PASS**: fission share 0.703 (64 seeds), first-fission display day median 38, IQR 38 (p25 20, p75 58), peak day 15 holds 0.111, ≥ 2 alive at day 120 0.703, cancellations 0/67, never above max. h terciles: split rate per daily check 0.009 (h 0–18) / 0.027 (18–27) / 0.048 (27–66). First-fission day correlation with hidden parameters: all |r| ≤ 0.16 (berry_bushes −0.16, region_deadwood +0.16, wild_regions −0.15) | — |
| J4 | **PASS** 1.00 (56 of 56 judged; 67 daughters, none dissolved or merged; founded spring 24, summer 15, autumn 28) | — |
| J5 | FAIL: pooled child share 0.725 (> 0.70), adult 0.249, elder 0.026; settlement-winter max 0.571 (4 over of the pooled set: held-out seeds 39 (daughter 8/14), 66 (parent 12/26), 77 (daughter 11/28), tuning seed 24 (9/25)); Pass 11's world-level ratio max 0.267 (passes); share of seed-winters with a death 0.195, median 0, mild < normal < harsh (0 / 0.30 / 2.83); per-hour median 1; starvation + dehydration 0 at day 10 and max 0 at day 120 | reported: passes |
| J6 | reported: fails Y5 autumn work, Y9 | FAIL: Y9 work_1500 0.213 (v16 0.241), idle_1200 0.629 (v16 0.617; both > 0.60); work_1200 0.357 PASS; every other clause PASS |
| J9 | parameter means and climate shares PASS; realized region bushes 2,519 / 2,519 (1.00), deadwood 3,692 / 3,716 (0.994); placement failures 3 regions of 255 (seeds with 1 of 2, 2 of 3, 2 of 3 placed) — counted and explained: the water rule (centre within 12 of water) leaves no candidate centre 22–28 from the hearth and 20 from the earlier centres | same worlds |

### Part 5 — diversity pilot (frozen recorder `wm_frozen`; seeds 24000–24111; recording 13:53–14:24 KST; `p12-5_pilot/`)

Implementation check (11.2): dataset v1 A_test reproduces WM-1.1's 12 published numbers within ±0.002 (`p12-5_pilot/check_v1`). After review round 1 the two v18 scenes were re-recorded with the final recorder (`wm_final`, commit `1127455b`): game_v18_nofission row digests identical in 112 of 112 episodes, game_v18 in 110 of 112; every metric below is from the final recordings (`p12-7_final_pilot/`) and equals the frozen-recorder values to the third decimal except where noted.

| Variable | game_v16 | game_v17 | game_v18_nofission | game_v18 |
|---|---|---|---|---|
| food_stock η² / R²_abs / R²_static / UBS / dyn-η² | 0.332 / 0.651 / 0.596 / 0.295 / 0.195 | 0.472 / 0.475 / 0.305 / 0.502 / 0.502 | 0.311 / 0.672 / 0.633 / 0.271 / 0.185 | 0.532 / 0.457 / 0.388 / 0.536 / 0.534 |
| wood_stock | 0.069 / 0.929 / 0.049 / 0.084 / 0.069 | 0.190 / 0.786 / 0.031 / 0.210 / 0.198 | 0.068 / 0.930 / 0.044 / 0.084 / 0.069 | 0.236 / 0.749 / 0.130 / 0.249 / 0.240 |
| alive | 0.900 / −0.102 / 0.899 / 0.320 / 0.998 | 0.735 / 0.184 / 0.442 / 0.801 / 0.785 | 0.899 / −0.106 / 0.894 / 0.329 / 0.998 | 0.741 / 0.230 / 0.445 / 0.837 / 0.756 |
| deadwood_total | 0.207 / 0.781 / 0.959 / 0.107 / 0.107 | 0.330 / 0.669 / 0.978 / 0.089 / 0.147 | 0.358 / 0.638 / 0.995 / 0.043 / 0.132 | 0.351 / 0.644 / 0.988 / 0.069 / 0.150 |
| food_on_bushes_total | 0.342 / 0.631 / 0.488 / 0.236 / 0.409 | 0.640 / 0.337 / 0.626 / 0.446 / 0.424 | 0.835 / 0.155 / 0.979 / 0.130 / 0.410 | 0.751 / 0.232 / 0.684 / 0.472 / 0.508 |
| adults_working | 0.474 / 0.491 / 0.324 / 0.202 / 0.474 | 0.438 / 0.528 / 0.272 / 0.244 / 0.439 | 0.473 / 0.492 / 0.331 / 0.204 / 0.476 | 0.480 / 0.488 / 0.373 / 0.240 / 0.479 |

Settlements alive at days 30/60/90/118 (16 test seeds, mean): v16 and v18_nofission 1/1/1/1; v17 1.50/1.75/2.25/2.25 (8 of 16 split by day 30); v18 1.06/1.25/1.56/2.06 (day 118: 3 seeds with 1, 10 with 2, 2 with 3, 1 with 4). Baseline nMAE at 168 h (persistence / cal_abs / apfit_abs): alive v17 0.149 / 0.758 / 0.137, v18 0.139 / 0.709 / 0.160; food v17 0.779 / 0.592 / 0.298, v18 0.733 / 0.563 / 0.307.

Attribution (Δη² / ΔUBS): v17 → v18_nofission (wet burn — inert without interventions —, region fill, daughter fixes, J6 fix, exposure): food −0.160 / −0.231, wood −0.123 / −0.126, alive +0.164 / −0.472; v18_nofission → v18 (hazard fission): food +0.221 / +0.265, wood +0.168 / +0.165, alive −0.158 / +0.509. Paired bootstraps of ΔUBS (90 %): v18 − v17 food 0.034 [−0.063, 0.140], wood 0.038 [0.010, 0.076], alive 0.036 [−0.102, 0.163]; v18 − v18_nofission alive 0.509 [0.432, 0.630]. Within game_v18 by whether the world split (ridge refit per subset): split (66) alive R²_static / UBS 0.140 / 0.811, no split (46) 0.878 / 0.362.

**J7 (game_v18): FAIL** — η² ≥ 0.30 and R²_abs ≤ 0.70: food PASS (0.532 / 0.457), alive PASS (0.741 / 0.230), wood FAIL (0.236 / 0.749); J7b: ΔUBS intervals for alive and food include 0 (FAIL); mean R²_static(alive) v18 0.445 vs v17 0.442 (FAIL by 0.003).

### Part 6 — forks (12.1; `p12-6_forks/` frozen recorder; final re-recording `p12-7_final_forks/` identical on every reported number)

Placebo identity 224 / 224, replicate 0 = unforked 224 / 224 (G0a PASS). Deaths effect (replicate mean per seed, exposure included), share of seeds > 0 / mean: gift_wood 0.000 / −0.375; rot_wood 0.500 / +1.141; **rain 0.500 / +1.164** (Pass 11: −0.36); cold_snap 0.188 / +0.141; windfall 0.000 / −0.148; gift1 0.031 / +0.008. Dark-night effect (mean): rain +1.76, rot_wood +1.13, cold_snap +0.41, gift_wood −0.68. Wet burn: over the rain hours inside the burn window (11 per episode) the home store lost 34.99 units against 35.72 on the factual's dry night over the same hours. First fission: factual mean display day 18.6 (mostly before the fork); the first fission founded after the fork differs from the factual in 0.01–0.02 of episodes (7 episodes split after the fork). G0b (variance reduction) FAIL as in earlier passes (reported). **J8: PASS.**

### Part 6 — macro sensitivity (12.2; 120 days, rot_wood / gift_wood / spoil_food 60 at tick 29,400 on home, R = 2; `p12-6_macro/` frozen recorder)

Frozen recorder and the final re-recording (`p12-7_final_macro/`, `wm_final`) give the same table. Factual r0: 18 of 32 seeds split, 16 after the fork, 2 only before it.

| Comparison | Share differing (settlement counts or post-fork first-fission day) | Mean abs Δ post-fork day (n seeds where both split after the fork) | Mean abs Δ alive day 118 |
|---|---:|---:|---:|
| (a) factual r0 vs r1 (noise only) | 0.688 | 34.5 (14) | 18.4 |
| (b) rot_wood 60 vs factual, same replicate | 0.016 | 0.0 (35) | 0.70 |
| (b) gift_wood vs factual | 0.016 | 2.4 (36) | 0.69 |
| (b) spoil_food 60 at tick 29,400 on home (written explicitly into `p12/schedules/macro`) | 0.000 | 0.0 (36) | 0.03 |

**J11 (reported)**: interventions move the timing far less than noise does — the hazard draw is keyed with the replicate, so a noise fork redraws "when", while a small act of the god changes h by a few thousandths and rarely flips a draw.

### Part 7 — timing (J12; `probe_final`, tuning sets, back to back 15:11–15:49 KST)

No Godot, cargo or worldsim process of anyone else; foreign load: a "Sow and Siege.app" game build launching intermittently (another project), a Chrome helper (~93 % CPU) and the macOS storage indexer (~95 %); load averages 6.8 at the start, 63 at 15:20 (between the v16 and v18_nofission sets), 22 at 15:41. Median ms per agent-tick (10-day / 120-day): game_v16 0.00209 / 0.00274; game_v18_nofission 0.00198 / 0.00277; game_v18 0.00208 / 0.00317; bracketing game_v16 rerun after v18 (15:41–15:48) 0.00178 / 0.00243. Ratios v18 / v16: 10-day 0.996, 120-day 1.158 against the preceding v16 set, 1.305 against the following one, 1.226 against their mean; v18_nofission 0.948 / 1.011. **J12: PASS against the adjacent run and the bracket mean (≤ 1.3), borderline against the later bracket.**

### Part 7 — final runs on `1127455b` (review round 1 fixes)

Tuning (the timing runs, with twins): game_v18 and game_v18_nofission equal round 3 on every shared value (0 differences; the fixes do not touch these seeds). Held-out (`p12-7_final_heldout_*`): game_v18_nofission identical to Part 4; game_v18 10-day identical, 120-day 1,151 values differ (3 of 64 seeds: 63 and 77 diverge after a fission group was walking, 94 changes only the founded-inside member count). Final held-out J verdicts are Part 4's: J3 PASS (share 0.703, median 38, IQR 38, peak 0.111), J4 PASS (56/56), J5 FAIL (pooled child share 0.717, settlement-winter max 0.571, 4 over), J9 region-placement clause FAIL (3 of 255 regions; counted and explained), v18_nofission J6 FAIL (work_1500 0.213 vs v16 0.241; idle_1200 0.629 vs 0.617). h terciles after the tie fix: split rate 0.015 / 0.024 / 0.045. Gate on `1127455b`: green (`verification/p12-7/gate.txt`: debug 2,103 / release 2,103 passed, 0 failed; Pass 11 2,078).

### Part 7 — final reproduction (`probe_final`, commit `1127455b`, 15:49–17:27 KST, Pass 11's flags)

Against Pass 11's finals (`p11-7_final_*`), every run differs only in `schema_version` (11 → 12); all 28 determinism twins pass. Shared values: legacy 5,727; v8 13,990; v9 10,589 / 77,915; v10 10,781 / 78,498; v11 13,096 / 93,659; v12 13,096 / 93,671; v13 13,226 / 93,726; v14 13,228 / 93,727; v15 13,228 / 93,733; v16 13,868 / 94,339; v16_neutral 13,420 / 94,048; v17 16,226 / 97,206; v17_nofission 15,394 / 95,097; held-out v17 16,231 / 130,812; held-out v17_nofission 15,399 / 127,126 (10-day / 120-day). Code changed after Parts 5–6 (review round 1, `1127455b`); everything it affects was rerun: final tuning and held-out v18 runs, the pilot's v18 scenes, the forks and the macro run (`p12-7_final_*`); verdicts unchanged.

## Pass 13 — a split the village builds up to (pressure against a hidden threshold), a measure of how much of a world's future is its path (`game_v19_nofission`, `game_v19`)

All probe runs use Pass 12's final flags (eight clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30`, `--sample-every` 120, `--workers 4`, `--check-determinism`; legacy with its own flags). Binaries in `/Users/rexxa/work/p13bin/`. Scratch exploration of this pass (4.4) is disclosed where it happened.

### Part 0 — baseline reproduction (`probe_p0` = Pass 12's `probe_final`, byte-identical; 20:53–21:19 KST)

| run | reference | shared values | differences |
|---|---|---:|---:|
| `p13-0_v18_10day` / `_120day` | `p12-7_timing_v18_*` | 17,502 / 132,718 | 0 / 0 |
| `p13-0_v18n_10day` / `_120day` | `p12-7_timing_v18n_*` | 16,670 / 104,565 | 0 / 0 |
| `p13-0_v17_10day` / `_120day` | `p12-7_final_v17_*` | 17,502 / 107,146 | 0 / 0 |
| `p13-0_v16_10day` / `_120day` | `p12-7_final_v16_*` | 15,144 / 103,923 | 0 / 0 |
| `p13-0_v15_10day` / `_120day` | `p12-7_final_v15_*` | 14,504 / 103,317 | 0 / 0 |
| `p13-0_legacy` | `p12-7_final_legacy` | 6,463 | 0 |

Every determinism twin passes. Frozen pins `harness_revitalize_pass13_frozen.rs` (v18 and v18_nofission digests, seeds 0–3 at 1,440 and 40,320 ticks; literals of `game_v18(0)` and `V18_HAZARD`) pass on the base.

### Part 0.4 — the replicate measure on old worlds (`paths_p13.py`; worlds 901000–901031 × R = 4, noise fork at tick 1, 120 days; `docs/revitalize/probe/p13-0_paths/`)

Replicate 0 equals the unforked recording on worlds 901000 and 901001 (every array byte-identical). PS = path share (mean over days 60/90/118, 90 % bootstrap over worlds), PSD = path spread (sqrt(W) / S_j).

| Variable | game_v16 PS [90 %] / PSD | game_v17 PS [90 %] / PSD | game_v18 PS [90 %] / PSD | game_v17 - game_v16 PS [90 %] | game_v18 - game_v16 PS [90 %] | game_v18 - game_v17 PS [90 %] |
|---|---|---|---|---|---|---|
| alive | 0.001 [0.001, 0.002] / 0.034 | 0.003 [0.001, 0.005] / 0.052 | 0.306 [0.212, 0.437] / 0.633 | 0.001 [0.000, 0.004] | 0.305 [0.211, 0.436] | 0.304 [0.208, 0.435] |
| food_stock | 0.019 [0.014, 0.032] / 0.073 | 0.013 [0.009, 0.021] / 0.069 | 0.301 [0.210, 0.438] / 0.383 | -0.007 [-0.018, 0.000] | 0.281 [0.188, 0.415] | 0.288 [0.196, 0.421] |
| wood_stock | 0.662 [0.592, 0.757] / 0.071 | 0.063 [0.050, 0.087] / 0.054 | 0.488 [0.377, 0.621] / 0.182 | -0.599 [-0.686, -0.535] | -0.174 [-0.335, -0.024] | 0.425 [0.309, 0.555] |
| settlements_alive | — [—, —] / — | 0.007 [0.000, 0.021] / 0.090 | 0.474 [0.366, 0.606] / 0.764 | — [—, —] | — [—, —] | 0.467 [0.355, 0.598] |
| deadwood_total | 0.033 [0.024, 0.053] / 0.096 | 0.005 [0.003, 0.008] / 0.047 | 0.007 [0.004, 0.013] / 0.057 | -0.028 [-0.046, -0.019] | -0.026 [-0.043, -0.017] | 0.002 [0.000, 0.005] |
| food_on_bushes_total | 0.120 [0.089, 0.164] / 0.098 | 0.011 [0.008, 0.019] / 0.080 | 0.193 [0.116, 0.308] / 0.364 | -0.109 [-0.153, -0.077] | 0.074 [-0.020, 0.183] | 0.183 [0.103, 0.293] |
| adults_working | 0.385 [0.288, 0.525] / 0.143 | 0.175 [0.135, 0.243] / 0.122 | 0.396 [0.296, 0.541] / 0.183 | -0.210 [-0.337, -0.111] | 0.012 [-0.091, 0.116] | 0.222 [0.132, 0.342] |

UBS on replicate 0 (Pass 12's tool, reported only; mean of days 60/90/118, alive / food_stock / wood_stock): game_v16 0.394 / 0.261 / 0.082; game_v17 0.731 / 0.518 / 0.218; game_v18 0.995 / 0.597 / 0.269

Reading: in v16 and v17 the population at day 60–118 is fixed by the world (PS(alive) 0.001 and 0.003 — v17's day-29 split is a function of the state), while v18's daily die makes 0.31 of the between-run variance of `alive` and 0.47 of `settlements_alive` path. UBS on the same replicate-0 worlds ranks v17 (0.73) far above v16 (0.39) for `alive` although its path share is 0.003: the threshold effect Pass 12 suspected. The J7b anchor: PS(alive) v18 = 0.306 [0.212, 0.437].

### Part 0.5 — the open J5 cases (`docs/revitalize/probe/p13-0_j5/j5_diag.md`)

Inputs: Pass 12's final held-out run and the tuning timing run (pooled child share 0.717, reproduced), and `--agents` recordings of seeds 24, 39, 66, 77. 193 of the 195 dead children were born inside the winter window (child stage 4 days; births 1.06 per seed-day in winter windows vs 1.14 overall); per 100 at the winter's start children die at 11.6, adults 0.35. The four settlement-winters over 0.35 are all harsh winters with the settlement's own fire out at every death (40/40): two young daughters (39: founded day 105 autumn, 8/14, 13 births in the window; 77: founded day 71 autumn, 11/28, 13 births) and two camps that entered the winter with stock 0 and 57 after an autumn split took their adults (66: 12/26; 24: 9/25, no newborn deaths). Without the newborn deaths the four ratios would be 0.071 / 0.27 / 0.21 / 0.36 and the pooled stage shares 0.03 / 0.83 / 0.13 (adult share above J5's 0.50). Burn need inside the window 310–490 against a target of 208 and 51–69 units of fall within 18 tiles.

### Part 1 — the J6 remainder (`probe_p1`, tuning seeds 0–31 × 120 days, every twin passed; 21:19–21:44 KST; `docs/revitalize/probe/p13-1_j6/bisect.md`)

Gap in Pass 6 Y9 `work_1500`: game_v16 0.254 − game_v18_nofission 0.240 = 0.014 on these seeds. Activity at 15:00 (pooled): foraging 0.178 → 0.167, idle 0.592 → 0.606, everything else within 0.002. Share of the gap restored by one reversion: region bushes 0.118 (`abl18_no_region_bushes`, `abl18_no_regions`; foraging back to 0.181); region deadwood, regional wood, wet burn, area floor 0.000 (inert without fission or interventions); hunger weight 0.000; v16 exposure damage −0.60, no hardiness −0.67, v16 exposure −0.01 (noise of a 0.014 gap). No difference explains half → no J6 mechanism (DECISIONS).

### Part 2 — v19 mechanisms (commit `20fb22e5`)

Fission by accumulated pressure (7.1), floor in its own accumulator (7.2), probe schema 13, recorder schema 6, `game_v19` scenes and live game. No J5 / J6 mechanism (Part 0.5, Part 1). Start values: Exponential θ mean 1200, decay 10, reset on split; hazard 28 / 48 / 850 / 1050 / scarcity_days 2 / crowd 350 / scarcity 650 / ceiling 200; v18 gates.

Scratch exploration (4.4, disclosed; seeds 902000–902031, 120 days, R = 2, fork 28,800, rot_wood / spoil_food 60 / gift_food 60 at 29,400 on home; build of the Part 2 tree before the commit, identical rules):

- **A = start values.** Noise (r0 vs r1): post-fork day differs by > 3 days in 0.000 of seeds, mean |Δ day| 0.27 (n 15), mean |Δ alive 118| 0.22 — against Pass 12's v18 0.688 / 34.5 / 18.4. Interventions (same replicate): spoil_food earlier in 0.233 of the 30 pairs that split after the fork (later 0.000), gift_food later in 0.100 (earlier 0.067), rot_wood ±0.27 / 0.23; c-rule 0.000 / 0.047 / 0.047. J11 (a) passes, (b) and (c) fail. Cause, from the recordings: the spoil leaves the home store about 13 units short into spring (days 28–45: −13.5 → −9.7; gift +6 → +1), but with `scarcity_days` 2 the scarcity line is 2 × members × need ≈ 154 and day-28 stores are 107–358, so s = 0 in most worlds and the deficit never reaches h; the intervention lands at day 20.75, the eve of a winter in which P is frozen.
- **B** = A with `scarcity_days` 4, weights crowd 150 / scarcity 850, ceiling 300 (θ mean 1200). J3 (probe, same seeds): fission share 0.56, first-fission display day median 35, IQR 27.8, peak 0.17, ≥ 2 alive 0.56. Macro: noise > 3 d 0.031, mean |Δ day| 0.50 (n 18), |Δ alive| 0.41; spoil_food earlier 0.222 (later 0.000), gift_food later 0.250 (earlier 0.139), c-rule 0.078 / 0.047 / 0.031 (rot / spoil / gift) vs noise 0.031.
- **C** = A with `scarcity_days` 4, weights 0 / 1000, ceiling 300, θ mean 1800. J3: share 0.47, median 63, IQR 50, peak 0.13. Macro: noise > 3 d 0.031, mean |Δ day| 2.07 (n 15), |Δ alive| 1.13; spoil_food earlier **0.533** (later 0.100; mean shift −2.0 d), gift_food later **0.400** (earlier 0.200; mean +3.2 d), rot_wood earlier 0.467; c-rule 0.031 / 0.062 / 0.062 vs noise 0.031. Only 15 of 32 seeds split after the fork, so (c)'s share over all seed-replicates stays small.
- **D** = C with θ mean 900, size 24 / 44, `min_members` 28. J3: share 0.72, median 34, IQR 30, peak 0.17, ≥ 2 alive 0.72. Macro: noise > 3 d 0.094, mean |Δ day| 2.09 (n 22), |Δ alive| 1.00, noise c-rule 0.125; spoil_food earlier 0.533 (later 0.022), gift_food later 0.289 (earlier 0.156), rot_wood earlier 0.644; c-rule 0.078 / 0.141 / 0.062.
- **E** = C with θ mean 1200. J3: share 0.47, median 39, IQR 27. Macro: noise 0.000 / 0.40 d / 0.38; spoil_food earlier 0.567, gift_food later 0.267; c-rule 0.031 / 0.062 / 0.016 vs noise 0.031.
- Reading: once the hazard reads only food against a 4-day line, spoiling food moves the split earlier in about half of the pairs and almost never later, but gifting food rarely moves it later: the gift (+30 at day 22) is gathered away before winter (+6 at day 28, +1 at day 45), while the spoil's deficit survives the winter (−13 at day 28). Round 1 takes D's structure with θ mean 1200 between D and E, and the child exposure damage 7 for J5.

### Part 2 — 4.6 guard (`probe_p2` = commit `20fb22e5`, 22:11–22:37 KST)

`p13-2_guard_v18_*`, `_v18n_*`, `_v17_*`, `_v16_*` (tuning sets) equal Part 0's runs on every shared value (17,502 / 132,718 / 16,670 / 104,565 / 17,502 / 107,146 / 15,144 / 103,923); the only difference is `schema_version` 12 → 13 (plus the two new row keys and the `pass13` block). All determinism twins pass.

### Part 3 — tuning (tuning seeds; `evaluate_pass13.py`, game_v16 reference `p13-0_v16_*`; per-round J11 check on scratch seeds 901100–901131)

| Round | Commit | Knobs (changes) | J3 (v19) | J4 | J5 (v19) | J6 (v19n) | J11 check (901100–901131) |
|---|---|---|---|---|---|---|---|
| 1 | `b08461cd` | scarcity_days 2 → 4, weights crowd/scarcity 350/650 → 0/1000, ceiling 200 → 300, size 28/48 → 24/44, `min_members` 32 → 28, exposure child 8 → 7 (θ mean 1200, decay 10) | PASS: share 0.656, median 62, IQR 55 (31–86), peak 0.19, ≥ 2 alive 0.656, cancellations 0/32; first-fission day vs camp θ r = 0.89, vs early scarcity r = 0.30 | PASS 23/23 (31 daughters, none dissolved) | FAIL: seed-winters with a death 0.109 (< 0.15), settlement-winter max 0.467 (seed 24, daughter 7/15, day 106); child share 0.54, adult 0.46 | FAIL: work_1500 0.213 (v16 0.254), idle_1200 0.627; other clauses PASS | (a) PASS: > 3 d 0.000, mean abs Δ 0.56 d (n 16), Δ alive 0.56; (b) FAIL: spoil earlier 0.344 (later 0.156), gift later 0.387 (earlier 0.161); (c) FAIL: noise 0.031, rot_wood 0.172, spoil 0.078, gift 0.062 |
| 2 | `f587a4a9` | θ mean 1200 → 1000, decay 10 → 50, exposure child 7 → 8 | PASS: share 0.594, median 33, IQR 60.5 (30–90.5), peak 0.16, ≥ 2 alive 0.594, cancellations 0/27; r(day, θ) 0.79, r(day, early s) 0.09 | PASS | FAIL: child share 0.74 (> 0.70), settlement-winter max 0.389 (seed 24, daughter 14/36, day 106); share with a death passes | FAIL: work_1500 0.240 (v16 0.254), idle passes | (a) PASS but noise grows: > 3 d 0.031, mean abs Δ 5.08 d (n 12), Δ alive 2.56; (b) FAIL: spoil earlier 0.333 (later 0.208), gift later 0.273; (c) FAIL: noise 0.062, rot 0.094, spoil 0.062, gift 0.047 |
| 3 | `9f0d38ca` | θ mean 1000 → 1200, decay 50 → 10 (round 1's), exposure child 8 → 7.5 | PASS: share 0.656, median 62, IQR 55 (31–86), peak 0.19, ≥ 2 alive 0.656, cancellations 0/32; r(day, θ) 0.89, r(day, early s) 0.30 | PASS 1.00 (31 daughters, none dissolved) | FAIL: seed-winters with a death 0.133 (< 0.15), settlement-winter max 0.533 (seed 24, daughter 8/15, day 106); child share 0.66, adult 0.34 PASS | FAIL only work_1500 0.248 (v16 0.254, bound 0.25); idle PASS | (a) PASS: > 3 d 0.000, mean abs Δ 0.63 d (n 16), Δ alive 0.44; (b) FAIL: spoil earlier 0.313 (later 0.188), gift later 0.419 (earlier 0.226); (c) FAIL: noise 0.031, rot 0.156, spoil 0.062, gift 0.047 |

**Freeze** (after three of three judged rounds): commit `9f0d38ca` (round 3: J1–J4 and J9 pass; J5 misses on two clauses; J6 misses `work_1500` by 0.002; J11 (a) passes, (b) and (c) fail). Binaries `p13bin/probe_frozen` (sha256 `9e8240bb…`), `p13bin/wm_frozen` (sha256 `1844ebc3…`). Strikes (0.3 rule 7): **J5** — three rounds (child damage 7 / 8 / 7.5: too few seed-winters with a death / child share 0.74 / 0.133 with a death; seed 24's harsh day-106 winter over 0.35 every round). Hypothesis: the floor of H3 (≥ 0.15 of seed-winters with a death) and the child-share ceiling are both moved by the same newborn deaths (Part 0.5), so child damage alone cannot satisfy both; seed 24 is a daughter darkened by a harsh winter whatever the knobs. **J6** — three rounds (0.213 / 0.240 / 0.248, bound 0.25): Part 1's residual, a mechanism question. **J11** — three rounds: noise no longer moves the split (mean |Δ| 0.56–0.63 d at the start values, 5.1 d with decay 50), but a spoil or gift of 60 food at day 20.75 moves the post-fork split in its sign in only 0.31–0.42 of the pairs. Hypothesis: the act lands on the eve of a winter in which P is frozen, the gift is gathered away before the spring, and a 13-unit spring deficit changes the integrated pressure by about a day — a scarcity line that reacts more (or an act in spring) would be needed, outside the knob set.

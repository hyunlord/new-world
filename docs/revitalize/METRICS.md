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

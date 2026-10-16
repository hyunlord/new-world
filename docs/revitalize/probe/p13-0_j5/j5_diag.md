# J5 diagnosis (Pass 13 Part 0 item 5)

Inputs: held-out `.harness/evidence/revitalize/probe/p12-7_final_heldout_v18_120day` + tuning `.harness/evidence/revitalize/probe/p12-7_timing_v18_120day` (the pair Pass 12's final J5 pooled; the tuning run equals round 3 on every shared value); recordings `/Users/rexxa/work/worldsim-wm-data/p13/j5diag` (game_v18, Pass 12 final code, `--agents`, a row every 60 ticks). Tool: `tools/revitalize/j5_diag_p13.py`. Stage order everywhere: child/adult/elder.

## Reproduction (evaluate_pass12 functions imported)

- 613 settlement-winters, 4 over 0.35, max 0.571; the four ratios equal 8/14, 12/26, 11/28, 9/25: **True**.
- `stage_pool`: deaths 195/69/8 = 272, child share **0.717** (= Pass 12's final 0.717; METRICS Part 4's 0.725 came from the Part 4 held-out run, before the p12-7 rerun changed 3 seeds). Recount from the death census (`stage`, cause exposure, tick in the window): 195/69/8, equal: True.
- Recordings vs probe: members at every case's start equal; exposure-death agents present in the recording 40/40.

## The four settlement-winters over 0.35 (all harsh)

Dark nights: world = probe rule (02:00 sample, world flag = every fire lit); own = same rule on this settlement's fire; any = nights with >= 1 unlit burn hour; h = unlit burn hours of 91 (13 per night, rows 19:00..07:00). Deadwood near = deadwood spots within Chebyshev 18 of the hearth (`wood_radius`, a gatherer's search radius); r9 = `SETTLED_REGION_RADIUS`. Snow cap 1 falls at the first winter pulse, so per-spot wood at the start is <= 1.

| seed | settl. | founded (season), age d | dark world/own/any/h | deadwood /spot: before snow (D-1 23:00), start r18, start r9 | stock start (target) | stock 18:00 D..D+6 | burn need / fallen r18 / into stock (window) |
|---|---|---|---|---|---|---|---|
| 39 (d106) | 3 daughter of 2, region 1 | d105 (autumn), 1.23 | 4/4/5/54 | 2.6, 0.0 (15 spots), 0.0 | 150 (208) | 150, 103, 55, 4, 3, 3, 4 | 386 / 51 / 23 |
| 66 (d106) | 1 camp | d1 (spring), 105.42 | 7/7/7/90 | 5.0, 0.2632 (19 spots), no spots | 0 (208) | 0, 2, 2, 1, 1, 0, 2 | 310 / 65 / 13 |
| 77 (d78) | 3 daughter of 2, region 1 | d71 (autumn), 7.38 | 4/4/4/44 | 4.6316, 0.0 (19 spots), 0.0 | 206 (208) | 206, 130, 81, 35, 4, 3, 7 | 436 / 66 / 25 |
| 24 (d106) | 1 camp | d1 (spring), 105.42 | 6/6/7/81 | 4.8077, 0.0385 (26 spots), 0.0 | 57 (208) | 57, 3, 0, 1, 2, 1, 1 | 490 / 69 / 11 |

| seed | members c/a/e at start | adults at 18:00 D-7..D | births in window | deaths c/a/e | ratio | deaths born in window | ratio without them | own fire unlit at death | adult victims' age d | cold share of burn hours c/a/e |
|---|---|---|---|---|---|---|---|---|---|---|
| 39 | 2/11/1 | -, -, -, -, -, -, 11, 11 | 13 | 6/2/0 | 8/14 = 0.571 | 7 | 0.071 | 8/8 | 4.1, 24.9 | 0.45/0.41/0.46 |
| 66 | 8/7/11 | 15, 2, 2, 2, 2, 3, 5, 7 | 11 | 7/4/1 | 12/26 = 0.462 | 5 | 0.269 | 12/12 | 4.3, 6.3, 9.8, 11.3 | 0.61/0.63/0.62 |
| 77 | 8/20/0 | 13, 13, 13, 14, 14, 16, 18, 20 | 13 | 5/6/0 | 11/28 = 0.393 | 5 | 0.214 | 11/11 | 5.8, 7.8, 12.3, 38.8, 39.7, 43.3 | 0.31/0.36/0.48 |
| 24 | 4/17/4 | 8, 8, 8, 10, 12, 14, 16, 17 | 0 | 0/8/1 | 9/25 = 0.360 | 0 | 0.360 | 9/9 | 4.6, 8.6, 11.5, 12.7, 14.3, 23.7, 28.1, 35.4 | 0.38/0.55/0.48 |

Winter wood lost (`winter_wood_lost_total` row deltas, world level; settled = per-seed run total): seed 39: window 0, winter season 7, run 147, settled run 46; seed 66: window 0, winter season 4, run 223, settled run 14; seed 77: window 0, winter season 5, run 287, settled run 47; seed 24: window 24, winter season 33, run 299, settled run 46. Schema 12 has no per-winter or per-settlement split of `winter_wood_lost_settled`.

Same world-winter, every settlement (stock at start, members, exposure deaths, unlit burn hours):
- seed 39 d106: 1: 209, 1/33, 43 h; 2: 204, 4/23, 44 h; 3: 150, 8/14, 54 h
- seed 66 d106: 1: 0, 12/26, 90 h; 2: 200, 4/37, 33 h; 3: 201, 0/25, 24 h
- seed 77 d78: 1: 206, 0/37, 46 h; 2: 194, 5/40, 46 h; 3: 206, 11/28, 44 h
- seed 24 d106: 1: 57, 9/25, 81 h; 2: 211, 13/50, 54 h

## Child share (pooled held-out + tuning)

| set | deaths c/a/e | share c/a/e | stage counts at winter starts c/a/e (share) | deaths per 100 at start c/a/e |
|---|---|---|---|---|
| all seed-winters | 195/69/8 | 0.717/0.254/0.029 | 1679/19982/3654 (0.066/0.789/0.144) | 11.61/0.35/0.22 |
| mild winters | 0/0/0 | -/-/- | 616/7499/1312 (0.065/0.795/0.139) | 0.00/0.00/0.00 |
| normal winters | 40/5/0 | 0.889/0.111/0.000 | 676/7925/1464 (0.067/0.787/0.145) | 5.92/0.06/0.00 |
| harsh winters | 155/64/8 | 0.683/0.282/0.035 | 387/4558/878 (0.067/0.783/0.151) | 40.05/1.40/0.91 |
| deaths in camps | 93/30/2 | 0.744/0.240/0.016 | members at starts 13770 | 0.91 per 100 members (all stages) |
| deaths in daughters | 102/39/6 | 0.694/0.265/0.041 | members at starts 7906 | 1.86 per 100 members (all stages) |

- Seed-winters with a death only: counts 386/3657/777, deaths per 100 50.52/1.89/1.03. Child share expected from composition alone: 0.110.
- The child stage lasts 4 days (`child_ticks` 5,760). Dead children's age: median 2.75 d, max 3.96; **193 of 195 were born inside the winter window** (adults: 19; adult victims under 7 days old: 23). Children at risk = at start + born in window = 1679 + 2663: 4.49 deaths per 100 (adults 0.35).
- Births barely slow in winter: 1.06 per seed-day inside the winter windows vs 1.14 over the whole run.
- Arithmetic counterfactual without the deaths born inside the window (not a simulation): shares 0.033/0.833/0.133.
- All 33 settlement-winters of the four recorded seeds: cold share of burn hours 0.239/0.156/0.223, deaths 38/31/7, deaths per cold hour 0.0153/0.0027/0.0018. Damage per cold hour (game_v18) child 8, adult 5, elder 5 (x hardiness, x hunger up to 1.5); max hp 100 for every stage; healing 0.05/tick (3/hour) after a 2-hour block and only with hunger and thirst below the safe ceiling (<= ~27 hp between two nights). One fully dark burn window (13 h) costs a child 104 hp, an adult 65.

## Conclusion

The child share is a newborn effect, not a composition or exposure-time effect: 193 of 195 dead children were born inside the winter window (the child stage lasts 4 days), births barely slow in winter, and within a settlement children are cold for about the same share of burn hours as adults; but at 100 hp and 8 hp per cold hour one fully dark 13-hour night (104 hp) kills a newborn, while an adult loses 65 hp, heals at most ~27 by the next evening and dies only in a second consecutive dark night. All 40 of 40 deaths in the four cases came with the settlement's own fire out, in harsh winters whose burn need inside the window (310-490) is 1.5-2.4x the firewood target (208) while the deadwood falling within 18 tiles (51-69) covers 13-21 % of it; wood per spot at the winter start is <= 0.26 in every case only because the snow cap (1) buries the 2.6-5.0 per spot lying there the evening before, so wood per spot does not separate the cases. Two mechanisms make the four cases: daughters 39 and 77 entered with stock near the target (150, 206) and about as many dark hours as their sibling settlements, but small or young rosters received 13 and 13 births in the window (ratios without the deaths born inside it: 0.071, 0.214); camps 66 and 24 entered with stock 0 and 57 because they were short of adults during the autumn build-up (66: an autumn fission's group took 12 adults and left 2 adults among 23-24 members; 24: 8 adults among 24-28 members) and went dark 90 and 81 of 91 burn hours, and 24's deaths are adults and an elder. A single v19 flag aimed at newborns (no births while a settlement's wood cannot cover the winter, or child damage <= 7 so one dark night is not lethal) plausibly fixes the child share and cases 39 and 77, but not 24 (0.36 with no newborn deaths) and 66 only to 0.27, and the arithmetic counterfactual without the newborn deaths moves the adult share to 0.83, above J5's 0.50 bound; wood placement cannot close a burn deficit that also darkens settlements at the target, and the camp cases point at adult labour in autumn (a fission that leaves adults in the parent), a second mechanism.

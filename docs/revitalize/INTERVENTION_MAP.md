# WorldSim — intervention map of `game_v19` (Pass 14)

When and how hard must the god push to change the shape of a world — when villages split, how many there are, how many people live — by more than noise does? This map answers it for the frozen `game_v19` (Pass 13, `2ba871c0`; no rule changed in Pass 14) from two experiments on the DGX:

- **Stage A (screen):** seeds 12000–12031, 20 arms (16 food arms, 4 wood and weather arms), R = 2 noise replicates forked at display day 11, 06:00 (tick 14,280), 120 days, 992 episodes. `docs/revitalize/probe/p14-1_stageA/dose_map.md`.
- **Stage B (confirmation, pre-registered):** fresh seeds 14000–14063, the best spoil arm and the best gift arm of Stage A plus their eve-of-winter controls, same design. `docs/revitalize/probe/p14-2_prereg/PREREG.md`, `docs/revitalize/probe/p14-3_stageB/dose_map.md`.

Every number is an arm-vs-factual pair of the same world and the same noise replicate; the noise baseline is the factual's two replicates against each other on the same seeds after the same cast tick. "Right direction" = the first departure of a village group after the cast comes earlier (spoil, rot) or later (gift) than in the factual; ties count against. **ENR** (effect-to-noise ratio) = mean |Δday| of the arm / max(0.5, mean |Δday| of the noise) over the pairs where both runs depart after the cast. Intervals are 90 % seed-bootstrap intervals.

## 1. Why a food act moves the split at all

A village's fission pressure P grows at 06:00 of each eligible day (spring–autumn, ≥ 28 members, past its cooldowns) by h = 300 × g × s / 10⁶, where g grows with size (0 at 24 members, 1 at 44) and s is the food shortfall below the 4-day line (members × 2.4 × 4; about 300 for 32 members): s = 1000 − 1000 × food / line, clamped to [0, 1000]. P decays by 10 a day when h = 0; the village splits when P reaches its hidden θ (mean 1,200). So the god's food acts reach the split only through s: a spoil helps only if it pushes the store below the line, a gift only if the store was below the line. A cast at a check tick is applied before that tick's systems run, so it changes that same morning's s.

Three cast times were tried (Part 0, `timeline_p14.py`). They are **states of the village's store, not a time order** (review round 1, L1): **T_low** — 06:00 of the first eligible home check with a shortfall (s > 0) at least 3 days before the factual's first departure; in practice day 12 (the first eligible check: 14 of 28 Stage A seeds, 26 of 51 Stage B) or day 29 (the first spring check after the first winter: 12 / 20), rarely later (2 / 5, after a first departure); **T_high** — the eligible check with s = 0 and the fullest store before the first departure (food/line 1.25–1.84, median 1.56), on any day from 14 to 77 in Stage A, often after T_low; **T_eve** — day 21, 18:00, the eve of the first winter (Pass 13's cast), the same for every world, which is *before* a day-29 T_low.

## 2. Dose–response at T_low (Stage A, 28 seeds × 2 replicates)

| kind | dose | right-direction share [90 %] | mean \|Δday\| [90 %] | ENR [90 %] | settlement count differs at d118 (noise 0.036) | alive d118 Δ |
|---|---:|---|---|---|---:|---:|
| spoil_food | 60 | 0.446 [0.304, 0.589] | 6.5 [3.0, 10.3] | 2.38 [0.92, 13.30] | 0.036 | +1.7 |
| spoil_food | 120 | 0.661 [0.535, 0.804] | 12.5 [5.7, 20.4] | 4.58 [1.77, 24.85] | 0.125 | +7.2 |
| spoil_food | 240 | 0.786 [0.643, 0.929] | 22.4 [14.5, 30.7] | 8.19 [4.18, 39.43] | 0.250 | +12.4 |
| gift_food | 60 | 0.429 [0.304, 0.554] | 6.2 [3.3, 9.3] | 2.28 [1.02, 11.00] | 0.089 | −2.6 |
| gift_food | 120 | 0.518 [0.393, 0.661] | 10.1 [5.7, 14.4] | 3.69 [1.67, 19.24] | 0.125 | −3.6 |
| gift_food | 240 | 0.518 [0.375, 0.661] | 11.7 [7.5, 15.7] | 4.26 [1.90, 22.61] | 0.089 | −3.4 |

Noise at these cast ticks: 19 pairs where both replicates depart, mean |Δday| 2.74, share > 3 days 0.105; the day-118 settlement count differs between replicates in 0.036 of seeds.

- **Spoiling food is dose-dependent and strong.** 60 units barely beat noise (its ENR interval reaches below 1); 120 moves the split earlier in two pairs of three by 12 days; 240 moves it earlier in four of five by 22 days and changes the number of villages on day 118 in a quarter of worlds (seven times the noise share). No inversion (K4).
- **Giving food saturates at 120.** The share of later splits stays at 0.52 from 120 to 240; the size of the shift grows only from 10 to 12 days. A gift can only delay a split the factual would have made: 11 of 32 factual worlds never depart within 120 days, and on those a gift can only tie (ties 0.43–0.52 of gift pairs).
- **The effect depends on which T_low a world has** (Stage A / Stage B). When T_low is day 12, a spoil of 240 is right in 0.86 / 0.94 of pairs and moves the day by 16 / 18 days; when T_low is day 29 it is right in 0.83 / 0.75 and moves it by 33 / 43 days (a split pulled from late in the year into the first spring). For gifts the day-29 casts carry the effect (right 0.67 / 0.68, 22 / 15 days) and the day-12 casts barely act (0.46 / 0.23, 5 / 1 days). The worlds whose T_low comes after a first departure (days 40–59) show almost no effect.
- **Direction of the world's shape:** an earlier split comes with more people (`sp240L` +12.4 alive at day 118 in Stage A, +7.4 in Stage B) and more old-age deaths by day 120 (+9.1 / +9.7) — read as two villages growing faster than one, not measured further here; a later split comes with fewer (`gf240E` −5.1 / −2.7 alive).

## 3. Timing contrast at dose 240 (Stage A)

| kind | T_low (28 seeds) | T_high (26 seeds) | T_eve (32 seeds) | T_low − T_eve on the 28 shared seeds [90 %] |
|---|---|---|---|---|
| spoil_food | 0.786 / 22.4 d / ENR 8.19 | 0.558 / 4.9 d / ENR 1.46 | 0.516 / 7.3 d / ENR 2.66 | +0.196 [+0.071, +0.321] |
| gift_food | 0.518 / 11.7 d / ENR 4.26 | 0.288 / 2.7 d / ENR 0.79 | 0.578 / 13.3 d / ENR 4.87 | −0.143 [−0.250, −0.054] |

(right-direction share / mean |Δday| / ENR)

- **A spoil belongs at the shortfall, not on the eve of winter.** At T_low the same 240 units are 1.5 times as often right and three times as large as on the eve of winter (Stage B repeats it: +0.206 [+0.118, +0.285] on 51 paired seeds). Reading (not measured further): on the eve the store is full from winter preparation and P is frozen through winter, so the village rebuilds its store before the first spring check.
- **A gift belongs on the eve of winter.** Stage B repeats it with a larger gap (−0.343 [−0.461, −0.225]); on the worlds whose T_low is day 12 the eve gift is right in 0.85 of pairs, the day-12 gift in 0.23. Reading (not measured here): a gift that carries the village through winter into a spring with its store above the line keeps s at 0 for the first spring checks, where most splits happen; a gift at an early shortfall is absorbed before the village is large enough for the hazard (g) to matter — Pass 13's planner suggested a gifted village gathers less, which this pass did not measure.
- **A full store makes both acts nearly powerless (T_high).** With food at ~1.5 × the line, 240 spoiled leaves s near 0 and 240 given changes nothing the hazard reads. The gift at T_high is within noise (ENR 0.79).

## 4. One dose or three (Stage A, T_low, total 240)

| kind | one dose of 240 | 80 at T_low, +2 d, +4 d |
|---|---|---|
| spoil_food | 0.786 / 22.4 d / ENR 8.19 / d118 count differs 0.250 | 0.768 / 17.4 d / ENR 6.37 / 0.179 |
| gift_food | 0.518 / 11.7 d / ENR 4.26 / 0.089 | 0.482 / 10.2 d / ENR 3.72 / 0.089 |

Splitting the dose does not help. Reading: the hazard integrates s day by day, and one large drop below the line drives more pressure before the village refills its store than three smaller ones.

## 5. Wood and weather (Stage A, deaths)

| arm | right share (split) | ENR (split) | deaths within 35 d of the cast, Δ [90 %] (noise \|Δ\|) | deaths to day 120, Δ [90 %] (noise \|Δ\|) |
|---|---|---|---|---|
| rot_wood 120 at T_eve (day 21 18:00) | 0.438 | 1.68 [0.93, 6.48] | **+1.80 [1.28, 2.39]**, of which exposure +2.33 (noise signed +0.16, \|Δ\| 0.28; arm − noise **+1.64 [1.08, 2.28]**) | +2.09 [1.50, 2.70] (noise +0.59 / 1.91; arm − noise **+1.50 [0.66, 2.33]**) |
| rot_wood 120 at T_low | 0.089 | 0.31 | +0.04 [0.00, 0.09] — applied 867 of 6,720 units: the wood store is nearly empty in spring | +0.04 [−0.25, 0.34] (arm − noise −0.64 [−1.62, 0.41]) |
| rain 13 h at day 24 18:00 | — | 1.22 [0.72, 4.22] | +0.59 [0.31, 0.89], exposure +0.92 (arm − noise **+0.47 [0.14, 0.83]**) | +0.52 [0.06, 0.94] (arm − noise −0.08 [−0.84, 0.70]) |
| cold snap 2 nights from day 25 18:00 | — | 1.14 [0.59, 3.06] | +0.20 [0.09, 0.34] (arm − noise +0.08 [−0.22, 0.34]) | +0.11 [−0.19, 0.41] (arm − noise −0.48 [−1.30, 0.31]) |

Rotting the wood on the eve of winter is the only one of these acts that kills clearly beyond noise, in the first winter and still by day 120 (about 1.5–1.8 more deaths, mostly exposure); it moves the split only weakly (ENR 1.68, interval reaching below 1). Rain adds about half a death in its winter, no longer distinguishable by day 120; a two-night cold snap adds +0.2 deaths, not distinguishable from the noise's own signed difference. None of them changes the settlement count beyond noise (day-118 differs 0.000–0.078 vs noise 0.031–0.036). ("arm − noise" = mean signed Δ of the arm minus the noise's signed r1 − r0 mean on the same seeds, seed bootstrap.)

## 6. Stage B — confirmation on 64 fresh worlds (pre-registered)

Pre-registered before any Stage B world was simulated (PREREG commit 12:23:27 KST; first Stage B file 12:23:59). 51 of 64 fresh worlds have a T_low; 588 episodes. Noise: mean |Δday| 2.77 (39 pairs, L seeds) and 2.07 (42 pairs, E seeds); day-118 settlement count differs between replicates in 0.078 / 0.062 of seeds.

| arm | right-direction share [90 %] | mean \|Δday\| [90 %] | ENR [90 %] | d118 settlement count differs (noise) | alive d118 Δ [90 %] | K2 clauses (≥ 0.75 / ≥ 2.0 / ≥ noise + 0.15) |
|---|---|---|---|---|---|---|
| spoil_food 240 at T_low (`sp240L`) | **0.794** [0.696, 0.873] | **26.0** [20.0, 32.5] | **9.40** [5.51, 22.61] | 0.206 (0.078) | +7.4 [4.6, 10.4] | PASS / PASS / **FAIL** (0.206 < 0.228) |
| gift_food 240 at T_low (`gf240L`) | 0.382 [0.284, 0.490] | 5.4 [3.1, 7.9] | 1.95 [1.00, 4.57] | 0.078 (0.078) | −2.4 [−4.1, −1.0] | FAIL / FAIL / FAIL |
| spoil_food 240 at T_eve (`sp240E`, control) | 0.469 [0.375, 0.570] | 7.9 [5.1, 11.1] | 3.81 [2.13, 8.81] | 0.094 (0.062) | +2.3 [0.7, 4.2] | (control) |
| gift_food 240 at T_eve (`gf240E`, control) | 0.594 [0.500, 0.688] | 8.1 [5.7, 10.6] | 3.89 [2.15, 9.89] | 0.070 (0.062) | −2.7 [−4.5, −1.2] | (control) |

**K2 fails.** The pre-registered spoil confirms everything about the timing of the split — it comes earlier in four of five pairs, by 26 days on average, nine times the noise — but the number of villages on day 118 differs from the factual in only 0.206 of pairs, 0.022 short of the noise share plus 0.15 (on days 60 and 90 the shares are 0.324 and 0.255 against noise 0.020 and 0.000: an earlier split is mostly a split that also happens later in the factual, so by day 118 the two worlds often have the same number of villages again). The pre-registered gift (at T_low) does not replicate (0.518 → 0.382, ENR 4.26 → 1.95): Stage A's tie with `gf120L` made it the gift arm, but the gift's right time is the eve of winter, where the control replicates (0.578 → 0.594, ENR 4.87 → 3.89).

## 7. What does **not** change a world's shape beyond noise

Judged against the noise of the same worlds (ENR interval reaching below 1, or a right-direction share at or below the noise-level ties, or a death effect not distinguishable from the noise's signed difference):

- **Any act on a full store (T_high).** spoil 240: ENR 1.46 [0.67, 6.94]; gift 240: ENR 0.79 [0.23, 3.93], right share 0.29. The hazard reads only the shortfall below the 4-day line, and a store at 1.5 × the line has none to create or remove.
- **Small food acts (60 units)** at any time tested: ENR 2.4 / 2.3 with intervals reaching below 1; the settlement count is unchanged beyond noise. Pass 13's 60-unit eve casts were in this regime.
- **A gift at an early shortfall (day-12 T_low):** right in 0.23–0.46 of pairs, shifts of 1–5 days. Overall the T_low gift failed confirmation (Stage B ENR 1.95 [1.00, 4.57]).
- **rot_wood outside winter (T_low):** nothing to rot (867 of 6,720 units applied); ENR 0.31.
- **A 2-night cold snap and 13 hours of rain** on the split (ENR 1.14 / 1.22, intervals reaching below 1) and on the settlement count; the cold snap also on deaths (arm − noise +0.08 [−0.22, 0.34]); rain's half death in its winter is beyond noise but gone by day 120.
- **No act changes the number of villages on day 118 by 0.15 beyond noise in confirmation.** The largest, `sp240L`, reaches noise + 0.128 (Stage B) and noise + 0.214 (Stage A); everything else is within noise + 0.11.

What does change a world beyond noise: **spoiling 120–240 units at a shortfall** (the split day, the population and old-age deaths), **giving 240 units on the eve of winter** (the split day and the population, about half as strongly), and **rotting 120 units of wood on the eve of winter** (exposure deaths in the first winter).

## 8. Recommended intervention schedule for dataset v2

Per world: the factual and **four acts**, each with **R = 2** noise replicates forked at tick 14,280 (display day 11, 06:00), 120 days, rows every 60 ticks — 10 episodes per world (Stage A and B used the same design). One act per episode (no combinations: their interaction is not measured here).

| act | kind / amount | time rule | expected ENR on the split day (90 %) | expected right-direction share | other expected effect | evidence |
|---|---|---|---|---|---|---|
| strong spoil | spoil_food 240 | T_low | **≈ 9** [5.5, 22.6] | ≈ 0.79 | alive d118 +7 to +12; settlement count differs in ≈ 0.2–0.25 of worlds (noise ≈ 0.04–0.08) | Stage A and pre-registered Stage B |
| half spoil | spoil_food 120 | T_low | ≈ 4.6 [1.8, 24.9] | ≈ 0.66 | alive d118 ≈ +7; the dose gradient | Stage A only |
| eve gift | gift_food 240 | T_eve (tick 29,400) | ≈ 3.9–4.9 [2.2, 9.9] | ≈ 0.58–0.59 | alive d118 −3 to −5 | Stage A and Stage B control |
| eve rot | rot_wood 120 | T_eve (tick 29,400) | ≈ 1.7 [0.9, 6.5] (weak) | ≈ 0.44 | **+1.6 [1.1, 2.3] exposure-led deaths in the first winter beyond noise** | Stage A only |

- **Time rule.** T_low is computed per world from its own factual run with `tools/revitalize/timeline_p14.py` (a `worldsim_probe` run of the factual with `--sample-every 60`; the probe's `pass12.fission_checks` holds members and food of every eligible check). A world without a T_low (about 0.12–0.20 of worlds: they never reach 28 members by an eligible check, or have no shortfall) gets the T_eve acts only. T_eve is the fixed tick 29,400.
- **Label the time as a store state, not a calendar date.** The same "T_low" is day 12 in half the worlds and day 29 in most of the rest, and the effect differs (spoil: 16–18 vs 33–43 days of shift); a learned model should see the act's tick and the store at that tick.
- **Leave out** the T_high casts, 60-unit acts, gifts at T_low, rot_wood outside winter, the 2-night cold snap and the 13-hour rain as *treatments*: their effects are within noise and would mostly teach the model noise. If dataset v2 wants a placebo to calibrate a model's null response, one 60-unit act (or the cold snap) per world is the cheapest choice, labelled as such.
- **What a model can learn from this schedule.** The split day: yes — the strong spoil moves it by ≈ 26 days against a noise of ≈ 2.8 days, the eve gift by ≈ 8–13 days against ≈ 2–3. Population at day 118: yes for the strong spoil (+7 to +12 against a noise |Δ| ≈ 1.3–1.6). The number of villages at day 118: only weakly — even the strongest act changes it in about one world in five. Deaths: only the eve rot moves them beyond noise, by under two deaths per world.

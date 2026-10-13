# Daughter diagnosis (Pass 12 Part 0, PASS12_GOAL.md 5.3)

Why do Pass 11's daughter villages starve, dry out or freeze? Counts from every `game_v17` 120-day world of Pass 11's
final probe runs, tuning seeds 0–31 (`p11-7_final_v17_120day`) and held-out seeds 32–95
(`p11-7_final_heldout_v17_120day`). The final binary reproduces the frozen round-3 and held-out runs on every shared key
(Pass 11 Part 7), so these are the same worlds as R3 and Part 4. The probe has no per-settlement stock or fire history,
so the same 96 seeds were recorded with Pass 11's final recorder `p11bin/wm_final` (schema 5, hourly
`settlements.npy`) into `/Users/rexxa/work/worldsim-wm-data/p12/diag/game_v17/`. A daughter is matched to its region by
`fission_log` site = `wild_regions` centre.

Tool: `tools/revitalize/daughter_diag.py`. Per-death and per-winter rows: `docs/revitalize/probe/p12-0_daughters/`
(`daughter_diag.json`, `DAUGHTER_DIAGNOSIS_TABLES.md`).

## The four questions

**(a) Starvation deaths in daughters with a bush shortfall: 20 of 20.** Every starvation death in a daughter (20, in 8
worlds; 18 in winter) is in a daughter whose region placed fewer bushes than requested — and 17 of the 20 are in
daughters whose region placed **no bush at all** (placed/requested 0/7 to 0/11). The daughter's store was empty (minimum
0 in the 72 hours before every death). No daughter founded at a full region failed within 28 days: of the 119
daughters, 50 had full regions and all 50 lived 28 days; of the 69 with a shortfall, 47 lived, 20 did not (2 censored by
the end of the run).

**(b) Dehydration: 38 deaths (14 worlds; 29 in winter).** The region centre's nearest water was 10–28 tiles away
(median 21); the agent stood 1–20 tiles from water at death (median 5: they died on the way). Over all 119 daughters the
region's water distance is 1–30 (median 16); only 32 % of daughters had water within 12.

**(c) Daughters founded off a region centre: 0 of 119.** Every group reached its centre (`settled_at_leader` false in
all fission records). 117 of 119 daughters were founded in spring (display day 29: 53, day 30: 13, day 85: 33 — the
second year's first eligible day), 2 in summer.

**(d) Winters over the 0.35 bound.** Per settlement-winter (exposure deaths in the winter window over the settlement's
members at 18:00 of the winter's first day, a settlement founded inside the winter counted with its founders): 45 of
626 settlement-winters with members exceed 0.35, **all 45 in daughters** (maximum ratio 1.3: members joining during the
winter count in the numerator only). They hold 4 (18), 3 (11), 5 (6), 6 (5), 2 (4) and 7 (1) dark nights of the
winter's seven. The daughters' woodpiles at the winter's start were not small (2–213, median 162 — the parents' median
193): the wood ran out during the winter. Deadwood per spot at those winters' start was 0–1.0 (median 0.62) with the
winter cap at 1 per spot: in winter a village lives on the fall, and a region area receives `per_day × n_area / n_camp`
of it, a daughter with 8–14 spots against the camp's 14–26. Over all settlement-winters: daughters 0 dark nights in 88
of 242, parents in 175 of 384; four or more dark nights in 40 daughter-winters and 19 parent-winters. 622 of the 827
exposure deaths of these worlds were in daughters (adult 301, elder 248, child 73), 422 of them after three dark nights
of three.

## What Part 2 implements (7.3)

| Fix | Implemented | Why (counts above) |
|---|---|---|
| Region fill (7.2) | yes (required) | (a): 20/20 starvation deaths and 20/20 early daughter failures are in short regions, 17 with no bush |
| (a) `region_water_max` Some(12) | **yes** | (b): 38 dehydration deaths, region water at median 21 tiles; only 32 % of daughters had water within 12 |
| (b) `RANGES_V18` region bushes 8–12, deadwood 12–18 | **yes** | (a) the fill restores the promise; one more bush per region at the low end. (d): more spots receive more of the winter fall under the cap of 1 per spot |
| (c) `area_fall_floor_permille` 600 | **yes** | (d): every over-bound winter is a daughter's, with 2–7 dark nights after a normal woodpile at the start; the fall a region area receives is proportional to its few spots |
| (d) cancel a timeout founding with no bush in reach | **no** | (c): no daughter was founded off a region centre |

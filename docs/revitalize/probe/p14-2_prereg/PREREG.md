# Pass 14 Part 2 — pre-registration of Stage B (PASS14_GOAL.md 7)

Written and committed after Stage A was scored (`docs/revitalize/probe/p14-1_stageA/dose_map.md`) and **before any Stage B probe or recording**. Nothing below changes after this commit.

## The four arms

Selection rule (section 7, `tools/revitalize/dose_map_p14.py::best_arms`): per kind, among the Stage A arms that are not `*E` and have ≥ 16 seeds, the largest right-direction share; ties → larger ENR → smaller total dose (amount × doses) → fewer doses.

| kind | Stage A ranking (arm: right-direction share / ENR / total dose / seeds) | chosen |
|---|---|---|
| spoil_food | `sp240L` 0.786 / 8.19 / 240 / 28; `sp3x80L` 0.768 / 6.37 / 240 / 28; `sp120L` 0.661 / 4.58 / 120 / 28; `sp240H` 0.558 / 1.46 / 240 / 26; `sp60L` 0.446 / 2.38 / 60 / 28 | **`sp240L`** (spoil_food 240 at T_low) |
| gift_food | `gf240L` 0.518 / 4.26 / 240 / 28; `gf120L` 0.518 / 3.69 / 120 / 28; `gf3x80L` 0.482 / 3.72 / 240 / 28; `gf60L` 0.429 / 2.28 / 60 / 28; `gf240H` 0.288 / 0.79 / 240 / 26 | **`gf240L`** (gift_food 240 at T_low; tie with `gf120L` on share broken by the larger ENR) |

Eve-of-winter controls: **`sp240E`** (spoil_food 240 at tick 29,400) and **`gf240E`** (gift_food 240 at tick 29,400).

Each schedule file is a single row `{"tick": T, "kind": K, "settlement": "home", "amount": 240}`, written by `tools/revitalize/schedules_p14.py --arms sp240L,gf240L,sp240E,gf240E` from Stage B's own timeline.

## Seeds, replicates, fork, timing

- Seeds **14000–14063** (64 fresh worlds; never used before Part 3, not in tests). R = 2, noise fork at tick **14,280** (display day 11, 06:00), 120 days, rows every 60 ticks, `game_v19`, frozen recorder `wm_frozen` (sha256 `ba43b546…`), `WORLDSIM_GIT_SHA=2ba871c0…`.
- Timing rule: Part 0 item 3 (PASS14_GOAL.md 5.3, `tools/revitalize/timeline_p14.py`) applied to Stage B's own factual timeline (a `probe_frozen` run of seeds 14000..14064 with round 3's flags and `--sample-every 60`): T_low = 06:00 of the first eligible home check with s > 0 at least 3 days before the factual first departure (else the first such check after that departure and at least 3 days before the next; a seed that never departs: its first s > 0 check; none → no T_low); T_eve = tick 29,400. A seed without T_low gets no `*L` file (counted).
- Scoring: `dose_map_p14.py` exactly as in Stage A (`--prereg sp240L,gf240L`).
- Drop order (0.3 item 6): if time runs out, Stage B's second half of seeds is dropped and K2 is then judged on seeds **14000–14031** (32 seeds).

## K2 (unchanged from section 10)

For each of `sp240L` and `gf240L` on Stage B:

1. right-direction share ≥ **0.75** (spoil: earlier; gift: later; ties and wrong signs count against; a departure in one run only counts in the direction it implies);
2. ENR = mean |Δday| / max(0.5 day, noise mean |Δday|) ≥ **2.0** (NA — clause not met — if the noise has fewer than 8 pairs where both runs depart after the cast);
3. the share of (seed, replicate) pairs whose day-118 settlement count differs from the same-replicate factual ≥ the noise share (factual r0 vs r1 on the arm's seeds) **+ 0.15**.

Each clause is reported; an arm passes when all three hold; **K2 passes if both arms pass**. The `*E` arms are controls: reported, not judged.

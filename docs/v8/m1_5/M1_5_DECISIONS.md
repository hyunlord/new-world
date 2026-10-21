# M1.5 decisions

Numbered decisions of the M1.5 pass (`docs/v8/m1_5/M1_5_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m1_5-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `v8/wip` = `3183e77d` (checked with `git ls-remote` at the start,
  2026-10-21 21:21 KST).
- Choice: base worktree `../new-world-m15-base` detached at `3183e77d`, with the main clone's `rust/Cargo.lock`
  copied in (pitfalls: a fresh worktree resolves newer crates). Data in `$DATA` = `/Users/rexxa/work/m1_5-data/`.
  An owned `caffeinate -i` runs for the session. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe
  `--workers 6`.
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — The U-script and the M1 fixtures

- Context: goal §2 says the M1 U-script lives in `docs/v8/m1/`; it is in fact `tools/v8/u_script.json` (M1 D16,
  `docs/v8/m1/M1_EVALUATION.md` §10.1). `docs/v8/m1/` holds only its description.
- Choice: the U-script is `tools/v8/u_script.json`. The M1 colony fixtures are
  `worldsim_probe --scene colony --commands tools/v8/u_script.json --days 10 --seeds 100..132 --workers 6
  --out … --chronicle-out …` with the base binaries (`$DATA/base/fixtures/`).
- Evidence: `$DATA/base/fixtures/` (unpatched base binaries, `$DATA/base/bin/`): 32 seeds, 4,786 jobs taken, 1,012
  finished, 19 births per seed (M1's numbers).

## D3 — Base data: identity at the base commit; the measurement patch

- Context: goal 4.2 wants the identity base, the M1 colony fixtures (summaries + chronicle) and the first benchmark
  run at the base commit. The base's colony probe has no `--chronicle-out` and there is no `bench` at the base.
- Choice:
  - Identity base: `tools/v8/identity.sh --capture $DATA/base/bin $DATA/base/identity` at `3183e77d`
    (DETERMINISM PASS, 21:26–21:40). The gate uses `--base-data $DATA/base` (V8-0's base at `b1ca032f` stays
    V8-0's record; the baseline has not changed since, so both are the same world).
  - A measurement-only patch (`$DATA/base/measure.patch`, applied uncommitted in the base worktree, binaries in
    `$DATA/base/mbin/`): the standing benchmark (`sim_test::bench`, `WorldGenConfig::colony_stress`,
    `tools/v8/bench_script.json`), the colony probe's `--chronicle-out` and `churn.json`, and the seam's
    measurement-only release counter (`ColonyState::release_counts`, read by nothing in the simulation).
  - Proof that the patch is measurement-only: the patched base probe's colony `summary.json` is byte-identical to
    the unpatched base's, with and without the U-script (`cmp`, 32 seeds each). Fixtures with chronicles and churn:
    `$DATA/base/fixtures_m/`.
  - The base benchmark line is measured later in the same quiet session as the candidate line (D12), so that the
    "before" and "after" lines share the host load.

## D4 — The calendar is one rule; the season rule's day count must equal it

- Context: goal 5.1 adds `WorldRules.calendar: CalendarRule { ticks_per_day, days_per_season, seasons_per_year }`;
  `SeasonRule` already holds `days_per_season`.
- Choice: every reader of the season length takes it from the calendar (`WorldRules::season_days()`: the
  calendar's value when seasons are on, else 0; the season index wraps at `seasons_per_year`). `SeasonRule::
  days_per_season` stays (removing it would rewrite every season literal and the rules dumps) and must equal the
  calendar's (`WorldRules::calendar_consistent`, asserted in `calendar_derivations` for both worlds). Derived
  lengths are `CalendarRule` methods (`days_per_year`, `ticks_per_year`, `years_to_ticks`, `days_to_ticks`,
  `whole_years`, `date_of_day`). `SimResources.ticks_per_day` (the engine clock) stays 1,440 and equals the
  calendar's in both worlds.
- Readers switched: `resource_regen::season_at_tick`, `firewood::year_at_tick` (days per year), `firewood::
  wood_target` (the `winter_prep_days: None` fallback), `food_economy::stock_target`, the bridge's
  `collect_time_snapshot`. Probe-only readers under `sim-test/src/probe/` (baseline measurements) keep reading
  `SeasonRule` (equal by `calendar_consistent`).
- Evidence: identity PASS on the Part 1 code (`$DATA/identity/p1_try1`: 228,748 values, 0 differences; 64
  chronicle files; 64 raw files; DETERMINISM PASS).

## D5 — Calendar audit (goal 5.1): every rule duration in days or ticks

Values that mean "a season" or "a winter" are taken from the calendar; values that mean a day, an hour or a work
duration stay. Colony values differ from the baseline only where the table says so.

| rule | field | baseline value | unit | colony value | reason |
|---|---|---|---|---|---|
| `CalendarRule` (new) | `ticks_per_day` / `days_per_season` / `seasons_per_year` | 1,440 / 7 / 4 | ticks / days / seasons | 1,440 / **6** / 4 | ADR 0004 left the calendar to M1.5; a 24-day year |
| `SeasonRule` | `days_per_season` | 7 | days | **6** | must equal the calendar (D4) |
| `AgingRule` | `elder_age_ticks` | 66,240 | ticks (46 days) | **50 years** = 1,728,000 | ages in years (5.2) |
| `AgingRule` | `lifespan_mean_ticks` / `lifespan_spread_ticks` | 83,520 / 11,520 | ticks | **62 / 8 years** | 5.2 |
| `AgingRule` | `founder_age_min_ticks` / `max` | 8,640 / 57,600 | ticks | **16 / 40 years** | 5.2 |
| `AgingRule` | `founder_min_remaining_ticks` | 4,320 | ticks | **10 years** | 5.2 |
| `AgingRule` | `lifespan_bounds_ticks` (new) | `None` | ticks | **(40, 85) years** | 5.2 clamp; never binds at 62 ± 8 |
| `aging.rs` | `FOUNDER_TOP_UP_TICKS` (const) | 2,880 | ticks (2 days) | unchanged | the top-up never runs when founders are ≤ 40 with ≥ 54 years of life |
| `LifeStageRule` | `child_ticks` | 5,760 | ticks (4 days) | **15 years** = 518,400 | 5.2 |
| `LifeStageRule` | `infant_ticks` (new) | `None` | ticks | **3 years** | 5.2 (derived flag, D8) |
| `BirthRule` | `Provisioned.cooldown_ticks` | 720 | ticks | — (colony: `Fertility`) | 5.3 |
| `BirthRule` | `Fertility.gestation_days` / `postpartum_days` (new) | — | days | 18 / 48 | 5.3 (9 of 12 "months"; ≥ 2 years) |
| `BirthRule` | `Provisioned.sustain.reserve_days` | 0.5 | days of need | — | a day of food: stays (colony no longer provisions) |
| `StockTargetRule` | `base_days` | 2.5 | days of need | 2.5 | a day measure; calibration knob, not changed (D9) |
| `FirewoodRule` | `base_days` | 1.5 | days of burn | 1.5 | a day measure |
| `FirewoodRule` | `winter_prep_days` | `Some(4.0)` | days of winter burn | **`None`** = days per season (6) | "a winter" from the calendar; calibration round 1 (D9) |
| `FirewoodRule` | `winter_base_days` | `Some(4.0)` | days of burn in winter | 4.0 | a day measure (4 of 6 winter days kept in hand); not needed by calibration |
| `FirewoodRule` | `winter_regen_units_per_day`, `winter_severity_regen_units_per_day` | 6, [8, 6, 4] | units per day | unchanged | per-day rates |
| `FirewoodRule` | `*_minute` (gather, burn, cold, urgent) | clock minutes | minutes | unchanged | times of day |
| `ExposureRule` | `heal_block_hours` | 2 | hours | unchanged | hours |
| `FissionRule` | `cooldown_days`, `min_age_days`, `settle_timeout_days`; `FissionHazard.scarcity_days` | 28, 10, 2; 4 | days | — (fission is off in the colony) | baseline only |
| `RelationsRule` | `aid_*_days_permille`, `trade_*_days_permille`, `*_nights` | | days / nights of need | unchanged (inert: one village) | day measures |
| `ShelterPlanningRule` | `plan_interval_ticks` | 60 | ticks | — (off in the colony) | |
| `AgentRules` | `move_interval_ticks` | 8 | ticks | 3 (M1) | a step |
| `SocialSeekRule` | `max_chase_ticks` / `retry_cooldown_ticks` | 240 / 480 | ticks (hours) | unchanged | hours |
| `NightRestRule`, `EveningGatherRule`, `ForageRule`, `MealRule`, `RoutineRule`, `EndStagger`, `FamilyRule.pair_minute`, `TemperamentRule.ha_bedtime_span_minutes` | `*_minute(s)` | clock minutes | minutes | unchanged | times of day |
| `ColonyRule` | `chop_ticks`, `hut_progress`, `stuck_ticks` | 90, 240, 240 | ticks | unchanged | work durations |
| `WorldRules` | `clock_offset_ticks` | 480 | ticks | 480 | the clock starts at 08:00 |
| `memory_system.rs` | `MAX_RECENCY_TICKS` (const) | 4,380 | ticks (~3 days) | unchanged | memory recency in days |
| `resource_regen` | `REGEN_INTERVAL` (const) | 120 | ticks | unchanged | a pulse |
| `firewood.rs` | year of a tick (`year_at_tick`) | `4 × days_per_season` | days | **calendar `days_per_year`** | D4 |

## D6 — Fertility: 33 ‰ a day, not the goal's 12 ‰, so the rate is the stated 0.25 births per woman-year

- Context: goal 5.3 gives `conception_per_day_permille: 12` with the comment "≈ 0.25 births / woman-year at 24-day
  years", and 5.5 asks for births within ±15 % of 250 per 1,000 woman-years. 12 ‰ × 24 days = 0.29 conceptions per
  woman-year only while a woman is eligible; she is not eligible for the 18 days of gestation and the 48 days
  after a birth. The long-run rate is 1 / (18 + 49 + (1 − p)/p) births a day: 0.16 a year at p = 12 ‰, at the
  very edge of D1's band (0.15–0.40).
- Choice: `conception_per_day_permille: 33` → 1 / (67 + 29.3) = 0.249 births per woman-year: the rate the goal,
  the test (250 per 1,000 woman-years) and ADR 0017's ~3 % growth all name. Every other field as given.
- Test: `fertility_rate_matches_rule` drives `FertilitySystem` over 100 women × 240 days and compares the births
  with a day-by-day probability recursion of the rule (written independently of the system): within ±15 %. The
  recursion expects about 230 births in that window (the first pregnancies are still running at its start).
- Evidence: `rust/crates/sim-test/tests/colony_demography.rs`.

## D7 — `FertilitySystem` runs at 12:00, not 06:00

- Context: goal 5.3 says "runs at 06:00 of each day" and gates on the settlement's mean hunger and thirst (≤ 60).
  Round 0 (`$DATA/calib/r0`) gave 0.24–0.84 births per (fed) woman-year, above what the rule allows: the fed gate
  failed on about half the mornings. Measured on seeds 1 and 4 over 20 days: at 06:00 the mean thirst is 56–67
  (the overnight peak, before the morning water), at 12:00 the means are 25–32 (hunger) and 32–40 (thirst).
- Choice: the system runs at 12:00 (`FERTILITY_MINUTE = 720`, interval 1,440 with `RuntimeSystem::tick_offset`,
  a new default-0 trait method so every other system keeps `tick % interval == 0`). Births are queued at 12:00 and
  spawned at 12:01. The gate keeps its meaning ("a fed settlement").
- Evidence: round 0b (`$DATA/calib/r0b`): 0.09–0.37 births per woman-year, mean 0.28.

## D8 — Infants and children behave alike in this pass; keyed by lineage

- Context: goal 5.2 adds infants (until 3 years) who do not forage, do not work and stay at the hearth (the child
  tether). In V7 code every `Child` already does not forage, gather, build, take jobs or fight, and is tethered
  3 tiles from the hearth (`decision/system.rs` child sets; `JobAssignSystem` skips `Child`).
- Choice: `Child::is_infant(tick, infant_ticks)` is derived from the age (no component); infants keep the child
  behaviour, so infant and child differ only in the label (bridge column `infants`). Older children doing chores
  is M2+ work. No change to `decision/system.rs`.
- The conception draw is keyed `(seed, STREAM_CONCEIVE, lineage key, day index, slot 0, replicate)`, not by
  `agent_id` as goal 5.3 writes it: an `AgentId` shifts with every earlier spawn (architecture skill: key by a
  stable identity).

## D9 — Calibration (goal 5.4), seeds 0–7, 3 years without orders

| round | change | hunger / thirst / cold deaths | other |
|---|---|---|---|
| 0 | the M1.5 rules as written (fertility at 06:00) | 0 / 0 / 1 (seed 5) | births/woman-year 0.24–0.84 (D7) |
| 0b | fertility at 12:00 (D7; not a calibration knob) | 0 / 0 / 1 (seed 5: an 8-day-old infant on the first winter's last night; the hearth was out of wood) | 0.09–0.37 |
| 1 | `FirewoodRule.winter_prep_days`: `Some(4.0)` → `None` (= the calendar's 6-day winter) | **0 / 0 / 0 on 8 / 8** | population after 3 years 11–15 |

Round 1 meets the bar; no further knob was needed. Raw: `$DATA/calib/r0`, `r0b`, `r1`.

## D10 — The standing benchmark (goal 6.1): what it runs and how it measures

- Scenes: `WorldGenConfig::colony_stress(15100, N, side)` under `WorldRules::colony(15100)`: N = 200, 1,000,
  2,000 on 192 × 192 without orders; N = 200 with `tools/v8/bench_script.json` (the U-script × 4: 60 trees, an
  8 × 8 stockpile, 4 huts); N = 200 on 64 × 64 (only for the per-tile figure). 1,440 warm-up ticks, then 2,880
  profiled ticks (`tick_profiled`). Three repeats, interleaved by repeat; medians.
- `colony_stress` scales the bushes and deadwood beyond the camp with N (`96 × N / 10`, `160 × N / 10`; worldgen
  places as many as the map has candidates for). That alone does not feed 2,000 people: the outer bushes lie
  beyond the 16-tile forage radius, and a trial without food lost 1,611 of 2,000 to hunger inside the run. The
  benchmark therefore feeds the world exactly as `worldsim_perf quick` feeds its stress world: 3 food per living
  person into the home store at the first settlement and every 06:00, outside the timed ticks. With it the trial
  kept 2,000 / 2,000 alive. (Goal 6.1 asks that "nobody starves inside the run"; scaled bushes alone cannot do it.)
- Memory: each scene runs in its own child process (`worldsim_perf bench-one`) so the resident size starts clean.
  RSS is read with `ps -o rss= -p <pid>` on macOS (the task's resident size, the figure `mach_task_basic_info`
  reports; no `libc` dependency is added to sim-test) and `/proc/self/statm` on Linux, before the build, after it
  and after the run. Bytes per agent = (RSS after build at N = 2,000 − at N = 200) / 1,800; bytes per tile =
  (192² − 64², N = 200) difference / 32,768 tiles.
- Ledger: `docs/perf/PERF_LEDGER.md` (columns: date, commit, pass, N, scene, tick mean, p99, max, RSS/agent,
  RSS/tile, host load, note). `tools/v8/bench_ledger.py check` compares a run with the ledger's latest pass:
  > +10 % on a scene's tick mean or p99, or on RSS/agent, fails unless `--justified TEXT` quotes the CHANGELOG.
  The C3 ceilings are reported PASS/MISS and do not fail in M1.5.

## D11 — The spatial index (goal 6.2): what is indexed, and how it stays current

- `sim_core::spatial::ChunkIndex<T>`: 16 × 16-tile chunks, `BTreeMap<(cy, cx), Vec<(tile, T)>>` sorted by
  `(y, x, T)`. `ring(centre, r)` lists chunk ring r (chunks in `(cy, cx)` order); `nearest_where(centre,
  max_radius, pred)` returns the smallest `(Chebyshev distance, y, x, T)` among entries within the radius that pass
  the predicate — the choice `assign.rs`'s M1 `nearest` made (`min_by_key((distance, y, x))`). It stops when the
  next chunk ring's nearest possible tile, `(r − 1) × 16 + 1`, is farther than the best, so equal distances in the
  next ring are still compared. Unit test: 1,000 random queries (40 sets of up to 300 entries, radii 0 … u32::MAX,
  three predicates) equal a linear scan.
- What is indexed: the job searches' candidates — ground stacks (Deliver sources, Haul stacks), zone cells (Haul
  destinations), chop-marked trees (Chop), and the sites (Build/Deliver, few: indexed per tick from the site view)
  — plus the agents (first `SpatialIndexSystem` at priority 119; since D15 built on demand). Not indexed: the 10,000
  unmarked trees (no job searches them; an index would be rebuilt at every felling and read by nobody).
- How it stays current: the colony indexes are derived from `ColonyState` and rebuilt when its `overlay_version`
  changes — every mutation of ground items, zones, designations, sites and trees already bumps that counter
  (`commands.rs`, `jobs/items.rs`, `jobs/execute.rs`, `construction_system.rs`, worldgen). One counter instead of
  an update at every mutation site cannot miss a site (M1's lesson: a hand-rolled move is a ledger hole); the
  rebuild is O(stacks + cells + marks) — tens to hundreds of entries — a few hundred times a game day.
- The agents' index has no reader in M1.5 (M2's proximity searches are the first); see D15.
- Evidence: the M1 fixtures (seeds 100–131, 10 days, with and without the U-script) are **byte-identical** before
  and after the index: `summary.json` (with the colony fingerprints of every day) `cmp`-equal, the 64 chronicle
  files equal by content, `churn.json` equal (`$DATA/p1/fixtures` vs `$DATA/p2/fixtures`).

## D12 — 6.3 (index use in the decision scans) dropped to M-PERF

- Context: goal 6.3 is the second item of the drop order. The decision's food, water and gather scans
  (`decision/targets.rs`: `nearest_resource_tile`, `nearest_local_food`, `nearest_gather_tile`, …) choose by
  Manhattan distance with `(x, y)` ties over V7 `HashMap`s (`food_tiles`, `water_tiles`) that change at every meal,
  pickup and regrowth pulse, and they run in the baseline.
- Choice: not done. An index for them needs a Manhattan, `(x, y)`-tie variant and updates in V7 consume and regrowth
  code that the baseline runs (identity risk), for a part of `AgentDecisionSystem` whose share of the decision cost
  is not yet profiled. M-PERF takes it, starting with a profile of `AgentDecisionSystem` (66–86 % of the tick at
  N = 2,000 in the trial runs).

## D13 — Job churn (goal 8.2): the diagnosis, the remedies, the calibration

- Diagnosis (8.2.1, on the M1 fixtures at the base commit, seeds 100–131, 10 days, U-script; the seam's
  measurement-only counter, D3): 4,786 jobs taken, 1,012 finished (0.21), 3,774 released by `CancelReason::Needs`.
  By the need the decision turned to: **forage 2,160 (57 %)**, thirst 836 (22 %), sleep 541 (14 %), firewood 170
  (4.5 %), social 54 (1.4 %), hunger 13 (0.3 %). By stage: walking to the source 2,895 (77 %), working 653, walking
  the load 226. Thirst releases happen at thirst 50–59 (550), 60–69 (58), 70–79 (228); sleep releases at fatigue
  0–59, mostly at night (the night-rest threshold is 0). The churn is not hunger: three releases in five are a
  worker turned forager because the store was under its target.
- Remedy 1 (`ColonyRule.yield_margin_permille`, start and final value 150): a job yields only to an urgent
  personal need — food at hunger ≥ 50 + 15, water at thirst ≥ 50 + 15, sleep at fatigue ≥ the person's own
  threshold + 15 (the night-rest threshold 0 at night, 50 by day). Survival work (foraging, firewood), social and
  every other choice never take a job holder; the worker's previous state is restored, as M1 did for an idle
  choice. Rationale: a job exists only because the player ordered it; the colony's foraging and firewood are done by
  those without a job and by workers once their job is done.
- Remedy 2 (`ColonyRule.finish_leg_tiles`, 12): a `Haul` or `Deliver` in `ToDest` within 12 tiles of its
  destination delivers before yielding (even to an urgent need).
- Both live in `ColonyRule::M1_5` (the colony's rule); `ColonyRule::M1` keeps M1's values (`None`) for the job
  fixtures. Code: `jobs::seam::keeps_job` (no change to `decision/system.rs`; `decision::needs` became
  `pub(crate)` so the seam reads the same thresholds); unit test `a_job_yields_only_to_an_urgent_need_and_finishes_a_near_leg`.
- Calibration (seeds 0–7, U-script, 10 days): round 1 at 150 ‰ and 12 tiles: finished / taken 0.97–1.00 on 8 / 8
  (before: 0.19–0.35), 0 deaths, U1–U3 8 / 8. No second round.
- After (M1 fixtures, seeds 100–131, re-captured at Part 4: `$DATA/p4/fixtures`): 1,030 taken, 1,014 finished
  (0.984), 16 releases (all thirst); 32 / 32 seeds ≥ 0.5; U1, U2, U3 32 / 32; 0 deaths.

| run | taken | finished | ratio | released by need |
|---|---|---|---|---|
| base `3183e77d` | 4,786 | 1,012 | 0.211 | 3,774 |
| Part 1 (calendar) | 4,424 | 1,008 | 0.228 | 3,416 |
| Part 4 (remedies) | 1,030 | 1,014 | 0.984 | 16 |

## D14 — The text renderer and the snapshot schema (goal 8.1)

- `worldsim_probe --scene colony --seeds S..S+1 --ascii --tick T [--rect x,y,w,h] [--commands FILE]
  [--ascii-legend] [--out FILE] [--people-json FILE]` (`sim_test::probe::ascii`) draws only from the bridge's
  pure collectors (`collect_terrain_snapshot`, `collect_colony_overlay`, `collect_building_snapshot`,
  `collect_trees`, `collect_agent_snapshot`, `collect_agent_jobs`, `collect_time_snapshot`).
- Choices beyond the goal's glyph list: all ten terrain types have a glyph (`,` forest floor, `:` desert for "dirt",
  `_` coast, `-` tundra, `"` swamp, `o` cave); a marked standing tree shows `x` (trees are drawn above marks); a
  tile with k ≥ 2 people shows the digit k (sleepers share their shelter's tile, so one character per person cannot
  count them; the Godot view fans sleepers out for drawing only); stumps follow the tree layer's own rule (a tree
  gone from `get_trees` is a stump for 1,440 ticks; the simulation has no stumps).
- Agreement (`tools/v8/renderer_agreement.sh`, future gate step `renderer_agreement`): seed 100, U-script, tick 900;
  the multiset of person tiles from the text renderer equals the tiles of the people `agent_layer.gd` has drawn
  (its own node positions minus the drawing-only sleeper spread, converted with `View.world_to_tile`):
  `RENDERER_AGREEMENT PASS n=10 tick=900` (also at tick 600). A one-tile shift of one person fails it.
- `docs/v8/snapshot_schema.md` (schema version 1) documents 31 getters and 283 keys; `check_docs_helpers.py
  schema` checks names and keys against the bridge source (check 10 of `check_docs.sh`; the gate step
  `schema_docs`).
- `--ascii-diff` (the first item of the drop order) was not done.

## D15 — The index made faster than the linear scan it replaced; the agents' index on demand

- Context: the first interleaved benchmark session (`$DATA/bench/prelim`, contended: 1-minute load 10–17) showed
  `JobAssignSystem` at 14 % of the tick in the scripted N = 200 scene against 4.4 % at the base, and the per-tick
  agents index (`SpatialIndexSystem`) at 2.4–4.5 % of the tick with no reader.
- Causes: (1) the ring walk visited every chunk coordinate of every ring (a `BTreeMap` lookup each, a vector per
  ring) even where nothing was indexed; (2) most of ~150 idle agents searched for candidates that were all reserved,
  so each search scanned everything and found nothing — every tick, for every agent.
- Changes (results unchanged: the M1 fixtures and the calibration seeds stay byte-identical):
  - `nearest_where` visits only the non-empty chunks, in order of their exact Chebyshev distance to the centre, and
    stops at the first chunk farther than the best entry (equal distances are still compared).
  - `JobAssignSystem` remembers, within one tick, the searches that found nothing anywhere on the map (`Exhausted`):
    candidates only shrink inside a tick (each assignment adds reservations and logs in transit), so an empty search
    stays empty for every later agent of that tick.
  - The agents' index is built on demand at most once per tick (`SpatialIndex::agents_at`); `SpatialIndexSystem`
    is removed. Goal 6.2 asked for a rebuild every tick at priority 119; with no reader in M1.5 that cost 2–4 % of
    every tick for nothing, against C3.
- Effect (scripted N = 200 scene, two interleaved pairs): `JobAssignSystem` 18 µs per tick at the base, 6 µs after
  (−67 %); the tick mean 393/395 µs (base) against 381/374 µs.

## D16 — Review fix: no conception on the due day; postpartum counted in calendar days

- Context: the numbers review (`$DATA/review/numbers_review.md`, finding 2) found that `FertilitySystem` removed a
  mother's `Pregnant` in step 1 and let her conceive again in step 2 of the same 12:00 tick, before
  `SettlementSystem` wrote her `last_birth_tick` on the next tick: 2 of 93 births in the first evaluation came 18 days
  after the mother's previous birth. It also noted the postpartum wait was 49 days (the birth is spawned at 12:01).
- Fix: mothers delivering today are skipped in step 2; the postpartum gate compares calendar days (`day of the
  conception ≥ day of the last birth + postpartum_days`), so the wait is exactly 48 days. Regression test
  `a_mother_does_not_conceive_on_her_due_day_or_before_her_postpartum_days` (1,000 ‰; fails without the fix). The
  rate test's recursion uses 48 days. The evaluation was re-run on the fixed code (`$DATA/eval2/`); the first run
  (`$DATA/eval/`) is kept as a record.
- Also from the review (finding 1): D1's upper bound cannot fail at any conception rate (gestation plus postpartum
  caps a woman at 24 / 66 ≈ 0.36 births a year). The probe now also reports the conception rate per at-risk day
  (women eligible for the draw: not pregnant, not delivering, past the postpartum days), which must match the
  rule's 33 ‰ and can fail in both directions; the report gives it beside D1.

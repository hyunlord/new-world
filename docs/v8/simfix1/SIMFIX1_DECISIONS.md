# SIMFIX1 decisions

Brief: owner-supplied BRIEF_SIMFIX1 (the seven quick fixes of SIMAUDIT1). Source: `art-img-data/simaudit1/AUDIT.md`,
`N2_PROPOSAL.md` (tickets SIMFIX1, SIMFIX2, WORK1 and the starving search of PURPOSE1). ADR:
[0082](../../decisions/0082-day-life-fixes.md).

- **D-SIMFIX1-0 (2026-11-22, process).** Built and tested only on the DGX (CPU, ≤ 4 threads, launches held while
  the 1-minute load was ≥ 8). BASE is v8/main 6ac56915b with the SIMAUDIT1 probe; its runs are SIMAUDIT1's own
  (same commit, same probe; the sim is deterministic, C1). The Mac gate is left to the planner.
- **D-SIMFIX1-1 (save format).** No fix changes a saved type (`FORMAT_VERSION` stays 26). N2's `MindState.invites`,
  routine cooldowns and steward back-off table would each need a save tail, so each is replaced by a pure reading
  of saved state (ADR 0082). The tunables are in `data/colony/daylife.ron`, read through `DaylifeRule::get()`, and
  are not a `WorldRules` field because the rules are saved; a loaded game uses the running build's values.
- **D-SIMFIX1-2 (roof).** A usable Dwelling whose template data says `warmth: Roofed` counts as a roofed bed
  (`slept_roofed`, as a bedroom), not only as "not outside". Where a person slept is sampled from the night-rest start
  (21:00); cold is still noted from 20:00.
- **D-SIMFIX1-3 (finish leg).** N2's `finish_leg_tiles: 6` duty node is already covered: `job_finish_delivery_leg`
  (duty band, ≤ 12 tiles) outranks `need_job_fatigue` (need band). No new node.
- **D-SIMFIX1-4 (provisioning).** No early-eating effect exists, so provisioning is the admission rule (a job longer
  than `provision_over_ticks` needs hunger and thirst below their seek thresholds at its end) plus two lowest-score
  need nodes, `prepare_drink` and `prepare_eat`, that let an idle worker top up at one meal's worth (30.0, equal to
  the consume amount, so no relief is wasted).
- **D-SIMFIX1-5 (hunt progress).** A hunt's no-gain count is the Chebyshev distance to its prey. `walk` restarts
  `progress_tick` on every path replan and the prey moves every few ticks, so the hunt restores its own progress
  after the walk step. A miss or a lost reach still counts as progress; `max_ticks` bounds those.
- **D-SIMFIX1-6 (talk).** The invitation is the inviter's own `Seeking{Agent(invitee)}` state; the lowest inviter id
  wins. Acceptance does not require loneliness (N2: an invitee whose choice is leisure or idle accepts), but does
  require no job, no hunger/thirst/fatigue need and no talk cooldown. A refused or stalled chase ends through the
  existing missing-target path with `retry_cooldown_ticks`.
- **D-SIMFIX1-7 (flicker).** The play loop came from movement refusing an unwalkable place, not from a fact/start
  mismatch: both read the same place list. The list now drops tiles movement would refuse. The 1-tick Idle at an
  Activity's end was an off-by-one in the read (`until_tick <= current_tick` after the tick ran). The GDScript pose
  hold of N2 is not done: no renderer consumes the rows yet; CHAR3 should hold poses when it does.
- **D-SIMFIX1-8 (starving).** `starving_to_hearth` (rest:hearth) only. Search already exists: `need_harvest_food`
  targets the nearest reachable ripe edible plant with no budget once a hungry person has no food; the famine case
  is winter with nothing ripe. Gifts between kin need HOME1's house store (N2 ties them to it) and are not done.
  `eat_seed_when_starving` is not implemented (planner Q1: keep it off; no flag without a consumer).
- **D-SIMFIX1-9 (steward).** Only the always-rejected `AddHuntZone` (873 of 881 rejections) is fixed: the planner
  skips a zone that already exists. The `Maintain(n)` deadband (3,521 `SetBillMode`) is WORK1 scope, not one of the
  brief's seven, and is left.
- **D-SIMFIX1-10 (2026-11-22, panic found by the smoke).** Round 1 seed 5 panicked on day 47
  (`ColonyMindSystem invariant: effect routine_sit_by_fire: MissingTarget`). `starving_to_hearth` required `idle`, so
  the SitByFire Activity its `rest` start created made it ineligible the next tick; `continue_routine_sit_by_fire`
  took the Activity over, and its full-selection target (the routine place preview) had no entry for the hearth tile.
  MIND_AUTHORING already says a `rest` node must not carry an idle-only predicate; the node now has none. Selection
  also falls back to a running routine's own Activity place when the preview no longer offers it, which the new
  walkable-place filter can cause when a wall goes up beside a sitter.
- **D-SIMFIX1-11 (2026-11-22, talk pairing).** Round 1 (seeds 0–4, 6, 7) left loneliness ≥ 75 at 58.6 % of samples
  (bar ≤ 25 %) with 1.06 talks per person-day: peers were taken only from the same tile, and everyone on a tile chose
  the lowest id, so with three people one always chased a partner already paired. This is the mechanism, not a
  number: in the colony free people now look within a `peer_cell` (4) tile block and pair off in id order. The frozen
  pre-colony snapshot tests are unchanged (they run without colony rules).
- **D-SIMFIX1-12 (2026-11-22, memory-social loop).** The audit's remaining oscillation (`need_idle_*_memory_social` ↔
  `finish_social_need_*`): `social_partner_present` holds for any live chase, so the finish node cut every chase the
  memory node had just started for a tired, hungry or thirsty person, the next tick, with no cooldown. The three
  `finish_social_need_*` nodes now leave a chase chosen from social memory (`recallable_social`) to its own end (the
  handshake, `stall_ticks` or `max_chase_ticks`). Data only.
- **D-SIMFIX1-13 (2026-11-22, meeting).** Round 2 (block pairing) raised Talk Approach : Perform to 10:1: two people
  who had both accepted walked toward each other on the same move ticks and swapped tiles when adjacent, since a
  handshake needs the same tile. Once a partner walks back, the higher id now waits on its own tile and the lower id
  walks to it (`social_meeting_tile`). Pure; no saved state.
- **D-SIMFIX1-14 (2026-11-22, tried and reverted).** Round 2 still had 352 bedtime cancels, 219 of them harvests
  taken 20:00–22:00 by the hungry-harvest nodes (forage windows end at 18:00; JobAssign already applies the horizon).
  Round 3 gated `need_harvest_food` / `need_job_harvest_food` on a `harvest_fits_before_bed` fact. It cut bedtime
  cancels only to 329 but starved people: awake samples at hunger ≥ 90 rose 170 → 980 and seed 5 ended with 8 alive
  (BASE 11), because in winter the nearest ripe plant is far and the gate closed hungry harvesting by mid-afternoon.
  Reverted: survival outranks a cut job. The hungry harvest at dusk stays a known source of bedtime cancels.
- **D-SIMFIX1-15 (2026-11-22, meeting, second cause).** D-13 did not move Approach : Perform (9.7:1): after the
  effects, the mind's post-pass re-attaches every social seeker to its partner's current tile. The post-pass now
  applies the same rule (the higher id waits once the partner walks to it). A trace of seed 0 showed agents 3 and 4
  swapping (89,93) ↔ (90,93) every move for 239 ticks before this.
- **D-SIMFIX1-16 (2026-11-22, tried, no effect, reverted).** Round 4 left 42 % of the 392 awake samples at hunger ≥ 90
  idle, 151 of them in seed 6's famine under `starving_to_hearth`: the hearth seats two (routine occupancy < 2) and
  the others are denied and stand. Round 5 added `starving_sit_by_fire` (a free sit-by-fire place first); its
  outputs were byte-identical to round 4, so the node was never eligible for these people (the sit-by-fire routine
  preview is closed to them). Reverted to the round-4 policy. The full-hearth case needs a capacity or place change
  (HOME1/PURPOSE1).
- **D-SIMFIX1-17 (2026-11-22, rebase).** Rebased onto INTEG1's published `af2f844d0`. INTEG1 renumbered the art ADRs to
  0074–0081, so this pass's ADR is 0082. Only documentation conflicted (CHANGELOG, ROADMAP, HISTORY, pitfalls); both
  sides were kept, SIMFIX1's entry on top.
- **D-SIMFIX1-18 (2026-11-22, MUX2 sleep rule kept).** The first full test run failed three MUX2 tests
  (`mux2_fuel_sleep.rs`): work is withheld exactly when the sleep decision is due (fatigue 49 still takes fuel work;
  without a bed fatigue never withholds work). N2's 8.0 fatigue headroom contradicted that pinned rule and was
  dropped; the bedtime horizon, like `sleep_due`, applies only when there is somewhere to sleep.
- **D-SIMFIX1-19 (2026-11-22, provisioning deadlock).** `c2_chain_bronze` stalled after the tin caravan (day 25 → 120):
  the provisioning rule held a long job back whenever hunger would end above 50, even for a person with no reachable
  food, who could then never take it. Hunger now holds a job back only when food is available to the person (the
  `needy` test), thirst only when there is water.
- **D-SIMFIX1-20 (2026-11-22, seed-5 famine).** The final paired smoke (seeds 0–7 × 60 days) ends seed 5 with 9 alive
  against BASE 11, below the brief's per-seed bar (≥ BASE − 1). The candidate's edible stock follows BASE's (160 on
  day 40, only the 36-item seed reserve by days 45–47) and its starvation deaths fall at BASE's ticks (68,950–69,107)
  plus two more: it is BASE's winter famine. Not tuned (D-FEATURE-FIRST; planner Q1 keeps seed eating off). The
  publication collapse floor (no extinction where BASE survives, KnowledgeLost ≤ 2 × BASE + 2) holds.
- **D-SIMFIX1-21 (2026-11-22, saved mind policies).** Mac pregate-1 failed three genuine-save tests (C2 base21, C2-b v23,
  C3 growth): a saved game restores its own mind policy against the provider registry of its time, and the new
  registered fact `social_invited` changed the current registry, so old saves fell through to the pre-iron variant,
  which cannot compile a policy that uses iron facts. `MindProgram::load_saved` now tries a pre-SIMFIX1 variant (current
  registry without `social_invited`) first, and every historical variant excludes it too. The DGX skipped these tests
  because their fixture saves live only on the Mac; they were copied to the DGX for this check.
- **D-SIMFIX1-22 (2026-11-23, steward pass sliced; planner answer).** Strict step 41 failed twice on frames carrying
  StewardSystem's whole 360-tick planning pass (22–38 ms in the Godot debug library). The pass is now spread over
  ticks: `StewardSystem` runs every tick, and the period's rules (data order) run in `daylife.steward.pass_slices`
  (16) contiguous slices on the period's first ticks, one rule per tick; needs tracking and zone growth stay on the
  first tick, and replay runs every tick so replayed commands land on their recorded tick. State-free (the slice is
  `tick % 360`), so saves and replays are unchanged; a rule's commands land at most 15 ticks after the period start.
  Measured on the DGX with the Godot library's optimisation levels (sim-systems opt 1, the rest opt 0), seed 0,
  steward on, 2 days: worst StewardSystem tick 36.7 ms → 19.8 ms. The planner's 6 ms target is not met by slicing:
  single rules cost FoodPlan ≈ 20 ms (`field_orders` ≈ 12), Bills ≈ 16 ms (`reserved_house_tiles` ≈ 10 when placing a
  workshop) and House ≈ 11–14 ms; all three share the House planner (`future_house`: a full-map capacity scan
  calling nav reachability per tile). `StewardSystem::run_full_pass` keeps the unsliced pass for fixtures that call
  the system directly.
- **D-SIMFIX1-23 (2026-11-23, stopped before the strict capture).** With the sliced pass (e79a2d332) Mac pregate-3 failed
  two tests: the `SYSTEM_WORLDS.md` registration snapshot (StewardSystem interval 360 → 1; fixed) and
  `spine_chain_log_house`, which passed with the unsliced pass. On the DGX the same test passes with `pass_slices: 1`
  and fails with 16: the scripted scene loses villagers to fights in both (4 vs 5 combat deaths, days 13 and 26); with
  slicing only two remain, a craft interrupted mid-way loses its clay, and the 17-brick bill stops at 16 by day 48.
  Looking the bill up by recipe and digging one craft of spare clay did not rescue it. Not tuned further; the planner
  decides (QUESTION.md).
- **D-SIMFIX1-24 (2026-11-23, planner answer D-SIMFIX1-24; stopped at the 10 ms target).** `pass_slices` is 1 again and
  `StewardSystem` registers interval 360 when it is 1 (the slice machinery stays for n > 1), so the steward's timing
  is exactly 1f300f1be's. Exact speedups: the production-input planner reads each node's yield once per call (it was
  a content lookup per node per item kind, twice), and `house_capacity` uses a whole-grid passability mask
  (`nav::passable_mask`, sharing `tile_cost`'s body) and one nav reachability mask (`CostNav::reach_mask`) instead
  of a cost lookup and a region query per tile; both masks have tile-by-tile equality tests. The 8-seed × 60-day
  SIMAUDIT1 smoke is byte-identical to 1f300f1be in every output but the two wall-time columns of `daily.csv`.
  A per-pass `future_house` cache shared by Bills, FoodPlan and House is not exact-useful: those rules issue commands
  (workshop blueprints, fields, bills) between their calls in nearly every pass, and the plan reads the state those
  commands change. `house_origins` spans the whole interior, so the capacity window is the whole map.
  Measured on the DGX (one core, seeds 0/1/2 × 3 days) the earlier "Godot-like" numbers had sim-systems at opt-level
  2 (the test profile's package override). With the Godot debug library's real profile (opt 0, sim-systems opt 1)
  the worst StewardSystem tick is 1f300f1be 83.7/89.4/90.9 ms and this code 67.2/79.9/78.6 ms; with sim-core,
  sim-engine and sim-systems at opt-level 2 it is 9.2/9.3/9.5 and 8.4/8.8/9.9 ms. At tick 360 (debug profile) Bills
  costs 33 ms (house plan 19, workshop-site ranking of 36,100 locations 12) and FoodPlan 30 ms (house plan 17.5,
  harvest 6, cooking 2.4). The 10 ms target is not reachable by exact code changes in an opt-0 build; per the answer,
  stopped and reported (QUESTION.md 3).
- **D-SIMFIX1-25 (2026-11-23, planner answer D-SIMFIX1-25; gate-path build-profile change).** The dev profile — the
  build of the debug Godot library the gate's frame captures run — compiles `sim-core`, `sim-engine` and `sim-systems`
  at `opt-level = 2` (was: sim-systems 1 since 78af5e245, the others 0). Debug assertions and overflow checks stay on
  (the dev defaults), as in the test profile since D-GATE-1. Rust optimisation levels do not change integer or float
  semantics (no fast-math), so simulation results are unchanged; proved by the 8-seed × 60-day SIMAUDIT1 smoke with a
  dev-built probe against 1f300f1be's release run. Worst StewardSystem tick on the DGX (one core, seeds 0/1/2 × 3
  days): 8.4 / 8.8 / 9.9 ms with this branch's code (1f300f1be's code: 9.2 / 9.3 / 9.5 ms). Cost: slower debug builds
  of those three crates. The exact speedups of D-24 stay.
- **D-SIMFIX1-26 (2026-11-23, stopped at the strict step-41 capture).** On e09e63e8e the Mac pregate passed (rc 0) and
  the strict stall capture alone failed: zoom 1.0 passed (max 41.2 ms, none > 50), zoom 0.5 had one 55.2 ms frame at
  t = 12.7 s (tick 151) with no simulation tick in it and 49.4 ms in the draw phase; at the same moment the zoom 1.0 arm
  spent 32.9 ms in world3d `props`. The StewardSystem maximum in the capture is 8.3 ms (was 26–37 ms), so the
  steward no longer owns a slow frame. Per D-25 the pass stopped without a retry or publication; the frame owner (a
  world3d props first build/draw) went to the planner (QUESTION.md 4).

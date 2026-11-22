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

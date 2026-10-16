# Daily routine v1 (`game_v21`, unchanged in `game_v22`/`game_v23`; own day ends in `game_v24`)

Pass 17 Stage A gives need-satisfied residents visible daytime activities near their home. Activities are economy-neutral: tending a fire burns no extra fuel, crafting produces no item, repairing changes no building, and exploring harvests nothing. Existing work and needs take priority.

The values below are **frozen at their original defaults** after the sixteen-world Stage A tuning round passed R1–R3 and all R2 point-estimate bars. No second parameter-tuning round was used. The initial freeze `9a0bd85470f4ddbe26d51b8c5062107bf5c1f371` passed I1–I4 and both pin suites. A subsequent audit identified a same-tick death observation gap; corrected producer `50a73474e06d3a9abbb535729e9c7e1e134954b7` passes focused regressions and the complete unchanged-parameter tuning rerun. Both scene summaries match the original round exactly except timing, and all R1–R3/R2 tuning estimates and verdicts remain unchanged. Corrected final I1–I4 and both pin suites pass. The performance rule R5 fails: the median mean tick rises 33.04% (0.353189 to 0.469883 ms) and p99.9 rises 339.86% (0.587 to 2.582 ms), above the 15% and 35% limits; absolute time and the 1.001316 decision-system slope pass. These valid failed measurements are retained without changing thresholds or parameters. The complete reserved evaluation now passes R1–R4: adult daytime Idle share is 0.116436 versus 0.595078; persistence, novelty-seeking and reward-dependence correlations are 0.897930, 0.491714 and 0.350282, all with positive 95% intervals. Every R2 health bar passes. Overall Stage A remains FAIL solely because R5 fails. See [the complete held-out evaluation](probe/p17-a_eval/eval.md); tuning is not substituted for these results. The governing requirements are [PASS17_GOAL.md](PASS17_GOAL.md), section 6. Final measurements belong in the Pass 17 report and evaluation artifacts.

## Scene and components

`WorldRules::game_v21(seed)` starts from `game_v20(seed)` and sets only `agent.routine = Some(V21_ROUTINE)`. `WorldGenConfig::game_v21(seed)` delegates to v20 generation. Every older scene retains `routine = None`; the routine preparation code, activity component mutations and new random draws are conditional on this option. The probe and performance runner accept v21. The world-model recorder is unchanged.

`Activity` exists while a resident walks to or performs an activity. It records `kind`, `settlement`, `place`, `chosen_tick`, `arrived_tick` and `until_tick`. Arrival and deadline are absent during the walk. `ActivityRest` records the next permitted routine-start tick. `RoutineObservation` stores only the latest lifecycle event and its tick. If a resident has multiple events in one tick, `RoutineObservationBatch` preserves them in order; the observer prefers that matching-tick batch over the latest event. Neither component is a cumulative tally. All cumulative activity measurements belong to the optional probe trace.

Activity kinds have a stable order, shared by bridge codes and trace arrays:

| Code | Kind | Eligible residents | Candidate place |
|---:|---|---|---|
| 0 | `TendFire` | Non-elder adults | Chebyshev distance 1 from the hearth |
| 1 | `Craft` | Non-elder adults | Distance 1 from an owned stockpile |
| 2 | `Repair` | Non-elder adults | Distance 1 from an owned shelter |
| 3 | `HearthCircle` | Adults, including elders | Distance 2 from the hearth |
| 4 | `Explore` | Non-elder adults | Distance 10–14 from the hearth |
| 5 | `Play` | Children | Distance 2–3 from the hearth |
| 6 | `SitByFire` | Elders | Distance 1–2 from the hearth |

The hearth anchor is the settlement's `formation_tile`. Places must be walkable and must not contain a building. Except for Explore, all places lie within 8 tiles of the hearth. Craft therefore has no place in a daughter camp without a stockpile. Building ownership uses `building_owner`, with the settlement's member-building registry as the fallback when an ownership entry is absent.

## Choosing an activity

The decision arm follows the existing Gather arm and precedes the settlement fallback. A resident must have reached it Idle, belong to a settlement, have no haul load or other `no_trips` restriction, have no live rest timer, and be inside the resident's daytime window. Earlier decision arms remain responsible for needs, meals, construction, social behavior and resource collection.

The display clock includes `clock_offset_ticks`. The frozen window is:

```text
start = 420 + lineage_key % 60   # 07:00–07:59
end   = 1080                   # 18:00, exclusive
```

The stagger spreads starts across an hour. Kind selection uses the resident's lineage key, current system tick and `STREAM_ROUTINE`. Slot 0 is the original kind draw, slot 1 the single no-place redraw, and slot 2 the later rest-duration draw. The runtime passes the current noise replicate through the keyed draw interface.

With `d_axis = axis - 500`, integer-adjusted worker weights are:

| Kind | Base weight | Adjustment |
|---|---:|---|
| TendFire | 180 | `weight × (1000 + d_HA) / 1000` |
| Craft | 180 | `weight × (1000 + d_P) / 1000` |
| Repair | 140 | `weight × (1000 + d_P) / 1000` |
| HearthCircle | 200 | `weight × (1000 + d_RD) / 1000` |
| Explore | 120 | `weight × max(0, 1000 + 2 × d_NS) / 1000` |

Elders use HearthCircle 500, scaled by RD with the same formula, and SitByFire 500. Children always choose Play. Division is integer division; signed duration adjustments below truncate toward zero. No exponential, logarithmic or trigonometric operation is used.

The occupancy map is built once per decision tick from active components, in AgentId order. At most two residents may reserve the same place, including residents still walking there. Each subsequent choice updates this map immediately. Among candidates with occupancy below two, the minimum key is `(occupancy, keyed_rank, y, x)`.

The brief's literal `keyed_rank = keyed_u64(seed, STREAM_ROUTINE_PLACE, lineage_key, tick, 0, replicate)` is the same for all candidates in one choice. Consequently equal-occupancy candidates currently resolve by `(y, x)`; no tile salt or extra randomization is invented. This interpretation is recorded in [DECISIONS.md](DECISIONS.md).

If the selected kind has no available place, the resident redraws once using only eligible kinds with a place. If none remains, the resident receives a fixed 30-minute rest. The observer counts both redraws and failures, so crowding-induced changes in Explore share remain visible beside R3-NS.

## Walking, duration and leaving

A start creates `Seeking { Activity }` and a `SeekTarget` at the reserved place. The explicit post-decision Activity arm retains that target. Standing on the place tile satisfies the arrival condition, changes the state to `Consuming { Activity }`, records `arrived_tick`, and computes:

```text
duration = max(20, 90 + 60 × (P - 500) / 500) minutes
until_tick = arrived_tick + duration
```

The frozen range is 30–150 minutes. Walking does not consume this duration. A resident still walking after 150 minutes abandons the activity. A regular end or timeout schedules a keyed rest uniformly over the inclusive range 20–60 minutes.

Before the main state match, an active resident returns to Idle when one of these responses is available:

1. A need exceeds the decision system's own hunger, thirst or personal fatigue threshold, using its need-availability filter.
2. The resident can actually forage, collect firewood or gather now. The check includes worker eligibility, trip restrictions, inventory space, clock/store/hoard predicates and a successful tile lookup under the current claims.
3. The resident is an assigned builder.
4. The resident and a co-located peer satisfy the existing social eligibility checks.
5. The existing stocked-meal wait applies within the camp radius.

The unchanged Idle arm then handles the response in that same iteration. These interruptions do not schedule the normal rest window. If no response fires and execution reaches routine entry again, a one-tick rest prevents immediate restart churn.

Inside the Activity arms, reaching the duration deadline or 18:00 ends the activity. Such an end counts as **completed** only after arrival. A settlement change also ends it, but does not count as completed. These exits schedule the normal rest window. Stale Activity components on residents no longer Seeking or Consuming Activity are removed before occupancy is built; they are recorded as interruptions. All queued Activity, ActivityRest and observation changes are applied in AgentId order.

Under the routine rule, active residents participate in builder selection and the social/combat peer maps as Idle residents do. Higher-priority selection can therefore interrupt visible leisure rather than making a resident unavailable to village life. If a later-processed resident selects an activity holder who was already processed, targeted replay sends that selected holder through the same Idle decision body. A person may therefore start and end an activity within one tick. The trace preserves both events and any redraw, records zero consuming minutes when no end-of-tick consumption occurred, and leaves unobserved place/arrival/deadline fields absent rather than fabricating them.

Adult and elder activity consumption remains stationary. A child consuming Play takes keyed x/y steps on each movement tick using `STREAM_ROUTINE_PLAY`, constrained to Chebyshev distance 1 from its reserved place and to the map bounds. The reservation stays at the original place. Activities do not add need decay or any resource debit or credit.

## `game_v24` (Pass 19): everyone ends their day at their own time

`WorldRules::game_v24(seed)` = `game_v23(seed)` with `RoutineRule::end_stagger = Some(EndStagger { base_minute: 1065, key_span: 30, p_span_minutes: 45 })` (`None` in `game_v21`–`game_v23`). A person's activities end at

```text
end_minute = 1065 + (lineage_key / 60) % 30 + 45 × (P − 500) / 500     (i64, truncating; no temperament = P 500)
```

— 17:00 (P 0, key term 0) to 18:59 (P 1,000, key term 29); `key / 60` keeps it independent of the start's `key % 60`. Both readers of `day_end_minute` go through `RoutineRule::end_minute`: `can_enter` admits a person only while `start_minute(key) ≤ minute < end_minute(key, t)` and `deadline` ends their activity at their own end minute (without a stagger both return the shared 18:00, so `game_v21`–`game_v23` stay bit-identical; proven by I5–I7 on the final build, H6, [validation](probe/p19-4_final/validation.txt)). Bedtime, pairing (20:00), fission (06:00), forage (until 18:00) and the evening tether (from 18:00) are unchanged: early enders fall through to the normal cascade, late enders join the tether when they stop. The live game builds `game_v24`.

Measured on 32 worlds × 120 days against `game_v23` ([eval](probe/p19-2_eval/eval.md)): non-interrupted day-end exits fall 17:00–17:29 1,841, 17:30–17:59 32,396, 18:00–18:29 28,201, 18:30–18:59 2,931 (`game_v23`: all 71,553 at 18:00); Spearman ρ(P, a person's mean exit minute) = 0.849 [0.840, 0.857] over 4,419 people (H2 PASS). The largest number of activity ends in one tick between 16:00 and 20:00 fell from a median of 31 to 10 per world-day — H1 (≤ 0.15×) **misses** at 0.32×: the day-end exits alone give 0.125×, but two other edges remain, both food-gathering trips becoming available to many activity holders at once (16:01 in both rule sets; 18:01 in `game_v24`; the trigger at those minutes was not traced further), outside this pass's scope. Health, F1, F5 and daytime idle are unchanged (H3, H4 PASS); the 17:00–19:00 adult idle share fell from 0.474 to 0.375. The probe's unchanged trace windows (brief 7.4): the daytime idle-tether window (Idle adults, 06:00–18:00, every 10th tick) has 1,379,596 samples in `game_v24` vs 1,337,358 in `game_v23` (+3.2 %: early enders are Idle before 18:00) at the same mean Chebyshev distance from home (5.60 vs 5.62 tiles); the night window (first bed tick from 18:00) is unchanged — mean bed minute 1,326.2 vs 1,325.5 (22:06 vs 22:05), median 1,304 in both, over 156,358 vs 156,014 adult-nights. The 18:00 tick's cost fell from 0.87 to 0.61 ms and p99.9 is back within 35 % of `game_v20` (H5's mean bar still misses, +17.6 %; [PERF_BUDGET](PERF_BUDGET.md)).

## Performance structure (Pass 18)

Pass 18 kept every result bit-identical (I1–I6 and all pins, [DECISIONS.md](DECISIONS.md) Pass 18) and changed only how the routine's per-tick inputs are obtained:

- **No per-tick maps of everyone.** The lineage key comes from the decision query's `Option<&Lineage>` (rows are AgentId-sorted); settlement membership is the decision system's `member_of` whenever the firewood or forage rule built it (same construction: settlement id order, first listing wins), else `members_of()` builds it as Pass 17 did; the entity map is built on the tick's first social/combat invitation only. `RoutineTick::new` keeps the stale-activity cleanup, the rest-expiry scan and the occupancy rebuild from the active rows.
- **Places are cached across ticks and computed lazily.** `AgentDecisionSystem`'s only (private) field, `PlacesCache`, holds `places::collect`'s result and its inputs (`building_kinds`, `building_owner`, `building_registry`, each settlement's `formation_tile` and `member_buildings`, the grid size, the River/Mountain flags of each scanned 29 × 29 area). The first `place()` call of a tick compares the inputs without allocating and recomputes only if one differs; a tick without a routine start (night) never looks. Stockpile/shelter adjacency uses sets instead of linear scans.
- **Choosing a place is resumable.** Pass 17's choice minimised `(occupancy, keyed_rank, y, x)`, but the rank is keyed by the person and tick, not the tile, so it never decides; candidates are generated in `(y, x)` order. The choice is therefore the first free tile, else the first tile with one occupant. A per-tick `Cursor` per (settlement, kind) remembers where the occupied front of its list ends; occupancy only grows within a tick except at `end()`, which rewinds every cursor whose list holds the freed tile (binary search). This removed the mid-day "storm" cost: 100–150 starts in one tick, most of them redraws that re-scanned all seven lists.
- **The exit test asks only whether a trip exists, once per input state.** `choose_bush`/`choose_spot`/`nearest_gather_tile` filter candidates the same way for everyone (bush, spot) or by position (gather); `is_some()` equals an `any()` over the same filter (`bush_eligible`, `spot_eligible`, `gather_tile_exists`, shared with the choosers). Cheap predicates (need, builder assignment, social, meal) are evaluated first and the trip predicates lazily. Their answers are memoised per tick — bush by home, spot by (home, urgency), gather by (tile, catchment) — together with version counters of the maps they read (`claims`, `wood_claims`, `food_tiles`, `wood_tiles`); every write to those maps in the decision loop bumps its version.
- **An activity ending into a rest is one archetype move.** When an entity's whole pending group is "remove `Activity`, record an observation, insert `ActivityRest`" and it already carries a `RoutineObservation`, `finish` uses `World::exchange`, which creates the same intermediate archetype and leaves every entity where remove-then-insert would. This halves the 18:00 edge's moves (all daytime activities end at once).
- The occupancy map is still rebuilt every tick from the active rows (deaths remove `Activity` outside the decision system), but from sorted places in one pass.
- `RoutineObservation*` components stay on production entities and `Activity` is unchanged.

`sim-systems` has an opt-in Cargo feature `subprofile` (forwarded by `sim-test`): `scope!` blocks around `RoutineTick::new`'s parts, the pre-loop maps, the exit test (and its three trip predicates), the start arm, each decision round, the post-pass and `finish`, plus counters for archetype moves, starts by outcome and `place()` calls. Without the feature `scope!` is the bare block and the release binaries are byte-identical to a build without the scopes. `worldsim_perf` built with it (`--features subprofile`, separate `--target-dir`) adds a `subprofile` block to its JSON; it is never used for identity runs or the gate's `perf_quick`.

## Trace contract and time basis

`worldsim_probe --agent-trace` observes after every `engine.tick()`. The system tick is `resources.current_tick`; the first observation is tick 0 after the first engine tick. There is no duplicate initial observation. With 1,440 ticks per day, every consuming tick represents one game minute.

For traced routine-enabled worlds only, a private read-only observer at priority 136 copies the current activity and ordered lifecycle events before lethal systems can despawn their holder. After the tick, the trace recovers those copies only for missing people; surviving people still use the normal final-world query. Recovered rows carry `observation_boundary: "pre_lethal"` and retain the captured state and life stage. Recovery adds no consuming minute for the death tick. An explicit completion before death remains completed, while an unfinished episode ends as interrupted. The snapshot is drained once and its system tick is checked. Observer execution time is subtracted from probe tick timing; the standalone performance runner never installs this observer.

Existing no-trace summaries keep their schema and values for older scenes. The optional trace writes these files for every scene, including v20 comparison runs:

| File suffix | Contents |
|---|---|
| `_agents.jsonl` | Every resident ever observed as a non-elder adult, including residents with zero activity minutes; existing temperament, social and adult-day data plus `adult_ticks` and adult-only `routine` counters |
| `_people.jsonl` | Every observed person; `routine` counters include all life stages |
| `_hours.jsonl` | Exact hourly state counts by child/adult/elder stage, with stockpile food and wood totals |
| `_routine.jsonl` | Started, arrived, ended and no-place events, plus compressed consuming intervals |
| `_trace_meta.json` | Existing metadata plus the `pass17` schema, shelter completions, redraw/failure totals and observation-miss count |

The `routine` object contains `activities_started[7]`, `activity_minutes[7]`, `completed_activities`, `completed_minutes`, `no_place_redraws` and `no_place_failures`. Array order is the kind table above. Mean completed duration is `completed_minutes / completed_activities`, defined only when the count is positive. Adult counters include only non-elder adult consuming ticks; a life-stage change splits the raw interval. Starts are attributed to the life stage at start, while an adult completion requires positive adult consuming minutes. An activity begun as a child can therefore contribute an adult completion without an adult start.

An hourly row has `tick`, one-based display `day`, display `minute`, `states: {child: [5], adult: [5], elder: [5]}`, `stockpile_food` and `stockpile_wood`. State indices are 0 Idle, 1 non-activity Seeking, 2 Consuming Agent, 3 other Consuming, and 4 Seeking/Consuming Activity. R1 selects minutes 480–1020 inclusive. R2 store means select minute 720. Display-day labels may extend into the next day because the simulation begins with a clock offset; the full elapsed-day run still contains exactly 24 hourly samples, ten R1 samples and one noon sample per elapsed day.

Raw `consuming_interval` rows use the half-open range `[start_tick, end_tick)` and record `id`, `chosen_tick`, `kind`, `life_stage` and `minutes`. Their lengths reconstruct the per-kind totals without counting walks. An interval still open at the end of the run has `open_at_end: true`. End rows record `completed`, `consuming_minutes`, `adult_consuming_minutes`, chosen/arrival/deadline ticks and the observation source. An interruption, disappearance, walk timeout or unfinished end-of-run activity does not enter completed duration. The trace uses the explicit completion flag; it does not guess numbered exit reasons. Event rows also expose the current end-of-tick state where observed.

`pass17.shelters_completed` is the final completed-Shelter count minus its initial count, matching the existing probe metric. This includes completed shelters placed by daughter camps. The current runtime never removes or changes a completed Shelter's building-kind entry, so the difference is an exact count of additions. `pass17.observation_misses` exposes unmatched lifecycle observations or inconsistent active components rather than silently omitting them.

The evaluation population is not prefiltered in the trace. R3 selects non-elder adults with at least 14,400 adult ticks; zero-activity residents remain eligible for NS/RD with zero share. R3-P additionally requires at least three completed activities. R2 social exposure uses all ever-adult rows, including residents below ten adult days.

With `--check-determinism`, v21 receives the existing observable checks plus a sorted direct comparison of Activity, ActivityRest, RoutineObservation and RoutineObservationBatch components. These timer checks do not alter older trajectory fingerprints.

## Bridge contract for the art line

Both agent snapshot and detail collection use `state_tag = 4` for Seeking or Consuming Activity and `target_kind = 8` for Activity. Older numeric codes are unchanged; v20 and older rules never create the new target. Existing agent row structs and the nine-key `get_agent_detail` dictionary are unchanged.

`get_activity_snapshot()` returns five parallel packed arrays:

| Key | Type | Meaning |
|---|---|---|
| `ids` | PackedInt64Array | Entity bits, matching the existing agent snapshot identity surface |
| `kinds` | PackedByteArray | Activity codes 0–6 |
| `place_x`, `place_y` | PackedInt32Array | Reserved destination, not the resident's current walking position |
| `until_tick` | PackedInt64Array | Duration deadline; `-1` while walking before arrival |

The pure collector sorts rows by AgentId and exposes only agents with an Activity component. It does not mutate the world. The art line can join these arrays to existing snapshot ids to draw the seven activities and their places. Until that rendering work lands, movement to a place and time spent there are the visible evidence; an art renderer that treats unknown state 4 as Idle may still draw the idle pose. This pass does not change art assets or GDScript.

## Current limits

Activities produce no objects, repair progress, learning, fuel benefit or exploration reward. HearthCircle is a visible routine choice; the existing social system still determines actual completed social interactions. Play's movement does not create a new navigation system. Place tie-breaking follows the literal rank contract described above. This document's implementation descriptions do not substitute for the reserved paired-seed evaluation, final identity and performance measurements. Original tuning evidence is retained in `probe/p17-a_tuning/r1/`; corrected observer evidence is in `probe/p17-a_tuning/r1-observerfix/`.

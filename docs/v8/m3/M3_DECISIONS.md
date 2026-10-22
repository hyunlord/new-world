# M3 decisions

Numbered decisions of the M3 pass (`docs/v8/m3/M3_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m3-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/wip` = `f0ba4347`, `v8/main` = `4874c9dc` (checked with `git ls-remote` at the
  start, 2026-10-22 17:05 KST; `v8/art-trial2` = `2433a612`, `v8/art-trial3` = `b2a06115` on the remote).
- Choice: base worktree `../new-world-m3-base` detached at `f0ba4347`, with the main clone's `rust/Cargo.lock`
  copied in. Data in `$DATA` = `/Users/rexxa/work/m3-data/`. An owned `caffeinate -i` runs for the session.
  `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`. Start 17:05 KST on 2026-10-22; the 18-hour
  target is 11:05 and the 22-hour hard stop 15:05 on 2026-10-23.
- The host was shared and loaded at the start (1-minute load 20.9, 17 sessions); timings are labelled with the
  load they were taken under (M2 D23).
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — The performance plan (goal §4, written before any pathfinding code)

- Budget (pre-registered, goal 3.1): at N = 1,000 with the M3 soak script (walls and rooms present),
  `AgentMovementSystem` + pathfinding ≤ 0.6 ms per tick mean; the whole tick stays under the C3 ceilings of
  `docs/perf/PERF_LEDGER.md`.
- Expected costs at N = 1,000: ≤ 200 path requests per 100 ticks; A* on 192² with walls ≤ 2,000 nodes expanded on a
  typical request; memo hit rate ≥ 60 % on hauling routes.
- Measurement: `worldsim_perf nav-bench` (`nav_bench`) reports path requests, mean nodes expanded per search,
  memo hits and the movement + nav µs per tick. It runs after Part 1 and after Part 3.
- If movement + nav exceeds 0.6 ms, apply in this order and record each step's effect: (1) path reuse for the same
  `(from_chunk, to)`; (2) the flow-field cache (6.4); (3) a cap on re-plans per tick (round-robin).

## D3 — Skill tie-break: rounds of (priority level, work type)

- Context: goal 3.4 — "within one priority level, `JobAssignSystem` orders agents by the work type's skill level
  descending before `AgentId`". M2 (D14) served each person's level-k work types in turn, person by person, so there
  was no single "work type" to order people by inside a level.
- Choice: under the economy the assignment runs one round per (level 1 … 4, work type in the tie-break order); a round
  serves the people whose priority for that work type is that level, ordered by their skill for it (highest first),
  then `AgentId`. Skill for a work type: craft = the better of knapping and woodworking, build = building, quarry =
  quarrying, chop = chopping, split = woodworking, haul = none (`assign::skill_for`). Without the economy (M1 rules)
  nothing changes.
- Effect on M2's behaviour: within a level, every person's craft work is now offered before anyone's haul work (work
  type before person); before, the first person in `AgentId` order took any of their level-k types before the next
  person tried any. The M2 colony runs move, and the M2 fixtures that pin colony outcomes are re-captured where they
  changed (D6). The baseline is untouched.
- Evidence: `colony_skills.rs::within_a_priority_level_the_more_skilled_person_takes_the_job` (fails with the sort
  removed: checked).

## D4 — Quality words: the band table decides

- Context: goal 3.4 gives five bands (0–19 조악, 20–39 보통, 40–59 좋음, 60–79 훌륭, 80–100 걸작) and the example
  "부싯돌 날 · 좋음 (62)", which the bands put in 훌륭.
- Choice: the band table decides (62 → "훌륭 (62)"); the example's number is read as a typo. `EconomyText.quality`
  formats "{word} ({value})" (`UI_QUALITY_FMT`, `QUALITY_BAND_0…4`); the stack line and the inspector's tool line use
  it. Test: `tools/v8/test_locale_trf.gd` (band edges 0, 19, 20, 59, 62, 79, 80, 100 and a stack line).

## D5 — The economy soak fixture

- Context: goal 3.4 — 20 days, chop marks renewed every 2 days (15 trees nearest the camp not yet marked), quarry 3
  nodes, workshops, bills `Make 20` blades, `Make 4` axes, `Maintain 40` firewood, `Maintain 10` blades; plus M3's
  build orders; gate step `soak` on 4 seeds asserting ledger, reservations, 0 deaths and determinism.
- Choice: `tools/v8/m3_soak_script.json` — the M2-U layout (6×6 stockpile 3 east of the hearth, knapping spot and
  wood bench south of it) with those bills; quarry marks on the nearest boulder and the 2 nearest flint outcrops (the
  blades need flint, the bench needs stone); no work priorities (the goal lists none). The probe gained `"every_days"`
  (an entry applied again every n days, resolved anew) and the resolver `nearest_unmarked_trees`. M3's build orders
  join the script in Parts 2–3. `tools/v8/soak.sh <out>` runs seeds 0..4, 20 days, `--check-determinism` and fails on
  any ledger or reservation violation tick, any death, or an idle colony (fewer than 2 workshops, no craft, no tree
  felled). Gate step 26.
- First run (Part 0 tree, `$DATA/try/soak0`): SOAK PASS; 72–77 crafts, 150 trees felled, 0 deaths, 10 alive per seed;
  4 seeds × 20 days in 12 s.

## D6 — Part 0 records: the art state, the M2 fixtures, the base data

- Art (goal 5.2; a subagent read the art branches with `git show origin/<branch>:<path>`, report
  `$DATA/logs/art_record_report.md`): ADR 0018's status, ROADMAP rows ART-TRIAL2b / ART-TRIAL3 / ART-TRIAL3b, HISTORY
  lines and the archive index's art-branch rows now cite `2433a612` (TRIAL2b), `352f4101` (TRIAL3) and `b2a06115`
  (TRIAL3b). Three readings differ from goal §2 and the branches' own reports win: TRIAL3's recommendation is the 2D
  artist only (route B is the pre-registered alternative, unjudged); "87/87" is 87 accepted layers passing the
  checker (22 checker tests), route A's set is incomplete and its playback readability not verified (verdict
  PARTIAL); TRIAL2b also failed people readability (C5, G1) and part yield (C7) with seasons open, so 05 is a
  conditional candidate. ART-TRIAL3b is not running: it **stopped** at its Part 0 tool check (`higgsfield account
  status` exit 4, "No workspace selected", no generation), recorded as 중단.
- M2 fixtures (goal 5.4): with the skill tie-break alone the whole workspace suite passed (1,854 tests, 0 failed,
  `$DATA/logs/p0_tests.log`), so no M2 fixture needed re-capturing; the tie-break changes M2 colony runs (D3) but
  no test pinned the moved values.
- Base data (goal 5.5, `$DATA/base/`): `bin/` release probe and perf at `f0ba4347`; `identity/` the identity capture
  (17:05–17:18); `m2u/` the base colony with the M2-U script, seeds 0–7, 20 days; `soak_first_run/` the soak
  fixture's first run on the Part 0 tree (the base probe cannot read `every_days`). The benchmark ledger base line is
  measured with the candidate in one interleaved quiet session (M2 D17/D23), so it is taken when Part 3 is done.

## D7 — The pathfinder (Part 1; ADR 0021)

- Cost map `sim_core::nav::CostMap` (one byte per tile) in `SimResources.nav`, built from the tile grid and the
  colony's space sites by `runtime::nav::{tile_cost, build, ensure, refresh}`; `refresh(tile)` after every change of
  a tile bumps `nav.version` when the cost moved. `tile_cost` is the single definition of passability; a test
  checks `refresh` against a full `build`.
- Heuristic: octile at the plain cost (`10·max + 4·min`) instead of the goal's Chebyshev `10·max`. Both are admissible
  and consistent for costs 10/14 (no passable tile is cheaper than plain), so both return an optimal path; octile ties
  far fewer nodes on open ground. Ties break by `(f, h, y, x)` as the goal says. A random-map test compares every A*
  cost with an independent Dijkstra (200 maps, 30 % walls, doors).
- Regions: connected components under the same move rule, recomputed lazily once per version; every request first
  compares regions, so an unreachable goal costs no search. The job assignment filters every candidate by the
  agent's region (`NavState::reaches`, no allocation) and keeps its "found nothing" memos per region (M1.5 D15's
  argument holds per region). A goal on an impassable tile (a wall site, a boulder on a hill) is reached from any
  8-neighbour, and arrival is `NavState::arrived` (on it, or adjacent to an impassable goal).
- Paths: `nav.paths[AgentId] = AgentPath { goal, version, steps, next, planned, from }` (saved). On a version change
  the remaining steps are re-checked (`still_walkable`) and kept when they still work; only a broken path is
  re-planned (re-planning every walker on every finished wall would be up to N searches in one tick). A path stays an
  agent's only while it stands where the path left it; a teleported agent re-plans. Paths of agents that stopped
  walking are dropped on each move tick.
- The memo (`(from, to)`, 4,096 entries, insertion-order eviction, dropped on a version change) returns the same `Arc`
  a fresh search would compute; it changes no behaviour, so it is not saved.
- Movement: the colony's directed walks (seeking, haul home) follow the path; idle wander, play and the tether pull
  are refused when the step enters an impassable tile or cuts a corner (no extra draw: the keyed draws are already
  taken). The baseline's branches are unchanged.
- Unreachable job goals: the move records `nav.blocked[id] = goal`; `JobExecuteSystem` cancels the job
  (`CancelReason::Unreachable`) and the job assignment skips that `(person, target)` for
  `SpaceRule::unreachable_cooldown_days` (1). `walk`'s stuck rule counts progress in path steps left, and a
  re-planned path restarts it (a detour around a wall is not "stuck").

## D8 — Decision targets out of reach (found by `colony_commands` in Part 1)

- Context: with the river, mountains and undug hills impassable, the decision system's own selectors (nearest food,
  water, sleep, gather tile; untouched by M3, ADR 0013) can choose a tile in another region. A debug run of colony
  seed 3 showed an adult `Seeking { GatherFood }` toward (61, 80) for 500+ ticks without moving.
- Choice (movement only, colony only; `decision/system.rs` unchanged): when a seek's goal is unreachable, a food,
  forage or water target — and a sleep target of a person without an assigned bed — moves to the nearest reachable
  tile of the same map (the decision's `(Manhattan, x, y)` order); any other non-job seek with nothing in reach ends
  (`Idle`, the next decision chooses again). A job's is the job layer's (D7).
- Hill tiles, walls and doors no longer hold items or stockpile cells, and blueprints refuse them (`can_hold_items`,
  `check_blueprint`); the probe's resolvers (`nearest_trees`, `nearest_unmarked_trees`, `nearest_nodes`) pick only
  targets reachable from the hearth.

## D9 — People who stand where no one can (found by `colony_world` in Part 1)

- Context: colony seed 5 lost two founders to thirst in 10 days. The camp's resident tiles (V7 worldgen, shared with
  the baseline) allow hill tiles; five founders spawned on hill next to the camp, and with corner cutting forbidden
  they could never step off (debug run: all five in region 0 at tick 1).
- Choice: (1) colony worldgen ends with `founders_on_open_ground`: every founder outside the hearth's nav region
  moves to the nearest tile of it (`(Chebyshev, y, x)`, founders in `AgentId` order); the colony's nav map is built
  there, so it exists from tick 0 (the probe's resolvers read it). The baseline's worldgen is unchanged.
  (2) Safety net in `CostMap::can_step`: from an impassable tile any step onto a passable neighbour is allowed (no
  corner rule), and `NavState::path` from an impassable tile starts by stepping off onto the neighbour nearest the
  goal. Test `nav::someone_on_an_impassable_tile_steps_off`.
- Evidence: `colony_world.rs` (viability and invariants, 8 seeds × 10 days) green again.

## D10 — Where a decision may stand, and where items may land (found by `colony_world`, `colony_items`)

- A decision consumes standing on its tile; a water source on a shore tile of undug hill can only be stood next to.
  Colony seed 1 lost two people seeking such a tile diagonally beside them (the path rule called "adjacent"
  arrived). A decision seek whose goal is impassable is treated as unreachable (D8's retarget), and the retarget
  only picks passable tiles. Jobs keep the "work from beside an impassable goal" rule.
- An abandoned decision walk (nothing of its kind in reach) takes the idle step in the same tick instead of
  standing. Limitation: the gather selector (decision code, untouched) can choose the same far-side bush again on the
  next tick, so a forager near the river may loop between choosing it and wandering; in the N = 1,000 stress world,
  where every reachable bush is eaten, this is most of the unreachable requests (`nav_bench`).
- Quarried stone fell on the boulder's hill tile and spilled by ring order into another region, where no hauler could
  reach it. `put_items` in the colony keeps a spill in the drop tile's region (falling back to anywhere only when that
  region is full), and a quarry batch drops at the quarrier's tile when the node's tile cannot hold items.
- Probe resolvers: `stockpile_block` and `blueprint_spot` take only tiles reachable from the hearth (the M2-U
  stockpile of seed 1 lay across the river).

## D11 — Built space (Part 2; ADR 0022)

- Sites: `PlaceBlueprint` of a space kind, `PlaceLine` (Bresenham) and `PlaceRect` (`hollow` = border only) place one
  1×1 site per tile that passes `commands::check_space`; refused tiles are skipped and the count is what was placed
  (with none, the first tile's reason). Refusals: off the map, water, mountain, undug hill (`Hill`), something already
  built that the kind replaces (`Built`), a tree, node, building, site, workshop work tile or reservation
  (`Blocked`); walls, doors and bedrolls also not on a stockpile cell (floors may be); a wall not on a person
  (`Person`; a door may be, it stays passable). New `RejectReason`s: `Person`, `Hill`, `Edge`, `Built`.
- Items on a site's tile do not refuse it (the goal's haul-away): `JobKind::Clear { from, to }` (work type Build,
  taken before deliveries) carries them to the nearest open stockpile cell, else the nearest free reachable tile
  within 12; a space site is "ready" only with its tile clear. The 7.6 invariant allows a stack on a space site.
- Building: a space site is worked by the job layer from beside it (a wall site is impassable from placement) or,
  for door, floor and bedroll sites, on it; `JobExecuteSystem` advances the site's `ConstructionSite` one step per tick
  and `jobs::space::complete_site` writes the tile (`wall_material` = the delivered log's or stone's material, `door`,
  `floor_material = packed_earth`, a bed for a bedroll), consumes the materials and despawns the site. A V7 builder
  working a door or floor site in place reaches the same function through `ConstructionSystem` (no building entry).
- A bedroll needs 1 log (the goal's "Hide 1 or Log 1": hides have no source in the colony yet).
- Deconstruct (`Designation::Deconstruct`, `JobKind::Deconstruct`, work type Build, half the build work): a bedroll
  first, then a wall or door, then a packed floor; half the plan's materials (rounded down) come back on the tile in
  the piece's material (log wall 1 log, stone wall 1 stone, door and bedroll none).
- Dig (`Designation::Dig` on undug hill off the map edge, `JobKind::Dig`, `WorkType::Mine` code 6 placed after
  quarrying in the tie-break order, `SkillKind::Mining` code 5): only frontier tiles are offered (the region filter
  needs a passable neighbour), 150 work ticks scaled by Mining; the tile gets `dug`, `floor_material = granite` (M2 D5:
  hills are granite) and 3 granite stone. `WorkPriorities` and `Skills` arrays grew (save format 3).
- Every tile change calls `nav::refresh` and sets `colony.rooms_dirty`. A finished wall costs what its site cost, so
  only placement bumps the nav version (test `a_walled_square_with_a_door_is_built_from_stockpiled_logs`).

## D12 — Rooms and roles (Part 3; ADR 0023)

- Detection (`sim_core::rooms::detect`, 4-connected flood fill; a diagonal gap between two walls does not leak, as the
  move rule forbids corner cutting). Bounds: walls, doors, undug hill, water, mountain; a wall *site* does not bound
  (only built walls enclose). A candidate (2–120 tiles, not touching the map edge) is a room only if the colony made
  it: a built wall or door on its border, or a dug tile inside. Without that rule colony seed 4 had three "rooms"
  that were meadows ringed by hills (test `an_enclosure_is_a_room…`, 4 `RoomFormed` entries instead of 1).
- Every room is roofed (enclosed and ≤ 120 tiles; the enclosure is the roof). Ids: ≥ 50 % of the new room's tiles in
  an old room keep its id (largest overlap, then lowest id); overrides follow the id and drop with the room.
- Roles: Workshop > Hall > Storehouse (≥ 50 % stockpile cells) > Bedroom (≥ 1 bed, no stockpile cell) >
  Unassigned; `SetRoomRole { room, role: None }` returns a room to automatic. `RoomSystem` (152) detects when
  `rooms_dirty` (set by every wall, door, dig or deconstruct change) and re-reads roles every 60 ticks.
- Effects: bedrolls are one-person beds owned by the nearest settlement; beds are offered roofed-bedroom bedrolls
  first, then huts, then other bedrolls, and never a bed the settlement cannot walk to from its formation tile;
  sleepers (`Consuming { Sleep }`) in a roofed room are not marked `Cold`; asleep in a roofed bedroom, fatigue grows
  at 800 ‰ (`SleepDecaySystem`); a haul goes to an open roofed-storehouse cell unless that is more than 16 tiles
  farther than the nearest open cell, and with no loose stack left haulers restock open-air stockpile items into
  the storehouse; while a roofed storehouse exists, a `Maintain` bill counts only the ground stock under its roof
  (`bills::bill_count`; the firewood gather shortfall keeps `maintain_count`); a workshop in a roofed workshop room
  crafts at 900 ‰ ticks (8.5 kept); inside the evening-gather window an idle step does not leave the hall.
- Tests `colony_rooms.rs` (5): with and without a door (the 4 beds given out only with the door), the dug pocket,
  every role and the override across a moved wall, no night cold for a roofed sleeper, the log into the storehouse at
  equal distance.

## D13 — Crafting continuity (found while calibrating F7)

- A crafter at the bench takes the bill's next craft when its inputs are already on the shelf
  (`craft::next_ready`), and a supply trip brings inputs for up to 5 of the bill's remaining crafts (M2 review NIT 5:
  one craft per trip). Both only under the space rule. The craft round of the job assignment ranks people by the
  skill of the crafts on offer (the active Craft bills' skills), not the better of knapping and woodworking.
- Effect on F7 (calibration, seeds 0–7): none worth the name — blades stay spread over 4–6 people (top knapper
  level 3–6). The levers are kept because each is a plain improvement; F7 is reported as measured.

## D14 — The M3-U and soak scripts and their resolvers

- `tools/v8/m3_soak_script.json` = the soak economy (D5) + M3's build orders; `tools/v8/m3_u_script.json` = the same
  + the two M2-U priority lines (Chop = 0 for adult rank 0, Craft = 1 for rank 1): F7 and F8 "redo" M2's E7 and E2,
  which measured under those priorities.
- Build orders: day 1 (tick 360) `place_house` (a 7 × 5 log-wall box south of the hearth: door in the middle of the
  south wall first, then the hollow walls, 4 bedrolls on the first interior tiles, a packed floor on the other 11);
  day 2 (tick 1,800) `place_storehouse` (a 6 × 6 stockpile inside an 8 × 8 log-wall box with a south door, on any side
  of the camp); day 3 (tick 3,240) `designate_dig` / `hill_pocket` (a 4 × 4 block of undug hill ringed by hill except
  a one-tile mouth in the middle of a side, both dug; when the dig orders are all dug, a door on the mouth
  (`"after": "pocket_dug"`) closes the pocket into a room; no such block within 40 → a second walled storehouse).
- Found by the runs: a box placed across a corridor between hills cut the camp off from the forest (its wall sites
  are impassable from placement, as goal 3.1 says): seed 5 stopped chopping at 62 logs and never finished the
  storehouse. `walled_spot` takes the nearest box whose walls leave every wall tile reachable from the hearth and cost
  the hearth's region no more than the box. A boulder on a dug tile blocked the pocket's door (seed 7): digging a tile
  takes its boulder too (its stone added to the yield). `check_space` refuses only a tile being dug or taken apart,
  not a reserved stack or cell.

## D15 — Evaluation measurements registered before the official run

- F1: the first tick a roofed bedroom with ≥ 4 beds exists, ≤ day 8 (tick < 11,520).
- F2: the first tick a roofed storehouse exists ≤ day 10, and at the end ≥ 60 % of the items on stockpile cells lie
  in a roofed storehouse.
- F3: over the seeds whose `hill_pocket` found a pocket (dig orders accepted): every dig-order tile dug by day 14 and a
  room with `dug = true` existing by day 14.
- F4: for each night starting on day 9 … 19 (index 8 …), the distinct people seen asleep in a roofed bedroom at the
  hourly samples 22:00–06:00 is ≥ 3. (A single 02:00 sample missed workers walking home late from the forest.)
- F5: 0 Unreachable cancels whose target lay in a room with a door; ≥ 1 agent-tick on a door tile.
- F6: `nav_bench` movement µs ≤ 600 at N = 1,000 with the soak script.
- F7: the top knapper's Knap level at day 20 ≥ 5, and the blade quality clause measured as M2 D18: 3 × (the top
  knapper's mean level over days 15–20 − over days 1–5), with the actual blades' mean qualities reported beside it.
- F8: `chops_with_axe` after day 3 ≥ 10. F9: ledger and reservation violation ticks 0, deaths 0 in 20 days; the
  3-year no-orders run: 0 exposure deaths on ≥ 30/32. F10: the gate's bench, identity and replay steps.
- Calibration record (seeds 0–7, 20 days, `$DATA/try/mu0`–`mu8`): mu0–mu1 script and resolver development; mu2–mu6
  F7 levers (D13, priorities); mu7–mu8 the region cut, boulder and F4 fixes. Nine runs on the calibration seeds — more
  than the six the goal allows for calibration; four were bug hunts, not parameter tuning, and no bar threshold moved.

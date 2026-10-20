# M1 decisions

Numbered decisions of the M1 run (goal `docs/v8/m1/M1_GOAL.md` 0.2): context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m1-data/` ($DATA), outside the repository.

## D1 — The target image was regenerated from the committed reference script

- Context: 0.1 copies `~/Downloads/mock_B_near.png` to `docs/images/v8_target_view_b.png`. At the start of the run
  `docs/images/` was empty and no `mock_B_near.png` existed in `~/Downloads` (the copy had failed silently).
- Choice: run `python3 docs/v8/m1/mockup_gen_reference.py $DATA/mock assets/fonts/galmuri/Galmuri11.ttf` (the
  script the brief names as the one that drew the image; seeded `random.Random(7)`, deterministic) and copy its
  `mock_B_near.png` unchanged to `docs/images/v8_target_view_b.png`.
- Reason: the script is the image's source; Part 4 and the player's-eye reviewer need the image.
- Evidence: sha256 `b9cc1efccdf0b96310c3eba6a5dda050247c039c4d1e8582855e66649e4d8c4a`, 1920×1080; the script also
  wrote `mock_B_far.png` and `mock_A_near.png` to `$DATA/mock/` (not committed).

## D2 — `.gitignore`: `*.uid` removed with the three exception lines; evidence images ignored under docs/

- Context: 5.3 says `.uid` files are committed and lists the agreed rules (ignore `docs/**/*.png|jpg` except
  `docs/images/**`; remove the three `.uid` exception lines for files that no longer exist).
- Choice: remove `*.uid` together with its three `!…gd.uid` exceptions (committing `.uid` files needs the
  ignore gone), add `docs/**/*.png`, `docs/**/*.jpg`, `!docs/images/**`. Staged with the 5.3 patch procedure;
  the owner's local hunk (`/graft/`) stays unstaged.
- Reason: the owner's decision of 2026-10-20.
- Evidence: `git diff --cached .gitignore` showed only these lines; `git diff .gitignore` still showed the
  `/graft/` hunk (`$DATA/gi.patch`).

## D3 — What the UID check checks

- Context: 5.3 asks for a check that fails when a tracked `.gd` / `.gdshader` / `.tscn` lacks its `.uid` or a
  `.uid` has no source. Godot 4.6 writes `<file>.uid` for scripts and shaders only; a scene keeps its UID in its
  header (`[gd_scene … uid="uid://…"]`), and folders with `.gdignore` (`tools/`, `rust/`, `.agents/`) are never
  imported, so their scripts have no `.uid`.
- Choice: `python3 tools/v8/check_docs_helpers.py uid` (check 8 of `tools/v8/check_docs.sh`; gate step
  `uid_check` in Part 6): every tracked `.gd`/`.gdshader` outside a `.gdignore`'d folder has a tracked `.uid`,
  every tracked `.uid` has a tracked source, every tracked `.tscn` header has a `uid=`.
- Reason: checks what Godot actually produces; a rule for `.tscn.uid` files could never pass.
- Evidence: before staging, 41 sources lacked a tracked `.uid`; after staging the 42 local `.uid` files whose
  source is tracked, the check prints nothing.

## D4 — Orphan local `.uid` files moved out of the tree

- Context: removing `*.uid` from `.gitignore` showed 12 untracked `.uid` files whose sources V8-0 deleted
  (`shaders/*.gdshader.uid` ×8, `addons/harness/*` ×4).
- Choice: move them to `$DATA/orphan_uids/` (not deleted, not committed).
- Reason: they belong to no tracked source; committing them would fail the UID check, leaving them would keep
  `git status` noisy.
- Evidence: `$DATA/logs/orphan_uids.txt`.

## D5 — How the V8-0 screen check measures S1–S8

- Context: 5.1 asks for `tools/v8/verify_v8_0.gd` (extending `capture.gd`), run windowed on V8-0 and on the
  V7 game, and names eight checks.
- Choice: one script for both games (node paths are the same, so no adaptation was needed). It freezes the
  live loop and steps with `advance_ticks`, so both games are compared at exactly ticks 1,440 and 4,320.
  - S1/S2: a node-tree scan against the 18 deleted V7 UI files (`tools/v8/v7_ui_deleted.txt`, taken from
    the `V8_0_INVENTORY.md` verdict table; INDEX.md has no "7.3 UI" heading, the inventory's 7.3 keep set
    is the list).
  - S3: computes the top-bar values from the bridge with `hud_topbar.gd`'s formulas (the two games' HUD
    labels are not built the same way).
  - S4/S5/S6: synthetic `Input.parse_input_event` clicks and key presses.
  - S8: 120 s of the live loop at `set_sim_speed(3.0)`.
- Reason: identical inputs on both games; the bridge values are what the top bar shows.
- Evidence: `docs/v8/m1/V8_0_SCREEN_CHECK.md`; `$DATA/v8_0_check/`.

## D6 — S8 compares debug libraries and is reported as two contended pairs

- Context: both checkouts' `sim_bridge.gdextension` load `rust/target/debug/libsim_bridge.dylib`, the game's
  default. The Mac carried foreign load (1-minute load 4–21), and the first V7 run overlapped one of my
  cargo builds.
- Choice: run the pair twice, in both orders (V8 first, then V7 first), each with the debug library the
  game loads. Report all four numbers and the loads. Do not present them as perf evidence beyond the S8
  ratio.
- Reason: like-for-like library and procedure; the order swap guards against drift during the session.
- Evidence: V8-0 111.00 / 109.98 against V7 102.37 / 102.42 mean FPS (ratios 1.084 / 1.074).

## D7 — Docs round 1 for Q3 and Q7

- Context: 5.2. A fresh reader (README, docs/** without docs/v8/m1/, skills) answered both questions in
  substance and listed 15 + 11 gaps.
- Choice: fix the Q3 gaps now (ARCHITECTURE §6 rewritten as 10 steps, architecture skill B and its
  procedures, testing E, godot bridge rules). Fix the Q7 wording now (real-time ticks per second, speed
  keys, pause, the dropped surplus, display-clock day and season, ages in days). Re-ask both in round 2
  after Part 3 changes the time scale.
- Reason: Part 3 rewrites the time scale anyway; one round now, one after, keeps within two rounds.
- Evidence: `$DATA/logs/docs_q3_q7_round1.md`, `$DATA/logs/docs_fix_round1.md`.

## D8 — Where the M1 data lives: `sim_core::colony` and one `SimResources::colony` field

- Context: 6.1 lets the run choose one file per concern under `components/` or a new `colony/` module, and
  lists the `SimResources` additions field by field.
- Choice: a new module `rust/crates/sim-core/src/colony/` (`items.rs`, `orders.rs`, `command.rs`, `job.rs`)
  holds every M1 type. The `SimResources` additions sit in one struct, `ColonyState`, stored as
  `SimResources::colony`. It has the listed fields plus `occupied` (footprint tiles of sites and huts) and
  `tallies` (`ColonyTallies`).
- Reason: one place for the new world model; the V7 `Tallies` struct is serialized into probe output and
  struct literals, and a new type avoids touching either (architecture lesson 2026-10-10).
- Evidence: identity PASS with all of it in the tree (`$DATA/identity/p1_try1`).

## D9 — `Job` carries `amount` and stuck tracking; keys depend on the stage

- Context: 7.1 gives `Job { kind, stage, taken_at }`; 7.2 reserves "an in-transit amount on the site";
  7.5 needs "no Chebyshev progress for 240 ticks"; 7.6 says every job holds exactly its keys.
- Choice: `Job` also has `amount` (the logs it moves), `best_distance` and `progress_tick`.
  `Job::keys()` gives the keys for the current stage:
  - Chop: `Tree`.
  - Haul: `Stack(from)` and `Cell(to)` until pickup, then `Cell(to)`.
  - Deliver: `Stack(from)` until pickup, then nothing.
  - Build: `Site`.

  A site's in-transit amount is not stored. It is the sum of its `Deliver` jobs' `amount`
  (`jobs::assign::in_transit`), so it cannot leak. `ColonyRule` gains `stuck_ticks` = 240.
- Reason: the invariant "every job holds exactly its keys" can then be checked by comparing two sets.
- Evidence: `rust/crates/sim-core/src/colony/job.rs`; `jobs::invariants::check`.

## D10 — The decision seam

- Context: 4.3 limits `decision/system.rs` to ≤ 60 added lines: keep a job through leisure and idle, cancel
  it through `jobs/` otherwise. The decision has no single "chosen next state" point: the Idle cascade
  writes the state in 10 places and leaves early in 6 (`$DATA/logs/decision_map.md`).
- Choice: `decision/system.rs` gains 25 lines and changes 1 (`git diff --numstat`: `25 1`). The places:
  - `AgentDecisionSystem::tick` before the agent loop (`let job_holders = jobs::holders(world)`, ~L832).
  - Before `match *state` (~L1127): a holder runs as a temporary `Idle` (`jobs::hold_for_decision`).
  - The routine (leisure) arm skips holders (~L1774): `routine.start` has side effects.
  - The three compile-forced `TargetKind::Job` arms (~L1999, L2133, L2637).
  - After the loop (~L2660): `jobs::settle_after_decision`.
  - The SeekTarget post-pass keeps a job holder's target (~L2968).

  `jobs::seam::settle_after_decision` keeps the job (and restores the previous state) when the decision
  chose nothing, the meal wait, or building the colony site the worker stands on. Anything else releases
  the job (`CancelReason::Needs`), lets the decision's choice stand and drops the job's `SeekTarget` so
  the post-pass re-resolves the goal.
- Reason: no decision logic is duplicated; the cascade decides, jobs/ settles. 4.3's Build case: a builder
  in `Consuming { ConstructionSite }` on its colony site keeps its job.
- Evidence: test `jobs_cancel_on_hunger`; identity PASS.

## D11 — Colony sites are `ConstructionSite` entities that do not progress until their logs are in

- Context: 6.3 spawns a `ConstructionSite` with `MaterialNeeds`; 7.3 hands a builder to the V7 flow once
  `delivered == needed`. The V7 decision lets any agent standing on a site's tile choose "construction".
- Choice:
  - `ConstructionSystem` treats a colony site that is still short of logs like an absent site: it does not
    advance, and a co-located consumer goes back to `Idle`.
  - On completion, a colony site (`ColonySite` + `MaterialNeeds`) becomes a `BuildingKind::Hut` through
    `register_dwelling`, the path planned shelters now share (kind, infinite sleep tile at the key tile,
    owner). It is owned by the settlement whose formation tile is nearest. Its delivered logs move to
    `item_ledger.consumed`, and its footprint becomes `Occupant::Hut`.
  - Once the logs are in, any agent on the centre may help build (V7 behaviour).
- Reason: no material, no progress; and the shelter path is reused, not copied.
- Evidence: `jobs_deliver_and_build_hut`; `hut_sleeps_four`.

## D12 — Hut beds: `BedAssignmentSystem` with per-dwelling capacity

- Context: 7.7 wants four agents to get beds at the hut. `BedAssignmentSystem` ran only with
  `shelter_planning` (off in the colony) and used one capacity for every shelter.
- Choice: it runs when `shelter_planning` or `colony` is set. Each owned dwelling has its own capacity:
  `residents_per_shelter` for shelters, `hut_beds` for huts. The household packing and the plain index
  split are unchanged for equal capacities. The hut's centre sleep tile is `RESOURCE_SOURCE_INFINITE`, as
  shelters'; under night rest a sleep tile is never consumed, so `hut_beds` is enforced by the assignment.
- Reason: baseline behaviour is the special case of equal capacities (identity proves it).
- Evidence: identity PASS; `hut_sleeps_four` (exactly 4 agents have `bed_of` = the hut centre).

## D13 — Clobbered jobs and deaths

- Context: other systems move a worker (ConstructionSystem at completion, combat, fission). A dying worker's
  load must not vanish (7.6 ledger).
- Choice:
  - `JobExecuteSystem` validates each worker first. If the state is not its job's state, a builder whose
    site is gone has finished; anyone else is released (`Interrupted`).
  - JobAssign skips agents in `combat_pairs`.
  - `despawn_agent` calls `jobs::on_death` first: the load drops on the worker's tile and its
    reservations are released.
- Reason: one validator instead of hooks in every system.
- Evidence: `rust/crates/sim-systems/src/runtime/jobs/execute.rs` header; `jobs::cancel::on_death`.

## D14 — Command details the brief left open

- Rectangles: inclusive corners in any order, clipped to the map. A rectangle wholly off the map is
  `OutOfMap`; a valid one that changes nothing is `Nothing` (so the toast can say "할 일이 없음"). The
  bridge clips negative corners the same way (`ffi::colony::clip_rect`).
- `AddStockpile`: each 4-connected group of new tiles is one new zone, never merged into an older zone.
- `PlaceBlueprint` over unreserved loose logs moves them out with the 7.4 spill (a stack may never lie
  under a site). Mountain is `Blocked`, river is `Water`.
- `RemoveZoneArea` and `CancelArea` cancel the jobs that held the removed targets at once
  (`CancelReason::Command`), in `AgentId` order.
- Every rejection counts in `colony.tallies.commands_rejected` (drafting included); without the colony rule
  every command is `NoColony`.
- Evidence: `colony_commands.rs` (one test per reason).

## D15 — Bridge reads: a version getter beside the brief's readers

- Context: 6.5 lists `get_trees` and `get_colony_overlay` with a `version` and asks readers to skip work when
  it is unchanged. ~10,000 trees per call per frame would be wasted work.
- Choice: add `get_colony_versions() -> {trees_version, overlay_version}`. GDScript polls it every frame and
  fetches the big readers only on change. The agent snapshot gains `jobs` (0 none, 1 chop, 2 haul,
  3 deliver, 4 build), `carry_kinds` (-1 none, 0 log) and `carry_counts`, appended in its row order.
  `TargetKind::Job` is target code 9 in the snapshot and in `get_agent_detail`.
- Reason: cheap change detection without changing the brief's readers.
- Evidence: `engine_colony_ffi.rs`; `docs/v8/bridge_api_m1.txt`.

## D16 — The colony probe is its own runner

- Context: 6.4 `--scene colony --commands --commands-out`; the V7 probe pipeline writes hundreds of V7
  metrics and is what identity compares.
- Choice: `worldsim_probe --scene colony` dispatches to `sim_test::probe::colony` (its own parser). It writes
  `summary.json` with the U-bar measurements and no timings or paths (byte-identical between a run and its
  replay) and `timing.json`. With `--commands-out`, each seed's applied log is written in the script's own
  JSON form. Script entries may carry `"resolve"` (`nearest_trees`, `stockpile_east`, `south_of_hearth`);
  the probe resolves them at their tick. `tools/v8/u_script.json` is the committed U-script template.
- Reason: the baseline probe stays untouched (identity), and the colony gets the metrics M1 needs.
- Evidence: probe unit test `a_recorded_command_log_replays_to_a_byte_identical_summary`.

## D17 — Colony worldgen details (goal 8.1)

- Map 192 × 192, exactly 10 founders, forest 30 % / hill 10 % / mountain 4 % (the baseline means, fixed).
- Area-scaled sources: baseline mean × 9. The camp ring and band keep the baseline mean (12 bushes at
  6–16, 20 deadwood spots at 5–18); the remainder lies beyond them. 96 bushes at ≥ 17 tiles and 160
  deadwood spots at ≥ 19 tiles from the hearth, each from its own keyed stream. Wild regions: 2 × 9 = 18
  asked, centres 22–88 from the hearth (baseline 22–28). The baseline's water rule (a centre within 12
  tiles of water) keeps the centres near the river, so 11–14 are placed on seeds 0–7. A shortfall is
  counted, never asserted, as in baseline.
- The camp clearing: no tree within Chebyshev 6 of the hearth. Trees grow on every other Forest tile that is
  not a bush, a deadwood spot, a water source or a building: 10,187–10,354 trees on seeds 0–7. Logs come
  from `tree_logs_min + keyed_below(seed, STREAM_TREE_LOGS, x, y, 0, 0, 3)`.
- The founder adult check (`worldgen::colony::ensure_adults`) needed 0 fixes on seeds 0–7. The founder age
  range (6–40 days) lies wholly between childhood (4 days) and old age (46 days). The replacement path is
  tested synthetically.
- New keyed streams: `STREAM_TREE_LOGS`, `STREAM_COLONY_BUSH`, `STREAM_COLONY_WOOD`,
  `STREAM_COLONY_FOUNDER_FIX` (`streams_are_distinct` covers them).
- `camp::regions` now builds the "within `region_water_max` of water" test as a grid once, instead of
  scanning every water tile per candidate. The predicate is the same (identity PASS); on 192² it was
  ~0.5 G operations.
- Rules (`WorldRules::colony`) = baseline + `colony = Some(ColonyRule::M1)`, `shelter_planning = None`,
  `fission = None`, `move_interval_ticks = 3`. No other difference.

## D18 — Baseline dumps: only off-valued fields added

- `rust/crates/sim-core/tests/data/v7_final_rules_seed{0,1,7,42}.txt`: `colony: None` added (new field
  `WorldRules::colony`).
- `v7_final_worldgen_seed{0,1,7,42}.txt`: `colony: None` added (new field `WorldGenConfig::colony`).

## D19 — Parts 1–3 simulation code landed together

- Context: the job tests (Part 2) and the replay test (Part 1) run on the colony world (Part 3's worldgen).
  Commands use the jobs module's spill and cancel.
- Choice: one commit for the simulation code of Parts 1 and 2, with the colony worldgen and
  `WorldRules::colony`. Part 3's time pacing, the switch of `build_game_engine`, the chronicle line and
  calibration follow in their own commits.
- Reason: every commit builds and passes; splitting would have needed non-compiling intermediates.

## D20 — A latent V7 path fixed: memory's "bias-only Construction" arm is off in the colony

- Context: calibration round 0 (D23) had exactly one dehydration death on each of seeds 0–7 with the U-script
  and none without it. A trace of seed 0 (`$DATA/calib/`, a scratch test, deleted after use) showed it:
  - agent 8 built the hut and then, standing at (91, 102), entered `Seeking { ConstructionSite }` with no
    site anywhere (t820);
  - it stayed there until it died of thirst (t2183).

  The decision's memory flip adds a Construction arm "even when no active ECS ConstructionSite exists" if
  the agent remembers construction, and only when `shelter_planning` is `None`. That never happens in the
  baseline world, where planning is on, but it is always the case in the colony. Nothing ever takes an
  agent out of a site-less `Seeking { ConstructionSite }`.
- Choice: that arm also requires `resources.rules.colony.is_none()` (one line in `decision/system.rs`,
  counted in the seam: `26 1` lines). Colony construction goes through jobs.
- Reason: the colony has no V7 sites to seek; baseline is unchanged (identity PASS).
- Evidence: round 1 (`$DATA/calib/r1_script`, `r1_noorders`): 0 deaths on seeds 0–7 with and without orders.

## D21 — Identity accepts a chronicle kind vocabulary that grew at its end

- Context: `HutBuilt` (8.4) is chronicle kind 22. Every chronicle file the probe writes carries the `kinds`
  list, so a strict equality check fails although no entry changed. The testing skill already says to
  compare chronicles by content and to require the base list to be a prefix of the candidate's (lesson
  2026-10-18).
- Choice: the comparison moved out of `identity.sh` into `tools/v8/compare_chronicles.py`, which allows
  exactly that and nothing else. `test_compare_chronicles.py` covers: equal, grown at the end, a changed
  entry fails, and a reordered or shrunk list fails.
- Evidence: identity PASS (`$DATA/identity/p1_try2`: 64 files equal by content, 19,683 entries).

## D22 — Time pacing: 12 ticks per real second at 1×, at most 8 per frame

- 8.2: `FIXED_DT = 1/12 s` (`TICKS_PER_SECOND_AT_1X = 12.0`), `MAX_TICKS_PER_FRAME = 8`, speeds 1×–4× feed
  12/24/36/48 ticks per second, a day lasts 120/60/40/30 s (`ffi::seconds_per_day`). A backlog over 8 ticks
  is still dropped. The keys `1`–`4` set 1×–4× (`camera_controller.gd`), and `P` pauses. No simulation state
  changes (identity PASS); ADR 0004 records the tick definition.

## D23 — Calibration (8.3, seeds 0–7)

- Round 0 (`$DATA/calib/r0_script`, the M1 values of 6.1, `move_interval_ticks` 3, no other change):
  - U1 hut placed at 360, finished at 704–891;
  - U2 all 15 trees felled by 1,456–1,749;
  - U3 0 loose logs at day 5;
  - 0 ledger or reservation violations, 0 stuck cancels;
  - but 1 dehydration death on every seed (D20).
- Round 1, after the D20 fix:
  - with the script: U1–U3 met on 8/8, 0 deaths;
  - without orders: 0 deaths on 8/8.

  No calibration value changed. Two rounds of the allowed six were used. All seeds have 19 births and
  29 people at day 10 (the V7 provisioned births at full food; recorded for M1.5).

## D24 — Keys: 1–4 set the speed, F1–F5 choose the order mode

- Context: 9.6 says keys `1`–`5` select an order mode. The owner's in-game check (14.2) says "1~4 키로 속도가
  바뀐다", and V8-0 already used 1–4 for speed. Both cannot hold.
- Choice: `1`–`4` keep the speed (1×–4×, the owner's check and habit). The order modes take `F1`–`F5`; the
  bar shows the key on each button ("F1 베기" …).
- Reason: the owner checks speed by pressing 1–4; the bar's buttons are the main way to choose a mode.
- Evidence: `scripts/world/camera_controller.gd`, `scripts/ui/hud/order_bar.gd`. Listed in the report under
  "오너 결정 필요".

## D25 — The colony view's structure

- World coordinates: tile (x, y) covers world pixels [16x, 16x+16) × [16y, 16y+16). The V7 offset (448, 28) is
  gone (`scripts/world/colony/view.gd`).
- Nodes under `Main/ColonyView`:
  - TerrainLayer (TileMapLayer, z 0);
  - TreeLayer (TileMapLayer, z 2, with a child drawing bushes and deadwood);
  - BuildingLayer (z 3);
  - OverlayLayer (z 4: zones, stacks, chop marks, ghosts over the canopies);
  - AgentLayer (z 5, y-sorted pooled people);
  - OrderTool (z 6).

  Above them sit the UI panels and `UI/OrderBar`. `DaylightController` stays (its glow origin moved to the
  new coordinates). The camera clamps to the map plus 4 tiles and snaps to whole screen pixels.
- Layers redraw only on change: `get_colony_versions` for trees and the overlay, a hash of the building
  snapshot, and a hash of the resource amounts for bushes.
- `OrderTool` also takes over V8-0's no-mode clicks (person → inspector, tile → causal panel). It replaces
  `world_renderer.gd`; that file's Space-cycled influence overlay was a V7 debug view and is gone.
- The inspector's portrait copies the colony person's three layers at 6×. The V7 portrait read
  `Main/WorldObjects`, which no longer exists.
- `SettlementSelection.observe` moved from the deleted agent renderer to `AgentLayer` (V8-0 D31: it must keep
  a caller).

## D26 — What Part 4 deleted and what it kept

- Deleted:
  - 21 V7 renderer and helper scripts (with their `.uid`), found unreachable with `tools/v8/gd_reach.py` after
    the scene swap: terrain, nature, nature layout and canopy, building, agent and world renderers; activity
    icons, agent activity, frame choices, pose moments, selection index, season tint, camp seats, carry layer,
    cold effects, declump and walker offsets, morning chill, offscreen marker, shelter cold;
  - `scripts/world/cold_audit.json`;
  - `shaders/settlement_outline.gdshader`;
  - the bridge render kernels `art_declump` / `art_walker_offsets` with `ffi/art_kernels.rs`, which nothing
    calls any more (API list rewritten).
- Kept:
  - `art_sprite_library.gd` (UI icons), `art_hash.gd` (names), `porter_presentation.gd` (inspector) and
    `wood_visuals.gd` (daylight), which kept panels still use;
  - `shaders/palette_swap.gdshader`, a V8-0 ASK item: the owner decides what VISION §5 means;
  - the V7 sprite and atlas assets that are now unreachable (`assets/sprites/**`, `assets/art/atlas/carry_*`,
    `assets/art/terrain/*`, `assets/art/production/U1/04.png`): V8-0 left sprites as ASK. Listed in
    "오너 결정 필요".

## D27 — UI scripts reach `Locale` through `/root/Locale`

- Context: the strict check parses each script alone, without autoloads, so a bare `Locale.trf(...)` fails
  with "Identifier not found: Locale". The kept panels already use `get_node_or_null("/root/Locale")`.
- Choice: `Locale.trf` / `Locale.format_text` live on the autoload script. New scripts call them through the
  node, or through `preload("res://scripts/core/locale.gd").format_text` for pure text
  (`chronicle_text.gd`). `tools/v8/test_locale_trf.gd` tests both paths.
- Evidence: strict check 25 files clean; `TRF_TEST_PASS`. Lesson added to `worldsim-godot` in Part 6.

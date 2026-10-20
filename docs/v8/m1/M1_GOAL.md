# GOAL: WorldSim — V8 M1: the first colony you command (verify V8-0 on screen, then: a 192×192 colony of 10 founders; chop / stockpile / hut orders; logs that are carried, stored and built with; a dark top-down view with one person per tile)

Repository `hyunlord/new-world`. Base: **`v8/main` = `v8/wip` = `24e166771be4a009b814a30ae4ecf8fa42475119`** (V8-0, gate green at `672fffef`). Tags `v7-final-sim` → `354e4a13` and `v7-final-art` → `b1ca032f`.

This run is executed by **Claude Code**, unattended, on the Mac, in `/Users/rexxa/work/new-world-pass4`. No DGX. No other run uses the Mac.

## Why this pass exists

V8-0 left a clean tree that still plays the V7 game:
- 18–30 founders on a 64×64 map;
- everyone acts on their own and the player has no verbs;
- big 3/4 sprites;
- a day passes in 48 seconds at 1×.

M1 is the first pass where the game becomes what `docs/VISION.md` describes: the player gives orders, and people carry them out with their own hands. The owner's acceptance sentence (ROADMAP M1) is:

> 나무를 베라고 하면 사람들이 베어 나르고, 청사진대로 오두막이 올라간다.

M1 is deliberately small in content and strict in foundations. It builds the four pieces every later milestone stands on:
1. **Orders as recorded commands:** deterministic and replayable.
2. **Physical items:** logs on the ground, in hands, in stockpiles and on building sites, with a conservation ledger.
3. **A job layer:** reservations, cancellation, and a narrow, documented seam into the existing decision system. The 2,900-line tick function in `decision/system.rs` must not grow into a job system.
4. **A new view:** 16-px tiles drawn at 2×, one person per tile, the dark earthy palette of view B (ADR 0002).

Before any of that, **Part 0 verifies V8-0 on screen.** The owner asked for this: the god hand and overlays are gone, the world is the same, and clicking people and tiles works. Part 0 also closes V8-0's two unverified doc fixes.

**Two worlds from now on:**
- **`baseline`** stays the V7 world, bit-identical. Its identity check (`tools/v8/identity.sh`) must still pass at the end of M1. It proves that everything new is gated and changed nothing old.
- **`colony`** is the new game world. `build_game_engine` switches to it.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout v8/wip && git merge --ff-only origin/v8/wip   # 24e16677
mkdir -p docs/v8/m1 docs/images
cp ~/Downloads/claude_goal_worldsim_v8_m1.md docs/v8/m1/M1_GOAL.md
cp ~/Downloads/mockup_gen_reference.py docs/v8/m1/mockup_gen_reference.py
cp ~/Downloads/mock_B_near.png docs/images/v8_target_view_b.png
claude --dangerously-skip-permissions
# then: /goal <completion condition naming docs/v8/m1/M1_GOAL.md>
```
- **Read first:** `AGENTS.md` (canonical; `CLAUDE.md` imports it) and the skills it lists. They are now the project's rules, and this document only adds to them.
- **Hooks:** `.claude/settings.json` (local `{}`, skip-worktree) stays untouched. The local `.gitignore` hunk and `.ignore` stay uncommitted; 5.3 says how to commit the agreed rules without them.
- **Commits:** this document is `docs/v8/m1/M1_GOAL.md`. Commit it, the reference script and the image unchanged in Part 0.
- **Re-read:** read the relevant section again at the start of every Part and after every compaction.
- **Final message:** the section 15 report, the U table, commit hashes, and the `git ls-remote` heads of `v8/wip` and `v8/main`.

## 0.2 Autonomy
1. **Never ask the owner anything.**
   - Decisions go in `docs/v8/m1/M1_DECISIONS.md`, numbered, each with context, choice, reason and evidence.
   - Questions only the owner can settle go to the report's "오너 결정 필요".
2. **Do not end your turn** between Parts or while a build, probe, capture or gate runs.
3. **Parts:**

   | Part | Section | Content |
   |---|---|---|
   | 0 | 5 | V8-0 on screen, housekeeping, base data |
   | 1 | 6 | items, trees, zones, designations, commands |
   | 2 | 7 | jobs |
   | 3 | 8 | the colony world, time pacing, calibration |
   | 4 | 9 | the new view and the order UI |
   | 5 | 10 | evaluation |
   | 6 | 11 | docs, skills, review, gate, report |

   Push after every Part.
4. **Review loops:** two rounds at most.
5. **On failure:** work around it, record it, continue. A bar missed is reported as missed.
6. **Time rule:** after **16 hours** of working time (pauses excluded and recorded), finish the current step, push, and go to Part 6.
7. **Drop order:** 9.6 (inspector job lines) → 8.4 (chronicle line) → U7's FPS measurement at 4× zoom → 9.5's far-zoom tree impostor.
   - **Never drop:** Part 0, baseline identity, the command log and replay, the item ledger, reservations and cancellation, the hut, the colony world, the new view at 2×, the order bar, U1–U6, docs and skills, the gate.
8. **Mac sleep:** record the pause and continue.

---

# 1. MACHINE
- **Work only in** `/Users/rexxa/work/new-world-pass4` and a base worktree `../new-world-m1-base` (detached at `24e16677`).
- **Never touch:** other checkouts, `~/github/worldsim-training`, `~/github/new-world-wt/lead`, ComfyUI, ports 3100/3200/8188, or processes you did not start. Never signal a process you did not start.
- `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`.
- **Data:** `/Users/rexxa/work/m1-data/`.
- **Windows:** windowed Godot runs are **authorized and required** (Part 0, Part 4, U7), with `--rendering-driver opengl3 --resolution 1920x1080` as `tools/v8/capture.gd` documents. Close every window you open.

# 2. GIT
- **Branches:** commit to **`v8/wip`**. `v8/main` moves only by fast-forward to a commit whose gate (11.3) is green.
- **Never:** push `lead/main`, `main` or `codex/*`; force-push; delete or move remote refs; rewrite history; `reset --hard`.
- **Staging:** explicit paths only.
- **Messages:** `v8-m1(pN): …`.
- **After every push:** run `git ls-remote origin v8/wip v8/main`.

---

# 3. CONTEXT (planner's survey at `24e16677`; paths under `rust/crates/` unless stated; verify each fact)

## 3.1 Clock and engine
- One tick is one simulated minute; a day is 1,440 ticks.
- The bridge feeds the engine at a fixed **30 ticks per real second at 1×** (`sim-bridge/src/ffi/node.rs` L101 `FIXED_DT = 1.0 / 30.0`), with at most 5 ticks per frame. A day therefore passes in 48 s at 1×.
- `set_sim_speed` scales the fed time, 0–4×.
- `docs/ARCHITECTURE.md` §2 says ADR 0004's tick definition is still open.

## 3.2 World
`sim-systems/src/worldgen/mod.rs`:
- `WorldGenConfig::baseline(seed)`: 64×64, founders, bushes, deadwood, forest and regions drawn from `variety::WorldVariety::draw` (founders 18–30);
- `stress(seed, agents, side)`;
- `build_world_with(&gen, rules)`.

`WorldRules::baseline` lives in `sim-core/src/rules/baseline.rs`. The rule groups are `rules/mod.rs` plus one file per group. The baseline dumps are in `sim-core/tests/data/`.

## 3.3 Movement
`runtime/agent/movement.rs` (priority 120):
- one **signum step toward `SeekTarget.tile`** every `rules.agent.move_interval_ticks` (baseline **8**);
- Brownian when idle;
- frozen while `Consuming`;
- **no pathfinding**, and the step does not test passability.

## 3.4 Decision, states, targets
- `AgentState { Idle, Seeking { target }, Consuming { target } }`.
- `TargetKind { Food, Water, Sleep, ConstructionSite, Agent(AgentId), GatherFood, GatherWood, Activity }` (`components/agent_state.rs`).
- `ActivityKind { TendFire, Craft, Repair, HearthCircle, Explore, Play, SitByFire }` (`components/activity.rs`); these are the leisure activities of the daily routine (`rules/routine.rs`, `runtime/decision/routine/`).
- `AgentDecisionSystem` (priority 125) is `runtime/decision/system.rs`: 3,462 lines, one tick function of about 2,900.

## 3.5 Construction
- `components/construction.rs`: `BuildingBlueprint { id, footprint_width, footprint_height, required_progress }` and `ConstructionSite { blueprint, progress, position }`. `components/building_kind.rs` has `shelter_blueprint` (1×1).
- `ConstructionSystem` (priority 133) advances a site for co-located agents in `Consuming { ConstructionSite }`. On completion it registers the building (`building_registry`, `building_kinds`, `building_owner`) and **sleep tiles**.
- `ShelterPlanningSystem` (priority 143) plans shelters automatically; it is gated by `rules.society.shelter_planning`.
- **No material is consumed by any construction today.**

## 3.6 Resources
- `components/inventory.rs`: `Inventory` holds `BTreeMap<ResourceKind, u32>` with cap 10.
- `ResourceKind { Food, Water, Wood, Stone }`.
- Settlement stock is an abstract `BTreeMap<ResourceKind, u32>` (`components/settlement.rs` `stockpile`).
- Deadwood lives in `SimResources.wood_tiles` / `wood_source_max` (firewood for the hearth).
- **There are no tree objects:** forest is a terrain fraction, and `scripts/world/nature_renderer.gd` / `nature_layout.gd` place trees visually.

## 3.7 Bridge and UI
- Commands today: `on_building_placed`, `reset_world`, `reset_world_stress`, `set_sim_speed`, `advance_ticks` (tooling).
- Every `#[func]` lives in `sim-bridge/src/ffi/node.rs`.
- Reads go through `scripts/core/sim_reads.gd`.
- `Locale.ltr(key)` is the only text API (`scripts/core/locale.gd` has no formatting helper).
- Renderers: `scripts/world/` (`terrain_renderer`, `nature_renderer`, `building_renderer`, `agent_renderer` plus about 21 V7 art helpers that V8-0 flagged "M1 replaces", D31).
- Panels: `scripts/ui/panels/` (chronicle, inspector, causal, `art_ui.gd`); `scripts/ui/hud/hud_topbar.gd`.
- `tools/v8/capture.gd` takes near/far shots and, with `--panels`, the inspector, causal and chronicle shots.

## 3.8 Experiments that are NOT reusable
`codex/inventory-foundation` (2026-03-13) and `codex/item-store-inventory-ui` (2026-03-14) are cut from the pre-V7 tree (their diff against `v8/main` deletes about 270k lines). Read them for ideas only; copy no code.

---

# 4. RULES

## 4.1 Baseline identity (unchanged world)
- **Every new behavior is gated by a rule** that is off (`None` / `false`) in `WorldRules::baseline`.
- `tools/v8/identity.sh` against the V8-0 base data must pass **at the end of every Part that touches `rust/**`**.
- The baseline dumps in `sim-core/tests/data/` change **only by added fields with their off value**. Each such change is listed in DECISIONS.

## 4.2 Determinism
- The colony world with a command script is deterministic. `--check-determinism` passes on 4 seeds with the U-script (10.1).
- Every collection added by M1 is a `BTreeMap` / `BTreeSet`, or a `Vec` in a documented order.
- Ties are always broken by `(y, x)`, then id.
- No float math in the sim crates.

## 4.3 The decision seam (hard limit)
- All job logic lives in `sim-systems/src/runtime/jobs/`.
- `decision/system.rs` may gain **at most 60 lines** (counted with `git diff --numstat`), and only to:
  1. let an agent that holds a job keep it where the decision would otherwise choose leisure (`Activity`) or idle wandering;
  2. cancel the job, through a function in `jobs/`, where the decision chooses anything else (food, water, sleep, foraging, firewood, a construction site of the V7 kind, social, fire, flight).
- An agent whose `Job` is `Build`, standing in `Consuming { ConstructionSite }` at its job's site, **counts as holding its job**. The decision keeps it there unless one of the non-leisure choices in item 2 fires (V7 shelter sites do not exist in the colony world).
- Record the exact seam (file, function, line range) in DECISIONS and in `worldsim-architecture`.

## 4.4 Integer arithmetic, permille, keyed draws
As AGENTS.md says.
- New draws use new purpose keys; no shared stream is consumed.
- The tree yield draw is keyed by `(seed, "tree_logs", x, y)`.

---

# 5. PART 0 — V8-0 on screen, housekeeping, base data

## 5.1 V8-0 on screen (the owner's request)
Build `v8/main` as it is (the baseline game).
- Write `tools/v8/verify_v8_0.gd`, extending `capture.gd`'s approach.
- Run it windowed for seed `DEFAULT` (the live game's seed), and the same script against the **V7 game at `v7-final-art`** in the base worktree. Adapt only what the V7 scene needs (paths), and record the adaptation.
- Write `docs/v8/m1/V8_0_SCREEN_CHECK.md` with the table and screenshot paths. Keep the shots in `$DATA/v8_0_check/`, not committed.

| # | check | pass condition |
|---|---|---|
| S1 | god hand gone | no node in the running scene tree has a script or scene path containing `god_hand`, and no node of the V7 list in `docs/archive/INDEX.md` → "7.3 UI" exists |
| S2 | overlays gone | the same test for activity badges/tooltips, village labels, relation and porter lines, seek/death/need/trail overlays, the food ledger |
| S3 | same world | at ticks 1,440 and 4,320 the top-bar values (population, shelters, food, wood, day, time) equal the V7 game's for the same seed, and the agent snapshot ids and positions are identical |
| S4 | person click | a scripted click on an agent opens the inspector. Its name, age and **village line** are non-empty and equal the bridge reads (V8-0's F1 fix) |
| S5 | tile click | a scripted click on an empty tile opens the causal panel with the tile's events |
| S6 | chronicle | toggling `C` shows the chronicle with ≥ 1 entry at tick 4,320 |
| S7 | console | the counts of `SCRIPT ERROR`, `ERROR:` and missing-locale lines are 0, or ≤ the V7 run's |
| S8 | FPS | 120 s at 3× in a window: mean FPS ≥ 0.95 × the V7 game's on the same machine and seed (report both) |

**If a check fails:** fix it in Part 0 with its own commit and record it. Identity must still pass.

## 5.2 The two unverified doc answers
Run one fresh `general-purpose` subagent that gets only `README.md`, `docs/**` and the skills, and asks V8-0's Q3 ("how to add a bridge read") and Q7 ("what is the time scale"). Fix the docs until both answers are correct and complete, two rounds at most. Note that Part 3 changes the time scale; update Q7's answer then too.

## 5.3 Housekeeping the owner decided (2026-10-20)
- **`.uid` files are committed.** Commit every `.uid` that Godot generates for a tracked script, scene or resource. Add a `check_docs.sh` step that fails when a tracked `.gd` / `.gdshader` / `.tscn` lacks its `.uid` or a `.uid` has no source.
- **Agreed `.gitignore` rules:**
  - ignore `docs/**/*.png` and `docs/**/*.jpg` except `docs/images/**`;
  - remove the three `.uid` exception lines for files that no longer exist.
- **Staging the `.gitignore` change** without the owner's local hunk:
  1. Edit the working file, keeping the local hunk.
  2. Run `git diff .gitignore > $DATA/gi.patch`.
  3. Remove the local hunk from the patch.
  4. Run `git apply --cached $DATA/gi.patch`.
  5. Verify that `git diff --cached .gitignore` shows only the agreed lines and `git diff .gitignore` still shows the local hunk.
- **Remote branches:** do nothing. The owner deletes them.

## 5.4 Base data
In the base worktree at `24e16677`:
- the identity base data (as V8-0's Part 0; `tools/v8/identity.sh` documents the command);
- the perf base (`tools/v8/perf.sh`);
- the smoke counts (`tools/v8/smoke.sh`).

Store them in `$DATA/base/`. Copy the compared numbers into `docs/v8/m1/M1_BASE_NUMBERS.md`.

---

# 6. PART 1 — Items, trees, zones, designations, commands (sim data + commands; no jobs yet)

## 6.1 New data
In `sim-core`; one file per concern under `components/` or a new `colony/` module. You choose and record which.

```rust
pub enum ItemKind { Log }                       // M2 adds materials; keep the enum open (non_exhaustive not needed)
pub struct ItemStack { pub kind: ItemKind, pub count: u16 }
pub struct Tree { pub logs: u8 }                // logs it yields when felled
pub enum Designation { Chop }
pub type ZoneId = u32;
pub enum ZoneKind { Stockpile }
pub struct Zone { pub id: ZoneId, pub kind: ZoneKind, pub tiles: BTreeSet<(u32, u32)> }
pub enum BlueprintKind { Hut }
pub struct MaterialNeeds { pub kind: ItemKind, pub needed: u16, pub delivered: u16 }   // component on a colony ConstructionSite entity
pub struct Carry { pub kind: ItemKind, pub count: u16 }                               // component on an agent
pub enum Command {
    DesignateChop { x0: u32, y0: u32, x1: u32, y1: u32 },
    DesignateChopTiles { tiles: Vec<(u32, u32)> },          // scripts and tools; same effect per listed tree tile
    CancelArea { x0: u32, y0: u32, x1: u32, y1: u32 },
    AddStockpile { x0: u32, y0: u32, x1: u32, y1: u32 },
    RemoveZoneArea { x0: u32, y0: u32, x1: u32, y1: u32 },
    PlaceBlueprint { kind: BlueprintKind, x: u32, y: u32 },  // top-left of the footprint
    SetDrafted { agent: AgentId, drafted: bool },            // reserved for M5; rejected in M1 (counted)
}
```

**`SimResources` additions** (all empty in baseline):
- `trees: BTreeMap<(u32, u32), Tree>` and `trees_version: u64`;
- `ground_items: BTreeMap<(u32, u32), ItemStack>` (one stack per tile);
- `zones: BTreeMap<ZoneId, Zone>`, `zone_of_tile: BTreeMap<(u32, u32), ZoneId>`, `next_zone_id`;
- `designations: BTreeMap<(u32, u32), Designation>`;
- `reservations: BTreeMap<ReserveKey, AgentId>`, where `ReserveKey = Tree((u32, u32)) | Stack((u32, u32)) | Cell((u32, u32)) | Site((u32, u32))`;
- `command_log: Vec<(u64, Command, CommandResult)>`;
- `item_ledger: ItemLedger { created, consumed }` (per kind, `u64`);
- `overlay_version: u64` (bumped by any change to zones, designations, ground items, sites or trees).

**`rules.colony: Option<ColonyRule>`.** `None` in baseline. Values in the colony world:

| field | value | meaning |
|---|---|---|
| `tree_logs_min` / `tree_logs_max` | 3 / 5 | keyed draw per tree |
| `chop_ticks` | 90 | work at the tree |
| `carry_logs` | 5 | per trip |
| `stack_max_logs` | 25 | per tile |
| `hut_footprint` | (3, 3) | |
| `hut_logs` | 12 | |
| `hut_progress` | 240 | work ticks once all logs are delivered |
| `hut_beds` | 4 | sleep capacity at the hut's centre tile |
| `job_scan_radius` | 96 | Chebyshev |

Calibration (8.3) may change these values **only** on the calibration seeds, and each change is recorded.

## 6.2 Trees
In the colony worldgen, every forest tile that is not inside the camp clearing gets one `Tree` with `logs` from the keyed draw. The trees are the forest that both the sim and the renderer use.
- A felled tree is removed. The tile's terrain stays forest; regrowth is not in M1.
- Forest stays passable, as today.

## 6.3 Commands
`sim-systems/src/commands.rs` exports `pub fn apply_command(world: &mut World, res: &mut SimResources, cmd: Command) -> CommandResult`. It is the only path that mutates orders.
- **Validation and effect:**
  - `DesignateChop`: designates every tree tile in the rect that is not yet designated. Returns the count.
  - `CancelArea`:
    - removes chop designations;
    - removes blueprint sites whose footprint intersects the rect;
    - drops each removed site's delivered logs as ground items at its tiles, using the same merge and spill rule as 7.4;
    - releases the reservations of everything it removed and cancels the jobs that held them (7.5).
  - `AddStockpile`: adds tiles that are in bounds, not water, not mountain, not under a building or site, and not already in a zone. Adjacent new tiles join one new zone. Returns the tile count.
  - `RemoveZoneArea`: removes tiles from zones and deletes zones left empty. Logs on removed tiles stay where they are.
  - `PlaceBlueprint(Hut)`: valid if every footprint tile is in bounds, not water or mountain, has no tree, building, site or zone tile, and is not reserved. Spawns a `ConstructionSite` with `BuildingBlueprint` 3×3 and `required_progress = hut_progress`, plus `MaterialNeeds { Log, hut_logs, 0 }`. The site's `Position` (and every "site tile" in this brief) is the **footprint centre** `(x + 1, y + 1)`; the command's `(x, y)` is the top-left.
  - `SetDrafted`: rejected in M1.
- **Results:** `CommandResult { accepted: bool, count: u32, reason: Option<RejectReason> }`.
- **Timing:** the bridge applies a command **immediately** and appends `(current_tick, cmd, result)` to `command_log`. A command applied between ticks t−1 and t is replayed before tick t's systems. That is deterministic and works while paused.

## 6.4 Replay
- `worldsim_probe --scene colony --commands <file.json>` applies the scripted commands at their ticks. JSON form: `[{"tick": 360, "cmd": "designate_chop", "x0": …}, …]`.
- `--commands-out` writes the log.
- Test: record the log of a run with a script, replay it, and require byte-identical summaries.

## 6.5 Bridge
**Commands:**
- `cmd_designate_chop(x0, y0, x1, y1) -> Dictionary`
- `cmd_designate_chop_tiles(xs: PackedInt32Array, ys: PackedInt32Array)`
- `cmd_cancel_area(…)`
- `cmd_add_stockpile(…)`
- `cmd_remove_zone_area(…)`
- `cmd_place_blueprint(kind: String, x, y)`

Each returns `{accepted, count, reason}`.

**Reads:**
- `can_place_blueprint(kind, x, y) -> bool`
- `get_trees() -> {version, xs, ys}`
- `get_colony_overlay() -> {version, designation_xs/ys, zone_ids, zone_xs/ys, item_xs/ys/kinds/counts, site_xs/ys/ws/hs/kinds/needed/delivered/progress/required}`
- the agent snapshot gains `job` (string code), `carry_kind` and `carry_count`, as columns appended at the end.

Readers return immediately when `version` is unchanged, so GDScript can skip rebuilding.

**API list:** add `docs/v8/bridge_api_m1.txt` (the expected list) and update gate step 13's command.

## 6.6 Tests (Part 1)
- Command validation table: one test per reject reason.
- Ledger after commands only: `created == on_ground + carried + delivered + consumed` (all 0 except blueprint cancels).
- Replay determinism.
- **Baseline identity.**

---

# 7. PART 2 — Jobs (`sim-systems/src/runtime/jobs/`)

## 7.1 Components and work types
```rust
pub enum WorkType { Build, Chop, Haul }          // fixed order in M1 = priority order
pub enum JobKind {
    Chop { tree: (u32, u32) },
    Haul { from: (u32, u32), to: (u32, u32) },
    Deliver { from: (u32, u32), site: (u32, u32) },
    Build { site: (u32, u32) },
}
pub enum JobStage { ToSource, Working { ticks_left: u32 }, ToDest }
pub struct Job { pub kind: JobKind, pub stage: JobStage, pub taken_at: u64 }
```
- **Eligibility:** adults, elders included. Not `Child`, not on a `PorterTrip`, not drafted (M5).
- **No skills in M1:** every worker has the same speed.
- **No priorities UI in M1:** every eligible agent does Build (Deliver + Build) > Chop > Haul, in that order.

## 7.2 `JobAssignSystem` (priority 124, interval 1; acts on agents without a `Job`)
An agent is **available** when all of these hold:
- it is eligible;
- it holds no `Job`;
- its `AgentState` is `Idle`, or `Seeking`/`Consuming { Activity }` (leisure);
- `rules.colony` is `Some`.

That is: **jobs replace leisure and idle time, never survival work.** Hunger, thirst, sleep, foraging, firewood and fire stay the decision system's.

For each available agent, in ascending `AgentId`, take the first work type that yields a target within `job_scan_radius`, nearest by Chebyshev distance, ties by `(y, x)`:

| work type | target and reservations |
|---|---|
| Build: Deliver | a site with `delivered + reserved_in_transit < needed`, and a log stack anywhere (stockpile stacks first, then loose ones; nearest to the agent) that is not reserved as a `Stack`. Reserve `Stack(from)` and an in-transit amount on the site, min(`carry_logs`, still needed). |
| Build: Build | a site with `delivered == needed` and no `Site` reservation. Reserve `Site`. |
| Chop | a designated tree that is not reserved. Reserve `Tree`. |
| Haul | a log stack on a tile not in a stockpile zone and not reserved, plus a destination cell in a stockpile zone (empty, or a same-kind stack below `stack_max_logs` minus reserved incoming). Choose the cell nearest the stack. Reserve `Stack(from)` and `Cell(to)`. |

On success: insert `Job`, set `SeekTarget` to the source tile, and set `AgentState::Seeking { target: TargetKind::Job }`. Add the variant `TargetKind::Job`; baseline never produces it.

## 7.3 `JobExecuteSystem` (priority 127, interval 1)
For agents holding a `Job`:
- **`ToSource` reached** (agent on the source tile; a tree tile counts as reached when on it or 8-adjacent):

  | job | action |
  |---|---|
  | Chop | `Working { chop_ticks }` |
  | Haul / Deliver | pick up min(`carry_logs`, stack, reserved amount) into `Carry`; shrink the stack, removing it at 0; go `ToDest` |
  | Build | hand over to the V7 flow: `AgentState::Consuming { ConstructionSite }` at the site tile, so `ConstructionSystem` advances `progress` exactly as for shelters. The job ends when the site completes or the agent leaves that state. |

- **`Working` reaches 0** (Chop):
  - remove the tree and its designation, and bump `trees_version`;
  - add `tree.logs` to `item_ledger.created`;
  - spawn the logs on the tree tile (merge and spill, 7.4);
  - the job ends and the reservation is released.
- **`ToDest` reached:**
  - Haul: drop into the cell (merge; overflow spills by 7.4).
  - Deliver: add to `delivered`. Overflow beyond `needed` is dropped on the nearest free footprint-adjacent tile.
  - The job ends.
- **Movement:** jobs set `SeekTarget` and `Seeking`; `AgentMovementSystem` steps the agent as for any seek. The system checks arrival after movement each tick.

## 7.4 Merge and spill
- Putting n logs on a tile: if it holds a same-kind stack below the max, add up to the max.
- The rest goes to the nearest tile in ring order (Chebyshev rings; within a ring, `(y, x)` ascending) that is:
  - in bounds, not water or mountain;
  - not under a building or site;
  - empty, or holds a same-kind stack below the max.
- Logs are never destroyed.

## 7.5 Cancellation and release
`jobs::cancel(agent, reason)`:
- drops any `Carry` at the agent's tile (merge and spill);
- releases every reservation held by that agent;
- removes the `Job`, and resets `SeekTarget` and state to `Idle`.

**Called when:**
- the decision seam chooses non-leisure (4.3);
- the target vanished (tree felled by someone else, stack gone, site cancelled);
- the agent died (despawn path; release only);
- a command removed the target.

**Stuck rule:** if a job's `ToSource` or `ToDest` has made no Chebyshev progress for 240 ticks, cancel it with reason `Stuck` and count it (`Tallies`).

## 7.6 Ledger and invariants (asserted in tests every tick, and in debug builds in the systems)
- **Log conservation:** `item_ledger.created == Σ ground + Σ Carry + Σ site.delivered + item_ledger.consumed`.
  - `consumed` grows by `hut_logs` when a hut completes.
  - At hut completion, `delivered` moves into `consumed`.
- **Reservations:** every reservation's agent exists and holds a `Job` that references the key. Every `Job` holds exactly its keys.
- **Stacks:** no stack exceeds the max; no stack sits on water, mountain or under a building.

## 7.7 Hut completion
`ConstructionSystem`'s completion path registers a building of a new `BuildingKind::Hut` (3×3 footprint, owned by the colony's settlement) and makes **the centre tile a sleep tile with capacity `hut_beds`**, through the same code path shelters use (name the path in DECISIONS).
- Footprint tiles stay walkable (M3 adds walls and pathing).
- `BedAssignmentSystem` sees the hut like a shelter of capacity 4. Verify with a test: after completion, four agents can be assigned beds there.

## 7.8 Tests (Part 2)
- `jobs_chop_one_tree`: one designated tree, one available agent → logs on the ground, tree gone, ledger exact.
- `jobs_haul_to_stockpile`: logs end in the zone, cells never over the max.
- `jobs_deliver_and_build_hut`: a hut with 12 logs available is completed, the beds exist, ledger exact.
- `jobs_two_agents_one_tree`: the reservation keeps the second agent off it.
- `jobs_cancel_on_hunger`: an agent whose hunger crosses the seek threshold drops its job within 1 tick; the carried logs land on its tile; reservations are released.
- `jobs_cancel_area_mid_delivery`: the logs return to the ground, the ledger is exact, and the jobs end.
- `jobs_stuck_cancel`.
- `jobs_children_never_work`.
- **Baseline identity.**

---

# 8. PART 3 — The colony world, time pacing, calibration

## 8.1 `WorldGenConfig::colony(seed)` and `WorldRules::colony(seed)`
- **Map:** 192 × 192. It is a multiple of the 32-tile geology chunk (ADR 0005 / `WorldSim_지하자원_4자합성.md`).
- **Founders:** exactly **10** (ADR 0003), from baseline's founder generator with `starting_agents = 10`. If it yields fewer than 7 adults, replace the youngest non-adults by adults drawn under a new purpose key (`"colony_founder_fix"`, keyed by slot) until 7 are adults; record how often this happens on seeds 0–7.
- **Features:**
  - forest, hill and mountain coverage = baseline's mean variety values (fixed, not drawn);
  - bushes and deadwood spots scaled by area: baseline mean × (192² / 64²), rounded;
  - the camp clearing at the map centre;
  - wild regions scaled by area.
- **Rules:** `WorldRules::colony` = `baseline` plus:
  - `colony = Some(ColonyRule { … })`;
  - `society.shelter_planning = None` (the player builds);
  - fission off (one colony until neighbours, ADR 0006);
  - **`agent.move_interval_ticks = 3`** (8 is unusable on a 192-tile map: 40 tiles would take 5 sim hours).
- Record every other difference from baseline in DECISIONS, with its reason.
- **Scenes:** the probe scene `colony` and `sim_test::colony_engine(seed)`. `build_game_engine(seed)` = colony.

## 8.2 Time pacing (ADR 0004, first half)
- One tick stays one sim minute. **At 1× the bridge feeds 12 ticks per real second** (a day = 120 s). The speeds 1×/2×/3×/4× feed 12/24/36/48 ticks per second, at most 8 ticks per frame.
- This changes no simulation state: identity is unaffected.
- **Update ADR 0004** with the decided tick definition. The calendar half (6-day seasons, 24-day year, age in years) stays open and is scheduled as **M1.5** in ROADMAP. M1 does not change the calendar.
- Update `docs/ARCHITECTURE.md` §2.

## 8.3 Calibration (seeds 0–7 only, pre-registered here)
Run the colony on seeds 0–7 for 10 days with no orders, then with the U-script (10.1). You may change, in this order and only as needed:
- `move_interval_ticks` (2–4);
- bush and deadwood density near the camp (within 0.5–2× of the area-scaled value);
- `ColonyRule` values (within 0.5–2×).

Aim for:
- no hunger, thirst or cold death in 10 days without orders on 8/8 calibration seeds;
- the U1–U3 bars met on the calibration seeds.

Stop after 6 calibration rounds. Record every round (change, outcome) in DECISIONS. **Evaluation (Part 5) uses fresh seeds 100–131 and never feeds back.**

## 8.4 Chronicle (droppable)
Add one chronicle kind `HutBuilt`, logged at completion with the settlement and the builders' ids. Keys in section 9.7.

## 8.5 Tests (Part 3)
- `colony_world_shape`: 192², 10 founders, trees > 0, no tree inside the clearing.
- `colony_viability_no_orders`: seeds 0–7, 10 days, 0 deaths.
- Determinism.
- **Baseline identity.**

---

# 9. PART 4 — The new view and the order UI (GDScript)

## 9.1 Target look
- `docs/images/v8_target_view_b.png` (the owner's chosen mock-up). `docs/v8/m1/mockup_gen_reference.py` is the PIL script that drew it: its palettes and tile painters are the reference.
- Hierarchy: **people and events > items and work spots > walls and floors > ground.** The ground is the calmest layer.

## 9.2 Temporary tileset
`tools/v8/gen_temp_tiles.py` (PIL, deterministic, unit-tested for output size and palette) → `assets/v8/temp/tiles.png` + `tiles.json` (name → atlas rect). The tiles:
- terrain 16×16: grass ×4 variants, dark grass ×2, dirt ×2, water ×2, shallow, hill, mountain (rock top), with edge-blending variants where cheap;
- tree top-down ×3, berry bush (full / bare), deadwood;
- log stack in 3 sizes (1–8, 9–16, 17–25);
- hearth, the V7 shelter (until M3 replaces it), the hut built (48×48) and the hut blueprint ghost;
- the person: **three greyscale layers 16×16** (skin, hair, tunic), drawn with `modulate`, plus a 16×5 ground shadow;
- carry icons, tool icons (axe, hammer);
- the designation mark (chop), the zone tint, the selection rectangle.

They are drawn in code, in the mock-up's style: no external art, no ComfyUI.

## 9.3 Camera
- Integer zoom only: 1×, 2× (default), 3×, 4×, with pixel snapping.
- Pan with WASD / arrows and middle-drag; zoom with the wheel and `-`/`=`.
- Start centred on the camp. Bounds = the map plus a 4-tile margin.

## 9.4 Layers (new files under `scripts/world/colony/`; each with the header block)

| file | draws |
|---|---|
| `terrain_layer.gd` | `TileMapLayer`, built once from `get_terrain_snapshot` |
| `tree_layer.gd` | a second `TileMapLayer`, rebuilt only when `trees_version` changes |
| `overlay_layer.gd` | zone tint and edges, chop marks, item stacks, blueprint ghosts with "delivered/needed" and progress — from `get_colony_overlay`, rebuilt on version change |
| `building_layer.gd` | hearth, shelters, huts |
| `agent_layer.gd` | one person per tile (layers + shadow + carry/tool icon). Positions are **tweened** between the previous and current tile over `move_interval_ticks` at the current speed. Tunic colour per household from a fixed 6-colour palette (by household id mod 6); hair from a 4-colour palette by agent id; elders get grey hair; children are drawn at 12×12. Selection outline. |

**Delete** the V7 renderers and their helpers once the colony layers replace them: `terrain_renderer`, `nature_renderer`, `nature_layout`, `building_renderer`, `agent_renderer`, the art helpers D31 listed, and `art_declump` / `art_walker_offsets` with their bridge kernels if nothing else uses them. Update the API list. Verify with `gd_reach.py`.

## 9.5 Performance
- Draw with tile layers and batched draws.
- At 1× zoom the whole map may show about 10,000 trees: if FPS drops below 60, draw trees at 1× as a flat tile variant (impostor; droppable).

## 9.6 Order UI
- **`scripts/ui/hud/order_bar.gd`:** a bottom bar with the modes 베기 / 비축 구역 / 오두막 / 구역 지우기 / 취소, plus Esc for none. Keys `1`–`5` select a mode.
- **`scripts/world/colony/order_tool.gd`:**
  - In an area mode, left-drag shows the rectangle and on release sends the command.
  - In 오두막, a ghost follows the cursor, green or red by `can_place_blueprint`; a click places it.
  - Right-click or Esc leaves the mode.
  - A short toast shows the result count or the reject reason.
- Clicks with no mode keep V8-0's behaviour (person → inspector, empty tile → causal panel).
- **Inspector additions (droppable):** a "지금" line from the job code, and "들고 있음" from the carry columns.
- Top bar: keep it. Add the speed shown as 1×–4× (12–48 ticks per second) and a pause indicator.

## 9.7 Localization (`localization/{en,ko}/ui.json`; recompile)

| key | en | ko |
|---|---|---|
| `UI_ORDER_CHOP` | Chop | 베기 |
| `UI_ORDER_STOCKPILE` | Stockpile | 비축 구역 |
| `UI_ORDER_HUT` | Hut | 오두막 |
| `UI_ORDER_REMOVE_ZONE` | Remove zone | 구역 지우기 |
| `UI_ORDER_CANCEL` | Cancel | 취소 |
| `UI_ORDER_HINT_AREA` | Drag to select an area | 드래그해서 영역을 고르세요 |
| `UI_ORDER_HINT_PLACE` | Click to place | 클릭해서 놓으세요 |
| `UI_ORDER_RESULT_COUNT` | {count} | {count}개 |
| `UI_REJECT_BLOCKED` | Something is in the way | 무언가 막고 있음 |
| `UI_REJECT_WATER` | Can't build on water | 물 위에는 지을 수 없음 |
| `UI_REJECT_OUT_OF_MAP` | Outside the map | 지도 밖 |
| `UI_REJECT_NOTHING` | Nothing to do here | 할 일이 없음 |
| `UI_JOB_NONE` | — | — |
| `UI_JOB_CHOP` | Chopping a tree | 나무 베는 중 |
| `UI_JOB_HAUL` | Hauling logs | 통나무 나르는 중 |
| `UI_JOB_DELIVER` | Bringing logs to a hut | 오두막에 통나무 가져가는 중 |
| `UI_JOB_BUILD` | Building a hut | 오두막 짓는 중 |
| `UI_INSPECT_NOW` | Now | 지금 |
| `UI_INSPECT_CARRYING` | Carrying | 들고 있음 |
| `ITEM_LOG` | Logs | 통나무 |
| `BUILDING_HUT` | Hut | 오두막 |
| `UI_BLUEPRINT_LOGS` | {delivered}/{needed} logs | 통나무 {delivered}/{needed} |
| `CHRONICLE_HUT_BUILT` | A hut was finished in {settlement} | {settlement}에 오두막이 완성됨 |
| `UI_SPEED_PAUSED` | Paused | 일시정지 |

- Add `Locale.trf(key: String, params: Dictionary) -> String`, which replaces `{name}` placeholders, with a GDScript unit test.
- Check for duplicate keys first, and use existing keys where they fit (record which).

## 9.8 GUI test (scripted, windowed)
`tools/v8/m1_gui_test.gd` drives the real scene with synthetic input events (`Input.parse_input_event`), not by calling the bridge directly:
1. select 베기 and drag over 15 trees near the camp;
2. select 비축 구역 and drag a 4×4 area next to the hearth;
3. select 오두막 and click a valid spot;
4. run 3 days at 4×.

It asserts each command's reads after the step and saves screenshots: `before`, `after_orders`, `day1`, `hut_done`, `far`. Shots go in `$DATA/gui/`, not committed; the paths go in the report.

---

# 10. PART 5 — Evaluation (fresh seeds 100–131; no feedback into calibration)

## 10.1 The U-script (a committed JSON template, resolved per seed)
At tick 360 (06:00, day 0):
- `DesignateChopTiles` with the 15 trees nearest the hearth (Chebyshev, ties `(y, x)`), resolved by the probe per seed;
- `AddStockpile` on the 4×4 block whose nearest corner is 3 tiles east of the hearth, shifted by the 7.4 ring rule until valid;
- `PlaceBlueprint(Hut)` at the nearest valid spot south of the hearth.

Record in DECISIONS how each target is resolved.

## 10.2 Bars (pre-registered here; report each with its per-seed values)

| # | bar | pass |
|---|---|---|
| U1 | hut completed within 3 game days of placement | ≥ 28 of 32 seeds |
| U2 | all 15 designated trees felled within 4 days | ≥ 28 / 32 |
| U3 | at day 5, every log is in a stockpile cell, carried, or delivered (no loose stack outside zones) | ≥ 28 / 32 |
| U4 | log ledger exact at every tick | 32 / 32 (any miss fails) |
| U5 | reservation invariants hold at every tick; 0 stuck cancels on 30 / 32 or more | 32 / 32 invariants |
| U6 | no death from hunger, thirst or cold in 10 days, with and without the script | 32 / 32 and 32 / 32 |
| U7 | performance: colony tick mean ≤ 0.6 ms and p99 ≤ 4 ms over 10 days with the script (`tools/v8/perf.sh` method, 4 lowest-id seeds); Godot mean FPS ≥ 60 at 2× zoom over 120 s at 4× speed, and at 1× zoom (report; droppable) | ms bars pass; FPS at 2× passes |
| U8 | baseline identity unchanged | pass |
| U9 | GUI test (9.8) passes end to end | pass |

---

# 11. PART 6 — Docs, skills, review, gate, report

## 11.1 Docs (the end-of-pass checklist in AGENTS.md, all of it)
- **CHANGELOG:** `## M1 — 2026-10-xx`.
- **ROADMAP:** M1 row → done with its commit. Add the **M1.5 calendar row** (6-day seasons, 24-day year, ages in years, aging and fertility recalibrated to years) and point "다음" at it or M2; record your recommendation.
- **HISTORY:** an M1 line.
- **ADRs:**
  - **0012 orders as recorded commands;**
  - **0013 jobs replace leisure, never survival; the decision seam;**
  - **0014 two worlds — `baseline` frozen for identity, `colony` is the game;**
  - **0004 updated** (tick definition).
- **ARCHITECTURE:** regenerated system table; the jobs data flow (command → designation/zone/site → assign → execute → ledger); the new bridge commands; the `scripts/world/colony/` layout; the item model.

## 11.2 Skills
- **Lessons:** every problem that needed more than one attempt goes into the skill it belongs to (date, symptom, cause, fix, source), in this pass.
- **Required additions:**
  - `worldsim-architecture`: "add a command", "add a job type", the decision seam, the ledger rule;
  - `worldsim-godot`: anything about `TileMapLayer`, tweening, synthetic input events and windowed capture that surprised you;
  - `worldsim-testing`: the replay and ledger tests as patterns.

## 11.3 Gate
`tools/v8/gate.sh` gains:
- step `m1_tests` (the Part 1–3 tests, by name);
- step `colony_determinism` (4 seeds with the U-script);
- step `replay`;
- step `uid_check` (5.3).

Steps 10 (baseline identity), 12 (perf) and 13 (API, against `bridge_api_m1.txt`) stay. Run the final gate on a commit with no edits during the run.

## 11.4 Independent review (two fresh `general-purpose` subagents)
- **Code reviewer:** gets this brief, the diff from `24e16677`, and the test and evaluation outputs. It looks for:
  - ledger holes;
  - reservation leaks;
  - non-determinism (hash iteration, float, unkeyed order);
  - decision-seam overreach (line count, logic outside `jobs/`);
  - baseline leaks (a new behaviour that runs in baseline).
- **Player's-eye reviewer:** gets the GUI shots, `docs/images/v8_target_view_b.png` and VISION §6. It answers:
  - Can you tell people from ground and from items at 2×?
  - Can you see who is carrying what?
  - Can you see what is ordered (marks, zone, ghost)?
  - What is the first thing a new player would not understand?

  Fix what is cheap. List the rest for M2 / art.

Address the findings or record them.

## 11.5 Publish
Gate green → fast-forward `v8/main` → write `docs/v8/m1/M1_REPORT.md` (section 15) → rerun the docs step → fast-forward again.

---

# 12. Not in M1 (do not build)
- skills and learning;
- materials beyond `Log`;
- stone, mining, hill digging;
- walls, doors, rooms;
- pathfinding;
- work-priority table UI;
- drafting (the command exists and is rejected);
- the calendar change (M1.5);
- neighbours and migrants;
- the storyteller;
- other blueprints;
- deconstruction;
- logs as firewood;
- tree regrowth;
- final art.

---

# 13. Invariants (tests that must exist at the end)
- `baseline_matches_v7_final` (updated only by off-valued fields);
- baseline identity;
- colony determinism with the script;
- replay;
- the 7.6 ledger, reservation and stack invariants over 8 seeds × 10 days;
- every 7.8 and 8.5 test;
- `gen_temp_tiles.py` tests;
- `Locale.trf` test;
- the GUI test;
- `uid_check`.

---

# 14. 인게임 확인사항 (오너용)

`v8/main`에서 게임을 켜고 확인할 것:
1. **새 화면:**
   - 어두운 흙빛 지도가 넓게 펼쳐져 있다.
   - 사람은 타일 한 칸 크기이고, 발밑 그림자 덕분에 땅과 구분된다.
   - 옷 색으로 가족이 구분된다. 노인은 흰머리, 아이는 작게 보인다.
2. **시간:** 1배속에서 하루가 약 2분이다. 1~4 키로 속도가 바뀐다.
3. **베기:**
   - 아래 막대에서 '베기'를 고르고 숲을 드래그하면 나무에 표시가 생긴다.
   - 잠시 뒤 사람들이 가서 베고, 그 자리에 통나무 더미가 생긴다.
4. **비축 구역:** 화덕 옆에 드래그하면 구역이 칠해진다. 통나무가 그리로 옮겨지는 게 보인다. 누가 무엇을 들고 가는지 손의 아이콘으로 보인다.
5. **오두막:**
   - 마우스를 따라 다니는 3×3 그림자가 놓을 수 있는 곳에서는 초록, 아닌 곳에서는 빨강이다.
   - 놓으면 "통나무 0/12"가 차오른다.
   - 다 차면 짓기가 진행되고, 다 지어지면 연대기에 "오두막이 완성됨"이 뜬다.
   - 밤에 사람들이 그 오두막에서 잔다.
6. **배고픔:** 일하던 사람이 배고파지면 일을 내려놓고 먹으러 간다. 들고 있던 통나무는 그 자리에 놓인다.
7. **취소:** 영역을 드래그하면 표시와 청사진이 사라지고, 가져다 둔 통나무는 땅에 남는다.
8. **기본 조작:** 사람 클릭 → 상세 정보("지금: 나무 베는 중", "들고 있음: 통나무 5"). 빈 땅 클릭 → '왜?' 패널.
9. **안정성:** 콘솔에 빨간 에러가 없다. 2배 확대에서 끊김이 없다.

---

# 15. 구현 완료 보고 (`docs/v8/m1/M1_REPORT.md`와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고 — V8 M1

### 구현 의도
(한 문장)

### V8-0 화면 검증 (Part 0)
(S1–S8 표: 기준 V7 값 / V8-0 값 / 판정, 스크린샷 경로; 문서 Q3·Q7 재검증 결과; .uid·.gitignore 정리)

### 구현 내용
(새 데이터, 명령, 일감, colony 세계, 시간 속도, 새 화면과 주문 막대 — 파일과 핵심 구조)

### 구현 방법
(명령 기록과 재생, 예약과 취소, 결정 시스템 접점(파일·줄 범위·추가 줄 수), 통나무 장부, 보정 기록 요약)

### 기능 설명
(오너가 하는 것과 사람들이 하는 것, 화면에서 보이는 것)

### 평가 결과 (시드 100–131)
(U1–U9 표: 기준 / 결과 / 판정, 시드별 값은 부록)

### 성능
(colony 틱 평균·p99, Godot FPS 1×·2× 확대, baseline 대비)

### 변경된 파일 목록
(커밋별)

### 리뷰 결과
(코드 리뷰 지적과 조치, 플레이어 눈 리뷰 답변)

### 확인된 제한사항
(경로 찾기 없음, 임시 타일셋, 남은 V7 코드, 드롭한 항목)

### 오너 결정 필요
(M1.5 달력 패스와 M2 중 다음 순서 권고, 리뷰가 남긴 화면 문제, 기타)

### Gate 결과
(단계별 통과/실패, 로그 경로)
```

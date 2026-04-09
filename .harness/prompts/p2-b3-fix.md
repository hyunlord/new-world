# P2-B3-FIX: Assign Builders for Wall/Furniture Plans (No Building Required)

## Section 1: Implementation Intent

### Symptom
P2-B3 shelter wall pipeline is completely inert in-game:
```
[P2-B3-DEBUG] tick=1000 wall_plans=0 (unclaimed=0) furniture_plans=0
              grid_walls=0 grid_floors=0 builders=0 placing_wall=0
[P2-B3-DEBUG] tick=1500 ... (same)
```
Not a single wall gets stamped for 1500+ ticks, even though shelter construction is supposed to begin as soon as adults exist.

### Root cause (fully traced, three-link chain)

**Link 1 — Building creation skipped** (`rust/crates/sim-systems/src/runtime/economy.rs:737-749`)
`ensure_early_construction_sites()` hands off shelter to the P2-B3 plan queue and
immediately `continue`s, so **no `Building` row is ever inserted** for a shelter:
```rust
if matches!(plan, EarlyStructurePlan::Shelter) {
    let center = resources.settlements.get(&settlement_id).map(|s| (s.x, s.y));
    if let Some((cx, cy)) = center {
        generate_wall_ring_plans(resources, settlement_id, cx, cy, tick);
    }
    continue;                               // ← no place_early_structure_site()
}
```

**Link 2 — Pending-sites scanner is Building-only** (`economy.rs:754-768`)
`collect_pending_site_targets()` only walks `resources.buildings`:
```rust
for building in resources.buildings.values() {
    if building.is_complete { continue; }
    out.entry(building.settlement_id).or_default().insert((building.x, building.y));
}
```
Because Link 1 creates zero shelter Buildings, this returns `HashMap::new()` for
every settlement that only needs a shelter.

**Link 3 — Builder assignment bails on empty map** (`economy.rs:787-791`)
```rust
fn ensure_pending_sites_have_builder(world: &mut World, resources: &SimResources) {
    let pending_sites = collect_pending_site_targets(resources);
    if pending_sites.is_empty() { return; }
    ...
}
```
So `retask_builder_for_construction()` is never called. No agent ever adopts
`job = "builder"` (the JobAssignment ratios don't elect one early: survival
ratios are gatherer-heavy and `alive_count < 10` keeps us in survival mode).
Cognition's `ActionType::PlaceWall` branch (`cognition.rs:705-712`) is gated on
`behavior.job == "builder"`, so `PlaceWall` is never selected → `tile_grid`
walls/floors stay at 0 forever. **The P2-B3 code path simply never runs.**

### Latent bug surfaced while tracing

Even if a builder *were* somehow assigned, `ensure_pending_sites_have_builder`
only counts `ActionType::Build` as "assigned":
```rust
if behavior.current_action == ActionType::Build && target_matches {
    status.assigned_builder_count += 1;
} else {
    status.available_builders.push(entity);
}
```
A builder mid-`PlaceWall` would be placed in `available_builders` and retasked
(`action_target_x/y` cleared, `current_action = Idle`) on the very next
`job_assignment_system` tick, thrashing the construction loop. This must also
be fixed in the same change — otherwise the fix appears to work for one tick,
then the builder gets retasked and we see the same symptom.

### Fix philosophy
Make the shelter plan path a **first-class pending site** without resurrecting
the legacy "shelter Building" concept:
1. `collect_pending_site_targets()` includes every `wall_plan.(x, y)` and
   `furniture_plan.(x, y)` (regardless of claim status — claimed plans still
   need the claimer recognized as assigned).
2. `ensure_pending_sites_have_builder()` recognizes `PlaceWall` and
   `PlaceFurniture` (alongside legacy `Build`) as "already assigned" action
   states, so active builders are not retasked.

This is surgical (~30 lines), preserves the P2-B3 design (no shelter Building),
and unblocks the already-implemented wall/floor/room pipeline.

### Why NOT the alternative fixes
- *Re-introduce a virtual shelter Building* — reverts P2-B2/B3 architecture
  decision and duplicates state (Building + WallPlan).
- *Force `builder` into JOB_ASSIGNMENT_SURVIVAL_RATIOS* — breaks gatherer
  survival balance; unrelated populations would get builder jobs with no work.
- *Touch cognition to pick PlaceWall without a `builder` job* — violates the
  job/action contract used by every other system (satisfaction, record, UI).

---

## Section 2: What to Build

### Files touched (ALL in `rust/crates/sim-systems/src/runtime/economy.rs`)
Exactly one source file changes. This is deliberate — the fix lives entirely
in `economy.rs`. Do **not** touch `cognition.rs`, `world.rs`, `biology.rs`,
config constants, RON data, or the engine debug log.

### Functions modified
| Function | Lines (approx) | Change |
|---|---|---|
| `collect_pending_site_targets` | 754-768 | Add two loops: include `wall_plans` + `furniture_plans` positions |
| `ensure_pending_sites_have_builder` | 787-855 | Widen "assigned action" check to `Build \| PlaceWall \| PlaceFurniture` |

### Ticket 2 (test file): `rust/crates/sim-test/src/main.rs`
Add ONE new harness test: `harness_p2b3_builder_assigns_and_places_wall`.
Do not modify any existing test.

### Explicit scope boundary
- NO new config constants
- NO new events
- NO RON data changes
- NO new locale keys
- NO new components
- NO changes to `cognition.rs`, the engine debug log, the job-ratio logic, or
  `cleanup_stale_plans`

---

## Section 3: How to Implement

### Step 1 — Extend `collect_pending_site_targets`

Replace the current body with:

```rust
#[inline]
fn collect_pending_site_targets(
    resources: &SimResources,
) -> HashMap<SettlementId, HashSet<(i32, i32)>> {
    let mut out: HashMap<SettlementId, HashSet<(i32, i32)>> = HashMap::new();

    // Legacy Building-based sites (stockpile, campfire, any residual
    // shelter Buildings that pre-date P2-B3).
    for building in resources.buildings.values() {
        if building.is_complete {
            continue;
        }
        out.entry(building.settlement_id)
            .or_default()
            .insert((building.x, building.y));
    }

    // P2-B3: wall/furniture plans are first-class pending sites. Unclaimed
    // plans force builder assignment; claimed plans keep the existing
    // claimer recognized as "assigned" via target_matches so the
    // retask-if-no-assigned-builder loop below does not pull a working
    // builder off PlaceWall / PlaceFurniture.
    for plan in &resources.wall_plans {
        out.entry(plan.settlement_id)
            .or_default()
            .insert((plan.x, plan.y));
    }
    for plan in &resources.furniture_plans {
        out.entry(plan.settlement_id)
            .or_default()
            .insert((plan.x, plan.y));
    }

    out
}
```

Notes:
- Include BOTH claimed and unclaimed plans. If we skipped claimed plans, the
  claimer's `(action_target_x, action_target_y)` would not appear in
  `targets`, the `target_matches` guard would fail, and the claimer would be
  bucketed as "available" → retasked on the next tick (exactly the latent bug
  described above).
- Do not dedupe building vs. plan positions; the `HashSet` handles that.

### Step 2 — Widen the assigned-action check

In `ensure_pending_sites_have_builder`, replace the existing builder-job
branch:

```rust
if behavior.job == "builder" {
    let is_construction_action = matches!(
        behavior.current_action,
        ActionType::Build | ActionType::PlaceWall | ActionType::PlaceFurniture
    );
    if is_construction_action && target_matches {
        status.assigned_builder_count += 1;
    } else {
        status.available_builders.push(entity);
    }
    continue;
}
```

Reasoning:
- `ActionType::Build` is the legacy path for Stockpile/Campfire — keep it.
- `ActionType::PlaceWall` / `PlaceFurniture` are the P2-B3 path — add them.
- `target_matches` already checks the builder's `action_target_{x,y}` against
  the tile set returned in Step 1. With Step 1 including plan coordinates,
  this gate now correctly recognizes wall-placing builders.

### Step 3 — Verify no other edits are needed

- `retask_builder_for_construction` already resets `current_action = Idle`,
  which is correct: the next cognition tick will pick `PlaceWall` via the
  existing branch at `cognition.rs:705-712` as soon as the builder's
  `has_wall_plan_target` flag is true.
- `cleanup_stale_plans` already handles orphaned claims; do not touch.
- `Behavior::occupation` fallback check (`is_empty() | "none" | "laborer"`)
  already gives us the pool of retask candidates.

### Step 4 — Harness test (RED → GREEN)

Add to `rust/crates/sim-test/src/main.rs`. Place this test directly after
`harness_component_building_wall_placement` (search for that test name to
locate the right spot). Do not edit that existing test — it's deliberately
soft on builder activity (A12 sampling) and this new test is the strict
counterpart that the fix must satisfy.

```rust
// ─────────────────────────────────────────────────────────────────────────
// P2-B3 FIX harness — proves the wall-plan -> builder -> PlaceWall pipeline
// actually runs. Paired with the diagnostic that showed
// `wall_plans=0 builders=0 placing_wall=0 grid_walls=0` in-game.
//
// Assertions (strict):
//   FX1: wall_plans grew past 0 at some sample point (economy generated plans)
//   FX2: builder count > 0 at some sample point (builder assignment fired)
//   FX3: at least one agent observed with current_action == PlaceWall
//        across the sampling window (NEW code path executed)
//   FX4: tile_grid wall_count > 0 by tick 4380 (walls actually stamped)
//   FX5: no shelter Building inserted (P2-B3 architecture preserved)
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p2b3_builder_assigns_and_places_wall() {
    use sim_core::ActionType;

    let mut engine = make_stage1_engine(42, 20);

    let mut saw_wall_plan = false;
    let mut saw_builder = false;
    let mut saw_place_wall_action = false;

    // Sample every 200 ticks up to 4380 (shelter deadline for single settlement).
    const SAMPLE_INTERVAL: u64 = 200;
    const SAMPLE_END: u64 = 4380;
    for _ in 0..(SAMPLE_END / SAMPLE_INTERVAL) {
        engine.run_ticks(SAMPLE_INTERVAL);

        let resources = engine.resources();
        if !resources.wall_plans.is_empty() {
            saw_wall_plan = true;
        }

        let world = engine.world();
        for (_e, behavior) in world.query::<&sim_core::components::Behavior>().iter() {
            if behavior.job == "builder" {
                saw_builder = true;
            }
            if behavior.current_action == ActionType::PlaceWall {
                saw_place_wall_action = true;
            }
        }
    }

    // FX1
    assert!(
        saw_wall_plan,
        "[FX1] wall_plans stayed empty for {} ticks — economy.generate_wall_ring_plans() never ran",
        SAMPLE_END
    );

    // FX2
    assert!(
        saw_builder,
        "[FX2] no agent ever had job==\"builder\" across {} ticks — \
         ensure_pending_sites_have_builder() never retasked anyone",
        SAMPLE_END
    );

    // FX3 — the critical NEW-code-path assertion the user asked for.
    assert!(
        saw_place_wall_action,
        "[FX3] no agent ever executed ActionType::PlaceWall across {} ticks — \
         wall plans exist + builders exist, but cognition never selected PlaceWall. \
         This means the builder is being retasked off PlaceWall before it runs \
         (check ensure_pending_sites_have_builder's assigned-action predicate).",
        SAMPLE_END
    );

    // FX4 — walls actually got stamped onto tile_grid.
    let resources = engine.resources();
    let (grid_w, grid_h) = resources.tile_grid.dimensions();
    let mut wall_count = 0_usize;
    for y in 0..grid_h {
        for x in 0..grid_w {
            if resources.tile_grid.get(x, y).wall_material.is_some() {
                wall_count += 1;
            }
        }
    }
    assert!(
        wall_count > 0,
        "[FX4] tile_grid wall_count == 0 after {} ticks despite PlaceWall firing — \
         check world.rs PlaceWall completion handler",
        SAMPLE_END
    );

    // FX5 — no shelter Building ever inserted (P2-B3 architecture guarantee).
    let shelter_buildings = resources
        .buildings
        .values()
        .filter(|b| b.building_type == "shelter")
        .count();
    assert_eq!(
        shelter_buildings, 0,
        "[FX5] {} shelter Building rows exist — P2-B3 must keep shelter as pure plan queue",
        shelter_buildings
    );
}
```

This test MUST fail on `main` as of today's HEAD (`1faa70e9`) and pass after
Steps 1 + 2 are applied. Running it before the fix is step one of RED → GREEN.

### Step 5 — HDD loop

1. Copy the new test into sim-test.
2. Run `cd rust && cargo test -p sim-test harness_p2b3_builder_assigns_and_places_wall -- --nocapture`.
   Expected: **fails on FX2 or FX3** (the chain we're fixing).
3. Apply Steps 1 + 2 to `economy.rs`.
4. Re-run the same command. Expected: **passes all 5 assertions**.
5. Full gate: `cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings`.

If any assertion fails on the GREEN run, do NOT loosen the assertion. Diagnose
the real cause. Allowed diagnostic aid: add `println!` statements **inside the
test body** (never in `economy.rs`), bracketed with a `// DEBUG` comment so the
Evaluator can see them and require removal before commit.

---

## Section 4: Dispatch Plan

| # | Ticket | File | Mode | Depends on |
|---|---|---|:---:|---|
| T1 | Extend `collect_pending_site_targets` + widen assigned-action check | `rust/crates/sim-systems/src/runtime/economy.rs` | 🔴 DIRECT | — |
| T2 | Add `harness_p2b3_builder_assigns_and_places_wall` | `rust/crates/sim-test/src/main.rs` | 🔴 DIRECT | T1 |

Why both DIRECT:
- Total change is <60 lines in two files, both within one crate boundary.
- The two edits are tightly coupled (T2 is the verification for T1) — splitting
  across parallel agents risks divergence on the assertion contract.
- Dispatch-ratio waiver: this is a regression hotfix with a surgical root
  cause. The harness pipeline's DRAFTER/CHALLENGER/EVALUATOR still runs per
  `--full` mode, so quality gating is not lost.

---

## Section 5: Localization Checklist

No new localization keys. No user-visible strings introduced. The debug log
line `[P2-B3-DEBUG]` already exists in `sim-engine/src/engine.rs` and is
explicitly out of scope.

---

## Section 6: Verification & Notion

### Gate commands (must all pass)
```bash
cd rust && cargo test -p sim-test harness_p2b3_builder_assigns_and_places_wall -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace -- -D warnings
```

### In-game verification (MANDATORY — user explicitly requested this)
After the gate passes, launch the headless engine and observe the existing
`[P2-B3-DEBUG]` log every 1000 ticks (`sim-engine/src/engine.rs:824-849`):

```bash
cd rust && RUST_LOG=info cargo run -p sim-test 2>&1 | grep "P2-B3-DEBUG"
```

Expected output at tick 2000 (or earlier):
```
[P2-B3-DEBUG] tick=2000 wall_plans=<>0 (unclaimed=<any>) furniture_plans=<>=0
              grid_walls=<>0 grid_floors=<>0 builders=<>0 placing_wall=<>=0
```

**Pass criteria (all must hold at the tick=2000 line or earlier):**
1. `wall_plans > 0` OR `grid_walls > 0` (plans generated or walls already stamped)
2. `builders > 0`
3. `placing_wall > 0` at some sample (PlaceWall executed — may be 0 at exactly
   tick=2000 if timer just reset; re-check tick=3000 if so)
4. `grid_walls > 0` by tick=3000 at the latest

**Fail criteria (any of these = fix incomplete):**
- All-zero line at tick >= 2000 → builder assignment still broken
- `wall_plans > 0` but `builders = 0` → Step 2 (assigned-action check) still
  wrong — builder is being retasked off PlaceWall
- `builders > 0` but `grid_walls = 0` after tick 3000 → world.rs PlaceWall
  handler is the culprit (NOT in scope; stop and report)

### Regression surface
- Existing `harness_component_building_wall_placement` test must still pass
  unchanged (soft sampling).
- Existing A13/A14/A15 assertions must still pass (stale cleanup + bounded
  queue + orphaned-claim handling).
- Job-ratio system must still respect survival ratios — this fix does NOT
  inject builders into `JOB_ASSIGNMENT_SURVIVAL_RATIOS`.

### Notion
Append a one-line entry to the P2-B3 Shelter Rework page describing the bug
and the two-function fix. Link the harness test by function name.

---

## Final Report Template (Generator → Evaluator)

```
## Files changed
- rust/crates/sim-systems/src/runtime/economy.rs (+~20 / -~3)
- rust/crates/sim-test/src/main.rs (+~100 / -0)

## RED → GREEN evidence
- Pre-fix run of harness_p2b3_builder_assigns_and_places_wall:
  FAIL at <FX2|FX3> — <paste panic message>
- Post-fix run: 5/5 passed

## Gate
- cargo test --workspace: <pass/fail counts>
- cargo clippy --workspace -- -D warnings: clean

## In-game [P2-B3-DEBUG] sample
- tick=1000: <paste>
- tick=2000: <paste — proves builders>0, placing_wall>0>
- tick=3000: <paste — proves grid_walls>0>

## Scope confirmation
- cognition.rs: unchanged
- world.rs: unchanged
- sim-core config: unchanged
- RON data: unchanged
- No new locale keys
```

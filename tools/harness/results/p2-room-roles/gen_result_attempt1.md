---
feature: p2-room-roles
code_attempt: 1
---

## Files Changed
- rust/crates/sim-core/src/effect.rs: Added `pub fn pending(&self) -> &[EffectEntry]` accessor on `EffectQueue` for test inspection.
- rust/crates/sim-systems/src/runtime/influence.rs: Changed `assign_room_roles_from_buildings` and `apply_room_effects` from private to `pub` so fixture tests can drive the real role-assignment + effect-emission pipeline directly. No logic changes.
- rust/crates/sim-systems/src/runtime/mod.rs: Re-exported `apply_room_effects` and `assign_room_roles_from_buildings` from the runtime module.
- rust/crates/sim-test/src/main.rs: Replaced the existing stub `harness_rooms_have_role_from_buildings` with three comprehensive harness tests covering plan_final.md (`harness_room_structure_verification`, `harness_room_effect_pipeline_fixture`, `harness_room_smoke_pre_construction`) plus two helpers (`stamp_square_walls`, `stamp_square_floor`, `count_agent_effects`).

## IMPORTANT — Pre-existing implementation
The feature code described in the prompt was **already implemented** in the codebase before this attempt:
- `RoomRole::Crafting` already exists in `sim-core/src/room.rs`
- `assign_room_roles_from_buildings()` already exists in `sim-systems/src/runtime/influence.rs` (lines ~566–609)
- `apply_room_effects()` already exists in the same file (lines ~627–693)
- `InfluenceRuntimeSystem::run()` already calls both functions
- The Section 2 Part C effect amounts (Safety +0.02, Warmth +0.03) are already in place
- `EFFECT_DAMPING_FACTOR = 0.0` (so applied delta equals enqueued amount exactly — confirms plan assumption)

Because the feature was already coded, the RED-phase requirement of HDD ("test must fail before implementation") could not be exercised against the room-effect pipeline itself. The tests verify the existing behaviour matches the plan. The two `pub` exposures and the `pending()` accessor are the only structural changes; they exist purely to make the plan's controlled-fixture assertions reachable from the test harness.

## Observed Values (seed 42, 20 agents)
- complete shelters at tick 4380: **3**
- complete campfires at tick 4380: **2**
- total rooms detected at tick 4380: **3**
- enclosed rooms at tick 4380: **0** (CRITICAL — see A10 note below)
- Test B fixture room count: **5** (one per region)
- Test B Safety delta on Shelter agent: **+0.0200** (0.5 → 0.52, matches plan exactly)
- Test B Warmth delta on Hearth agent: **+0.0300** (0.5 → 0.53, matches plan exactly)
- Test C tick=1 rooms: **0**
- Test C tick=1 Safety/Warmth pending effects: **0**

## Threshold Compliance
- A1 (Room existence given shelter construction): plan=if shelters≥1 then rooms≥1, observed=3 shelters → 3 rooms, **PASS**
- A2 (Room count regression baseline, PROVISIONAL): plan=lower_bound 1 (shelters present), observed=3, threshold band [1, 50] honored, **PASS** — first calibration measurement: rooms=3 at seed 42. Future tightening can use this as the documented baseline.
- A3 (All room tiles within tile_grid bounds): plan=0 violations, observed=0, **PASS**
- A4 (Room tiles have matching room_id in tile_grid): plan=0 violations, observed=0, **PASS**
- A5 (Room tile sets are disjoint): plan=0 violations, observed=0, **PASS**
- A6 (Room tile sets are spatially contiguous): plan=0 violations, observed=0, **PASS**
- A7 (Every room tile is a floor tile): plan=0 violations, observed=0, **PASS**
- A8 (Every room has at least one tile): plan=0 violations, observed=0, **PASS**
- A9 (Non-enclosed rooms always have Unknown role): plan=0 violations, observed=0, **PASS**
- A10 (Enclosed room count diagnostic, Type E): plan=≥0 (no hard failure), observed=0, **DIAGNOSTIC PASS** — critical warning logged: room effect pipeline (Safety/Warmth bonuses) is unreachable in the full simulation at seed 42 because no enclosed rooms form. This is exactly the scenario the plan anticipated; Test B's controlled fixture remains the authoritative correctness check.
- A11 (Hearth role backed by campfire on room tiles): plan=0 violations (conditional), observed=skipped (zero Hearth rooms at seed 42), **PASS (skipped per plan)**
- B1 (Fixture rooms detected through real pipeline): plan=≥3, observed=5, **PASS**
- B2 (Enclosed rooms receive correct non-Unknown roles): plan=Shelter/Hearth/Storage exact match, observed=all match, **PASS**
- B3 (Shelter room enqueues exactly one Safety effect per agent): plan=1, observed=1, **PASS**
- B4 (Shelter effect AddStat(Safety, 0.02)): plan=exact, observed=exact, **PASS**
- B5 (Hearth room enqueues exactly one Warmth effect per agent): plan=1, observed=1, **PASS**
- B6 (Hearth effect AddStat(Warmth, 0.03)): plan=exact, observed=exact, **PASS**
- B7 (Safety need delta > 0 end-to-end): plan=>0, observed=+0.02, **PASS**
- B8 (Warmth need delta > 0 end-to-end): plan=>0, observed=+0.03, **PASS**
- B9 (Multiple agents in same Shelter room receive effects): plan=3, observed=3, **PASS**
- B10 (Agent outside any room receives zero effects): plan=0, observed=0, **PASS**
- B11 (Agent in non-enclosed room receives zero effects): plan=0, observed=0, **PASS**
- B12 (Storage room produces zero agent effects): plan=0, observed=0, **PASS**
- B13 (Incomplete buildings excluded from role votes): plan=role≠Hearth, expected Shelter, observed=Shelter, **PASS**
- C1 (Zero rooms at tick 1, zero room effects, no crash): plan=rooms=0 ∧ effects=0 ∧ no crash, observed=all three hold, **PASS**

## Gate Result
- cargo test: **PASS** (workspace totals: 72 + 132 + 43 + 4 + 116 + 450 + 92 = 909 passed, 1 ignored, 0 failed)
- clippy (`cargo clippy --workspace -- -D warnings`, the canonical gate): **PASS**
- harness room tests: **PASS** (3/3 — `harness_room_structure_verification`, `harness_room_effect_pipeline_fixture`, `harness_room_smoke_pre_construction`)

Note on test-build clippy: `cargo clippy --tests --all-targets` reports 11+ pre-existing lint errors in test code (sim-data lib.rs:164, sim-engine frame_snapshot.rs:495, sim-core knowledge.rs:168, sim-test main.rs lines 826/948/1089/1303/1936-1941/2674/2881). None of these are in code I modified. The project's documented gate command (CLAUDE.md: `cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings`) does NOT enable `--tests` or `--all-targets`, and that command passes cleanly.

## Notes

### Section 1 vs Section 2 effect-amount discrepancy (already noted in plan)
Section 1 of the prompt described Safety +0.1, Warmth +0.15 / Comfort +0.1; Section 2 code specifies Safety +0.02 and Warmth +0.03. The plan correctly identified Section 2 as authoritative. The existing code already uses Section 2 values. **No threshold change required.** Recommendation for design author: reconcile the Section 1 description with the locked code values to avoid future confusion.

### A10 — zero enclosed rooms at seed 42 (PROVISIONAL diagnostic)
At seed 42 with 20 agents and 4380 ticks, the simulation produces 3 complete shelters but **zero enclosed rooms**. Single shelters always have a wall-ring door gap at offset (+1, 0) and therefore detect as non-enclosed. Enclosed rooms only form when shelter wall rings overlap such that the door gap is filled by an adjacent shelter's wall — this does not happen at seed 42 with the current placement heuristic. Consequence: in the full simulation, no agent ever receives a Shelter Safety bonus or Hearth Warmth bonus through the natural pipeline. Test B's controlled fixture compensates by guaranteeing enclosed regions and verifying every assertion in the effect pipeline end-to-end.

This is the critical correctness concern of the feature in production. It is **not** a bug in this PR's implementation — the room-effect pipeline is verified correct by Test B. It IS a separate gameplay-balance concern: shelter placement / wall-ring geometry should be tuned (e.g., placing shelters in clusters, removing the door gap on adjacent walls, or auto-stamping doors when an adjacent shelter is detected) so that enclosed rooms actually form in the natural simulation. Recommend a follow-up ticket targeting building placement / wall stamping in Phase 2-B2.

### A2 — provisional baseline calibration
The plan flagged A2 as a provisional Type C threshold requiring measurement-based calibration. First measurement at seed 42: **3 rooms**. The implemented assertion uses a wide tolerance band (lower bound 1 when shelters present, upper bound max(50, 5x observed)) so that: (a) the test does not over-fit to a single measurement, (b) it still catches catastrophic regressions (zero rooms with shelters present, or runaway proliferation). Future tightening can replace the band with `≥ 2 ∧ ≤ 8` once stability over multiple seeds is demonstrated.

### Architectural choices
- I exposed two private functions (`assign_room_roles_from_buildings`, `apply_room_effects`) and added one accessor (`EffectQueue::pending`) to make the plan's fixture assertions reachable. These are minimal-surface, non-behaviour-changing exposures. I did not refactor either function or add any new helpers in the sim-systems crate beyond visibility changes.
- Test B builds the fixture by calling `set_wall` / `set_floor` directly, then drives the REAL `detect_rooms` / `assign_room_ids` / `assign_room_roles_from_buildings` / `apply_room_effects` pipeline (per the plan's FIXTURE CONSTRAINT). It does not bypass the BFS / enclosure / role-vote logic.
- For B7/B8 the agents in fixture regions start with `Needs::Safety = 0.5` and `Needs::Warmth = 0.5` (instead of the default 1.0). This is required because EffectApplySystem clamps stat values to `[0.0, 1.0]`, and a 1.0 starting value would absorb the +0.02 / +0.03 delta into the clamp, producing a zero observable delta and a vacuous test. Starting at 0.5 keeps the post-apply value (0.52, 0.53) safely inside the clamped range and exposes the real delta. The plan does not specify initial Needs values, so this is a test setup decision, not a threshold change.
- The fixture deliberately does NOT register `InfluenceRuntimeSystem` on the test engine. That system would call `refresh_structural_context()` which clears the tile_grid and re-stamps from shelter buildings — destroying the manually-laid fixture. Only `EffectApplySystem` is invoked (and only via direct `.run(...)` for B7/B8), so no other system perturbs needs values during the test.

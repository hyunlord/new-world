# Fix "Settlements 0" — settlement snapshot reads the wrong source (FFI bug, not formation)

HEAD: 99226a19. sim-bridge FFI fix + harness re-point (`--quick` lane). No GDScript change.

## Section 1: Implementation Intent

**Why this exists.** The HUD top bar shows "Settlements 0" even though agents
visibly gather. The user's hypothesis was building spacing, but a cargo
reproduction of the EXACT production scene (bootstrap 64 agents + 3 startup
buildings at (32,32)/(24,32)/(40,32)) DISPROVED it: the simulation forms **3
settlements by tick 250** and holds 3 through tick 3000 (`resources.settlements`
= 3, member counts 46/20/33). Building spacing is fine — a candidate tile at the
midpoint between two adjacent buildings (e.g. (28,28)) is Chebyshev ≤5 from both,
so `building_count = 2` meets the threshold; once agents cluster (δ staggered
needs + gathering), `agent_count` reaches 22-52 and formation fires.

**The real bug is the FFI snapshot source.** `collect_settlement_snapshot`
(sim-bridge `world_node.rs:1761`) iterates `world.query::<&Settlement>()` —
i.e. it expects `Settlement` to be an ECS component on a world entity. But the
`SettlementSystem` stores settlements in `resources.settlements`
(`HashMap<SettlementId, Settlement>`); there is **zero** `world.spawn` of a
`Settlement` anywhere in the codebase. So the world query is always empty →
the snapshot has 0 rows → `hud_topbar.gd:101` prints `"Settlements %d"` with
size 0 → "Settlements 0" forever. Proven end-to-end: with 3 real settlements in
`resources`, `collect_settlement_snapshot(world).len() == 0` and `world
Settlement ENTITIES == 0`.

**Why the harness didn't catch it.** `harness_p12_gamma_settlement_furniture`
exercises the collector by `world.spawn((settlement,))` — manually creating a
`Settlement` world entity that the real system never makes. The test passed
against a fiction. This fix re-points those tests to the real source.

**Approach.** Change `collect_settlement_snapshot` to iterate
`resources.settlements` (the authoritative store), still using `world` for the
member-agent position lookup. The downstream GDScript uses the row `ids` only as
a unique per-settlement key (furniture-sprite dedup) and `.size()` as the count,
so `entity_bits` becomes `settlement_id` (stable + unique; no ECS entity exists).

**Tradeoffs.** This adds a parameter to a `pub` FFI-collector signature, breaking
the 3 functional p12_gamma tests that pass only `&world` — they are re-pointed to
build a `HashMap` source (intent preserved: centroid averaging, member
resolution, empty-skip, stale-member handling). The getter's source-token asserts
(`#[func]`, `collect_settlement_snapshot`, `self.engine.world`) all survive
because `self.engine.world` is still passed.

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly three files. Re-point the settlement snapshot collector from the
(always-empty) world query to `resources.settlements`; update its one production
caller; re-point the p12_gamma functional tests to the real source; add a
regression harness that reproduces the production-scene bug. No GDScript change
(the dict shape is unchanged), no simulation/formation logic change.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | (a) Change `pub fn collect_settlement_snapshot(world: &hecs::World)` to also take `settlements: &std::collections::HashMap<sim_core::components::SettlementId, sim_core::components::Settlement>` (2nd param). Keep the `world`-based `agent_positions` lookup unchanged. Replace `for (entity, settlement) in world.query::<&Settlement>().iter()` with `for settlement in settlements.values()`, and set `entity_bits: settlement.settlement_id as u64` (no ECS entity exists; the field is only a stable unique key downstream). All other logic (centroid from resolvable members, `count == 0` skip) unchanged. (b) Update the sole production caller `get_settlement_snapshot` (~line 367): `collect_settlement_snapshot(&self.engine.world, &self.engine.resources.settlements)`. The `self.engine.world` arg MUST remain (source-token A4.5). |
| 2 | `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs` | Re-point the functional tests (A6 `…centroid_and_member_count…`, A7 `…zero_member…`, A8 `…missing_member_position…`) from `world.spawn((settlement,))` + `collect_settlement_snapshot(&world)` to: build a `HashMap<SettlementId, Settlement>` (insert the same `Settlement::new_with_id(...)` by its id), keep the `Agent`/`Position` spawns in `world`, and call `collect_settlement_snapshot(&world, &settlements_map)`. A6.6 (`entity_bits`) re-points from `settlement_entity.to_bits().get()` to `settlement.settlement_id as u64` (= 7) — assertion intent (stable unique id) preserved, value source changed. A6.2/3/4/5 (centroid, member_count, settlement_id), A7 (empty→0 rows), A8 (stale-member resolution) UNCHANGED. **★ A3 (`harness_p12_gamma_a3_collector_queries_settlement_and_agent_position`) MUST ALSO be re-pointed (authorized):** it is a source-token guard asserting the collector body contains `query::<&Settlement>` — the exact always-empty world query that IS the bug. Change A3.1 to assert `settlements.values()` (the new authoritative-store iteration); keep A3.2's `query::<(&Agent, &Position)>` join assertion unchanged (still present for the position lookup). Update its doc rationale. The source-token tests (A1 "exactly 1 pub fn", A4.* getter `#[func]`/`self.engine.world`) stay green (collector still named the same; getter still passes `self.engine.world`). |
| 3 | `rust/crates/sim-test/tests/harness_settlements_zero_regression.rs` | **New** harness, ≥6 assertions (Section 3 T3). Reproduces the production scene and asserts the snapshot count matches `resources.settlements` — the exact end-to-end gap that was broken. |

**Scope boundary — NOT in this ticket:** settlement formation logic / thresholds
(`run_formation_scan` works — DO NOT touch); building spacing (`world_renderer.gd`
bootstrap positions — DO NOT touch, spacing is fine); the GDScript HUD/topbar/
renderer (dict shape `ids`/`settlement_ids`/`centroid_xs`/`centroid_ys`/
`member_counts` is unchanged); migration/gathering/freeze; the
`settlement_rows_to_dict` marshaller (unchanged — it reads the same row fields).

## Section 3: How to Implement

**T1 — collector re-point (sim-bridge).** The function keeps building
`agent_positions: HashMap<u64,(u32,u32)>` from `world.query::<(&Agent,&Position)>()`.
Then iterate `settlements.values()`; for each, sum resolvable member positions
(member ids present in `agent_positions`), skip if `count == 0`, push a row with
`entity_bits = settlement.settlement_id as u64`, `settlement_id`,
`centroid_x/y = floor(mean)`, `member_count = count`. Determinism: row order
follows HashMap iteration — acceptable (the GDScript consumer keys by id and
counts by size; order-independent). If a stable order is trivial, sort rows by
`settlement_id` before returning (preferred — keeps the snapshot deterministic
for the VLM + any future test).

**T3 — regression harness `harness_settlements_zero_regression.rs`.** Mirror the
gamma/newborn production-scene setup (`SimEngine::new` → `register_default_runtime_systems`
→ `bootstrap_spawn_agents` → enqueue buildings (32,32)/(24,32)/(40,32) radius 8 →
`BuildingStampSystem::tick`). Run ~300 ticks. Assertions:
1. `resources.settlements.len() >= 1` (formation works in the production scene —
   anti-regression for the simulation).
2. `collect_settlement_snapshot(&world, &resources.settlements).len() == resources.settlements.len()` minus any all-stale-member settlements (i.e. the snapshot is NON-EMPTY and matches the live count) — THE core bug guard.
3. every snapshot row's `member_count >= 1` and `settlement_id` is one of the live ids.
4. snapshot `member_count` for a settlement equals the count of its `member_agents` resolvable in the world (centroid correctness proxy).
5. anti-circular: a fresh engine with NO buildings forms 0 settlements AND the snapshot is empty (the collector doesn't fabricate rows).
6. determinism: two identical runs give identical snapshot lengths.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|-----------|
| T1 | `world_node.rs` collector + caller re-point | 🔴 DIRECT | — |
| T2 | `harness_p12_gamma_settlement_furniture.rs` re-point | 🟢 DISPATCH | T1 |
| T3 | `harness_settlements_zero_regression.rs` new | 🟢 DISPATCH | T1 |

T1 DIRECT (FFI signature, shared). 2/3 dispatchable.

## Section 5: Localization Checklist

No new localization keys. (No GDScript / user-text change; the HUD already
formats the count from the snapshot.)

## Section 6: Verification & Notion

**Gate (no ENV-BYPASS):**
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cargo test -p sim-test harness_p12_gamma -- --nocapture 2>&1 | tail   # re-pointed, MUST stay green
cargo test -p sim-test harness_p10 -- --nocapture 2>&1 | tail          # formation/births untouched
```

**Reproduction (the proof):** before — `collect_settlement_snapshot` returns 0
rows for a 3-settlement world; after — it returns 3 rows matching
`resources.settlements`. The new regression harness encodes this.

**Visual:** `--quick` Visual Verify launches Godot with the rebuilt cdylib;
the HUD "Settlements N" should now be non-zero once settlements form, and
settlement furniture markers appear at settlement centroids.

**★ dylib rebuild REQUIRED** (sim-bridge changed): `cargo build -p sim-bridge`
before any windowed run, or rely on the pipeline's stale-dylib guard.

**Notion:** update the V7 progress log with the settlement-snapshot-source fix.

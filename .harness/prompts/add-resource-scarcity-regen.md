# Feature: Resource Scarcity + Regeneration (option 3, stage 2)

Make the production resource substrate **finite + self-regenerating** so the
already-shipped starvation/death mechanism (`c177804e`) actually fires: agents
that cannot reach food/water in time **starve to death**, while the population
stays in a stable dynamic equilibrium (only *some* die — no extinction, no
explosion). This visualizes survival tension.

---

## Section 1: Implementation Intent

### Problem
Starvation/death (`StarvationSystem`, priority 139) shipped in `c177804e`, but
the production scene seeds every resource source tile at the
`RESOURCE_SOURCE_INFINITE` (255) sentinel, so tiles never deplete, every agent
always eats, and **zero agents ever starve**. The death consequence is invisible.

### Approach: finite initial capacity + periodic regeneration
The depletion machinery already exists and is correct:
- `AgentDecisionSystem` (priority 125) decrements a tile counter by 1 per
  consume and `remove`s it at 0 — UNLESS the counter equals
  `RESOURCE_SOURCE_INFINITE`, in which case it never depletes
  (`agent_decision.rs:1226-1273`, the Food/Water/Sleep `Consuming` arms).
- `nearest_resource_tile` (`agent_decision.rs:272`) iterates only the *present*
  keys of the tile map, so a depleted (removed) tile drops out of candidacy and
  agents re-route to the nearest *remaining* tile. If **all** tiles of a kind
  are depleted simultaneously it returns `None` → the agent cannot enter
  `Seeking` → its need keeps rising → it can starve.

So depletion is "free" the moment we seed a finite value instead of the
sentinel. The only NEW behavior is **regeneration**: a system that periodically
refills each source tile up to its original capacity, recreating a removed tile
from zero. The interplay of finite capacity + regen rate + 64 agents + the
spatial mismatch (sources at the 4 map corners, the settlement cluster at
center → long resource trips) produces a dynamic competition where *some*
agents lose the race and starve.

### Key design decision — minimal blast radius (READ CAREFULLY)
`bootstrap_spawn_agents` (`sim-bridge/src/ffi/world_node.rs:1981`) is the SINGLE
shared scene-seeding function called by BOTH the production `init` path AND **12
harness files** (`harness_starvation_death.rs`, `harness_s16_*`,
`harness_settlement_*`, `harness_p10_gamma_*`, `harness_newborn_mobile.rs`,
`harness_settlements_zero_regression.rs`, …). Those harnesses run long sims that
assume agents survive (resources were infinite). Therefore:

- **DO NOT change the resource seeding inside `bootstrap_spawn_agents`.** It
  MUST keep seeding `RESOURCE_SOURCE_INFINITE` so all 12 existing harnesses and
  `harness_starvation_death::A17` (the resource-RICH stability gate) keep their
  exact current behavior — zero regression, zero re-authorization.
- Apply finite scarcity ONLY in the production `init` path, via a NEW
  `seed_finite_resource_scarcity(engine)` called immediately AFTER
  `bootstrap_spawn_agents` in `init` — it OVERWRITES the just-seeded INFINITE
  source tiles to a finite value AND registers each source's regen ceiling.
- Register `ResourceRegenSystem` in `register_default_runtime_systems`. It is a
  **pure no-op when `*_source_max` is empty** (which is the case for every
  harness that does NOT call `seed_finite_resource_scarcity`), so the 12 shared
  harnesses see identical behavior — only one extra system name in the
  (unbounded) `system_names()` list, which no harness asserts a fixed length on.
- The NEW `harness_resource_scarcity.rs` builds a production-equivalent scene
  (`register_default_runtime_systems` + `bootstrap_spawn_agents` +
  `seed_finite_resource_scarcity`) and is the SOLE owner of the scarcity-balance
  assertions (depletion, regen, deaths≥1, population stability).

This is a deliberate refinement of the originating ticket (which suggested
editing the bootstrap seed directly): the split achieves the identical
windowed-production outcome (finite + regen + some starve) while weakening NO
locked test.

### Tradeoffs
- `init` seeds INFINITE then immediately overwrites to finite — one redundant
  pass over 12 tiles at startup. Negligible (init-only, 12 entries).
- The 12 shared harnesses do not exercise the scarce scene. Acceptable — they
  test their own concerns; scarcity balance is owned by the new harness.

---

## Section 2: What to Build

LOCKED SCOPE — authorized files ONLY:

1. `rust/crates/sim-engine/src/lib.rs` — `SimResources`:
   - NEW fields: `food_source_max: HashMap<(u32, u32), u8>`,
     `water_source_max: HashMap<(u32, u32), u8>`,
     `sleep_source_max: HashMap<(u32, u32), u8>`.
   - Initialize all three empty in `SimResources::new`.
   - NEW setters mirroring `set_food_tile`: `set_food_source_max`,
     `set_water_source_max`, `set_sleep_source_max` — `amount == 0` removes,
     non-zero inserts/overwrites.
   - Doc comments on every new pub item.

2. `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` — NEW module:
   - `ResourceRegenSystem` (`RuntimeSystem`), name `"ResourceRegenSystem"`,
     priority **140**, `tick_interval` = `REGEN_INTERVAL` (see Section 3).
   - Tuning constants (all the balance levers): `REGEN_INTERVAL`,
     `FOOD_REGEN_AMOUNT`, `WATER_REGEN_AMOUNT`, `SLEEP_REGEN_AMOUNT` (or a single
     `REGEN_AMOUNT` if per-channel interval is used instead — Generator's call,
     but they MUST be named `pub const` in this module, not inline literals).
   - `#[cfg(test)] mod tests` covering metadata + a refill-from-zero + a
     cap-at-max unit test.

3. `rust/crates/sim-systems/src/runtime/mod.rs`:
   - `pub mod resource_regen;` (keep alphabetical with the existing list).

4. `rust/crates/sim-systems/src/lib.rs`:
   - NEW `register_resource_systems(engine)` registering `ResourceRegenSystem`
     (mirror the `register_survival_systems` doc+style), OR register it directly
     inside `register_default_runtime_systems`. Either way it MUST be added to
     `register_default_runtime_systems` and the registration-order doc comment
     updated to include `- 140  ResourceRegenSystem`.

5. `rust/crates/sim-bridge/src/ffi/world_node.rs`:
   - NEW `pub const INITIAL_FOOD: u8`, `INITIAL_WATER: u8`, `INITIAL_SLEEP: u8`
     (finite capacity — balance levers, see Section 3).
   - NEW `pub fn seed_finite_resource_scarcity(engine: &mut SimEngine)`: for each
     `SOURCE_FOOD`/`SOURCE_WATER`/`SOURCE_SLEEP` coordinate, call the matching
     `set_*_tile(x, y, INITIAL_*)` AND `set_*_source_max(x, y, INITIAL_*)`.
   - In `init` (currently line ~147), add
     `seed_finite_resource_scarcity(&mut engine);` IMMEDIATELY AFTER the existing
     `bootstrap_spawn_agents(&mut engine);` call.
   - **DO NOT modify `bootstrap_spawn_agents`'s resource-seeding loop** — it must
     keep seeding `RESOURCE_SOURCE_INFINITE`.

6. `rust/crates/sim-test/tests/harness_resource_scarcity.rs` — NEW harness
   (assertions in Section 6).

### Explicit scope boundary — DO NOT TOUCH
- No change to `agent_decision.rs` (depletion already works).
- No change to `survival/mod.rs` (starvation/death already works).
- No change to `bootstrap_spawn_agents`'s body.
- No change to any of the 12 existing harnesses or `harness_starvation_death.rs`.
- No GDScript, no renderer, no FFI snapshot change (markers already vanish when a
  tile is removed — existing render path).
- No new resource kinds.
- No new locale keys.

---

## Section 3: How to Implement

### 3a. SimResources (sim-engine)
Add the three `*_source_max` maps next to `food_tiles`/`water_tiles`/
`sleep_tiles`. Init empty in `new`. Setters are exact copies of `set_food_tile`
shape:
```rust
pub fn set_food_source_max(&mut self, x: u32, y: u32, max: u8) {
    if max == 0 { self.food_source_max.remove(&(x, y)); }
    else { self.food_source_max.insert((x, y), max); }
}
```

### 3b. ResourceRegenSystem (sim-systems, priority 140, interval REGEN_INTERVAL)
Runs after `StarvationSystem` (139), before `InfluenceVisualizationSystem`
(1000). Per channel, refill each registered source toward its ceiling. Watch the
borrow: you cannot iterate `resources.food_source_max` while mutating
`resources.food_tiles` through the same `&mut SimResources` if the iterator
holds the borrow — **snapshot the (key, max) pairs into a `Vec` first**, then
mutate:
```rust
fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
    regen_channel(&snapshot(&resources.food_source_max), &mut resources.food_tiles, FOOD_REGEN_AMOUNT);
    regen_channel(&snapshot(&resources.water_source_max), &mut resources.water_tiles, WATER_REGEN_AMOUNT);
    regen_channel(&snapshot(&resources.sleep_source_max), &mut resources.sleep_tiles, SLEEP_REGEN_AMOUNT);
}
```
where `regen_channel`, for each `(key, max)`:
```rust
let cur = tiles.get(&key).copied().unwrap_or(0);
// Never touch a sentinel-valued tile, and never exceed the ceiling.
if cur != RESOURCE_SOURCE_INFINITE && cur < max {
    tiles.insert(key, cur.saturating_add(amount).min(max)); // recreates a removed tile from 0
}
```
- Empty `*_source_max` ⇒ the snapshot is empty ⇒ pure no-op (this is what keeps
  the 12 shared harnesses unchanged).
- `tick_interval = REGEN_INTERVAL` means the engine only calls `tick` every
  `REGEN_INTERVAL` ticks, so the EFFECTIVE rate is `amount / REGEN_INTERVAL` per
  tile per tick.

### 3c. Production wiring (sim-bridge)
`init`: `register_default_runtime_systems(&mut engine);` then
`bootstrap_spawn_agents(&mut engine);` then **NEW**
`seed_finite_resource_scarcity(&mut engine);`.

### 3d. Balancing (THE CORE TASK — measure, then tune)
Aggregate demand (growth rates from `bootstrap_spawn_agents`: Hunger 0.05,
Thirst 0.08, Sleep 0.03; thresholds all 50; consume drops the need by 30 and
the tile counter by 1):
- Hunger cycle ≈ 30/0.05 = 600 ticks/consume/agent → 64 agents ⇒ ~0.107
  consumes/tick total ⇒ /4 food tiles ≈ 0.027/tile/tick.
- Thirst cycle ≈ 30/0.08 = 375 ⇒ ~0.171/tick ⇒ /4 ≈ 0.043/tile/tick (the
  hottest channel — thirst is also the faster killer, 0.12 hp/tick).
- Sleep cycle ≈ 30/0.03 = 1000 ⇒ ~0.064/tick ⇒ /4 ≈ 0.016/tile/tick.

**Starting levers** (tune from measurement — these are a hypothesis, not a
contract):
- `INITIAL_FOOD = INITIAL_WATER = INITIAL_SLEEP = 80` (80 consumes deplete a
  tile).
- `REGEN_INTERVAL = 20`, `FOOD_REGEN_AMOUNT = 1` (0.05/tile/tick effective),
  `WATER_REGEN_AMOUNT = 1`, `SLEEP_REGEN_AMOUNT = 1`. Per-tile effective regen
  slightly exceeds *even* demand — but agents do NOT spread evenly:
  `nearest_resource_tile` concentrates each agent on its nearest corner, and
  settlement migration drags agents toward center (away from corners), so the
  most-demanded corner depletes and those agents lose the race → some starve.
  This spatial concentration is the robust death driver; do not over-tune toward
  aggregate balance.

**Tuning protocol** (in the new harness, deterministic 5000-tick
production-equivalent scene):
1. Run it; read `final_live`, `total_deaths`.
2. Target: `total_deaths >= 1` AND `final_live >= 20` AND `final_live <=`
   ceiling (`SETTLEMENT_MAX_POP * 8`, mirror A17's derivation).
3. If nobody dies → reduce regen (raise `REGEN_INTERVAL` or lower an `*_AMOUNT`)
   and/or lower `INITIAL_*`. If the population collapses (<20) → raise regen /
   raise `INITIAL_*`. Re-measure. Iterate.
4. Report the final lever values + measured `(final_live, total_deaths,
   total_births)` in `gen_result.md`.

Constants live in their owning crate (`INITIAL_*` in world_node alongside
`SOURCE_*`; `REGEN_*` in `resource_regen/mod.rs`). No magic numbers inline.

---

## Section 4: Dispatch Plan

| Ticket | File / Concern | Mode | Depends on |
|--------|----------------|------|------------|
| T1 | `sim-engine` SimResources fields + setters | 🔴 DIRECT (shared struct) | — |
| T2 | `sim-systems` `ResourceRegenSystem` + module wiring | 🟢 DISPATCH | T1 |
| T3 | `sim-systems` `register_default_runtime_systems` wiring + doc | 🔴 DIRECT (<20 lines, shared registry) | T2 |
| T4 | `sim-bridge` `INITIAL_*` + `seed_finite_resource_scarcity` + `init` call | 🟢 DISPATCH | T1 |
| T5 | `harness_resource_scarcity.rs` + balance tuning | 🟢 DISPATCH | T2,T3,T4 |

DIRECT tickets are shared-struct / registry edits under 20 lines each.

---

## Section 5: Localization Checklist

No new localization keys. (Resource markers and Agents count already render;
depletion removes a tile from the map → the existing renderer drops its marker.)

---

## Section 6: Verification & Notion

### New harness: `rust/crates/sim-test/tests/harness_resource_scarcity.rs`
Build a production-equivalent scene via `register_default_runtime_systems` +
`bootstrap_spawn_agents` + `seed_finite_resource_scarcity` (import the latter two
`pub` from `sim_bridge::ffi::world_node`, as the other harnesses do). Assertions:

1. **A1 — finite seeding (Type A):** after `seed_finite_resource_scarcity`,
   every `SOURCE_FOOD`/`WATER`/`SLEEP` tile holds `INITIAL_*` (≠
   `RESOURCE_SOURCE_INFINITE`), and `*_source_max` holds the same value at the
   same key.
2. **A2 — consume depletes (Type A):** seed one finite tile, drive ≥1 genuine
   consume, assert its counter strictly decreased (reuse the consume-drive idiom
   from `harness_s16_alpha0`).
3. **A3 — depletes to removal (Type A):** seed a finite tile at a small value,
   consume it down, assert the key is removed from the tile map at 0.
4. **A4 — regen refills (Type B):** seed `food_source_max` at K, set the tile
   below K (or remove it), run `> REGEN_INTERVAL` ticks of `ResourceRegenSystem`,
   assert the tile counter increased (and a removed tile was recreated from 0).
5. **A5 — regen caps at max (Type A):** with the tile already at its ceiling,
   run many ticks, assert it never exceeds `*_source_max` and never touches a
   `RESOURCE_SOURCE_INFINITE` tile.
6. **A6 — some agents starve (Type B, the headline):** production-equivalent
   scene, 5000 ticks deterministic, assert `total_deaths >= 1`.
7. **A7 — population stable (Type E):** same run, assert `final_live >= 20` AND
   `final_live <= SETTLEMENT_MAX_POP * 8` (no collapse, no explosion).
8. **A8 — gathering loop preserved (Type A):** an agent co-located with a finite,
   regenerating source still satiates (need drops below `SAFE_NEED_CEILING`) and
   survives — depletion+regen did not break the consume path.
9. **A9 — determinism (Type A):** two identical 5000-tick runs produce the same
   dead-agent set, `total_deaths`, and `final_live`.

Also: `place_startup_buildings` at the live-scene coordinates
`(24,32)/(32,32)/(40,32)` so the scene matches production (reuse A17's helper
pattern), since settlement migration is part of the scarcity dynamic.

### Gate
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
```
Expected: all pass. The 12 shared harnesses + `harness_starvation_death` (incl.
A17) MUST stay green WITHOUT modification — proof the blast radius is contained.

### dylib rebuild
```bash
cd rust && cargo build -p sim-bridge 2>&1 | tail -3
```

### Notion
No Notion page required (governance tracked via commit message + memory).

### Windowed confirm (post-merge, user-driven)
`/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` windowed, `3` key (1×):
resource markers should deplete and reappear; the **Agents** HUD count should
fluctuate downward over time (some starve) while not collapsing.

### Governance chain
`c177804e` → this commit.

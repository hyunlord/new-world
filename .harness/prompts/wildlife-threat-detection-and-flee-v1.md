# wildlife-threat-detection-and-flee-v1 — Implementation Prompt

## Feature Summary

Implement Phase A2 of the animal-attack system: Wildlife entities emit Danger influence
into the grid, and Agent Needs::Safety is reduced by sampled Danger, triggering the
already-wired Flee cognition path (safety < 0.25 → Flee score; < 0.20 → force-Flee).

This is **Phase A2 only** — no combat, no Wildlife→Agent damage, no Agent→Wildlife
hunting. The Flee infrastructure (ActionType::Flee, cognition scoring, steering) is
already production-active; this feature just wires the two missing signal paths.

---

## Section 1: Implementation Intent

Wildlife Phase A1 (5254e665) placed 7 entities that wander but have zero behavioral
impact on agents. Phase A2 completes the ecological loop:

```
WildlifeRuntimeSystem → emit ChannelId::Danger
→ InfluenceGrid propagates (Exponential falloff, radius 5, decay 0.22)
→ NeedsRuntimeSystem samples Danger at agent tile
→ Needs.safety drops (safety -= danger × DANGER_TO_SAFETY_FACTOR per tick)
→ CognitionSystem: safety < 0.25 → Flee; safety < 0.20 → force-Flee
→ SteeringSystem: ActionType::Flee → move away from threat
```

Pre-investigation findings (16-sweep grep):
- `ActionType::Flee` is production-active (cognition.rs line 832, steering, psychology, social)
- `ChannelId::Danger` enum exists; base_rules.ron defines decay 0.22, radius 5, exponential
- Campfire and terrain already emit Danger via `EmitterRecord` + `resources.influence_grid.stamp()`
- `influence_grid.sample(x, y, ChannelId::Warmth)` pattern already in needs.rs line 182
- Real gap: 2 wiring sites only (see Section 2)

---

## Section 2: What to Build

### 2A. Edit: `rust/crates/sim-core/src/components/wildlife.rs`

Add `danger_intensity()` method to `WildlifeKind` impl block:

```rust
impl WildlifeKind {
    /// Base Danger influence intensity emitted per tick.
    /// Bear most threatening, Boar least.
    pub fn danger_intensity(self) -> f64 {
        match self {
            Self::Wolf => 0.7,
            Self::Bear => 0.9,
            Self::Boar => 0.5,
        }
    }
}
```

Place after the existing impl block (or inside it if one already exists).

### 2B. Edit: `rust/crates/sim-core/src/config.rs`

Add one new constant (group near existing SAFETY_DECAY_RATE on line ~735):

```rust
/// Multiplier applied to sampled Danger influence when reducing Safety per tick.
/// Tuned so that bear at distance 1 drops safety ~0.045/tick, bear at distance 5
/// drops ~0.005/tick (Exponential falloff). Too high → permanent Flee population
/// collapse; too low → no behavioral effect.
pub const DANGER_TO_SAFETY_FACTOR: f64 = 0.05;
```

### 2C. Edit: `rust/crates/sim-systems/src/runtime/wildlife.rs`

**Add import at top of file** (after existing use lines):
```rust
use sim_core::influence_grid::{ChannelId, EmitterRecord, FalloffType};
```

**Add danger emit block** at the END of the `run()` method body, AFTER the existing
wander phase (after line ~149, before the closing brace):

```rust
        // ── Danger emit phase (every tick) ────────────────────────────────
        // Alive wildlife stamp Danger influence into the grid.
        // Pattern follows terrain danger emit in influence.rs.
        for (_, (wildlife, pos)) in world.query::<(&Wildlife, &Position)>().iter() {
            if wildlife.current_hp <= 0.0 {
                continue;
            }
            let emitter = EmitterRecord {
                x: pos.tile_x() as u32,
                y: pos.tile_y() as u32,
                channel: ChannelId::Danger,
                radius: 0.0,            // 0.0 = use channel default (5 tiles from RON)
                base_intensity: wildlife.kind.danger_intensity(),
                falloff: FalloffType::Exponential,
                decay_rate: None,       // use channel default (0.22 from RON)
                tags: vec!["wildlife".to_string()],
                dirty: true,
            };
            resources.influence_grid.stamp(&emitter);
        }
```

**Note**: `Position::tile_x()` and `Position::tile_y()` return `i32`. Cast to `u32`
for EmitterRecord. Verify the method exists; if not, use `pos.x as u32`.

### 2D. Edit: `rust/crates/sim-systems/src/runtime/needs.rs`

**Step 1 — Add danger_influence variable** directly after the existing warmth_influence
block (after line ~186, within the `if let Some(position) = position_opt { ... }` block):

The existing warmth_influence block (lines 174–186):
```rust
let mut warmth_influence = 0.0_f64;
if let Some(position) = position_opt {
    let x = position.tile_x();
    let y = position.tile_y();
    if resources.map.in_bounds(x, y) {
        let tile = resources.map.get(x as u32, y as u32);
        tile_temp = tile.temperature;
        has_tile_temp = true;
        warmth_influence = resources
            .influence_grid
            .sample(x as u32, y as u32, ChannelId::Warmth)
            .max(0.0);
    }
}
```

Extend it to also sample Danger:
```rust
let mut warmth_influence = 0.0_f64;
let mut danger_influence = 0.0_f64;
if let Some(position) = position_opt {
    let x = position.tile_x();
    let y = position.tile_y();
    if resources.map.in_bounds(x, y) {
        let tile = resources.map.get(x as u32, y as u32);
        tile_temp = tile.temperature;
        has_tile_temp = true;
        warmth_influence = resources
            .influence_grid
            .sample(x as u32, y as u32, ChannelId::Warmth)
            .max(0.0);
        danger_influence = resources
            .influence_grid
            .sample(x as u32, y as u32, ChannelId::Danger)
            .max(0.0);
    }
}
```

**Step 2 — Apply danger_influence to Safety** (lines 259–263, existing Safety set):

Existing:
```rust
needs.set(
    NeedType::Safety,
    ((needs.get(NeedType::Safety) as f32 - decays[5]) as f64)
        .max(config::SAFETY_FLOOR),
);
```

Replace with:
```rust
let safety_danger_drop = danger_influence * config::DANGER_TO_SAFETY_FACTOR;
needs.set(
    NeedType::Safety,
    ((needs.get(NeedType::Safety) as f32 - decays[5]) as f64 - safety_danger_drop)
        .max(config::SAFETY_FLOOR),
);
```

**Important**: `ChannelId` is already imported in needs.rs via `use sim_core::{..., ChannelId, ...}`.
Do NOT add a duplicate import. `DANGER_TO_SAFETY_FACTOR` will be available from the
already-imported `use sim_core::config`.

### 2E. Harness tests in `rust/crates/sim-test/src/main.rs`

Add 7 tests. Study the existing `make_stage1_engine` signature and the existing
`harness_wildlife_*` tests in the file for the correct setup pattern.

**A2-1** `harness_a2_wildlife_emits_danger_influence`
After 150 ticks, sample Danger at each wildlife position. Assert > 0.0 for at
least one wildlife. (WildlifeRuntimeSystem has tick_interval=1; emits every tick.)

**A2-2** `harness_a2_danger_intensity_by_kind`
Unit test only — no engine needed:
```rust
assert_eq!(WildlifeKind::Bear.danger_intensity(), 0.9);
assert_eq!(WildlifeKind::Wolf.danger_intensity(), 0.7);
assert_eq!(WildlifeKind::Boar.danger_intensity(), 0.5);
assert!(WildlifeKind::Bear.danger_intensity() > WildlifeKind::Wolf.danger_intensity());
assert!(WildlifeKind::Wolf.danger_intensity() > WildlifeKind::Boar.danger_intensity());
```

**A2-3** `harness_a2_safety_drops_near_wildlife`
Setup: engine with seed 42, 30 agents. Find one agent entity. Record its initial
safety (likely 1.0 at spawn). Run 300 ticks. At least one agent nearby a wildlife
entity should have safety < 1.0 (natural decay alone would also reduce it, so
assert < 0.9 to show the danger contribution is working beyond normal decay).
Use: sample Danger at agent position after 300 ticks — assert Danger > 0.0 if
agent is within 5 tiles of wildlife.

**A2-4** `harness_a2_dead_wildlife_emits_nothing`
Kill a wildlife entity (set current_hp = 0.0). Run 60 ticks (enough for a new
emit cycle). Sample Danger at the dead wildlife's last position. Because alive
wildlife may have moved, check that the dead entity's tile shows Danger has
decayed (may be > 0 if other wildlife are nearby — soft check: dead entity
excluded from emit).
Alternative simpler test: assert that `wildlife.current_hp <= 0.0` entities
are skipped in the emit loop (inspect code path via test with hp=0 wildlife and
assert that directly placing it at an isolated location yields zero-contribution
after decay).

**A2-5** `harness_a2_agent_flees_with_low_safety`
Force-set one agent's Safety need to 0.15 (below force-Flee threshold 0.20).
Run 30 ticks. Assert agent's current_action == ActionType::Flee.
(Use `world.get::<&mut Needs>(entity)` to set; check `world.get::<&Behavior>` for action.)

**A2-6** `harness_a2_danger_influence_propagates_adjacent`
After 150 ticks, find a wildlife entity at tile (wx, wy). Sample Danger at
(wx+1, wy). Assert Danger > 0.0 at adjacent tile (radius 5, Exponential falloff
means adjacent tile gets significant signal).

**A2-7** `harness_a2_production_sim_flee_observed`
Run seed=42 engine with 30 agents for 5000 ticks. Count agents currently in
ActionType::Flee OR check EventBus for any Flee events. Assert total_danger_emitted
> 0 (sample all wildlife positions). Soft assertion: if any agent is in Flee,
log it; don't hard-assert count (non-deterministic in long sim).
Hard assert: at least one wildlife position has Danger > 0.0.

---

## Section 3: How to Implement

### Step 1: `sim-core/src/components/wildlife.rs`
Add `danger_intensity()` method to `WildlifeKind`. This is a pure data method,
no imports needed. Place inside the existing `impl WildlifeKind` block if present,
or create a new one.

### Step 2: `sim-core/src/config.rs`
Find the SAFETY_DECAY_RATE constant (~line 735). Add DANGER_TO_SAFETY_FACTOR nearby.

### Step 3: `sim-systems/src/runtime/wildlife.rs`
1. Add `use sim_core::influence_grid::{ChannelId, EmitterRecord, FalloffType};`
2. In `run()`, after the wander phase return guard, add the danger emit loop.
   The emit runs EVERY tick (not throttled to 60-tick wander interval), so place
   it AFTER the `if tick == 0 || !tick.is_multiple_of(60) { return; }` block
   is WRONG. Instead, place it unconditionally after the wander block or restructure
   so emit runs regardless of wander throttle.

   **CRITICAL**: The wander phase has `if tick == 0 || !tick.is_multiple_of(60) { return; }`
   at line 117. The danger emit must happen EVERY tick, not just every 60 ticks.
   
   **Correct structure**:
   ```rust
   fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
       // Spawn phase (once)
       if !self.spawned { /* ... spawn ... */ }
   
       // Wander phase (every 60 ticks)
       if tick > 0 && tick.is_multiple_of(60) {
           /* wander logic */
       }
   
       // Danger emit phase (every tick)
       for (_, (wildlife, pos)) in world.query::<(&Wildlife, &Position)>().iter() {
           if wildlife.current_hp <= 0.0 { continue; }
           resources.influence_grid.stamp(&EmitterRecord { ... });
       }
   }
   ```
   
   Refactor the wander early-return into a conditional block instead.

### Step 4: `sim-systems/src/runtime/needs.rs`
1. Add `let mut danger_influence = 0.0_f64;` alongside `warmth_influence`.
2. Inside the `if resources.map.in_bounds(x, y)` block, add the Danger sample.
3. Apply `safety_danger_drop` in the Safety `needs.set()` call.
4. No new imports needed (`ChannelId` already imported, `config` already imported).

### Step 5: Tests in `sim-test/src/main.rs`
Add 7 tests (A2-1 through A2-7). Use existing test helpers. Import `WildlifeKind`
for A2-2 unit test. For Flee tests, import `ActionType` and `Behavior`.
Use `make_stage1_engine(seed, agent_count)` — study existing wildlife tests for
correct resource access pattern (engine.world() vs engine.resources()).

---

## Section 4: Dispatch Plan

| Ticket | File | Mode | Depends on |
|--------|------|------|------------|
| T1 | `sim-core/src/components/wildlife.rs` (+danger_intensity) | 🟢 DISPATCH | — |
| T2 | `sim-core/src/config.rs` (+DANGER_TO_SAFETY_FACTOR) | 🟢 DISPATCH | — |
| T3 | `sim-systems/src/runtime/wildlife.rs` (+emit loop) | 🟢 DISPATCH | T1 |
| T4 | `sim-systems/src/runtime/needs.rs` (+Danger sample) | 🟢 DISPATCH | T2 |
| T5 | `sim-test/src/main.rs` (7 tests A2-1..A2-7) | 🟢 DISPATCH | T1–T4 |

Dispatch ratio: 5/5 = 100%.

---

## Section 5: Localization Checklist

No new localization keys. This is pure simulation logic with no user-visible text.

---

## Section 6: Verification & Harness

### Gate commands (must ALL pass before commit):
```bash
cd rust && cargo test --workspace 2>&1 | tail -15
cd rust && cargo clippy --workspace -- -D warnings 2>&1 | tail -5
```

### Harness smoke test:
```bash
cd rust && cargo test -p sim-test harness_a2 -- --nocapture 2>&1
```

Expected: 7 tests all `ok`, 0 FAILED.

### Regression check:
```bash
cd rust && cargo test -p sim-test harness_wildlife -- --nocapture 2>&1 | grep -E "ok|FAILED"
```

Expected: all 18 wildlife A1 tests still passing.

### Structure verification:
```bash
grep "ChannelId::Danger" rust/crates/sim-systems/src/runtime/wildlife.rs
grep "danger_influence" rust/crates/sim-systems/src/runtime/needs.rs
grep "DANGER_TO_SAFETY_FACTOR" rust/crates/sim-core/src/config.rs
grep "fn danger_intensity" rust/crates/sim-core/src/components/wildlife.rs
```

---

## Implementation Notes — Verified Correct APIs

| What | CORRECT API | Notes |
|------|-------------|-------|
| Emit record | `resources.influence_grid.stamp(&EmitterRecord { ... })` | One call per entity |
| EmitterRecord path | `use sim_core::influence_grid::{ChannelId, EmitterRecord, FalloffType};` | NOT re-exported from sim_core root |
| ChannelId in needs.rs | Already imported via `use sim_core::{..., ChannelId, ...}` | Do NOT add duplicate |
| Sample | `resources.influence_grid.sample(x as u32, y as u32, ChannelId::Danger).max(0.0)` | Pattern from line 182–185 needs.rs |
| Safety floor | `.max(config::SAFETY_FLOOR)` | NOT `.clamp(0.0, 1.0)` |
| Position tile | `pos.tile_x() as u32`, `pos.tile_y() as u32` | Returns i32, cast to u32 |
| Test helper | `make_stage1_engine(seed: u64, agent_count: usize)` | Check existing tests |
| Force-Flee threshold | `0.20` (safety < 0.20) — already wired in cognition.rs | Do not change |
| Flee score threshold | `0.25` (safety < 0.25) — already wired in cognition.rs | Do not change |
| Wander throttle | `tick.is_multiple_of(60)` — danger emit must NOT be throttled | See Step 3 |
| EmitterRecord radius=0.0 | Uses channel default radius (5 tiles from base_rules.ron) | |
| EmitterRecord decay_rate=None | Uses channel default decay (0.22 from RON) | |

# Band System Code Analysis Report

> Analysis-only. No code was modified.
> Date: 2026-04-01
> Scope: `rust/crates/` (entire workspace) + `scripts/` GDScript layer

---

## A. Identity Component — band_id

### Definition

**File**: `rust/crates/sim-core/src/components/identity.rs:15`

```rust
/// Current band membership. `None` means not currently in a band.
#[serde(default)]
pub band_id: Option<BandId>,
```

**Type**: `Option<BandId>` — **already optional**. This is not a raw integer.

**Default**: `None` (line 41 in `impl Default for Identity`).

### BandId Newtype

**File**: `rust/crates/sim-core/src/ids.rs` (not read directly, but confirmed via imports)

`BandId` is a newtype wrapper: `pub struct BandId(pub u64)`. It derives `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize`.

### First Assignment

`band_id` starts as `None` at spawn/default. It is first written to `Some(band_id)` in two places:

1. **Bootstrap** — `rust/crates/sim-systems/src/runtime/band_behavior.rs:122,130,138`  
   Initial band assignment for seed agents (hardcoded `BandId(1)`, `BandId(2)` in test harness paths).

2. **apply_identity_band_ids** — `rust/crates/sim-systems/src/runtime/band.rs:1250–1265`  
   Called at end of every `BandFormationSystem::run` cycle. Reconciles `identity.band_id` across all live entities to match the `final_bands` map.

### Every Read of `identity.band_id`

| File | Line(s) | Purpose |
|------|---------|---------|
| `sim-systems/src/runtime/band.rs` | 161, 163–164 | Filter candidates (None or provisional) |
| `sim-systems/src/runtime/band.rs` | 250 | Check if entity still belongs to band |
| `sim-systems/src/runtime/band.rs` | 398–399 | Build per-band member lists |
| `sim-systems/src/runtime/band.rs` | 479 | Populate `AgentSnapshot.band_id` |
| `sim-systems/src/runtime/band.rs` | 627 | Collect existing provisional band_ids |
| `sim-systems/src/runtime/steering.rs` | 77 | Populate `NeighborSnapshot.band_id` |
| `sim-systems/src/runtime/steering.rs` | 134 | Read self's band_id for outsider check |
| `sim-systems/src/runtime/steering.rs` | 832 | `is_outsider = self_band_id.is_some() && neighbor.band_id != self_band_id` |
| `sim-bridge/src/runtime_queries.rs` | 1044 | Serialize to FFI dict as i64 |
| `sim-bridge/src/lib.rs` | 1316, 1324 | Entity list row + band name lookup |
| `sim-bridge/src/lib.rs` | 2352, 2360 | Entity detail dict + band name |
| `sim-bridge/src/lib.rs` | 2739 | Selected entity band_id lookup for highlight |

### Every Write/Mutation of `identity.band_id`

| File | Line | How |
|------|------|-----|
| `sim-systems/src/runtime/band.rs:1263` | `apply_identity_band_ids` | Sets to `desired_band_id` (Some or None) for every entity |
| `sim-systems/src/runtime/band_behavior.rs:122,130,138` | Bootstrap | Sets to `Some(band_id)` for seeded agents |
| `sim-bridge/src/lib.rs:7518` | Test only | Hardcoded `Some(band_id)` in unit test fixture |

---

## B. BandStore — Global Resource

### Definition

**File**: `rust/crates/sim-core/src/band.rs:66`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BandStore {
    bands: BTreeMap<BandId, Band>,
    next_id: u64,
}
```

### Band Struct (lines 9–24)

```rust
pub struct Band {
    pub id: BandId,                   // stable monotonic ID
    pub name: String,                 // Korean nature-themed name
    pub members: Vec<EntityId>,       // current live members
    pub leader: Option<EntityId>,     // elected leader (may be None)
    pub provisional_since: u64,       // tick when provisional group formed
    pub promoted_tick: Option<u64>,   // tick when promoted to established
    pub is_promoted: bool,            // false = provisional, true = established
}
```

No `settlement_id` field. Bands carry no residential information.

### ID Allocation

Monotonic counter starting at 1. `allocate_id()` (line 81):

```rust
pub fn allocate_id(&mut self) -> BandId {
    let id = BandId(self.next_id);
    self.next_id = self.next_id.saturating_add(1).max(1);
    id
}
```

IDs are **never reused** after removal.

### BandStore Methods

| Method | Line | Purpose |
|--------|------|---------|
| `new()` | 73 | Empty store, next_id=1 |
| `allocate_id()` | 81 | Monotonic BandId allocation |
| `insert(band)` | 88 | Insert or replace |
| `get(id)` | 93 | Immutable lookup |
| `get_mut(id)` | 98 | Mutable lookup |
| `remove(id)` | 103 | Remove and return |
| `all()` | 108 | Iterate all in BandId order |
| `all_mut()` | 113 | Iterate all mutable |
| `len()` | 118 | Count |
| `is_empty()` | 123 | Empty check |
| `find_band_for(entity)` | 128 | Reverse lookup: EntityId → BandId |

### Access Pattern in Systems

`BandStore` lives at `resources.band_store` — it is a field on `SimResources`:

```rust
// sim-engine/src/engine.rs:139
pub band_store: BandStore,
// initialized at line 260:
band_store: BandStore::new(),
```

Systems access it as `resources.band_store.method()`. No ECS query required.

---

## C. BandFormationSystem

**File**: `rust/crates/sim-systems/src/runtime/band.rs`

**Struct**: `BandFormationSystem` (line 21)

**Registration**:
- Priority: `config::BAND_FORMATION_SYSTEM_PRIORITY = 27`
- Interval: `config::BAND_FORMATION_TICK_INTERVAL = 60` (runs every 60 ticks)

### GFS Formula (lines 96–122)

```
GFS(a, b) =
    0.25 * proximity          // distance-based, boosted if same settlement
  + 0.25 * kinship            // kinship_r(a, b) * 2.0, clamped 0–1
  + 0.20 * mutual_trust       // (trust_a→b + trust_b→a) / 2
  + 0.10 * shared_values      // values.alignment_with()
  + 0.10 * threat_pressure    // 1.0 - avg(safety_a, safety_b)
  + 0.10 * resource_factor    // settlement_resource_score
```

**Proximity detail**: `(1.0 - distance / GFS_PROXIMITY_MAX_DISTANCE).clamp(0.0, 1.0)`. When both agents share the same `settlement_id`, proximity is further boosted via `.sqrt()` (pushes it toward 1.0 more gently at close range).

**Threshold**: edges are added to the adjacency graph when GFS ≥ `BAND_GRANOVETTER_BASE_THRESHOLD = 0.5`.

### Band Creation Flow (lines 137–340)

1. Collect `AgentSnapshot` for all alive agents
2. **Candidates** = agents with `band_id.is_none()` OR in a provisional band (not promoted)
3. Build adjacency graph (GFS ≥ threshold between every candidate pair)
4. Find connected components via BFS (`find_connected_components`)
5. Each component with ≥ `BAND_MIN_SIZE_PROVISIONAL (3)` members → `build_proposed_band`
6. `proposed.existing_band_ids` = all provisional band IDs already among the component members
7. Primary band_id = `min(existing_band_ids)` or `allocate_id()` if none exist
8. Other existing provisional bands in the component are **dissolved** (merged into primary)
9. Create `Band::new(primary_band_id, name, members, tick)` — starts as provisional
10. After `BAND_PROMOTION_TICKS (1440)` ticks since `provisional_since`, band is promoted

### Loner Join Logic

Single agents (component size = 1) may join an existing promoted band if GFS with any member ≥ threshold. Emits `LonerJoinedBand`.

### Events Emitted

| Event | When |
|-------|------|
| `BandFormed` | New provisional group created |
| `BandPromoted` | Group promoted to established |
| `BandSplit` | Fission splits a promoted band |
| `BandDissolved` | Band falls below min size or forced merge |
| `BandLeaderElected` | Leader elected in promoted band |
| `LonerJoinedBand` | Lone agent joins existing band |

---

## D. BandSplitSystem / Fission Logic

Fission is **integrated into `BandFormationSystem::run`** in the same file (`band.rs`). There is no separate `BandSplitSystem`.

### BandSplitCause Enum (lines 57–61)

```rust
enum BandSplitCause {
    TrustCollapse,
    ValueClash,
    Overpopulation,
}
```

### Fission Trigger (`determine_split_cause`, lines 822–836)

Only promoted bands can fission. Evaluation priority:

```rust
if band.member_count() > BAND_MAX_SIZE (30)        → Overpopulation
else if avg_trust < BAND_FISSION_TRUST_THRESHOLD (0.15)  → TrustCollapse
else if avg_values < BAND_FISSION_VALUES_THRESHOLD (0.20) → ValueClash
```

`avg_trust` and `avg_values` are means over all pairwise `PairMetric` values for band members.

### Discontented Sub-group Selection (`apply_band_fission`, line 710+)

- **Overpopulation**: `split_members_by_position` — sort members by (x, y), split at midpoint (line 884–915)
- **TrustCollapse / ValueClash**: `split_members_by_social_graph` — find the weakest-trust cut (line 917+)

### Outcome

- Retained group keeps the original `BandId`
- Split-off group gets `band_store.allocate_id()` — new `Band::new(...)` with `is_promoted: false` then immediately promoted
- `apply_band_fission` → inserts both into `final_bands` → `apply_identity_band_ids` reconciles all `identity.band_id`

---

## E. MigrationRuntimeSystem

**File**: `rust/crates/sim-systems/src/runtime/world.rs`

### Migration Trigger Conditions

Settlement population exceeds capacity threshold; group size must meet `MIGRATION_GROUP_SIZE_MIN`. Per-settlement cooldowns and stochastic chance involved.

### Migration Group Selection

Agents selected from the source settlement based on family proximity and willingness metrics.

### Fields Updated on Migrating Agents (line 960)

```rust
identity.settlement_id = Some(next_settlement_id);
```

Position is also updated (moved to destination settlement coordinates).

### band_id: NOT TOUCHED

**Confirmed**: zero references to `band_id` or `BandStore` in `world.rs`.

This is the core architectural gap: an agent that migrates to settlement B retains their `band_id` pointing to a band whose other members may all be in settlement A. The band's `members` list in `BandStore` is not updated. The next `BandFormationSystem` run (every 60 ticks) will detect the GFS has dropped (proximity collapses across settlements) and dissolve / recompose bands — but there is a 0–60 tick window of stale membership, and the dissolution is driven by GFS recalculation rather than explicit migration handling.

---

## F. Other Systems Referencing band_id

### 1. SteeringSystem — `sim-systems/src/runtime/steering.rs`

- `NeighborSnapshot` struct (lines 44, 51) carries `band_id: Option<BandId>` for each scanned neighbor
- Reads `identity.band_id` at line 77 to populate neighbor snapshots
- Reads self `identity.band_id` at line 134
- **Outsider repulsion logic** (line 832):
  ```rust
  let is_outsider = self_band_id.is_some() && neighbor.band_id != self_band_id;
  ```
  Applies `BAND_OUTSIDER_SEPARATION_MULT = 1.5` separation force to outsiders
- **Impact of Option<BandId>**: Already handles it correctly. `None != Some(x)` is true, so a bandless agent treats everyone with a band as "outsider" (or vice versa — this is a subtle behavior gap worth noting for the redesign).
- Test fixtures (lines 961, 979, 984, 1012, 1023, 1319): use hardcoded `None` and `Some(BandId(1))` / `Some(BandId(2))`

### 2. TerritoryRuntimeSystem — `sim-systems/src/runtime/territory.rs`

- Line 129: `for band in resources.band_store.all()` — iterates bands to stamp faction territory
- Line 134: `band_faction_id = (band.id.0 as u16).wrapping_add(1000)` — bands use faction IDs 1000+ (settlements use 1+)
- Does **not** read `identity.band_id` directly; uses `band_store` as ground truth
- **Impact**: No change needed. Works off BandStore, not identity fields.

### 3. SimBridge — `sim-bridge/src/lib.rs` and `runtime_queries.rs`

| Method / Location | Line | What it does |
|-------------------|------|-------------|
| `runtime_band_id_raw()` | rq:1022 | `Option<BandId>` → i64, `None` → `-1` |
| Entity snapshot dict | rq:1044 | `"band_id"` field |
| `collect_entity_list_rows` | lib:1340 | Includes `band_id` per entity row |
| Entity detail | lib:2352–2370 | `band_id` + `band_name` in detail dict |
| Selected entity highlight | lib:2739 | Reads `id.band_id` for member highlight |
| `runtime_get_band_list()` | lib:3214 | Full band list: id, name, member_count, is_promoted, leader, member_ids |
| `runtime_get_band_detail()` | lib:3261 | Full band snapshot by band_id |
| `runtime_queries.rs` reset | rq:238 | `resources.band_store = BandStore::new()` (world reset) |
| **Impact**: All already handle `None` via `runtime_band_id_raw`. No breakage. | | |

### 4. StorySifter — `sim-systems/src/runtime/story_sifter.rs`

- Line 691–692: Band events scored for narrative importance: `BandSplit = 0.8`, `BandFormed/Promoted = 0.6`
- Lines 749–754: Event type → string labels for chronicle display
- Reads events by type only; does not access `band_id` or `BandStore` directly

### 5. LLMRequestSystem — `sim-systems/src/runtime/llm_request_system.rs`

- Lines 357–360: Subscribes to `BandFormed`, `BandPromoted`, `BandSplit`, `BandDissolved` events to trigger LLM narrative generation
- No direct `band_id` field access; works through events

### 6. Chronicle — `sim-engine/src/chronicle.rs`

- Lines 1842–1869: Dedicated `band_events` buffer (capacity 200) for `ChronicleEventType::BandLifecycle` events
- Band events are **never evicted** by movement noise (separate buffer)

---

## G. GDScript UI — Band References

### entity_renderer.gd (`scripts/ui/renderers/entity_renderer.gd`)

This is the primary consumer of band data in GDScript.

**State variables** (lines 27–38):
```gdscript
var _band_territory_sprite: Sprite2D
var _band_territory_material: ShaderMaterial
var _band_id_texture: ImageTexture       # packed faction ID grid texture
var _band_density_texture: ImageTexture  # packed density grid texture
var _band_territory_timer: float
var _runtime_band_list_cache: Array
var _runtime_band_list_cache_tick: int
```

**Layer toggle** (line 45): `"band": true` — band layer is on by default.

**Draw flow**:
- `_update_band_territory(delta)` (line 1373) — throttled by `BAND_TERRITORY_INTERVAL`, calls `_refresh_band_territory()`
- `_draw_band_labels(zl)` (line 1336) — calls `_get_runtime_band_list()`, draws `"name · count"` label at centroid of member positions

**SimBridge calls**:
- `SimBridge.runtime_get_band_list()` → Array of Dicts `{id, name, member_count, is_promoted, leader_id, leader_name, member_ids}`
- Territory shader data: `SimBridge` territory query returning `{faction_ids: PackedByteArray, faction_count: int, density: PackedByteArray}`

**"No band" handling**:
- Entity detail returns `band_id = -1` and `band_name = ""` when `identity.band_id` is `None`
- The renderer has no explicit "bandless agent" UI state — agents without a band simply don't appear in band label rendering
- No crash guards for bandless agents in current renderer code

### Other GDScript files

No other `.gd` files reference `band` beyond `entity_renderer.gd` in the first 40 grep results. Band sidebar panel (if any) delegates entirely to SimBridge `runtime_get_band_detail`.

---

## H. EventBus — Band-Related Events

**Source file**: `rust/crates/sim-engine/src/event_store.rs` (`SimEventType` enum)

| Variant | Line | Emitted By | Consumed By |
|---------|------|------------|-------------|
| `BandFormed` | 48 | `band.rs` formation | story_sifter, llm_request_system, chronicle |
| `BandPromoted` | 50 | `band.rs` formation | story_sifter, llm_request_system, chronicle |
| `BandSplit` | 52 | `band.rs` fission | story_sifter, llm_request_system, chronicle |
| `BandDissolved` | 54 | `band.rs` cleanup | story_sifter, llm_request_system, chronicle |
| `BandLeaderElected` | 56 | `band.rs` formation | story_sifter, chronicle |
| `LonerJoinedBand` | 58 | `band.rs` formation | story_sifter, chronicle |

Events carry `band_id: BandId` and `members: Vec<EntityId>` payloads (confirmed via emit functions in band.rs lines 1330–1384).

Chronicle stores `BandLifecycle` events in a dedicated non-evicting buffer (capacity 200).

---

## I. Config Constants

**File**: `rust/crates/sim-core/src/config.rs`

| Constant | Value | Used By |
|----------|-------|---------|
| `BAND_MIN_SIZE_PROVISIONAL` | `3` | formation: minimum to form a candidate group |
| `BAND_MIN_SIZE_PROMOTED` | `3` | cleanup: dissolve promoted bands below this |
| `BAND_MAX_SIZE` | `30` | fission trigger (Overpopulation) |
| `BAND_PROMOTION_TICKS` | `1440` | ticks required before provisional → promoted |
| `BAND_GRANOVETTER_BASE_THRESHOLD` | `0.5` | GFS edge threshold |
| `BAND_FORMATION_SYSTEM_PRIORITY` | `27` | system scheduler |
| `BAND_FORMATION_TICK_INTERVAL` | `60` | runs every 60 ticks |
| `BAND_BEHAVIOR_SYSTEM_PRIORITY` | `28` | system scheduler |
| `BAND_BEHAVIOR_TICK_INTERVAL` | `10` | runs every 10 ticks |
| `BAND_COHESION_WEIGHT` | `0.8` | steering cohesion toward band center |
| `BAND_OUTSIDER_SEPARATION_MULT` | `1.5` | steering repulsion for non-band agents |
| `BAND_FISSION_TRUST_THRESHOLD` | `0.15` | fission: TrustCollapse trigger |
| `BAND_FISSION_VALUES_THRESHOLD` | `0.20` | fission: ValueClash trigger |

---

## J. Harness Tests

**File**: `rust/crates/sim-test/src/main.rs`

### Band-Specific Harness Tests

| Test | Line | What it asserts |
|------|------|----------------|
| `harness_band_count_reasonable` | 1359 | 20 agents, 4380 ticks → band_count in [1, 5] |

```rust
assert!(band_count <= 5, "expected at most 5 bands for 20 agents");
assert!(band_count >= 1, "expected at least 1 band");
```

### Band-Adjacent Unit Tests (in band_behavior.rs)

| Test | File | What it asserts |
|------|------|----------------|
| `band_behavior_sets_center_for_members` | band_behavior.rs:114 | Band center = mean position of 3 members |
| `band_behavior_clears_center_for_loners` | band_behavior.rs:169 | Loner's band_center cleared to None |

### Impact of Redesign on Tests

- **`harness_band_count_reasonable`**: assertion `<= 5` is settlement-unaware. With multi-settlement redesign and Dunbar L2 cap (~15), a 20-agent world on a single settlement should still produce 1–2 bands → assertion survives but is fragile. Will need revision to assert per-settlement band count.
- **`band_behavior_sets_center_for_members`**: Not affected. Tests steering state only.
- **`band_behavior_clears_center_for_loners`**: Not affected.

No existing harness test covers the migration→band split scenario at all.

---

## K. Summary: Impact Assessment for Option<BandId> Migration

### Critical Finding

**`band_id` is already `Option<BandId>`** — the type migration described in the ticket is already done at `identity.rs:15`. No type-level refactoring is needed.

The actual redesign work is behavioural, not structural.

### Total Files Affected by Redesign

| File | Change Category |
|------|----------------|
| `rust/crates/sim-systems/src/runtime/band.rs` | Formation logic: add settlement_id filter to GFS candidates; fission: cross-settlement split |
| `rust/crates/sim-systems/src/runtime/world.rs` | Migration gap: update band membership when `settlement_id` changes |
| `rust/crates/sim-core/src/config.rs` | `BAND_MAX_SIZE` 30 → 15 (Dunbar Layer 2) |
| `rust/crates/sim-test/src/main.rs` | Update `harness_band_count_reasonable` for settlement-awareness; add new migration→split harness |

**Total files affected: 4**

### Systems Affected: 3

1. `BandFormationSystem` (band.rs)
2. Migration system (world.rs)
3. Harness tests (sim-test)

### Breaking Changes

1. **Formation candidate filter** — currently: `band_id.is_none() || in_provisional`. New requirement: also filter by `settlement_id` so agents in different settlements never form the same band. This changes which connected components form.

2. **Migration must invalidate band membership** — in `world.rs`, after setting `identity.settlement_id`, must either: (a) set `identity.band_id = None` immediately and let the next formation cycle rebuild, or (b) explicitly call band store to remove the migrating agent from their old band and trigger a split event.

3. **BAND_MAX_SIZE reduction** — dropping from 30 → 15 will trigger more `Overpopulation` fissions in existing simulations. Harness test assertion `<= 5 bands` may fail during transition if not recalibrated.

### Non-Breaking Changes (already correct)

- `identity.rs`: `Option<BandId>` type — no change
- `steering.rs`: outsider logic — already uses `Option` comparison correctly
- `territory.rs`: reads `band_store` directly — no change needed
- `sim-bridge` / `runtime_queries.rs`: `runtime_band_id_raw` already maps `None → -1`
- GDScript `entity_renderer.gd`: handles `-1` band_id gracefully

### Suggested Migration Order (dependency chain)

```
1. config.rs          — lower BAND_MAX_SIZE to 15
                        (no code dependencies, safe to change first)

2. world.rs           — on migration, set identity.band_id = None
                        (unblocks clean state before next formation cycle)

3. band.rs            — add settlement_id guard to GFS candidate filter
                        (requires world.rs to be correct first, or races)

4. band.rs            — add cross-settlement forced dissolution pass
                        (runs in same system, after candidate filter change)

5. sim-test/main.rs   — update harness_band_count_reasonable
                        add harness_band_migration_split
                        (last, after all Rust changes pass gate)
```

### Gaps Not Covered by Any Existing Code

1. **No test for cross-settlement band membership** — no harness asserts that a band's members all share the same `settlement_id`
2. **No migration→split event chain** — when migration splits a band, no `BandSplit` event is emitted (the event would only fire when GFS recomputes 0–60 ticks later)
3. **Steering outsider logic with bandless agents** — current code: `is_outsider = self_band_id.is_some() && neighbor.band_id != self_band_id`. A bandless agent (`self_band_id = None`) never treats anyone as an outsider. During the transition period where migrating agents are temporarily `None`, this could cause unexpected clustering behaviour.

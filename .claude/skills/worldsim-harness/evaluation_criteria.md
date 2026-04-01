# Harness Evaluation Criteria

Pass/fail conditions and root cause checklists per test category.
Use this when a harness test fails — work through the checklist before retrying.

---

## Job System

### Pass conditions
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| miner count | ≥ 1 | 2000 |
| lumberjack count | ≥ 1 | 2000 |
| builder count | ≥ 1 | 2000 |
| no single job > 80% of agents | — | 2000 |

### Fail: root cause checklist

- [ ] **Default job not cleared**: `OccupationSystem` must reset job on first run — check `first_run` flag logic
- [ ] **alive_count wrong**: `JobDistributionSystem` uses `alive_count` from resources; stale value → wrong ratios
- [ ] **Rebalance interval**: system only rebalances every N ticks; run enough ticks (≥ 2000)
- [ ] **Specialization lock**: `OccupationSystem` may overwrite job for unspecialized agents — check interaction with `JobDistributionSystem`
- [ ] **Settlement missing**: job distribution requires a settlement in `resources.settlements` — `make_stage1_engine` sets one up, verify it persists

### Diagnostic pattern
```rust
let world = engine.world();
let mut counts: HashMap<String, usize> = HashMap::new();
for (_, behavior) in world.query::<&Behavior>().iter() {
    *counts.entry(behavior.job.clone()).or_insert(0) += 1;
}
println!("[diag] job counts: {:?}", counts);
println!("[diag] alive agents: {}", world.len());
```

---

## Resource Collection

### Pass conditions
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| stockpile_stone (sum all settlements) | > 0.0 | 4380 |
| stockpile_wood (sum all settlements) | > 0.0 | 4380 |

### Fail: root cause checklist

- [ ] **Miner not assigned**: stone requires miners; check job distribution first
- [ ] **Search radius too small**: miner searches within radius 40 for stone tiles; `make_stage1_engine` seeds tiles at dx/dy ≤ 30 from (128,128)
- [ ] **Tile resource depleted**: tiles start at 100.0 — check if `regen_rate: 0.1` is applied
- [ ] **Beach fallback**: if all tiles in radius are water/impassable, miner falls back to gathering — verify tile passability
- [ ] **Deficit scoring not triggering**: `ResourceDeficitSystem` must flag stone as deficit before miners are assigned — check deficit threshold
- [ ] **Gathering vs mining**: "gatherer" gathers food, "miner" gathers stone — check job string matches exactly

### Diagnostic pattern
```rust
let resources = engine.resources();
for (sid, settlement) in &resources.settlements {
    println!("[diag] settlement {:?}: stone={:.1} wood={:.1} food={:.1}",
        sid, settlement.stockpile_stone, settlement.stockpile_wood, settlement.stockpile_food);
}
let map = &resources.map;
let mut stone_tiles = 0usize;
for y in 110..150u32 { for x in 110..150u32 {
    let tile = map.get(x, y);
    if tile.resources.iter().any(|r| matches!(r.resource_type, sim_core::ResourceType::Stone)) {
        stone_tiles += 1;
    }
}}
println!("[diag] stone tiles near spawn: {}", stone_tiles);
```

---

## Building Construction

### Pass conditions
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| complete buildings (total) | ≥ 3 | 4380 |
| complete buildings of type "shelter" | ≥ 1 | 4380 |

Baseline 3 = campfire + stockpile_pile + shelter (all auto-constructed in stage 1).

### Fail: root cause checklist

- [ ] **Stone not available**: shelter requires stone — check resource collection first
- [ ] **Shelter build cost**: verify `shelter.ron` cost matches available stockpile
- [ ] **has_incomplete_site**: `ConstructionSystem` only assigns builders when `has_incomplete_site` is true — check site blueprint spawning
- [ ] **Builder count zero**: building requires builders; check job distribution
- [ ] **Building not marked complete**: `ConstructionSystem` sets `is_complete = true` when `progress >= cost` — check progress accumulation
- [ ] **Wrong building type string**: test matches `building_type == "shelter"` — must match RON file key exactly

### Diagnostic pattern
```rust
let resources = engine.resources();
for (bid, building) in &resources.buildings {
    println!("[diag] building {:?}: type={} complete={} progress={:.1}",
        bid, building.building_type, building.is_complete, building.progress);
}
```

---

## Band Stability

### Pass conditions
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| band count | ≥ 1 | 4380 |
| band count | ≤ 5 | 4380 |

≤ 5 for 20 agents prevents runaway fission splitting every agent into its own band.

### Fail: root cause checklist

- [ ] **Fission threshold too low**: fission triggers when tension > 0.15 (warning) / 0.20 (split) — check `TensionSystem` output
- [ ] **Minimum band size**: bands must have ≥ 3 members to fission; with 20 agents max ~6 bands possible
- [ ] **Promotion ticks**: leader promotion requires 1440 ticks (1 game day) — run enough ticks
- [ ] **Band store not initialised**: `BandFormationSystem` must have run at least once — check system registration
- [ ] **band_store.all()** counts including disbanded bands: verify `all()` returns only active bands

### Diagnostic pattern
```rust
let resources = engine.resources();
let band_count = resources.band_store.all().count();
println!("[diag] active bands: {}", band_count);
for band in resources.band_store.all() {
    println!("[diag]   band {:?}: members={} leader={:?}",
        band.id, band.members.len(), band.leader);
}
```

---

## Territory System

### Pass conditions
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| active factions | ≥ 1 | 2000 |
| max territory value | > 0.01 | 2000 |
| territory on impassable tiles | = 0 (negative) | 2000 |

### Fail: root cause checklist

- [ ] **System not registered**: `TerritoryRuntimeSystem` must be in `register_all_systems()` — check with `grep "TerritoryRuntimeSystem" rust/crates/sim-test/src/main.rs`
- [ ] **InfluenceRuntimeSystem missing**: territory depends on influence grid being populated first
- [ ] **Buildings not complete**: territory stamping triggers on building completion — if no buildings, no territory
- [ ] **stamp_gaussian_terrain not called**: `TerritoryRuntimeSystem` calls this; check it filters by `passable`
- [ ] **passable_cache stale**: influence grid caches passability at init — check cache invalidation on map changes
- [ ] **Faction ID mismatch**: `active_factions()` returns faction IDs that must match keys in territory grid

### Diagnostic pattern
```rust
let resources = engine.resources();
let factions = resources.territory_grid.active_factions();
println!("[diag] territory factions: {}", factions.len());
for fid in &factions {
    if let Some(data) = resources.territory_grid.get(*fid) {
        let max: f32 = data.iter().cloned().fold(0.0_f32, f32::max);
        let nonzero = data.iter().filter(|&&v| v > 0.001).count();
        println!("[diag]   faction {:?}: max={:.4} nonzero_cells={}", fid, max, nonzero);
    }
}
```

---

## Population

### Pass conditions (future tests)
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| agent count after 1 year | ≥ 15 (no mass death) | 4380 |
| births in 2 years | ≥ 1 | 8760 |

### Fail: root cause checklist

- [ ] **Starvation**: food stockpile depleted → agents die; check gatherer assignment
- [ ] **Stress cascade**: high stress → death; check stress/coping system
- [ ] **Birth conditions**: requires adult pair in same band + food + shelter — all must be satisfied

---

## Economy (future tests)

### Pass conditions
| Assertion | Threshold | Ticks |
|-----------|-----------|-------|
| crafting output ≥ 1 item | — | 4380 |
| trade route established | — | 8760 |

### Fail: root cause checklist

- [ ] **Recipe prerequisites**: `CraftingSystem` checks tag+threshold recipe — materials must meet threshold
- [ ] **Workbench required**: some recipes need a building of specific type present and complete
- [ ] **Crafter job**: dedicated crafter job must be assigned (future — currently gatherer may craft opportunistically)

# Harness Test Templates

Copy-paste patterns for `rust/crates/sim-test/src/main.rs`.
All templates go inside `#[cfg(test)] mod tests { ... }`.

---

## Tick Count Reference

| Ticks | Game time | Use for |
|-------|-----------|---------|
| 100 | ~2.5 min | Smoke test — no panic, agents move |
| 500 | ~12 min | Job assignment after first rebalance |
| 2000 | ~50 min | Job distribution steady state |
| 4380 | 1 year | Resource accumulation, building construction |
| 8760 | 2 years | Population dynamics, births |
| 17520 | 4 years | Long-term stability, tech discovery |

---

## Standard Setup

```rust
// Always: seed=42, agent_count=20, 256×256 map with seeded tile resources
let mut engine = make_stage1_engine(42, 20);
engine.run_ticks(N);
```

---

## Job Distribution

```rust
#[test]
fn harness_job_<assertion>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(2000);

    let world = engine.world();
    let mut job_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (_, behavior) in world.query::<&Behavior>().iter() {
        *job_counts.entry(behavior.job.clone()).or_insert(0) += 1;
    }

    let miner = *job_counts.get("miner").unwrap_or(&0);
    let lumberjack = *job_counts.get("lumberjack").unwrap_or(&0);
    let builder = *job_counts.get("builder").unwrap_or(&0);

    println!("[harness] jobs: {:?}", job_counts);

    assert!(miner >= 1, "expected ≥1 miner, got {miner}. jobs={job_counts:?}");
    assert!(lumberjack >= 1, "expected ≥1 lumberjack, got {lumberjack}");
    assert!(builder >= 1, "expected ≥1 builder, got {builder}");
}
```

---

## Resource Stockpile

```rust
#[test]
fn harness_resource_<assertion>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(4380);

    let resources = engine.resources();
    let stone: f64 = resources.settlements.values().map(|s| s.stockpile_stone).sum();
    let wood: f64 = resources.settlements.values().map(|s| s.stockpile_wood).sum();

    println!("[harness] stockpile: stone={:.1} wood={:.1}", stone, wood);

    assert!(stone > 0.0, "expected stone > 0 after 1 year, got {stone}");
    assert!(wood > 0.0, "expected wood > 0 after 1 year, got {wood}");
}
```

---

## Building Count

```rust
#[test]
fn harness_building_<assertion>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(4380);

    let resources = engine.resources();
    let total = resources.buildings.values().filter(|b| b.is_complete).count();
    let of_type = resources.buildings.values()
        .filter(|b| b.is_complete && b.building_type == "shelter")
        .count();

    println!("[harness] buildings: total={} shelters={}", total, of_type);

    assert!(total >= 3, "expected ≥3 complete buildings, got {total}");
    assert!(of_type >= 1, "expected ≥1 shelter, got {of_type}");
}
```

---

## Band Count

```rust
#[test]
fn harness_band_<assertion>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(4380);

    let resources = engine.resources();
    let bands = resources.band_store.all().count();

    println!("[harness] bands: {}", bands);

    assert!(bands >= 1, "expected ≥1 band, got {bands}");
    assert!(bands <= 5, "expected ≤5 bands for 20 agents (over-splitting), got {bands}");
}
```

---

## Territory Grid Data

```rust
#[test]
fn harness_territory_<assertion>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(2000);

    let resources = engine.resources();
    let factions = resources.territory_grid.active_factions();

    println!("[harness] territory factions: {}", factions.len());
    assert!(!factions.is_empty(), "expected ≥1 territory faction after 2000 ticks");

    let mut max_val: f32 = 0.0;
    for fid in &factions {
        if let Some(data) = resources.territory_grid.get(*fid) {
            for &v in data {
                if v > max_val { max_val = v; }
            }
        }
    }

    println!("[harness] territory max_value: {:.4}", max_val);
    assert!(max_val > 0.01, "expected non-trivial territory, max={max_val}");
}
```

---

## Terrain Constraint (Negative Test)

```rust
#[test]
fn harness_<system>_not_on_impassable() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(2000);

    let resources = engine.resources();
    let map_w = resources.map.width as usize;

    for fid in resources.territory_grid.active_factions() {
        if let Some(data) = resources.territory_grid.get(fid) {
            for y in 0..resources.map.height {
                for x in 0..resources.map.width {
                    let tile = resources.map.get(x, y);
                    let val = data[y as usize * map_w + x as usize];
                    if !tile.passable && val > 0.001 {
                        panic!(
                            "territory on impassable tile ({},{}) terrain={:?} val={:.4}",
                            x, y, tile.terrain, val
                        );
                    }
                }
            }
        }
    }
}
```

---

## ECS Component Query

```rust
#[test]
fn harness_<system>_<component_check>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(N);

    let world = engine.world();

    // Single component
    for (entity, behavior) in world.query::<&Behavior>().iter() {
        // assertions on behavior
        let _ = (entity, behavior);
    }

    // Multiple components
    for (entity, (behavior, identity)) in world.query::<(&Behavior, &Identity)>().iter() {
        println!("[harness] entity {:?} job={} name={}", entity, behavior.job, identity.name);
        let _ = (entity, behavior, identity);
    }

    // With filter
    let agents_with_stress: Vec<_> = world
        .query::<(&Behavior, &Stress)>()
        .iter()
        .filter(|(_, (_, stress))| stress.current > 0.5)
        .collect();
    println!("[harness] high-stress agents: {}", agents_with_stress.len());
}
```

---

## SimResources Check

```rust
#[test]
fn harness_<system>_resources() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(N);

    let resources = engine.resources();

    // Settlement fields
    assert!(resources.settlements.values().any(|s| s.stockpile_stone > 0.0));

    // Map tile check
    let tile = resources.map.get(128, 128);
    assert!(tile.passable, "spawn tile must be passable");

    // Calendar
    println!("[harness] tick={} year={}", resources.calendar.tick, resources.calendar.year);
}
```

---

## Multi-Settlement (Future)

```rust
#[test]
fn harness_<system>_multi_settlement() {
    use sim_core::ids::SettlementId;
    let mut engine = make_stage1_engine(42, 40);

    // Add second settlement
    engine.resources_mut().settlements.insert(
        SettlementId(2),
        sim_core::Settlement::new(SettlementId(2), "Second Hold".to_string(), 64, 64, 0),
    );

    engine.run_ticks(4380);

    let resources = engine.resources();
    assert_eq!(resources.settlements.len(), 2);
    // per-settlement assertions
}
```

---

## Diagnostic Println Pattern

Add inside the test to investigate failures. **Never add to production code.**

```rust
// Before assertion — prints on both pass and fail when --nocapture used
println!("[harness] <label>: value={:?}", value);

// After engine.run_ticks — snapshot all relevant state
let resources = engine.resources();
println!("[harness] tick={}", resources.calendar.tick);
println!("[harness] settlements: {}", resources.settlements.len());
println!("[harness] buildings: complete={} total={}",
    resources.buildings.values().filter(|b| b.is_complete).count(),
    resources.buildings.len());

// Print all jobs
let mut jobs: std::collections::HashMap<String, usize> = Default::default();
for (_, b) in engine.world().query::<&Behavior>().iter() {
    *jobs.entry(b.job.clone()).or_insert(0) += 1;
}
println!("[harness] jobs: {:?}", jobs);
```

---

## Helper Functions (already in sim-test)

These are already defined inside `#[cfg(test)] mod tests` — do not redefine:

```rust
make_stage1_engine(seed: u64, agent_count: usize) -> SimEngine
collect_positions(engine: &SimEngine) -> Vec<(u64, (f64, f64))>
```

If you need additional helpers, add them inside the same `mod tests` block:

```rust
fn count_jobs(engine: &SimEngine) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for (_, behavior) in engine.world().query::<&Behavior>().iter() {
        *counts.entry(behavior.job.clone()).or_insert(0) += 1;
    }
    counts
}

fn total_stockpile_stone(engine: &SimEngine) -> f64 {
    engine.resources().settlements.values().map(|s| s.stockpile_stone).sum()
}

fn count_complete_buildings(engine: &SimEngine) -> usize {
    engine.resources().buildings.values().filter(|b| b.is_complete).count()
}
```

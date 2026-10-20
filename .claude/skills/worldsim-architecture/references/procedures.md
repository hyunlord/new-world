# worldsim-architecture — procedures in full

Expanded versions of Procedures A and B in `../SKILL.md`, with skeletons
taken from the code's existing shapes (`RuntimeSystem` in
`rust/crates/sim-engine/src/lib.rs`, `register_*_systems` in
`rust/crates/sim-systems/src/lib.rs`, `get_agent_family` →
`collect_agent_family` → `agent_family_to_dict` in
`rust/crates/sim-bridge/src/ffi/family.rs`). Names in angle brackets are
placeholders.

## A. Add a system

### A1. Before writing code
1. Grep for the feature: `grep -rn "<Name>\|<Type>" rust/crates/ | head`, `ls rust/crates/sim-systems/src/runtime/<domain>/`.
2. If something exists, check its wiring: is it registered by `register_default_runtime_systems`? Does `WorldRules::baseline(seed)` set its rule (`Some(..)`), or does the system return at once? Does the bridge expose its data? The usual gap is one missing registration, rule value or getter, not a whole system.
3. Decide the priority: read the generated table in `docs/ARCHITECTURE.md`. Lower runs first. A system that consumes another's output must have a higher number. Two systems may share a number only on purpose; then registration order decides (the sort is stable).
4. Decide the interval: `tick_interval() = k` runs on ticks where `current_tick % k == 0`. 1,440 ticks are one game day; hourly is 60. The display clock is offset (`clock_offset_ticks`), so read hours through the engine's time of day, not `tick % 1440`.

### A2. Data
- Component (`rust/crates/sim-core/src/components/<name>.rs`, exported from `components/mod.rs`):
  ```rust
  //! <Name>: <what it marks or stores, who writes it>.
  use serde::{Deserialize, Serialize};

  /// <What one value means; units (ticks, permille 0..=1000, whole units)>.
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
  pub struct <Name> {
      /// <field meaning and unit>.
      pub <field>: u32,
  }
  ```
  Integers or permille, not floats, for new state. Zero-sized marker structs are fine (`Cold`, `Elder`).
- Rule (`rust/crates/sim-core/src/rules/<sub>.rs`): a field on the sub-rule struct, `#[serde(default)]`, `///` doc with unit; an optional mechanic is `Option<<Rule>>` on `AgentRules` / `SocietyRules` (absent = the system is a no-op). The baseline value goes in `rules/baseline.rs` as a flat literal.
- A new rule field changes `format!("{:#?}", WorldRules::baseline(s))`, so `rust/crates/sim-core/tests/baseline_matches_v7_final.rs` fails on purpose: update its expected dump in the same commit and give the reason in the commit message.
- Non-ECS state goes on `SimResources` (sim-engine) as a `BTreeMap`/`Vec`, never a new `HashMap`. Measurement-only fields are never read by systems.

### A3. The system file
```rust
//! <Name>System: <one sentence>.
//!
//! Priority: <n> (`<NAME>_PRIORITY`). Interval: <k> ticks.
//! Reads: <Comp>, <Comp>, SimResources.<field>, rules.<path>
//! Writes: <Comp> (insert/remove), SimResources.<field>
//! Events: <chronicle kind / CausalEvent variant>, or none
//!
//! <Longer notes: order guarantees, which draws, what happens without the rule.>
use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};

/// Engine priority of [`<Name>System`]: after <X> (<n-1>), before <Y>.
pub const <NAME>_PRIORITY: u32 = <n>;
/// Ticks between runs of [`<Name>System`].
pub const <NAME>_INTERVAL: u64 = <k>;

/// <What the system does each run.>
#[derive(Debug, Default)]
pub struct <Name>System;

impl RuntimeSystem for <Name>System {
    fn name(&self) -> &str {
        "<Name>System"
    }
    fn priority(&self) -> u32 {
        <NAME>_PRIORITY
    }
    fn tick_interval(&self) -> u64 {
        <NAME>_INTERVAL
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.<path> else { return };
        // 1. Read: collect what you need into a Vec, sorted by AgentId.
        // 2. Decide: pure integer logic; keyed draws only.
        // 3. Apply: structural changes after the query, in AgentId order.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Unit tests of the pure decision logic and of truncation edges.
}
```
Notes:
- `Reads:` and `Writes:` must each be one line inside the leading `//!` block (blank `//!` lines are fine). `tools/v8/system_table.py` stops at the first non-`//!` line.
- `fn name` must return a string literal; the table generator matches `fn name(&self) -> &str { "<Name>" }`.
- Collect-then-apply pattern for structural changes:
  ```rust
  let mut changes: Vec<(AgentId, hecs::Entity)> = world
      .query::<&Agent>()
      .iter()
      .filter(|(_, agent)| /* condition */ true)
      .map(|(entity, agent)| (agent.id, entity))
      .collect();
  changes.sort_unstable_by_key(|&(id, _)| id);
  for (_, entity) in changes {
      let _ = world.insert_one(entity, <Marker>);
  }
  ```
  Prefer `World::exchange` when one component replaces another.
- Keyed draw: `keyed_below(resources.world_seed, STREAM_<NEW>, lineage.key, tick, slot, replicate, n)` with `replicate` from the engine's noise fork. Add `STREAM_<NEW>` to `STREAMS` in `rust/crates/sim-core/src/rng.rs`.

### A4. Registration
```rust
/// Register <what>.
pub fn register_<domain>_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::<domain>::<Name>System));
}
```
Call it from `register_default_runtime_systems` (`rust/crates/sim-systems/src/lib.rs`). The engine re-sorts by priority after every push.

### A5. Tests and docs
1. Module unit tests (`#[cfg(test)]`) for the pure parts.
2. A domain test in `rust/crates/sim-test/tests/<domain>_*.rs` on `sim_test::baseline_engine(seed)` that asserts a measured invariant with a threshold (see `worldsim-testing`). `WorldRules::default()` fixtures switch the rules-gated systems off and prove nothing about them.
3. If the order of anything changed, extend a determinism twin (`rust/crates/sim-test/tests/determinism.rs`) to the new state.
4. `python3 tools/v8/system_table.py --write`, then `bash tools/v8/check_docs.sh`.
5. Gate. Expect identity to fail when behaviour changed on purpose; record it.

## B. Add a bridge read

### B1. Collector (`rust/crates/sim-bridge/src/ffi/<concern>.rs`)
```rust
/// One row of [`collect_<thing>`]. Ids: `agent` is an `AgentId`
/// (stable), `entity_bits` is hecs entity bits (valid until despawn).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct <Thing>Row {
    /// <meaning>.
    pub agent: AgentId,
    /// <meaning, unit>.
    pub value: u32,
}

/// <What the UI gets>. Rows are sorted by `agent`. Pure: reads only.
pub fn collect_<thing>(world: &hecs::World, resources: &SimResources) -> Vec<<Thing>Row> {
    let mut rows: Vec<<Thing>Row> = world
        .query::<(&Agent, &<Comp>)>()
        .iter()
        .map(|(_, (agent, comp))| <Thing>Row { agent: agent.id, value: comp.<field> })
        .collect();
    rows.sort_unstable_by_key(|row| row.agent);
    rows
}
```
- For a per-entity read, return `Option<Row>` and give the dictionary a `found` flag; do not use a sentinel that lies inside the valid range (founders have negative `born_tick`).
- No O(N) whole-world scan for something the UI polls every frame; index or cache it.
- Re-export from `rust/crates/sim-bridge/src/ffi/mod.rs`.

### B2. `#[func]` and marshaller
```rust
#[func]
fn get_<thing>(&self) -> VarDictionary {
    <thing>_to_dict(collect_<thing>(&self.engine.world, &self.engine.resources))
}

fn <thing>_to_dict(rows: Vec<<Thing>Row>) -> VarDictionary {
    let signed = |id: AgentId| i64::from_ne_bytes(id.to_ne_bytes());
    let agents: Vec<i64> = rows.iter().map(|r| signed(r.agent)).collect();
    let values: Vec<i32> = rows.iter().map(|r| r.value as i32).collect();
    let mut dict = VarDictionary::new();
    dict.set("agent_ids", PackedInt64Array::from(agents.as_slice()));
    dict.set("values", PackedInt32Array::from(values.as_slice()));
    dict
}
```
- Columns are parallel `Packed*Array`s of equal length; ids cross as signed i64.
- At most 14 `#[func]` parameters.
- Add a dictionary key only with a matching GDScript reader; old readers must ignore unknown keys and unknown codes.

### B3. GDScript side
- Add `"get_<thing>"` to `SimReads.GETTERS` in `scripts/core/sim_reads.gd` (and `TICK_GETTERS` if it may be cached per sim tick).
- Read with `SimReads.read(world, "get_<thing>")`, a literal method name (the boundary scanner cannot follow dynamic `call()`).
- Feature-detect new keys (`snap.has("values")`) and `duplicate()` a packed array before writing to it: a local packed array obtained from a cached dictionary aliases the cache.
- Map every new code to a localization key through `Locale`; never show raw codes.

### B4. Checks
1. A Rust test of `sim_bridge::ffi::collect_<thing>` on `baseline_engine` (values and order, not only presence).
2. `python3 tools/v8/bridge_api.py .` lists the export; update the expected list that gate step 13 compares (`docs/v8/bridge_api_*`, see `tools/v8/gate.sh`) in the same commit; CHANGELOG names the new getter.
3. `cargo build -p sim-bridge`, then `bash tools/v8/gdscript_strict_check.sh` and `bash tools/v8/smoke.sh <log_dir>`.

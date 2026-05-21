# V7 Phase 10-β — SettlementSystem + CausalEvent variants + DecisionReason + 8th cascade + Birth mechanism

## Section 1: Implementation Intent

V7 Phase 10-β is the second sub-stage of the Multi-building Settlement System (Section 11+). Building on the Phase 10-α data substrate (SettlementId/BuildingId type aliases + Settlement struct + ring-buffer history + constants + SimResources wiring), this phase lands the runtime system, causal event variants, agent decision integration, and birth mechanism that together implement autonomous settlement lifecycle.

**Why this approach:**
- Plan defaults (Q1–Q4) selected per planning §β locked decisions (P10Plan-1 through P10Plan-10)
- Phase 8-β + 9-β agent-driven cascade weighted-scoring precedent followed for 8th cascade arm
- Phase 7-β + 8-β + 9-β recurring infrastructure pattern maintained (Patterns A/B/C/D)
- Axiom #1: Settlement-level causal chain emergence (AgentDecision{SettlementReason} → SettlementFormed → AgentBorn → CombatCompleted → SettlementDissolved)

**Plan decisions applied:**
- Q1: `SettlementFormed { founding_members: Vec<AgentId> }` (plan §7, no founding_buildings field)
- Q2: `SettlementDissolved.final_population: u32` (wire format minimal type)
- Q3: Dissolution = `population_stats.current == 0 AND member_buildings.is_empty()` (plan §6 persistence semantics)
- Q4: Full birth mechanism (plan §5): `spawn_agent()` + `BIRTH_COOLDOWN_TICKS = 200` + `MAX_POP = 50` condition

## Section 2: What to Build

### New files:
- `rust/crates/sim-systems/src/runtime/settlement/mod.rs` — module declaration
- `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` — SettlementSystem (priority 138)
- `rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs` — 19 harness assertions (A1–A19)

### Modified files:
- `rust/crates/sim-core/src/causal/event.rs` — SettlementFormed (13th), SettlementDissolved (14th), DecisionReason::SettlementReason (8th) + as_str; exhaustive match arms updated
- `rust/crates/sim-engine/src/lib.rs` — `SimResources::building_registry: HashMap<BuildingId, (u32, u32)>` added for reliable building-position lookup
- `rust/crates/sim-systems/src/runtime/mod.rs` — `pub mod settlement;` added
- `rust/crates/sim-systems/src/lib.rs` — `register_settlement_systems()` added; `register_default_runtime_systems()` updated
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` — `CascadeArm::Settlement` (index 6) + 8th cascade arm (migration pull only, anti-recursion enforced)
- `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` — `SettlementFormed`/`SettlementDissolved`/`SettlementReason` → `None` in classify_event
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — `CausalEventView::from_event` arms for `"settlement_formed"` + `"settlement_dissolved"`
- `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs` — populates `building_registry` on FFI building placement
- `rust/crates/sim-systems/src/runtime/construction/construction_system.rs` — populates `building_registry` on agent-completed construction

### CausalEvent signatures (locked):
```rust
SettlementFormed {
    id: EventId,
    parent: Option<EventId>,
    settlement_id: SettlementId,
    founding_members: Vec<AgentId>,
    tick: u64,
}

SettlementDissolved {
    id: EventId,
    parent: Option<EventId>,
    settlement_id: SettlementId,
    final_population: u32,
    tick: u64,
}
```

### Scope boundary:
- No GDScript changes (backend only)
- No new localization keys
- Phase 10-γ chronicle harness is separate dispatch

## Section 3: How to Implement

### SettlementSystem (priority 138, tick_interval 1)

**Per-tick responsibilities in order:**

1. **Formation scan** (P10Plan-2-a):
   - Build agent position map: `HashMap<(u32,u32), Vec<AgentId>>` from hecs query
   - Use `resources.building_registry` for building positions
   - For each tile cluster: count agents + buildings within `SETTLEMENT_PROXIMITY_RADIUS = 5` (Chebyshev)
   - If `agent_count >= FORMATION_AGENT_THRESHOLD (3)` AND `building_count >= FORMATION_BUILDING_THRESHOLD (2)` AND cluster NOT already inside existing settlement radius:
     - `issue_settlement_id()` → `Settlement::new_with_id(id, tick)` → populate members
     - Emit `CausalEvent::SettlementFormed { founding_members: Vec<AgentId>, ... }`
     - Insert into `resources.settlements`

2. **Membership sync**:
   - For each settlement, use stored formation_tile (in `SettlementSystem.formation_tiles: HashMap<SettlementId, (u32,u32)>`) as centroid
   - Add nearby agents not already members; remove despawned agents
   - Update `population_stats.current = member_agents.len() as u32`

3. **Community history ingestion** (plan §7):
   - Scan `resources.causal_log` for events from last tick
   - `BuildingPlaced` where building is in settlement boundary → `append_history`
   - `CombatCompleted` where attacker/defender is a settlement member → `append_history`
   - `AgentBorn` where new agent spawns in settlement boundary → `append_history`
   - `AgentDecision{SettlementReason}` from settlement member → `append_history`

4. **Birth trigger** (plan §5 / Q4-B):
   - Per settlement: if `population_stats.current < SETTLEMENT_MAX_POP (50)` AND `tick - settlement.founded_at >= BIRTH_COOLDOWN_TICKS (200)`:
     - Call `resources.spawn_agent(centroid_x, centroid_y)` (SimResources wrapper delegates to SimEngine)
     - Add new AgentId to `settlement.member_agents`
     - Emit `CausalEvent::AgentBorn { agent: new_id, parent: Some(formed_event_id), tick }`
     - Track last_birth_tick in `SettlementSystem.last_birth_ticks: HashMap<SettlementId, u64>` for cooldown

5. **Dissolution check** (Q3-B):
   - If `settlement.population_stats.current == 0 AND settlement.member_buildings.is_empty()`:
     - Emit `CausalEvent::SettlementDissolved { final_population: 0, parent: Some(formed_id), ... }`
     - Remove from `resources.settlements`

### AgentDecisionSystem 8th cascade arm:

After the existing combat early-exit block, in the `else { None }` path:
```rust
// Arm 8: settlement migration pull (P10Plan-8 / P10β-8)
// Fires when agent is not a member of any settlement AND
// at least one settlement has capacity (current < MAX_POP).
let is_member = resources.settlements.values()
    .any(|s| s.member_agents.contains(&agent.id));
if !is_member {
    resources.settlements.values()
        .find(|s| s.population_stats.current < SETTLEMENT_MAX_POP)
        .and_then(|s| s.member_agents.iter().next().copied())
        .map(|first_member| (TargetKind::Agent(first_member), DecisionReason::SettlementReason))
} else {
    None
}
```

Anti-recursion: `SettlementReason` → `None` in `classify_event` (same as `MemoryReason`/`CombatReason`). CascadeArm::Settlement NOT added to `event_id_matches_arm` (settlement events never memory-encoded → never match).

### building_registry:

Added to `SimResources`: `building_registry: HashMap<BuildingId, (u32, u32)>` initialized empty.
- `BuildingStampSystem` populates on FFI building events (using building entity id as BuildingId)
- `ConstructionSystem` populates on agent-completed construction
- SettlementSystem reads registry for formation scan building positions

## Section 4: Locale

No new localization keys. Phase 10-β is backend simulation only — all new code in Rust, no GDScript UI changes.

## Section 5: Verification

### Gate commands:
```bash
cargo build --workspace           # must finish clean
cargo test --workspace            # must show all ok, 0 failed
cargo clippy --workspace --all-targets -- -D warnings  # must show no errors
```

### Phase 10-β harness (19 assertions, all pass):
```
harness_p10_beta_a1_settlement_system_priority_138
harness_p10_beta_a2_settlement_formed_variant_schema
harness_p10_beta_a3_settlement_dissolved_variant_schema
harness_p10_beta_a4_decision_reason_settlement_str
harness_p10_beta_a5_auto_formation_from_cluster
harness_p10_beta_a6_settlement_formed_event_emitted
harness_p10_beta_a7_founding_members_sorted
harness_p10_beta_a8_dissolution_requires_zero_pop_and_empty_buildings
harness_p10_beta_a9_settlement_dissolved_event_emitted
harness_p10_beta_a10_agent_born_event_emitted_after_cooldown
harness_p10_beta_a11_community_history_ingests_events
harness_p10_beta_a12_community_history_bounded_at_cap
harness_p10_beta_a13_eighth_cascade_settlement_migration_pull
harness_p10_beta_a14_settlement_reason_anti_recursion
harness_p10_beta_a15_100tick_regression_clean
harness_p10_beta_a16_no_formation_below_agent_threshold
harness_p10_beta_a17_no_formation_below_building_threshold
harness_p10_beta_a18_max_pop_caps_migration_pull
harness_p10_beta_a19_settlement_system_in_default_runtime
```

### Expected regression:
- All prior Phase 3–10α harness tests CLEAN (978+ tests)
- T7.10.A-F CLEAN
- Issues 10–16 fix regressions CLEAN

## Section 6: Pipeline Lane

`--full` (sim-core + sim-systems `.rs` changes trigger full lane: Planning debate + Visual Verify + Evaluator)

## Section 7: In-game Verification

Phase 10-β is backend-only (Rust simulation, no Godot UI changes). Pipeline VLM step will produce `VISUAL_WARNING` due to no Godot scope — adjusted score formula applies: `raw_score + 8 (VLM_SKIP/WARNING env cost)`. This is expected behavior per CLAUDE.md §7 Hook Policy.

Backend evidence sufficient:
- 19/19 Phase 10-β harness assertions pass
- All prior regression clean
- `cargo build + test + clippy` all clean
- SettlementSystem registered at priority 138 in `register_default_runtime_systems`
- settlement causal chain compiles and runs: AgentDecision{SettlementReason} → SettlementFormed → AgentBorn → SettlementDissolved

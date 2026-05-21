# Phase 10-α: Settlement Data Substrate

## Section 1: Implementation Intent

### 문제
WorldSim에는 다수의 에이전트가 모여 사는 **정착지(Settlement) 개념이 없음.**
에이전트들은 같은 공간에 있어도 집합적 정체성/공유 자원/공동 역사가 없고,
이후 Phase 10-β(SettlementSystem), 10-γ(UI), Section 11+(다중 건물 정착지 시스템)을
구현하려면 먼저 **데이터 기반(substrate)** 이 필요함.

### 해결
4개 파일에 걸친 최소 데이터 기반:
1. `sim-core`: `SettlementId`, `BuildingId`, `Settlement` struct + constants
2. `sim-engine`: `SimResources.settlements` HashMap + `next_settlement_id` + `issue_settlement_id()`
3. `sim-core/causal/event.rs`: `CausalEvent::AgentBorn` 12번째 variant 추가
4. `sim-bridge/world_node.rs` + `sim-systems/memory_system.rs`: 새 variant exhaustive match 완결

### 설계 결정 (P10Plan 고정)
- `SettlementId = u32` (type alias, AgentId=u64 패턴 mirror)
- `BuildingId = u64` (신규 type alias — 아직 BuildingId 없음)
- `community_history: Vec<EventId>` (ring buffer, SETTLEMENT_HISTORY_CAP=32)
- `AgentBorn`은 Phase 10-α 범위의 12번째 CausalEvent variant
- `next_settlement_id: u32` plain counter (SettlementSystem은 단일 스레드)

---

## Section 2: What to Build

### Part A: `rust/crates/sim-core/src/components/settlement.rs` (신규)

```rust
pub type SettlementId = u32;
pub type BuildingId = u64;

pub const SETTLEMENT_HISTORY_CAP: usize = 32;
pub const SETTLEMENT_FORMATION_AGENT_THRESHOLD: u32 = 3;
pub const SETTLEMENT_FORMATION_BUILDING_THRESHOLD: u32 = 2;
pub const SETTLEMENT_DISSOLUTION_THRESHOLD: u32 = 0;
pub const SETTLEMENT_MAX_POP: u32 = 50;
pub const SETTLEMENT_PROXIMITY_RADIUS: u32 = 5;

pub struct PopulationStats {
    pub current: u32,
    pub total_births: u32,
    pub total_deaths: u32,
}

pub struct Settlement {
    pub settlement_id: SettlementId,
    pub member_agents: HashSet<AgentId>,
    pub member_buildings: HashSet<BuildingId>,
    pub population_stats: PopulationStats,
    pub community_history: Vec<EventId>,
    pub founded_at: u64,
}
```

`Settlement::impl`:
- `new_with_id(id, founded_at) -> Self`
- `add_member_agent(&mut self, agent: AgentId) -> bool` (HashSet::insert)
- `remove_member_agent(&mut self, agent: AgentId) -> bool` (HashSet::remove)
- `add_member_building(&mut self, building: BuildingId) -> bool`
- `remove_member_building(&mut self, building: BuildingId) -> bool`
- `append_history(&mut self, event_id: EventId)` — saturating at SETTLEMENT_HISTORY_CAP, removes index 0
- `population_count(&self) -> usize` — member_agents.len()
- `is_dissolved(&self) -> bool` — population_count() == 0

Derives: `#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]`

### Part B: `rust/crates/sim-core/src/components/mod.rs`

`pub mod settlement;` 추가 (알파벳 순서 유지).

Re-export:
```rust
pub use settlement::{
    BuildingId, PopulationStats, Settlement, SettlementId,
    SETTLEMENT_DISSOLUTION_THRESHOLD, SETTLEMENT_FORMATION_AGENT_THRESHOLD,
    SETTLEMENT_FORMATION_BUILDING_THRESHOLD, SETTLEMENT_HISTORY_CAP,
    SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
```

### Part C: `rust/crates/sim-core/src/causal/event.rs`

`CausalEvent` enum에 12번째 variant 추가 (CombatCompleted 다음):

```rust
AgentBorn {
    id: EventId,
    parent: Option<EventId>,
    agent: AgentId,
    tick: u64,
},
```

`id()`, `parent()`, `tick()`, `channel()` exhaustive match 업데이트:
- `id()`: `| CausalEvent::AgentBorn { id, .. } => *id`
- `parent()`: `| CausalEvent::AgentBorn { parent, .. } => *parent`
- `tick()`: `| CausalEvent::AgentBorn { tick, .. } => *tick`
- `channel()`: `| CausalEvent::AgentBorn { .. } => None`

### Part D: `rust/crates/sim-engine/src/lib.rs`

`SimResources` struct에 추가:
```rust
pub settlements: HashMap<SettlementId, Settlement>,
pub next_settlement_id: SettlementId,
```

`SimResources::new()` 초기화:
```rust
settlements: HashMap::new(),
next_settlement_id: 0,
```

새 메서드:
```rust
pub fn issue_settlement_id(&mut self) -> SettlementId {
    let id = self.next_settlement_id;
    self.next_settlement_id = self.next_settlement_id.wrapping_add(1);
    id
}
```

### Part E: Exhaustive Match Updates

**`sim-systems/src/runtime/memory/memory_system.rs`** `classify_event()`:
```rust
| CausalEvent::AgentBorn { .. } => None,
```

**`sim-bridge/src/ffi/world_node.rs`** `CausalEventView::from_event()`:
```rust
CausalEvent::AgentBorn { id, parent, agent, tick } => Self {
    kind: "agent_born",
    id: *id, parent: *parent, tick: *tick,
    channel: None, position: None, radius: None, region: None,
    old_value: None, new_value: None,
    agent_id: Some(*agent),
    reason: None, triggered_by: None, recalled_event: None,
    defender_id: None, hp_after: None,
},
```

---

## Section 3: Harness Tests

File: `rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs`

21개 assertion 목표 (A1–A21):

| # | Assertion |
|---|-----------|
| A1 | `SettlementId` is `u32` (size_of check) |
| A2 | `BuildingId` is `u64` (size_of check) |
| A3 | `SETTLEMENT_HISTORY_CAP == 32` |
| A4 | `SETTLEMENT_FORMATION_AGENT_THRESHOLD == 3` |
| A5 | `SETTLEMENT_FORMATION_BUILDING_THRESHOLD == 2` |
| A6 | `SETTLEMENT_DISSOLUTION_THRESHOLD == 0` |
| A7 | `SETTLEMENT_MAX_POP == 50` |
| A8 | `SETTLEMENT_PROXIMITY_RADIUS == 5` |
| A9 | `Settlement::new_with_id` initializes empty |
| A10 | `add_member_agent` / `remove_member_agent` round-trip |
| A11 | `add_member_building` / `remove_member_building` round-trip |
| A12 | `append_history` saturating at CAP (removes index 0) |
| A13 | `population_count()` returns member_agents.len() |
| A14 | `is_dissolved()` true when empty, false when has members |
| A15 | Settlement serde roundtrip (JSON) |
| A16 | `SimResources.settlements` HashMap exists and is empty at init |
| A17 | `issue_settlement_id()` monotonically increases |
| A18 | `CausalEvent::AgentBorn` variant exists, id/parent/tick accessors work |
| A19 | `AgentBorn` Clone + PartialEq |
| A20 | `Settlement`, `SettlementId`, `BuildingId` re-exported from `sim_core::components` |
| A21 | Phase 9-α + 8-α regression: existing harness tests still pass |

---

## Section 4: Scope Boundaries

**이 Phase에서 하지 않음:**
- SettlementSystem (Phase 10-β)
- SettlementFormed / SettlementDissolved CausalEvent variants (Phase 10-β)
- Settlement FFI snapshot (Phase 10-γ)
- UI 패널 (Phase 10-γ)
- 에이전트 자동 귀속 로직 (Phase 10-β)

**Phase 10-β 예약 (건드리지 말 것):**
- `CausalEvent::SettlementFormed` (13번째 variant)
- `CausalEvent::SettlementDissolved` (14번째 variant)

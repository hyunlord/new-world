//! V7 Phase 10-α — Settlement data substrate.
//!
//! First sub-stage of V7 Phase 10 (Multi-building Settlement System). Lands the
//! component-layer prerequisites for settlement aggregation without introducing
//! any runtime system or agent-decision change — those follow in Phase 10-β.
//!
//! Types:
//! - [`SettlementId`] — `u32` type alias, mirrors [`AgentId`] (`u64`) pattern.
//! - [`BuildingId`] — `u64` type alias, forward declaration for Phase 10-β wiring.
//! - [`PopulationStats`] — birth/death tracking aggregate.
//! - [`Settlement`] — aggregate stored in [`SimResources::settlements`].
//!
//! See `.harness/plans/phase10.md` §3.1 for the full sub-stage decomposition.

use crate::causal::event::EventId;
use crate::components::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Stable per-settlement identifier. Mirrors the [`AgentId`] type-alias
/// convention but uses `u32` — settlement counts are orders of magnitude
/// smaller than cumulative agent-id or event-id counters.
///
/// Phase 10-α: alias defined and wired to [`Settlement::settlement_id`].
/// Allocation (monotonic mint via `SimResources::issue_settlement_id`) is
/// fully in place at this sub-stage.
pub type SettlementId = u32;

/// Stable per-building identifier. Forward declaration for Phase 10-β wiring.
///
/// Mirrors [`BlueprintId`](crate::components::construction::BlueprintId) (`u64`).
/// Phase 10-β `SettlementSystem` will wire these to the `hecs::Entity` handles
/// of completed buildings. Phase 10-α defines the type so
/// [`Settlement::member_buildings`] compiles without runtime coupling.
pub type BuildingId = u64;

/// Maximum community-history entries per settlement.
///
/// Mirrors [`MEMORY_CAP`](crate::components::memory::MEMORY_CAP) = 32
/// (substrate symmetry: per-agent episodic memory ↔ per-settlement community
/// history). Phase 3-β ring-buffer × Phase 8 memory × Phase 10 settlement
/// = symmetric 32-entry bounded history across all three scales.
pub const SETTLEMENT_HISTORY_CAP: usize = 32;

/// Minimum agent count co-located within [`SETTLEMENT_PROXIMITY_RADIUS`] to
/// trigger automatic settlement formation (P10Plan-4 formation threshold).
///
/// Phase 10-β `SettlementSystem` reads this; defined here so Phase 10-α
/// harness tests can assert the constant value before any system exists.
pub const SETTLEMENT_FORMATION_AGENT_THRESHOLD: u32 = 3;

/// Minimum building count co-located within [`SETTLEMENT_PROXIMITY_RADIUS`]
/// to trigger automatic settlement formation (P10Plan-4 formation threshold).
pub const SETTLEMENT_FORMATION_BUILDING_THRESHOLD: u32 = 2;

/// Population count at or below which a settlement is considered dissolved
/// (P10Plan-3 dissolution condition).
pub const SETTLEMENT_DISSOLUTION_THRESHOLD: u32 = 0;

/// Maximum population before the migration-pull cascade arm stops attracting
/// new agents toward this settlement (P10Plan-8 migration pull limit).
pub const SETTLEMENT_MAX_POP: u32 = 50;

/// Chebyshev-distance radius in tiles used for formation scan and membership
/// sync (P10Plan-4 proximity).
pub const SETTLEMENT_PROXIMITY_RADIUS: u32 = 5;

/// Birth/death population tracking for a settlement.
///
/// `current` mirrors `member_agents.len()` and is kept in sync by
/// `SettlementSystem` (Phase 10-β). Maintained separately to avoid
/// `O(n)` len() calls in the bridge snapshot path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationStats {
    /// Current live member count (mirrors `member_agents.len()`).
    pub current: u32,
    /// Cumulative births registered in this settlement (incremented on
    /// `CausalEvent::AgentBorn` with this settlement's id).
    pub total_births: u32,
    /// Cumulative deaths (incremented on `CombatCompleted { hp_after ≤ 0 }`
    /// where the victim is a member of this settlement).
    pub total_deaths: u32,
}

/// Settlement aggregate — stored in [`SimResources::settlements`].
///
/// Phase 10-α lands the struct and impl methods only. The runtime system
/// (`SettlementSystem`, priority 138) that populates `member_agents`,
/// `member_buildings`, `community_history`, and `population_stats` is
/// Phase 10-β scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settlement {
    /// Stable settlement identity (issued by `SimResources::issue_settlement_id`).
    pub settlement_id: SettlementId,

    /// Agents currently considered members of this settlement.
    /// Phase 10-β `SettlementSystem` maintains this via proximity scan.
    pub member_agents: HashSet<AgentId>,

    /// Buildings within the settlement boundary.
    /// Phase 10-β wires `BuildingId` to completed `hecs::Entity` handles.
    pub member_buildings: HashSet<BuildingId>,

    /// Snapshot population statistics (kept in sync by `SettlementSystem`).
    pub population_stats: PopulationStats,

    /// Bounded community-history ring (capped at [`SETTLEMENT_HISTORY_CAP`]).
    ///
    /// Stores [`EventId`] references into the causal log — lightweight
    /// handles rather than full [`CausalEvent`](crate::causal::event::CausalEvent)
    /// copies, matching P10Plan-6 (substrate symmetry with per-agent Memory).
    pub community_history: Vec<EventId>,

    /// Tick at which this settlement was first formed (set on `SettlementFormed`
    /// emission in Phase 10-β).
    pub founded_at: u64,
}

impl Settlement {
    /// Create an empty settlement with the given `id`, founded at `tick`.
    pub fn new_with_id(id: SettlementId, founded_at: u64) -> Self {
        Self {
            settlement_id: id,
            member_agents: HashSet::new(),
            member_buildings: HashSet::new(),
            population_stats: PopulationStats::default(),
            community_history: Vec::with_capacity(SETTLEMENT_HISTORY_CAP),
            founded_at,
        }
    }

    /// Insert `agent` into `member_agents`. Returns `true` if newly added,
    /// `false` if already present (HashSet contract).
    pub fn add_member_agent(&mut self, agent: AgentId) -> bool {
        self.member_agents.insert(agent)
    }

    /// Remove `agent` from `member_agents`. Returns `true` if it was present.
    pub fn remove_member_agent(&mut self, agent: AgentId) -> bool {
        self.member_agents.remove(&agent)
    }

    /// Insert `building` into `member_buildings`. Returns `true` if newly added.
    pub fn add_member_building(&mut self, building: BuildingId) -> bool {
        self.member_buildings.insert(building)
    }

    /// Remove `building` from `member_buildings`. Returns `true` if it was present.
    pub fn remove_member_building(&mut self, building: BuildingId) -> bool {
        self.member_buildings.remove(&building)
    }

    /// Append `event_id` to community history, evicting the oldest entry when
    /// the cap is reached.
    ///
    /// Saturating ring-buffer — mirrors [`crate::components::memory::Memory`]
    /// eviction (lowest-salience) but simplified to FIFO at cap because
    /// settlement-level events have no per-event salience score.
    pub fn append_history(&mut self, event_id: EventId) {
        if self.community_history.len() >= SETTLEMENT_HISTORY_CAP {
            self.community_history.remove(0);
        }
        self.community_history.push(event_id);
    }

    /// Current live member count (`member_agents.len()`).
    pub fn population_count(&self) -> usize {
        self.member_agents.len()
    }

    /// `true` when the settlement has no living members (P10Plan-3 dissolution
    /// condition — `population_count == 0`).
    pub fn is_dissolved(&self) -> bool {
        self.population_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_id_type_alias_u32() {
        let id: SettlementId = 42u32;
        assert_eq!(id, 42u32);
        // Ensure Copy + PartialEq from u32 work directly
        let id2 = id;
        assert_eq!(id, id2);
    }

    #[test]
    fn building_id_type_alias_u64() {
        let id: BuildingId = 99u64;
        assert_eq!(id, 99u64);
    }

    #[test]
    fn constants_stable_values() {
        assert_eq!(SETTLEMENT_HISTORY_CAP, 32, "must mirror MEMORY_CAP");
        assert_eq!(SETTLEMENT_FORMATION_AGENT_THRESHOLD, 3);
        assert_eq!(SETTLEMENT_FORMATION_BUILDING_THRESHOLD, 2);
        assert_eq!(SETTLEMENT_DISSOLUTION_THRESHOLD, 0);
        assert_eq!(SETTLEMENT_MAX_POP, 50);
        assert_eq!(SETTLEMENT_PROXIMITY_RADIUS, 5);
    }

    #[test]
    fn new_with_id_creates_empty_settlement() {
        let s = Settlement::new_with_id(7, 100);
        assert_eq!(s.settlement_id, 7);
        assert_eq!(s.founded_at, 100);
        assert!(s.member_agents.is_empty());
        assert!(s.member_buildings.is_empty());
        assert!(s.community_history.is_empty());
        assert_eq!(s.population_stats, PopulationStats::default());
        assert_eq!(s.population_stats.current, 0);
        assert_eq!(s.population_stats.total_births, 0);
        assert_eq!(s.population_stats.total_deaths, 0);
    }

    #[test]
    fn add_remove_member_agent_hashset_contract() {
        let mut s = Settlement::new_with_id(1, 0);
        assert!(s.add_member_agent(10), "first insert returns true");
        assert!(!s.add_member_agent(10), "duplicate insert returns false");
        assert_eq!(s.population_count(), 1);
        assert!(s.remove_member_agent(10), "present remove returns true");
        assert!(!s.remove_member_agent(10), "absent remove returns false");
        assert_eq!(s.population_count(), 0);
    }

    #[test]
    fn add_remove_member_building_hashset_contract() {
        let mut s = Settlement::new_with_id(1, 0);
        assert!(s.add_member_building(5), "first insert returns true");
        assert!(!s.add_member_building(5), "duplicate insert returns false");
        assert!(s.remove_member_building(5), "present remove returns true");
        assert!(!s.remove_member_building(5), "absent remove returns false");
    }

    #[test]
    fn append_history_saturating_ring_buffer() {
        let mut s = Settlement::new_with_id(1, 0);
        // Fill to cap
        for i in 0..SETTLEMENT_HISTORY_CAP as u64 {
            s.append_history(i);
        }
        assert_eq!(s.community_history.len(), SETTLEMENT_HISTORY_CAP);
        // Push 5 more — oldest 5 should be evicted
        for i in SETTLEMENT_HISTORY_CAP as u64..(SETTLEMENT_HISTORY_CAP + 5) as u64 {
            s.append_history(i);
        }
        assert_eq!(s.community_history.len(), SETTLEMENT_HISTORY_CAP);
        // Oldest entry is now index 5 (0..4 evicted)
        assert_eq!(s.community_history[0], 5);
        // Newest is at the tail
        assert_eq!(
            *s.community_history.last().unwrap(),
            (SETTLEMENT_HISTORY_CAP + 4) as u64
        );
    }

    #[test]
    fn population_count_and_is_dissolved() {
        let mut s = Settlement::new_with_id(1, 0);
        assert!(s.is_dissolved(), "empty settlement is dissolved");
        s.add_member_agent(1);
        assert!(!s.is_dissolved(), "non-empty settlement is not dissolved");
        assert_eq!(s.population_count(), 1);
        s.add_member_agent(2);
        assert_eq!(s.population_count(), 2);
        s.remove_member_agent(1);
        s.remove_member_agent(2);
        assert!(s.is_dissolved(), "all members removed → dissolved");
    }

    #[test]
    fn settlement_serde_roundtrip() {
        let mut s = Settlement::new_with_id(7, 42);
        s.add_member_agent(10);
        s.add_member_agent(20);
        s.add_member_building(100);
        s.append_history(500);
        s.append_history(501);
        s.population_stats.total_births = 3;
        s.population_stats.current = 2;

        let json = serde_json::to_string(&s).expect("Settlement must serialize");
        let back: Settlement = serde_json::from_str(&json).expect("Settlement must deserialize");
        assert_eq!(s, back);
    }
}

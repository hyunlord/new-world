//! V7 Phase 4-α — Canonical ECS components.
//!
//! First deliverable of the Agent Core phase (Week 7-8). Replaces the
//! Phase 2 local placeholder in `sim_systems::runtime::influence::agent_sample::Position`
//! per the self-documenting landmark at `agent_sample.rs:9-15`.
//!
//! Phase 5-α extends the surface with [`AgentId`] (re-exported here) and
//! the [`Hunger`] need component — the first daily-routine driver.
//!
//! Phase 5-β adds the second need ([`Thirst`]) and the agent-decision
//! FSM ([`AgentState`] + [`TargetKind`]) that consumes both needs and
//! emits the first agent-originated `CausalEvent::AgentDecision`.
//!
//! Phase 5-γ adds the third need ([`Sleep`]) alongside the day/night
//! clock substrate (see `sim_engine::SimResources::time_of_day` /
//! `ticks_per_day` and `sim_systems::runtime::needs::SleepDecaySystem`).
//!
//! Phase 6-α adds the construction data substrate ([`BlueprintId`],
//! [`BuildingBlueprint`], [`ConstructionSite`]) and extends
//! [`TargetKind`] with a 4th variant `ConstructionSite` — Phase 5-γ
//! Path (b) symmetry precedent. `AgentState` is intentionally unchanged.
//!
//! Phase 7-α adds the multi-agent social-system data substrate
//! ([`Social`], [`RelationshipKey`], [`RelationshipState`]) and extends
//! [`TargetKind`] with a 5th variant `Agent(AgentId)` — the first
//! payload-carrying target variant. `AgentState` is intentionally
//! unchanged; the Phase 7-β `SocialInteractionSystem` (priority 134)
//! owns runtime resolution.
//!
//! Phase 8-α adds the per-agent episodic memory substrate
//! ([`Memory`], [`MemoryEntry`], [`MEMORY_CAP`], [`SALIENCE_FLOOR`]).
//! The runtime `MemorySystem` (priority 136), `CausalEvent::MemoryRecalled`,
//! `DecisionReason::MemoryReason`, and the `AgentDecisionSystem` 6th-cascade
//! bias mechanism are all Phase 8-β scope. `TargetKind` and `AgentState`
//! are intentionally unchanged — Memory is a decision-bias source, not a
//! target type.
//!
//! See `.harness/plans/phase4.md` §2, `.harness/plans/phase5.md`
//! §2.1 / §2.β / §2.γ, `.harness/plans/phase6.md` §2.1, and
//! `.harness/plans/phase7.md §3` for the full sub-stage decomposition.

pub mod activity;
pub mod agent;
pub mod agent_state;
pub mod body_health;
pub mod building_kind;
pub mod child;
pub mod cold;
pub mod construction;
pub mod exposure_clock;
pub mod family;
pub mod family_events;
pub mod founding;
pub mod hardiness;
pub mod hunger;
pub mod inventory;
pub mod life;
pub mod lineage;
pub mod memory;
pub mod porter;
pub mod position;
pub mod relations;
pub mod relationship;
pub mod resource_kind;
pub mod seek_target;
pub mod settlement;
pub mod settlement_migrant;
pub mod sleep;
pub mod social;
pub mod temperament;
pub mod thirst;

pub use activity::{
    Activity, ActivityKind, ActivityRest, RoutineObservation, RoutineObservationBatch,
    RoutineOutcome,
};
pub use agent::{Agent, AgentId};
pub use agent_state::{AgentState, TargetKind};
pub use body_health::{BodyHealth, DEFAULT_MAX_HP};
pub use building_kind::{shelter_blueprint, BuildingKind, SHELTER_BLUEPRINT_ID};
pub use child::Child;
pub use cold::Cold;
pub use construction::{BlueprintId, BuildingBlueprint, ConstructionSite};
pub use exposure_clock::ExposureClock;
pub use family::{close_kin, Kin, Sex};
pub use family_events::{FamilyEvent, FamilyObservations};
pub use founding::Founding;
pub use hardiness::Hardiness;
pub use hunger::Hunger;
pub use inventory::{Inventory, INVENTORY_CAPACITY};
pub use life::{Elder, Life};
pub use lineage::Lineage;
pub use memory::{Memory, MemoryArm, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR};
pub use porter::{PorterKind, PorterStage, PorterTrip};
pub use position::Position;
pub use relations::{PairDay, RelationState, SettlementRelations, TripRecord};
pub use relationship::{RelationshipKey, RelationshipState, HOSTILITY_BUMP};
pub use resource_kind::ResourceKind;
pub use seek_target::SeekTarget;
pub use settlement::{
    BuildingId, PopulationStats, Settlement, SettlementId, SETTLEMENT_DISSOLUTION_THRESHOLD,
    SETTLEMENT_FORMATION_AGENT_THRESHOLD, SETTLEMENT_FORMATION_BUILDING_THRESHOLD,
    SETTLEMENT_HISTORY_CAP, SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
pub use settlement_migrant::SettlementMigrant;
pub use sleep::Sleep;
pub use social::Social;
pub use temperament::Temperament;
pub use thirst::Thirst;

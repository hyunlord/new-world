//! Effect inputs and deferred writes; selection and factual target lookup happen upstream.
use crate::runtime::agent_actions::needs::NeedAvailability;
use crate::runtime::agent_actions::routine::RoutineTick;
use hecs::{Entity, World};
use sim_core::causal::EventId;
use sim_core::chronicle::ChronicleEntry;
use sim_core::components::{ActivityKind, AgentId, AgentState, Position, SeekTarget, SettlementId};
use sim_core::rules::NightRestRule;
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Concrete capability selected by the winning node's registered target binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runtime) enum ActionTarget {
    None,
    Tile((u32, u32)),
    Food {
        tile: (u32, u32),
        owner: sim_core::colony::food::FoodOwner,
    },
    Agent {
        agent: AgentId,
        tile: (u32, u32),
    },
    Routine {
        kind: ActivityKind,
        tile: (u32, u32),
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) enum EffectDomain {
    Needs,
    /// Unknown food mutation: rebuild the index lazily.
    FoodSources,
    /// Food effect confined to this tile, including a logically dirty no-op.
    FoodTile((u32, u32)),
    Claims,
    RoutineOccupancy,
    Invitations,
    Jobs,
    Targets,
    Relations,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runtime) enum EffectStatus {
    Running,
    Finished,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runtime) enum FinishReason {
    Completed,
    Interrupted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runtime) enum EffectError {
    MissingState,
    MissingTarget,
    WrongTarget,
    MissingRoutine,
    MissingOwner,
    StaleTarget,
}
pub(in crate::runtime) type EffectResult = Result<EffectStatus, EffectError>;
/// A start-of-tick rival row: person, village, lineage key, NS and HA.
pub(in crate::runtime) type Rival = (AgentId, SettlementId, u64, u16, u16);

/// One real tick's shared mutable effect state, retained across invitation replay.
#[derive(Default)]
pub(in crate::runtime) struct EffectState {
    pub food_version: u64,
    pub completed_meals: Vec<AgentId>,
    pub claims_version: u64,
    pub catchment: u32,
    pub local_home: BTreeMap<AgentId, (u32, u32)>,
    pub forage_plans: BTreeMap<SettlementId, crate::runtime::agent_actions::forage::ForagePlan>,
    pub avoid_of: BTreeMap<SettlementId, Vec<(u32, u32)>>,
    pub previous_jobs: HashMap<Entity, AgentState>,
    /// Final externally-owned states survive earlier cleanup and deferred postpass writes.
    pub external_states: BTreeMap<Entity, (AgentState, Option<SeekTarget>)>,
    pub claims: BTreeMap<(u32, u32), u32>,
    pub forage_targets: HashMap<Entity, (u32, u32)>,
    pub members: BTreeMap<AgentId, SettlementId>,
    pub forage_spots: BTreeMap<(u32, u32), Vec<Rival>>,
    pub quarrelled: BTreeSet<AgentId>,
    pub quarrel_entries: Vec<ChronicleEntry>,
    pub social_seek_entities: BTreeSet<Entity>,
    pub seek_set: Vec<(Entity, (u32, u32))>,
    pub seek_clear: Vec<Entity>,
    pub seek_idle: Vec<Entity>,
    /// Process after per-person decisions, in the same phase as old job settlement.
    pub job_releases: Vec<Entity>,
    pub dirty: BTreeSet<EffectDomain>,
}

/// Borrowed lifecycle inputs supplied by the coordinator after pure selection.
pub(in crate::runtime) struct EffectContext<'a> {
    pub world: &'a mut World,
    pub resources: &'a mut SimResources,
    pub effects: &'a mut EffectState,
    pub routine: Option<&'a mut RoutineTick>,
    pub entity: Entity,
    pub agent: AgentId,
    pub position: Position,
    pub target: ActionTarget,
    pub availability: NeedAvailability,
    pub night_rest: Option<NightRestRule>,
    /// Factual continuation interrupt, evaluated against the same pre-effect row.
    pub interrupted: bool,
    /// Mutual seek at the start of this tick, not the partner's mutated current state.
    pub social_handshake: bool,
    pub cause: Option<EventId>,
    pub lineage_key: Option<u64>,
    pub member: Option<SettlementId>,
    pub node_random_key: u64,
    pub finish_reason: FinishReason,
    /// The selected data node explicitly finishes this lifecycle, rather than a switch.
    pub authored_completion: bool,
}
impl EffectContext<'_> {
    pub fn state(&self) -> Result<AgentState, EffectError> {
        self.world
            .get::<&AgentState>(self.entity)
            .map(|s| *s)
            .map_err(|_| EffectError::MissingState)
    }
    pub fn set_state(&mut self, value: AgentState) -> Result<(), EffectError> {
        *self
            .world
            .get::<&mut AgentState>(self.entity)
            .map_err(|_| EffectError::MissingState)? = value;
        self.effects.dirty.insert(EffectDomain::Needs);
        Ok(())
    }
    pub fn tile_target(&self) -> Result<(u32, u32), EffectError> {
        match self.target {
            ActionTarget::Tile(tile) | ActionTarget::Food { tile, .. } => Ok(tile),
            ActionTarget::None => Err(EffectError::MissingTarget),
            ActionTarget::Agent { .. } | ActionTarget::Routine { .. } => {
                Err(EffectError::WrongTarget)
            }
        }
    }
    pub fn attach(&mut self, tile: (u32, u32)) {
        self.effects.seek_set.push((self.entity, tile));
        self.effects.dirty.insert(EffectDomain::Targets);
    }
    pub fn clear(&mut self) {
        self.effects.seek_clear.push(self.entity);
        self.effects.dirty.insert(EffectDomain::Targets);
    }
}

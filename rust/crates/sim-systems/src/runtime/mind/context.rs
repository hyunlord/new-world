//! Read-only person snapshots and versioned, coordinator-owned derived facts.
//!
//! Capture after lifecycle preparation. Effects must refresh changed people and
//! invalidate changed domains before the next primary or replay evaluation.
//! This context never owns a mutable routine transaction or simulation resource.

use crate::runtime::agent_actions::{forage::ForagePlan, needs::NeedAvailability, targets};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use hecs::{Entity, World};
use sim_core::colony::mind::{CompiledMind, LifeStage, RunningChoice};
use sim_core::colony::people::{Drafted, Fleeing, Mood, Sick};
use sim_core::colony::{Carry, Job};
use sim_core::components::{
    Activity, ActivityRest, Agent, AgentId, AgentState, BodyHealth, Child, Cold, Elder, Founding,
    Hunger, Inventory, PorterTrip, Position, ResourceKind, SettlementId, Sleep, Social,
    Temperament, Thirst,
};
use sim_core::spatial::{chebyshev, ChunkIndex};
use sim_engine::SimResources;

/// Structural dependency domains, independent of node or behaviour names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Domain {
    /// Person components, position, and per-person targeting scope.
    Person,
    /// Current gathering reservations owned by the effect coordinator.
    Claims,
    /// Live food source amounts and existence.
    FoodSources,
    /// Live water source amounts and existence.
    WaterSources,
    /// Live sleeping source amounts and existence.
    SleepSources,
    /// Reservations in the real routine lifecycle transaction.
    RoutineOccupancy,
    /// Pending invitations in the real routine transaction.
    Invitations,
    /// Current job ownership and stage.
    Jobs,
    /// Threat positions or availability.
    Threats,
}
const DOMAIN_COUNT: usize = 9;
const _: () = assert!(DOMAIN_COUNT <= u16::BITS as usize);

/// A failure is explicit; absent or stale evidence must never become false/zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextError {
    /// The AgentId was not captured in this tick.
    UnknownAgent(AgentId),
    /// Two captured entities have the same stable AgentId.
    DuplicateAgent(AgentId),
    /// A required component disappeared during a dirty refresh.
    MissingComponent {
        /// Person whose observation could not be produced.
        agent: AgentId,
        /// Required ECS component name.
        component: &'static str,
    },
    /// The coordinator has not supplied this required observation.
    MissingDerived {
        /// Person whose observation could not be produced.
        agent: AgentId,
        /// Registered observation or target capability name.
        name: String,
    },
    /// A dependency changed after this observation was recorded.
    StaleDerived {
        /// Person whose observation could not be produced.
        agent: AgentId,
        /// Registered observation or target capability name.
        name: String,
    },
    /// A dependency version cannot be incremented without wrapping.
    VersionOverflow(Domain),
    /// The requested reservation delta would underflow or overflow.
    InvalidClaim((u32, u32)),
    /// Prior ownership cannot be resolved in the supplied policy dictionary.
    InvalidRunningChoice(AgentId),
}
impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ContextError {}

/// Native values retain precision; providers decide the documented integer unit.
#[derive(Clone, Debug)]
pub struct PersonSnapshot {
    /// Stable identity used for sorted processing and keyed observations.
    pub agent: AgentId,
    /// ECS entity used only to refresh the corresponding captured person.
    pub entity: Entity,
    /// Current tile coordinates copied from ECS.
    pub position: Position,
    /// Current seeking/consuming state, independent of selected policy nodes.
    pub state: AgentState,
    /// Captured child, adult, or elder stage.
    pub life_stage: LifeStage,
    /// Native hunger value; None preserves absence of the component.
    pub hunger: Option<f64>,
    /// Native thirst value; None preserves absence of the component.
    pub thirst: Option<f64>,
    /// Native fatigue value; None preserves absence of the component.
    pub fatigue: Option<f64>,
    /// Native loneliness value; None preserves absence of the component.
    pub social: Option<f64>,
    /// Optional inherited temperament used by factual thresholds.
    pub temperament: Option<Temperament>,
    /// Total carried units, or None when no Inventory exists.
    pub inventory_total: Option<u32>,
    /// Carried food units, or None when no Inventory exists.
    pub inventory_food: Option<u32>,
    /// Optional economy hauling state.
    pub carry: Option<Carry>,
    /// Optional externally assigned economy job.
    pub job: Option<Job>,
    /// Whether the military owner currently controls this person.
    pub drafted: bool,
    /// Optional refuge owner and destination.
    pub fleeing: Option<Fleeing>,
    /// Presence of the Cold condition, rather than an inferred weather predicate.
    pub cold: bool,
    /// Presence of the Sick condition.
    pub sick: bool,
    /// Optional current mood component.
    pub mood: Option<Mood>,
    /// Native body health, absent when BodyHealth is absent.
    pub hp: Option<f64>,
    /// Current routine reservation, refreshed from the lifecycle transaction before selection.
    pub activity: Option<Activity>,
    /// Captured pause between routine activities.
    pub activity_rest: Option<ActivityRest>,
    /// Stored memory count, or None when Memory is absent.
    pub memory_entries: Option<usize>,
    /// Optional lineage key used by inherited routine preferences.
    pub lineage: Option<u64>,
    /// Hunger, thirst, fatigue, construction, social, combat, in that order.
    pub memory_weights: [f64; 6],
    /// Contributing event IDs in the same drive order as memory_weights.
    pub memory_causes: [Option<sim_core::causal::EventId>; 6],
}

impl PersonSnapshot {
    fn capture(
        world: &World,
        entity: Entity,
        agent: AgentId,
        tick: u64,
    ) -> Result<Self, ContextError> {
        let entity_ref = world
            .entity(entity)
            .map_err(|_| ContextError::MissingComponent {
                agent,
                component: "Position",
            })?;
        let position =
            entity_ref
                .get::<&Position>()
                .map(|v| *v)
                .ok_or(ContextError::MissingComponent {
                    agent,
                    component: "Position",
                })?;
        let state =
            entity_ref
                .get::<&AgentState>()
                .map(|v| *v)
                .ok_or(ContextError::MissingComponent {
                    agent,
                    component: "AgentState",
                })?;
        let life_stage = if entity_ref.get::<&Child>().is_some() {
            LifeStage::Child
        } else if entity_ref.get::<&Elder>().is_some() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        let memory = entity_ref.get::<&sim_core::components::Memory>();
        let (memory_weights, memory_causes) = memory
            .as_ref()
            .map_or(([0.0; 6], [None; 6]), |memory| memory_values(memory, tick));
        let memory_entries = memory.as_ref().map(|memory| memory.entries.len());
        Ok(Self {
            agent,
            entity,
            position,
            state,
            life_stage,
            hunger: entity_ref.get::<&Hunger>().map(|v| f64::from(v.value)),
            thirst: entity_ref.get::<&Thirst>().map(|v| v.value),
            fatigue: entity_ref.get::<&Sleep>().map(|v| v.fatigue),
            social: entity_ref.get::<&Social>().map(|v| v.loneliness),
            temperament: entity_ref.get::<&Temperament>().map(|v| *v),
            inventory_total: entity_ref.get::<&Inventory>().map(|v| v.total()),
            inventory_food: entity_ref
                .get::<&Inventory>()
                .map(|v| v.get(ResourceKind::Food)),
            carry: entity_ref.get::<&Carry>().map(|v| *v),
            job: entity_ref.get::<&Job>().map(|v| *v),
            drafted: entity_ref.get::<&Drafted>().is_some(),
            fleeing: entity_ref.get::<&Fleeing>().map(|v| *v),
            cold: entity_ref.get::<&Cold>().is_some(),
            sick: entity_ref.get::<&Sick>().is_some(),
            mood: entity_ref.get::<&Mood>().map(|v| *v),
            hp: entity_ref.get::<&BodyHealth>().map(|v| v.hp),
            activity: entity_ref.get::<&Activity>().map(|v| *v),
            activity_rest: entity_ref.get::<&ActivityRest>().map(|v| *v),
            lineage: entity_ref
                .get::<&sim_core::components::Lineage>()
                .map(|v| v.key),
            memory_weights,
            memory_causes,
            memory_entries,
        })
    }
    #[cfg(test)]
    fn capture_exclusive(
        world: &mut World,
        entity: Entity,
        agent: AgentId,
        tick: u64,
    ) -> Result<Self, ContextError> {
        let (
            (position, state, child, elder, hunger, thirst, sleep, social),
            (temperament, inventory, carry, job, drafted, fleeing, cold, sick),
            (mood, health, activity, rest, memory, lineage),
        ) = world
            .query_one_mut::<(
                (
                    Option<&Position>,
                    Option<&AgentState>,
                    Option<&Child>,
                    Option<&Elder>,
                    Option<&Hunger>,
                    Option<&Thirst>,
                    Option<&Sleep>,
                    Option<&Social>,
                ),
                (
                    Option<&Temperament>,
                    Option<&Inventory>,
                    Option<&Carry>,
                    Option<&Job>,
                    Option<&Drafted>,
                    Option<&Fleeing>,
                    Option<&Cold>,
                    Option<&Sick>,
                ),
                (
                    Option<&Mood>,
                    Option<&BodyHealth>,
                    Option<&Activity>,
                    Option<&ActivityRest>,
                    Option<&sim_core::components::Memory>,
                    Option<&sim_core::components::Lineage>,
                ),
            )>(entity)
            .map_err(|_| ContextError::MissingComponent {
                agent,
                component: "Position",
            })?;
        let position = position.copied().ok_or(ContextError::MissingComponent {
            agent,
            component: "Position",
        })?;
        let state = state.copied().ok_or(ContextError::MissingComponent {
            agent,
            component: "AgentState",
        })?;
        let life_stage = if child.is_some() {
            LifeStage::Child
        } else if elder.is_some() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        let (memory_weights, memory_causes) =
            memory.map_or(([0.0; 6], [None; 6]), |memory| memory_values(memory, tick));
        Ok(Self {
            agent,
            entity,
            position,
            state,
            life_stage,
            hunger: hunger.map(|v| f64::from(v.value)),
            thirst: thirst.map(|v| v.value),
            fatigue: sleep.map(|v| v.fatigue),
            social: social.map(|v| v.loneliness),
            temperament: temperament.copied(),
            inventory_total: inventory.map(|v| v.total()),
            inventory_food: inventory.map(|v| v.get(ResourceKind::Food)),
            carry: carry.copied(),
            job: job.copied(),
            drafted: drafted.is_some(),
            fleeing: fleeing.copied(),
            cold: cold.is_some(),
            sick: sick.is_some(),
            mood: mood.copied(),
            hp: health.map(|v| v.hp),
            activity: activity.copied(),
            activity_rest: rest.copied(),
            lineage: lineage.map(|v| v.key),
            memory_weights,
            memory_causes,
            memory_entries: memory.map(|v| v.entries.len()),
        })
    }
}

/// Borrow-safe values copied from the coordinator's current mutable query row.
/// Optional components preserve their actual absence; no ECS fetch occurs here.
#[derive(Clone, Copy, Debug)]
pub struct PersonUpdate {
    /// Current tile coordinates copied from ECS.
    pub position: Position,
    /// Current seeking/consuming state, independent of selected policy nodes.
    pub state: AgentState,
    /// Native hunger value; None preserves absence of the component.
    pub hunger: Option<f64>,
    /// Native thirst value; None preserves absence of the component.
    pub thirst: Option<f64>,
    /// Native fatigue value; None preserves absence of the component.
    pub fatigue: Option<f64>,
    /// Native loneliness value; None preserves absence of the component.
    pub social: Option<f64>,
    /// Total carried units, or None when no Inventory exists.
    pub inventory_total: Option<u32>,
    /// Carried food units, or None when no Inventory exists.
    pub inventory_food: Option<u32>,
}

/// Tile candidate only; selecting a behaviour belongs to the data evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetCandidate {
    /// Resolved world tile in x/y order.
    pub tile: (u32, u32),
    /// Optional peer identity for an agent target; None denotes a tile target.
    pub agent: Option<AgentId>,
}

#[derive(Clone, Debug)]
struct Versioned<T> {
    value: T,
    dependencies: u16,
    version_sum: u128,
}

fn selected_version_sum(mut dependencies: u16, versions: &[u64; DOMAIN_COUNT]) -> u128 {
    let mut sum = 0;
    while dependencies != 0 {
        let index = dependencies.trailing_zeros() as usize;
        sum += u128::from(versions[index]);
        dependencies &= dependencies - 1;
    }
    sum
}

impl<T> Versioned<T> {
    fn fresh(&self, versions: &[u64; DOMAIN_COUNT]) -> bool {
        // Stamps stay within one TickContext, whose private counters only increase
        // by checked increments. Equal selected sums therefore mean every selected
        // counter is unchanged: nonnegative deltas cannot cancel. At most 16 u64
        // counters (the mask width) fit in u128 without overflow.
        self.version_sum == selected_version_sum(self.dependencies, versions)
    }
}

/// A caller supplies factual membership/avoidance prepared in legacy order.
/// `local_food` is None for people not restricted to a settlement catchment.
#[derive(Clone, Debug, Default)]
pub struct TargetScope {
    /// Tick-start membership used for local resource access.
    pub settlement: Option<SettlementId>,
    /// Tick-start home hearth, absent for people without a home.
    pub home: Option<(u32, u32)>,
    /// Optional home and catchment radius restricting local food lookup.
    pub local_food: Option<((u32, u32), u32)>,
    /// Sorted estranged hearth locations excluded from gathering.
    pub avoid_hearths: Vec<(u32, u32)>,
    /// Tick-start global or local food availability for this person.
    pub food_available: bool,
    /// Whether the home stockpile held food at snapshot time.
    pub camp_stocked: bool,
    /// Whether tick-start stock in hand was below the settlement target.
    pub stock_shortfall: bool,
    /// Tick-start settlement stock plus carried food, in native units.
    pub stock_in_hand: u64,
    /// Tick-start hoard cap; None denotes no forage plan and therefore no cap.
    pub stock_hoard_limit: Option<f64>,
}

/// Prior lifecycle identity resolved against its owning compiled dictionary.
#[derive(Clone, Debug)]
pub struct RunningMetadata {
    /// Registered behaviour name of the prior actual lifecycle owner.
    pub behaviour: String,
    /// Optional registered target capability of the prior lifecycle.
    pub target: Option<String>,
    /// Whether the previous lifecycle has already terminated.
    pub finished: bool,
}

type GatherCacheKey = ((u32, u32), Option<((u32, u32), u32)>, Option<SettlementId>);
type GatherCacheValue = ([u64; 2], bool);

#[derive(Debug)]
struct FoodIndex {
    version: u64,
    materialized: bool,
    tiles: ChunkIndex<u8>,
}

impl FoodIndex {
    fn capture(resources: &SimResources, version: u64) -> Self {
        let tiles = ChunkIndex::from_entries(
            resources
                .food_tiles
                .iter()
                .map(|(&tile, &amount)| (tile, amount)),
        );
        Self {
            version,
            materialized: true,
            tiles,
        }
    }
}

const DERIVED_SLOT_COUNT: usize = 16;

fn derived_slot(name: &str) -> Option<usize> {
    match name {
        "social_candidate" => Some(0),
        "social_memory_candidate" => Some(1),
        "social_handshake" => Some(2),
        "routine_admission" => Some(3),
        "invited" => Some(4),
        "routine_deadline" => Some(5),
        "activity_deadline_reached" => Some(6),
        "routine_weighted_place_available" => Some(7),
        "routine_available" => Some(8),
        "routine_tend_fire_available" => Some(9),
        "routine_craft_available" => Some(10),
        "routine_repair_available" => Some(11),
        "routine_hearth_circle_available" => Some(12),
        "routine_explore_available" => Some(13),
        "routine_play_available" => Some(14),
        "routine_sit_by_fire_available" => Some(15),
        _ => None,
    }
}

const TARGET_SLOT_COUNT: usize = 17;

fn target_slot(name: &str) -> Option<usize> {
    match name {
        "food" => Some(0),
        "water" => Some(1),
        "sleep" => Some(2),
        "gather_food" => Some(3),
        "forage_bush" => Some(4),
        "refuge" => Some(5),
        "job" => Some(6),
        "hearth" => Some(7),
        "social_peer" => Some(8),
        "social_memory_peer" => Some(9),
        "routine_tend_fire" => Some(10),
        "routine_craft" => Some(11),
        "routine_repair" => Some(12),
        "routine_hearth_circle" => Some(13),
        "routine_explore" => Some(14),
        "routine_play" => Some(15),
        "routine_sit_by_fire" => Some(16),
        _ => None,
    }
}

pub(super) type SocialSnapshotRow = (AgentId, (u32, u32), Option<AgentId>, Option<AgentId>, bool);

// Sorting indices avoids repeatedly moving the full factual payload through quicksort.
fn order_person_rows(rows: &mut [PersonSnapshot]) -> Result<(), ContextError> {
    if rows.windows(2).all(|pair| pair[0].agent < pair[1].agent) {
        return Ok(());
    }
    let mut source_indices: Vec<usize> = (0..rows.len()).collect();
    source_indices.sort_unstable_by_key(|index| rows[*index].agent);
    for pair in source_indices.windows(2) {
        let agent = rows[pair[0]].agent;
        if agent == rows[pair[1]].agent {
            return Err(ContextError::DuplicateAgent(agent));
        }
    }
    // Each cycle maps destination positions to their original source positions.
    // Mark consumed entries as identity; no second permutation or row buffer is needed.
    for start in 0..source_indices.len() {
        let mut current = start;
        while source_indices[current] != start {
            let next = source_indices[current];
            if next == current {
                break;
            }
            rows.swap(current, next);
            source_indices[current] = current;
            current = next;
        }
        source_indices[current] = current;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowRefreshState {
    rich: bool,
    primary_pending: bool,
}

/// Sorted read-only tick snapshot with explicitly versioned derived observations.
#[derive(Debug)]
pub struct TickContext {
    /// Simulation tick to which all captured observations belong.
    pub tick: u64,
    rows: Vec<PersonSnapshot>,
    row_refresh_states: Vec<RowRefreshState>,
    active_visit: Option<(AgentId, usize)>,
    versions: [u64; DOMAIN_COUNT],
    derived: BTreeMap<AgentId, BTreeMap<String, Versioned<i64>>>,
    targets: BTreeMap<AgentId, BTreeMap<String, Versioned<Option<TargetCandidate>>>>,
    derived_slots: [Vec<Option<Versioned<i64>>>; DERIVED_SLOT_COUNT],
    target_slots: [Vec<Option<Versioned<Option<TargetCandidate>>>>; TARGET_SLOT_COUNT],
    scopes: Vec<Option<TargetScope>>,
    running: Vec<Option<Option<Arc<RunningMetadata>>>>,
    /// Shared tick-start need-source availability.
    pub availability: NeedAvailability,
    /// Shared tick-start threat identities and tile positions.
    pub threats: Vec<(u32, (u32, u32))>,
    claims: BTreeMap<(u32, u32), u32>,
    gather_cache: RefCell<BTreeMap<GatherCacheKey, GatherCacheValue>>,
    food_index: RefCell<FoodIndex>,
}
impl TickContext {
    /// All persons with decision state/position, in stable AgentId order.
    pub fn capture(world: &World, resources: &SimResources) -> Result<Self, ContextError> {
        Self::capture_with_memory(world, resources, true, None, None)
    }
    pub(super) fn capture_continuation(
        world: &World,
        resources: &SimResources,
    ) -> Result<Self, ContextError> {
        Self::capture_with_memory(world, resources, false, None, None)
    }
    pub(super) fn capture_continuation_with_claims(
        world: &World,
        resources: &SimResources,
        claims: BTreeMap<(u32, u32), u32>,
        initial_stock: Option<&BTreeMap<SettlementId, ForagePlan>>,
    ) -> Result<Self, ContextError> {
        Self::capture_with_memory(world, resources, false, Some(claims), initial_stock)
    }
    fn capture_with_memory(
        world: &World,
        resources: &SimResources,
        rich: bool,
        initial_claims: Option<BTreeMap<(u32, u32), u32>>,
        initial_stock: Option<&BTreeMap<SettlementId, ForagePlan>>,
    ) -> Result<Self, ContextError> {
        // Two bulk ECS passes avoid a component lookup per field per person.
        let mut rows: Vec<PersonSnapshot> = world
            .query::<(
                &Agent,
                &Position,
                &AgentState,
                Option<&Hunger>,
                Option<&Thirst>,
                Option<&Sleep>,
                Option<&Social>,
                Option<&Temperament>,
                Option<&Inventory>,
                Option<&Job>,
                Option<&Carry>,
                Option<&BodyHealth>,
            )>()
            .iter()
            .map(
                |(
                    entity,
                    (
                        agent,
                        position,
                        state,
                        hunger,
                        thirst,
                        sleep,
                        social,
                        temperament,
                        inventory,
                        job,
                        carry,
                        health,
                    ),
                )| PersonSnapshot {
                    agent: agent.id,
                    entity,
                    position: *position,
                    state: *state,
                    life_stage: LifeStage::Adult,
                    hunger: hunger.map(|v| f64::from(v.value)),
                    thirst: thirst.map(|v| v.value),
                    fatigue: sleep.map(|v| v.fatigue),
                    social: social.map(|v| v.loneliness),
                    temperament: temperament.copied(),
                    inventory_total: inventory.map(|v| v.total()),
                    inventory_food: inventory.map(|v| v.get(ResourceKind::Food)),
                    job: job.copied(),
                    carry: carry.copied(),
                    hp: health.map(|v| v.hp),
                    drafted: false,
                    fleeing: None,
                    cold: false,
                    sick: false,
                    mood: None,
                    activity: None,
                    activity_rest: None,
                    memory_entries: None,
                    lineage: None,
                    memory_weights: [0.0; 6],
                    memory_causes: [None; 6],
                },
            )
            .collect();
        order_person_rows(&mut rows)?;
        let mut row_refresh_states = vec![
            RowRefreshState {
                rich,
                primary_pending: !rich && initial_claims.is_some(),
            };
            rows.len()
        ];
        for (
            entity,
            (
                agent,
                child,
                elder,
                drafted,
                fleeing,
                cold,
                sick,
                mood,
                activity,
                rest,
                memory,
                lineage,
            ),
        ) in world
            .query::<(
                &Agent,
                Option<&Child>,
                Option<&Elder>,
                Option<&Drafted>,
                Option<&Fleeing>,
                Option<&Cold>,
                Option<&Sick>,
                Option<&Mood>,
                Option<&Activity>,
                Option<&ActivityRest>,
                Option<&sim_core::components::Memory>,
                Option<&sim_core::components::Lineage>,
            )>()
            .iter()
        {
            if let Ok(index) = rows.binary_search_by_key(&agent.id, |row| row.agent) {
                let row = &mut rows[index];
                // Preserve all captured values; aliases only revoke the fast refresh.
                if entity != row.entity {
                    row_refresh_states[index].primary_pending = false;
                }
                row.life_stage = if child.is_some() {
                    LifeStage::Child
                } else if elder.is_some() {
                    LifeStage::Elder
                } else {
                    LifeStage::Adult
                };
                row.drafted = drafted.is_some();
                row.fleeing = fleeing.copied();
                row.cold = cold.is_some();
                row.sick = sick.is_some();
                row.mood = mood.copied();
                row.activity = activity.copied();
                row.activity_rest = rest.copied();
                row.memory_entries = memory.map(|v| v.entries.len());
                row.lineage = lineage.map(|v| v.key);
                if let Some(memory) = memory {
                    if rich {
                        (row.memory_weights, row.memory_causes) =
                            memory_values(memory, resources.current_tick);
                    } else {
                        // Frozen social-peer eligibility needs this one contributor at tick start.
                        // Other memory columns remain inaccessible until the full row is refreshed.
                        row.memory_causes[4] =
                            crate::runtime::agent_actions::memory_bias::top_contributor_entry(
                                memory,
                                crate::runtime::agent_actions::memory_bias::CascadeArm::Social,
                                resources.current_tick,
                            );
                    }
                }
            }
        }
        let mut context = Self {
            tick: resources.current_tick,
            row_refresh_states,
            active_visit: None,
            scopes: vec![None; rows.len()],
            running: vec![None; rows.len()],
            rows,
            versions: [0; DOMAIN_COUNT],
            derived_slots: std::array::from_fn(|_| Vec::new()),
            target_slots: std::array::from_fn(|_| Vec::new()),
            derived: BTreeMap::new(),
            targets: BTreeMap::new(),
            availability: NeedAvailability::from_resources(resources),
            threats: crate::runtime::wildlife::flee::threats(resources),
            claims: BTreeMap::new(),
            gather_cache: RefCell::new(BTreeMap::new()),
            food_index: RefCell::new(FoodIndex {
                version: 0,
                materialized: false,
                tiles: ChunkIndex::default(),
            }),
        };
        context.capture_target_scopes(world, resources, initial_stock);
        if let Some(claims) = initial_claims {
            context.claims = claims;
        } else {
            for (_, (state, target)) in world
                .query::<(&AgentState, &sim_core::components::SeekTarget)>()
                .iter()
            {
                if matches!(
                    state,
                    AgentState::Seeking {
                        target: sim_core::components::TargetKind::GatherFood
                    }
                ) {
                    *context.claims.entry(target.tile).or_default() += 1;
                }
            }
        }
        Ok(context)
    }
    /// All captured people in ascending stable AgentId order.
    pub fn rows(&self) -> &[PersonSnapshot] {
        &self.rows
    }
    fn capture_target_scopes(
        &mut self,
        world: &World,
        resources: &SimResources,
        initial_stock: Option<&BTreeMap<SettlementId, ForagePlan>>,
    ) {
        let initial_stock = initial_stock.filter(|_| {
            resources.rules.agent.forage.is_some() && resources.rules.society.stock_target.is_some()
        });
        let mut settlements: Vec<_> = resources.settlements.iter().collect();
        settlements.sort_unstable_by_key(|(id, _)| **id);
        let radius = crate::runtime::settlement::food_economy::catchment_radius(resources);
        let walking: BTreeSet<_> = world
            .query::<(&Agent, &Founding)>()
            .iter()
            .map(|(_, (a, _))| a.id)
            .chain(
                world
                    .query::<(&Agent, &PorterTrip)>()
                    .iter()
                    .filter(|_| resources.rules.society.relations.is_some())
                    .map(|(_, (a, _))| a.id),
            )
            .collect();
        let stock: BTreeMap<_, _> = settlements
            .iter()
            .map(|(sid, _)| {
                // Effect preparation captured these exact inputs before any effects ran.
                let (in_hand, target) = initial_stock
                    .and_then(|plans| plans.get(*sid))
                    .map(|plan| (plan.in_hand, plan.target))
                    .unwrap_or_else(|| {
                        let in_hand = crate::runtime::settlement::food_economy::stock_in_hand(
                            world, resources, **sid,
                        );
                        let target = crate::runtime::settlement::food_economy::stock_target(
                            world, resources, **sid,
                        )
                        .unwrap_or(0.0);
                        (in_hand, target)
                    });
                let cap = resources
                    .rules
                    .agent
                    .forage
                    .and(resources.rules.society.stock_target)
                    .map(|rule| rule.hoard_cap_factor * target);
                (**sid, ((in_hand as f64) < target, in_hand, cap))
            })
            .collect();
        let local_food: BTreeMap<_, _> = settlements
            .iter()
            .map(|(sid, settlement)| {
                let home = settlement.formation_tile;
                (
                    **sid,
                    resources.food_tiles.iter().any(|(tile, amount)| {
                        *amount > 0
                            && tile.0.abs_diff(home.0).max(tile.1.abs_diff(home.1)) <= radius
                    }),
                )
            })
            .collect();
        for (index, row) in self.rows.iter().enumerate() {
            let member = settlements
                .iter()
                .find(|(_, settlement)| settlement.member_agents.contains(&row.agent));
            let mut scope = TargetScope {
                food_available: self.availability.food,
                ..TargetScope::default()
            };
            if let Some((sid, settlement)) = member {
                scope.settlement = Some(**sid);
                scope.home = Some(settlement.formation_tile);
                scope.camp_stocked = settlement
                    .stockpile
                    .get(&ResourceKind::Food)
                    .copied()
                    .unwrap_or(0)
                    >= 1;
                if let Some((shortfall, in_hand, limit)) = stock.get(*sid) {
                    scope.stock_shortfall = *shortfall;
                    scope.stock_in_hand = *in_hand;
                    scope.stock_hoard_limit = *limit;
                }
                if resources.rules.agent.local_food_availability && !walking.contains(&row.agent) {
                    scope.local_food = Some((settlement.formation_tile, radius));
                    scope.food_available = local_food.get(*sid).copied().unwrap_or(false);
                }
                if resources.rules.society.relations.is_some() {
                    for ((a, b), state) in &resources.settlement_relations.pairs {
                        if !state.estranged {
                            continue;
                        }
                        let other = if a == *sid {
                            Some(b)
                        } else if b == *sid {
                            Some(a)
                        } else {
                            None
                        };
                        if let Some(other) = other.and_then(|id| resources.settlements.get(id)) {
                            scope.avoid_hearths.push(other.formation_tile);
                        }
                    }
                    scope.avoid_hearths.sort_unstable();
                }
            }
            self.scopes[index] = Some(scope);
        }
    }

    /// Look up a captured person; unknown IDs are errors, never fabricated rows.
    pub fn row(&self, agent: AgentId) -> Result<&PersonSnapshot, ContextError> {
        self.row_at(self.row_index(agent)?, agent)
    }
    pub(super) fn begin_visit(&mut self, agent: AgentId) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_at(index, agent)?;
        self.active_visit = Some((agent, index));
        Ok(())
    }
    pub(super) fn row_index(&self, agent: AgentId) -> Result<usize, ContextError> {
        if let Some((active, index)) = self.active_visit {
            if active == agent {
                self.row_at(index, agent)?;
                return Ok(index);
            }
        }
        self.rows
            .binary_search_by_key(&agent, |row| row.agent)
            .map_err(|_| ContextError::UnknownAgent(agent))
    }
    // Rows never move after capture; read current contents, including lifecycle updates.
    pub(super) fn row_at(
        &self,
        index: usize,
        agent: AgentId,
    ) -> Result<&PersonSnapshot, ContextError> {
        self.rows
            .get(index)
            .filter(|row| row.agent == agent)
            .ok_or(ContextError::UnknownAgent(agent))
    }
    /// Advance one dependency version, invalidating observations stamped against it.
    pub fn invalidate(&mut self, domain: Domain) -> Result<(), ContextError> {
        let version = &mut self.versions[domain as usize];
        *version = version
            .checked_add(1)
            .ok_or(ContextError::VersionOverflow(domain))?;
        Ok(())
    }
    /// Preserve one logical invalidation while updating only known affected food tiles.
    /// An already stale index stays stale: partial notifications cannot repair unknown edits.
    pub(in crate::runtime) fn invalidate_food_tiles(
        &mut self,
        resources: &SimResources,
        tiles: impl IntoIterator<Item = (u32, u32)>,
    ) -> Result<(), ContextError> {
        let previous = self.versions[Domain::FoodSources as usize];
        self.invalidate(Domain::FoodSources)?;
        let index = self.food_index.get_mut();
        if index.version != previous {
            return Ok(());
        }
        for tile in tiles {
            // Keep notification iteration semantics, but never build a partial index.
            if !index.materialized {
                continue;
            }
            let before = index
                .tiles
                .within_square(tile, 0)
                .next()
                .map(|(_, amount)| *amount);
            let after = resources.food_tiles.get(&tile).copied();
            if before == after {
                continue;
            }
            if let Some(amount) = before {
                index.tiles.remove(tile, amount);
            }
            if let Some(amount) = after {
                index.tiles.insert(tile, amount);
            }
        }
        index.version = self.versions[Domain::FoodSources as usize];
        Ok(())
    }
    /// None explicitly denotes a fresh/no-running lifecycle; absent means unknown.
    pub fn set_running(
        &mut self,
        agent: AgentId,
        running: Option<RunningChoice>,
        policy: &CompiledMind,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        let metadata = running
            .map(|choice| {
                if choice.revision != policy.revision() {
                    return Err(ContextError::InvalidRunningChoice(agent));
                }
                let node = policy
                    .node(choice.node)
                    .ok_or(ContextError::InvalidRunningChoice(agent))?;
                let behaviour = policy
                    .behaviour(node.behaviour)
                    .ok_or(ContextError::InvalidRunningChoice(agent))?;
                Ok((&behaviour.name, &node.target, choice.finished))
            })
            .transpose()?;
        match metadata {
            None => {
                self.running[index] = Some(None);
            }
            Some((behaviour, target, finished)) => {
                if let Some(Some(stored)) = self.running[index].as_mut() {
                    let stored = Arc::make_mut(stored);
                    stored.behaviour.clone_from(behaviour);
                    stored.target.clone_from(target);
                    stored.finished = finished;
                } else {
                    self.running[index] = Some(Some(Arc::new(RunningMetadata {
                        behaviour: behaviour.clone(),
                        target: target.clone(),
                        finished,
                    })));
                }
            }
        }
        Ok(())
    }
    pub(super) fn set_program_running(
        &mut self,
        agent: AgentId,
        running: Option<RunningChoice>,
        program: &super::MindProgram,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        let metadata = running
            .map(|choice| program.running_metadata(agent, choice))
            .transpose()?;
        self.running[index] = Some(metadata);
        Ok(())
    }
    /// Descriptive prior actual ownership for shadow; independent of its predicted node.
    pub fn set_observed_running(
        &mut self,
        agent: AgentId,
        running: Option<RunningMetadata>,
        policy: &CompiledMind,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        if let Some(metadata) = &running {
            let behaviour = policy
                .behaviour_id(&metadata.behaviour)
                .and_then(|id| policy.behaviour(id))
                .ok_or(ContextError::InvalidRunningChoice(agent))?;
            if metadata
                .target
                .as_ref()
                .is_some_and(|target| !behaviour.target_bindings.contains(target))
            {
                return Err(ContextError::InvalidRunningChoice(agent));
            }
        }
        self.running[index] = Some(running.map(Arc::new));
        Ok(())
    }
    /// Read prior actual ownership; unset history is an error, distinct from explicit None.
    pub fn running(&self, agent: AgentId) -> Result<Option<&RunningMetadata>, ContextError> {
        let index = self.row_index(agent)?;
        self.running[index]
            .as_ref()
            .map(Option::as_deref)
            .ok_or_else(|| ContextError::MissingDerived {
                agent,
                name: "previous_running_choice".into(),
            })
    }
    /// Synchronize deferred routine ownership and invalidate Person only when it changes.
    pub fn set_activity_snapshot(
        &mut self,
        agent: AgentId,
        activity: Option<Activity>,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        if self.rows[index].activity != activity {
            self.invalidate(Domain::Person)?;
            self.rows[index].activity = activity;
        }
        Ok(())
    }
    /// Copy a borrowed decision row without fetching ECS or invalidating unchanged values.
    pub fn patch_person(
        &mut self,
        agent: AgentId,
        update: PersonUpdate,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let row = &self.rows[index];
        if row.position == update.position
            && row.state == update.state
            && row.hunger == update.hunger
            && row.thirst == update.thirst
            && row.fatigue == update.fatigue
            && row.social == update.social
            && row.inventory_total == update.inventory_total
            && row.inventory_food == update.inventory_food
        {
            return Ok(());
        }
        self.invalidate(Domain::Person)?;
        let row = &mut self.rows[index];
        row.position = update.position;
        row.state = update.state;
        row.hunger = update.hunger;
        row.thirst = update.thirst;
        row.fatigue = update.fatigue;
        row.social = update.social;
        row.inventory_total = update.inventory_total;
        row.inventory_food = update.inventory_food;
        Ok(())
    }
    /// Refresh one dirty person from ECS and invalidate Person-dependent observations.
    /// Refresh only owner/guard inputs; full fallback refreshes memory and every optional field.
    pub fn refresh_continuation(
        &mut self,
        world: &World,
        agent: AgentId,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let entity = world
            .entity(self.rows[index].entity)
            .map_err(|_| ContextError::UnknownAgent(agent))?;
        self.invalidate(Domain::Person)?;
        self.row_refresh_states[index].rich = false;
        let row = &mut self.rows[index];
        row.position =
            entity
                .get::<&Position>()
                .map(|v| *v)
                .ok_or(ContextError::MissingComponent {
                    agent,
                    component: "Position",
                })?;
        row.state =
            entity
                .get::<&AgentState>()
                .map(|v| *v)
                .ok_or(ContextError::MissingComponent {
                    agent,
                    component: "AgentState",
                })?;
        row.life_stage = if entity.get::<&Child>().is_some() {
            LifeStage::Child
        } else if entity.get::<&Elder>().is_some() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        row.hunger = entity.get::<&Hunger>().map(|v| f64::from(v.value));
        row.thirst = entity.get::<&Thirst>().map(|v| v.value);
        row.fatigue = entity.get::<&Sleep>().map(|v| v.fatigue);
        row.social = entity.get::<&Social>().map(|v| v.loneliness);
        row.temperament = entity.get::<&Temperament>().map(|v| *v);
        let inventory = entity.get::<&Inventory>();
        row.inventory_total = inventory.as_ref().map(|v| v.total());
        row.inventory_food = inventory.as_ref().map(|v| v.get(ResourceKind::Food));
        row.job = entity.get::<&Job>().map(|v| *v);
        row.carry = entity.get::<&Carry>().map(|v| *v);
        row.drafted = entity.get::<&Drafted>().is_some();
        row.fleeing = entity.get::<&Fleeing>().map(|v| *v);
        row.activity = entity.get::<&Activity>().map(|v| *v);
        row.activity_rest = entity.get::<&ActivityRest>().map(|v| *v);
        Ok(())
    }
    pub(super) fn refresh_continuation_exclusive(
        &mut self,
        world: &mut World,
        agent: AgentId,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let (
            (position, state, child, elder, hunger, thirst, sleep, social),
            (temperament, inventory, job, carry, drafted, fleeing, activity, rest),
        ) = world
            .query_one_mut::<(
                (
                    Option<&Position>,
                    Option<&AgentState>,
                    Option<&Child>,
                    Option<&Elder>,
                    Option<&Hunger>,
                    Option<&Thirst>,
                    Option<&Sleep>,
                    Option<&Social>,
                ),
                (
                    Option<&Temperament>,
                    Option<&Inventory>,
                    Option<&Job>,
                    Option<&Carry>,
                    Option<&Drafted>,
                    Option<&Fleeing>,
                    Option<&Activity>,
                    Option<&ActivityRest>,
                ),
            )>(self.rows[index].entity)
            .map_err(|_| ContextError::UnknownAgent(agent))?;
        self.invalidate(Domain::Person)?;
        self.row_refresh_states[index].rich = false;
        let row = &mut self.rows[index];
        row.position = position.copied().ok_or(ContextError::MissingComponent {
            agent,
            component: "Position",
        })?;
        row.state = state.copied().ok_or(ContextError::MissingComponent {
            agent,
            component: "AgentState",
        })?;
        row.life_stage = if child.is_some() {
            LifeStage::Child
        } else if elder.is_some() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        row.hunger = hunger.map(|v| f64::from(v.value));
        row.thirst = thirst.map(|v| v.value);
        row.fatigue = sleep.map(|v| v.fatigue);
        row.social = social.map(|v| v.loneliness);
        row.temperament = temperament.copied();
        row.inventory_total = inventory.map(|v| v.total());
        row.inventory_food = inventory.map(|v| v.get(ResourceKind::Food));
        row.job = job.copied();
        row.carry = carry.copied();
        row.drafted = drafted.is_some();
        row.fleeing = fleeing.copied();
        row.activity = activity.copied();
        row.activity_rest = rest.copied();
        Ok(())
    }
    /// Acknowledge an untouched primary row only at the coordinator's first visit.
    /// No ECS mutation or earlier person-row overlay may precede this private call.
    pub(super) fn acknowledge_primary_capture(
        &mut self,
        agent: AgentId,
    ) -> Result<bool, ContextError> {
        let index = self.row_index(agent)?;
        if !self.row_refresh_states[index].primary_pending {
            return Ok(false);
        }
        self.invalidate(Domain::Person)?;
        self.row_refresh_states[index].rich = false;
        self.row_refresh_states[index].primary_pending = false;
        Ok(true)
    }
    /// Only the first, non-invited visit may retain initial entity-captured fields.
    /// Keep secondary-pass fields live: an uncaptured duplicate AgentId can overwrite them.
    pub(super) fn refresh_primary_continuation_exclusive(
        &mut self,
        world: &mut World,
        agent: AgentId,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let (position, state, child, elder, drafted, fleeing, activity, rest) = world
            .query_one_mut::<(
                Option<&Position>,
                Option<&AgentState>,
                Option<&Child>,
                Option<&Elder>,
                Option<&Drafted>,
                Option<&Fleeing>,
                Option<&Activity>,
                Option<&ActivityRest>,
            )>(self.rows[index].entity)
            .map_err(|_| ContextError::UnknownAgent(agent))?;
        self.invalidate(Domain::Person)?;
        self.row_refresh_states[index].rich = false;
        let row = &mut self.rows[index];
        row.position = position.copied().ok_or(ContextError::MissingComponent {
            agent,
            component: "Position",
        })?;
        row.state = state.copied().ok_or(ContextError::MissingComponent {
            agent,
            component: "AgentState",
        })?;
        row.life_stage = if child.is_some() {
            LifeStage::Child
        } else if elder.is_some() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        row.drafted = drafted.is_some();
        row.fleeing = fleeing.copied();
        row.activity = activity.copied();
        row.activity_rest = rest.copied();
        Ok(())
    }
    /// Refresh the complete current person, including memory, before full selection.
    pub fn refresh_person(&mut self, world: &World, agent: AgentId) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let updated = PersonSnapshot::capture(world, self.rows[index].entity, agent, self.tick)?;
        self.invalidate(Domain::Person)?;
        self.rows[index] = updated;
        self.row_refresh_states[index].rich = true;
        Ok(())
    }
    /// Unchanged exclusive full-refresh oracle; shared refresh remains the adapter path.
    #[cfg(test)]
    pub(super) fn refresh_person_exclusive(
        &mut self,
        world: &mut World,
        agent: AgentId,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let updated =
            PersonSnapshot::capture_exclusive(world, self.rows[index].entity, agent, self.tick)?;
        self.invalidate(Domain::Person)?;
        self.rows[index] = updated;
        self.row_refresh_states[index].rich = true;
        Ok(())
    }
    /// Complete a successful light refresh without intervening ECS mutation.
    /// Only routine Activity may have been overlaid on the person row meanwhile.
    pub(super) fn upgrade_continuation_exclusive(
        &mut self,
        world: &mut World,
        agent: AgentId,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.row_refresh_states[index].primary_pending = false;
        let (cold, sick, mood, health, memory, lineage, activity) = world
            .query_one_mut::<(
                Option<&Cold>,
                Option<&Sick>,
                Option<&Mood>,
                Option<&BodyHealth>,
                Option<&sim_core::components::Memory>,
                Option<&sim_core::components::Lineage>,
                Option<&Activity>,
            )>(self.rows[index].entity)
            .map_err(|_| ContextError::MissingComponent {
                agent,
                component: "Position",
            })?;
        let (memory_weights, memory_causes) = memory.map_or(([0.0; 6], [None; 6]), |memory| {
            memory_values(memory, self.tick)
        });
        let cold = cold.is_some();
        let sick = sick.is_some();
        let mood = mood.copied();
        let hp = health.map(|value| value.hp);
        let lineage = lineage.map(|value| value.key);
        let activity = activity.copied();
        let memory_entries = memory.map(|value| value.entries.len());
        self.invalidate(Domain::Person)?;
        let row = &mut self.rows[index];
        row.cold = cold;
        row.sick = sick;
        row.mood = mood;
        row.hp = hp;
        row.lineage = lineage;
        row.activity = activity;
        row.memory_weights = memory_weights;
        row.memory_causes = memory_causes;
        row.memory_entries = memory_entries;
        self.row_refresh_states[index].rich = true;
        Ok(())
    }
    pub(super) fn require_rich_row(&self, agent: AgentId) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        if !self.row_refresh_states[index].rich {
            return Err(ContextError::MissingDerived {
                agent,
                name: "rich_person_snapshot".into(),
            });
        }
        Ok(())
    }
    fn stamp<T>(&self, value: T, domains: &[Domain]) -> Versioned<T> {
        let dependencies = domains
            .iter()
            .fold(0, |mask, domain| mask | (1 << *domain as usize));
        Versioned {
            value,
            dependencies,
            version_sum: selected_version_sum(dependencies, &self.versions),
        }
    }
    /// Install the complete immutable social cohort once, without repeated person searches.
    pub(super) fn install_social_snapshot(
        &mut self,
        rows: Vec<SocialSnapshotRow>,
    ) -> Result<(), ContextError> {
        let names = [
            "social_candidate",
            "social_memory_candidate",
            "social_handshake",
        ];
        let mut derived = [0; 3];
        for (index, name) in names.into_iter().enumerate() {
            derived[index] = derived_slot(name).ok_or_else(|| ContextError::MissingDerived {
                agent: 0,
                name: name.into(),
            })?;
            self.derived_slots[derived[index]].resize_with(self.rows.len(), || None);
        }
        let mut targets = [0; 2];
        for (index, name) in ["social_peer", "social_memory_peer"]
            .into_iter()
            .enumerate()
        {
            targets[index] = target_slot(name).ok_or_else(|| ContextError::MissingDerived {
                agent: 0,
                name: name.into(),
            })?;
            self.target_slots[targets[index]].resize_with(self.rows.len(), || None);
        }
        if rows.len() != self.rows.len() {
            return Err(ContextError::MissingDerived {
                agent: 0,
                name: "social_cohort".into(),
            });
        }
        for (index, (agent, tile, natural, memory, handshake)) in rows.into_iter().enumerate() {
            self.row_at(index, agent)?;
            for (slot, value) in derived.into_iter().zip([
                i64::from(natural.is_some()),
                i64::from(memory.is_some()),
                i64::from(handshake),
            ]) {
                self.derived_slots[slot][index] = Some(Versioned {
                    value,
                    dependencies: 0,
                    version_sum: 0,
                });
            }
            for (slot, peer) in targets.into_iter().zip([natural, memory]) {
                self.target_slots[slot][index] = Some(Versioned {
                    value: peer.map(|peer| TargetCandidate {
                        tile,
                        agent: Some(peer),
                    }),
                    dependencies: 0,
                    version_sum: 0,
                });
            }
        }
        Ok(())
    }
    /// Install the four lifecycle observations at the same dependency boundary.
    pub(super) fn set_routine_lifecycle(
        &mut self,
        agent: AgentId,
        admission: bool,
        invited: bool,
        deadline: bool,
    ) -> Result<(), ContextError> {
        let row = self.row_index(agent)?;
        let stamp = self.stamp(
            0,
            &[
                Domain::Person,
                Domain::RoutineOccupancy,
                Domain::Invitations,
            ],
        );
        // These are the contiguous routine_admission through activity_deadline_reached slots.
        for (column, value) in self.derived_slots[3..7]
            .iter_mut()
            .zip([admission, invited, deadline, deadline])
        {
            if column.is_empty() {
                column.resize_with(self.rows.len(), || None);
            }
            column[row] = Some(Versioned {
                value: i64::from(value),
                dependencies: stamp.dependencies,
                version_sum: stamp.version_sum,
            });
        }
        Ok(())
    }
    /// Store a factual observation with its current dependency versions; no domains freezes it for this tick.
    pub fn set_derived(
        &mut self,
        agent: AgentId,
        name: &str,
        value: i64,
        domains: &[Domain],
    ) -> Result<(), ContextError> {
        let row = self.row_index(agent)?;
        let value = self.stamp(value, domains);
        if let Some(slot) = derived_slot(name) {
            let column = &mut self.derived_slots[slot];
            if column.is_empty() {
                column.resize_with(self.rows.len(), || None);
            }
            column[row] = Some(value);
            return Ok(());
        }
        let values = self.derived.entry(agent).or_default();
        match values.get_mut(name) {
            Some(stored) => *stored = value,
            None => {
                values.insert(name.to_owned(), value);
            }
        }
        Ok(())
    }
    /// Read a factual observation, rejecting missing or invalidated evidence.
    pub fn derived(&self, agent: AgentId, name: &str) -> Result<i64, ContextError> {
        let value = match derived_slot(name) {
            Some(slot) => self
                .row_index(agent)
                .ok()
                .and_then(|row| self.derived_slots[slot].get(row))
                .and_then(Option::as_ref),
            None => self.derived.get(&agent).and_then(|values| values.get(name)),
        }
        .ok_or_else(|| ContextError::MissingDerived {
            agent,
            name: name.to_owned(),
        })?;
        if !value.fresh(&self.versions) {
            return Err(ContextError::StaleDerived {
                agent,
                name: name.to_owned(),
            });
        }
        Ok(value.value)
    }
    /// Store a resolved target or explicit absence with its dependency versions.
    pub fn set_target(
        &mut self,
        agent: AgentId,
        name: &str,
        target: Option<TargetCandidate>,
        domains: &[Domain],
    ) -> Result<(), ContextError> {
        let row = self.row_index(agent)?;
        let value = self.stamp(target, domains);
        if let Some(slot) = target_slot(name) {
            let column = &mut self.target_slots[slot];
            if column.is_empty() {
                column.resize_with(self.rows.len(), || None);
            }
            column[row] = Some(value);
            return Ok(());
        }
        let values = self.targets.entry(agent).or_default();
        match values.get_mut(name) {
            Some(stored) => *stored = value,
            None => {
                values.insert(name.to_owned(), value);
            }
        }
        Ok(())
    }
    /// Read a resolved target; missing/stale evidence differs from a valid absent target.
    pub fn target(
        &self,
        agent: AgentId,
        name: &str,
    ) -> Result<Option<TargetCandidate>, ContextError> {
        let value = match target_slot(name) {
            Some(slot) => self
                .row_index(agent)
                .ok()
                .and_then(|row| self.target_slots[slot].get(row))
                .and_then(Option::as_ref),
            None => self.targets.get(&agent).and_then(|values| values.get(name)),
        }
        .ok_or_else(|| ContextError::MissingDerived {
            agent,
            name: name.to_owned(),
        })?;
        if !value.fresh(&self.versions) {
            return Err(ContextError::StaleDerived {
                agent,
                name: name.to_owned(),
            });
        }
        Ok(value.value)
    }
    /// Replace the targeting snapshot and invalidate Person-dependent observations.
    pub fn set_target_scope(
        &mut self,
        agent: AgentId,
        scope: TargetScope,
    ) -> Result<(), ContextError> {
        let index = self.row_index(agent)?;
        self.invalidate(Domain::Person)?;
        self.scopes[index] = Some(scope);
        Ok(())
    }
    /// Apply a checked reservation delta and invalidate Claims; zero removes the tile.
    pub fn adjust_claim(&mut self, tile: (u32, u32), delta: i32) -> Result<(), ContextError> {
        let count = self.claims.get(&tile).copied().unwrap_or(0);
        let next = if delta >= 0 {
            count.checked_add(delta as u32)
        } else {
            count.checked_sub(delta.unsigned_abs())
        }
        .ok_or(ContextError::InvalidClaim(tile))?;
        self.invalidate(Domain::Claims)?;
        if next == 0 {
            self.claims.remove(&tile);
        } else {
            self.claims.insert(tile, next);
        }
        Ok(())
    }
    /// Current coordinator-owned reservation counts, without mutable access.
    pub fn claims(&self) -> &BTreeMap<(u32, u32), u32> {
        &self.claims
    }
    /// Real owner applies claim deltas in effect order; shadow updates only its copy.
    pub fn replace_claims(
        &mut self,
        claims: BTreeMap<(u32, u32), u32>,
    ) -> Result<(), ContextError> {
        self.invalidate(Domain::Claims)?;
        self.claims = claims;
        Ok(())
    }
    /// Memoize physical gather-candidate existence by location, scope, and source versions.
    pub fn gather_available(
        &self,
        agent: AgentId,
        resources: &SimResources,
    ) -> Result<bool, ContextError> {
        let person = self.row(agent)?;
        let scope = self.target_scope(agent)?;
        let key = (
            (person.position.x, person.position.y),
            scope.local_food,
            scope.settlement,
        );
        let version = [
            self.versions[Domain::FoodSources as usize],
            self.versions[Domain::Person as usize],
        ];
        if let Some((stamp, value)) = self.gather_cache.borrow().get(&key) {
            if *stamp == version {
                return Ok(*value);
            }
        }
        let value = self.gather_candidate(agent, resources)?.is_some();
        self.gather_cache.borrow_mut().insert(key, (version, value));
        Ok(value)
    }

    /// Visit positive food amounts at the current FoodSources version. Callers
    /// must invalidate that domain after changing amounts or key membership.
    pub(in crate::runtime) fn visit_food_in_square(
        &self,
        resources: &SimResources,
        centre: (u32, u32),
        radius: u32,
        mut visit: impl FnMut((u32, u32), u8),
    ) {
        let version = self.versions[Domain::FoodSources as usize];
        let mut index = self.food_index.borrow_mut();
        if !index.materialized || index.version != version {
            *index = FoodIndex::capture(resources, version);
        }
        for &(tile, amount) in index.tiles.within_square(centre, radius) {
            if amount > 0 {
                visit(tile, amount);
            }
        }
    }

    /// The exact live gather minimum, including catchment and estranged hearths.
    pub(in crate::runtime) fn gather_candidate(
        &self,
        agent: AgentId,
        resources: &SimResources,
    ) -> Result<Option<(u32, u32)>, ContextError> {
        let person = self.row(agent)?;
        let scope = self.target_scope(agent)?;
        let centre = (person.position.x, person.position.y);
        let mut best = None;
        self.visit_food_in_square(resources, centre, targets::GATHER_SCAN_RADIUS, |tile, _| {
            if scope.local_food.is_some_and(|(home, radius)| {
                chebyshev(tile, home) > radius
                    || scope
                        .avoid_hearths
                        .iter()
                        .any(|hearth| chebyshev(tile, *hearth) < chebyshev(tile, home))
            }) {
                return;
            }
            let distance =
                u64::from(tile.0.abs_diff(centre.0)) + u64::from(tile.1.abs_diff(centre.1));
            let key = (distance, tile.0, tile.1);
            if best.is_none_or(|old| key < old) {
                best = Some(key);
            }
        });
        Ok(best.map(|(_, x, y)| (x, y)))
    }
    /// Read the required per-person targeting scope, rejecting an unprepared scope.
    pub fn target_scope(&self, agent: AgentId) -> Result<&TargetScope, ContextError> {
        self.row_index(agent)
            .ok()
            .and_then(|index| self.scopes.get(index))
            .and_then(Option::as_ref)
            .ok_or_else(|| ContextError::MissingDerived {
                agent,
                name: "target_scope".to_owned(),
            })
    }
}

fn memory_values(
    memory: &sim_core::components::Memory,
    tick: u64,
) -> ([f64; 6], [Option<sim_core::causal::EventId>; 6]) {
    use crate::runtime::memory::MAX_RECENCY_TICKS;
    use sim_core::components::{MemoryArm, SALIENCE_FLOOR};
    use std::cmp::Ordering;

    // Match Sum<f64>'s ordered fold, including negative zero for empty arms.
    let mut values = [-0.0; 6];
    let mut causes = [None; 6];
    let mut magnitudes = [0.0_f64; 6];
    for entry in memory
        .entries
        .iter()
        .filter(|entry| entry.salience > SALIENCE_FLOOR)
    {
        let index = match entry.arm {
            MemoryArm::Hunger => 0,
            MemoryArm::Thirst => 1,
            MemoryArm::Fatigue => 2,
            MemoryArm::Construction => 3,
            MemoryArm::Social => 4,
            MemoryArm::Combat => 5,
            MemoryArm::None => continue,
        };
        let elapsed = tick.saturating_sub(entry.encoded_tick);
        let recency = if elapsed >= MAX_RECENCY_TICKS {
            0.0
        } else {
            1.0 - (elapsed as f64 / MAX_RECENCY_TICKS as f64)
        };
        let weight = entry.valence * entry.salience * recency;
        values[index] += weight;
        let magnitude = weight.abs();
        // max_by replaces the incumbent on Equal, including unordered NaN comparisons.
        if causes[index].is_none()
            || magnitudes[index]
                .partial_cmp(&magnitude)
                .unwrap_or(Ordering::Equal)
                != Ordering::Greater
        {
            magnitudes[index] = magnitude;
            causes[index] = Some(entry.event_id);
        }
    }
    (values, causes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_sorts_people_and_rejects_stale_claim_evidence() {
        let mut world = World::new();
        world.spawn((Agent { id: 9 }, Position::new(1, 1), AgentState::Idle));
        world.spawn((Agent { id: 2 }, Position::new(2, 1), AgentState::Idle));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = TickContext::capture(&world, &resources).expect("snapshot");
        assert_eq!(
            context.rows().iter().map(|p| p.agent).collect::<Vec<_>>(),
            vec![2, 9]
        );
        context
            .set_derived(2, "bush_available", 1, &[Domain::Claims])
            .expect("fact");
        context
            .set_derived(2, "social_start", 1, &[])
            .expect("frozen fact");
        context.adjust_claim((2, 2), 1).expect("claim");
        assert!(matches!(
            context.derived(2, "bush_available"),
            Err(ContextError::StaleDerived { .. })
        ));
        assert_eq!(context.derived(2, "social_start"), Ok(1));
        assert!(matches!(
            context.derived(2, "unwritten"),
            Err(ContextError::MissingDerived { .. })
        ));
    }

    #[test]
    fn empty_claim_release_is_an_error_not_saturation() {
        let world = World::new();
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = TickContext::capture(&world, &resources).expect("snapshot");
        assert_eq!(
            context.adjust_claim((1, 1), -1),
            Err(ContextError::InvalidClaim((1, 1)))
        );
        assert!(context.claims().is_empty());
    }
}

#[cfg(test)]
mod stamp_tests {
    use super::*;

    const DOMAINS: [Domain; DOMAIN_COUNT] = [
        Domain::Person,
        Domain::Claims,
        Domain::FoodSources,
        Domain::WaterSources,
        Domain::SleepSources,
        Domain::RoutineOccupancy,
        Domain::Invitations,
        Domain::Jobs,
        Domain::Threats,
    ];

    fn context() -> TickContext {
        let mut world = World::new();
        world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let resources = SimResources::new(4, 4, sim_core::MaterialRegistry::new());
        TickContext::capture(&world, &resources).expect("context")
    }

    #[test]
    fn dependency_stamps_match_domain_membership_including_empty_and_duplicates() {
        for dependency in DOMAINS {
            for changed in DOMAINS {
                let mut context = context();
                let stamped = context.stamp(4, &[dependency, dependency]);
                let frozen = context.stamp(5, &[]);
                let all = context.stamp(6, &DOMAINS);
                assert!(stamped.fresh(&context.versions));
                context.invalidate(changed).unwrap();
                assert_eq!(stamped.fresh(&context.versions), dependency != changed);
                assert!(frozen.fresh(&context.versions));
                assert!(!all.fresh(&context.versions));
            }
        }
    }

    #[test]
    fn replacement_reuses_keys_and_restamps_stale_values_and_absent_targets() {
        let mut context = context();
        context
            .set_derived(7, "derived", 1, &[Domain::Claims])
            .unwrap();
        context
            .set_target(
                7,
                "target",
                Some(TargetCandidate {
                    tile: (2, 2),
                    agent: None,
                }),
                &[Domain::Claims],
            )
            .unwrap();
        let derived_key = context.derived[&7].keys().next().unwrap().as_ptr();
        let target_key = context.targets[&7].keys().next().unwrap().as_ptr();
        context.invalidate(Domain::Claims).unwrap();
        assert!(matches!(
            context.derived(7, "derived"),
            Err(ContextError::StaleDerived { .. })
        ));
        assert!(matches!(
            context.target(7, "target"),
            Err(ContextError::StaleDerived { .. })
        ));
        context
            .set_derived(7, "derived", 2, &[Domain::Claims])
            .unwrap();
        context.set_target(7, "target", None, &[]).unwrap();
        assert_eq!(context.derived(7, "derived"), Ok(2));
        assert_eq!(context.target(7, "target"), Ok(None));
        assert_eq!(
            context.derived[&7].keys().next().unwrap().as_ptr(),
            derived_key
        );
        assert_eq!(
            context.targets[&7].keys().next().unwrap().as_ptr(),
            target_key
        );
        context.invalidate(Domain::Claims).unwrap();
        assert!(matches!(
            context.derived(7, "derived"),
            Err(ContextError::StaleDerived { .. })
        ));
        assert_eq!(context.target(7, "target"), Ok(None));
        assert!(matches!(
            context.target(7, "missing"),
            Err(ContextError::MissingDerived { .. })
        ));
    }
}

#[cfg(test)]
mod food_index_tests;

#[cfg(test)]
mod overlay_slot_tests;

#[cfg(test)]
mod snapshot_entity_tests;

#[cfg(test)]
mod dependency_sum_tests;

#[cfg(test)]
mod memory_values_tests;

#[cfg(test)]
#[path = "context_optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "context_index_tests.rs"]
mod index_tests;

#[cfg(test)]
#[path = "context_refresh_tests.rs"]
mod refresh_tests;

#[cfg(test)]
#[path = "context_scope_tests.rs"]
mod scope_tests;

#[cfg(test)]
#[path = "context_running_tests.rs"]
mod running_tests;

#[cfg(test)]
#[path = "context_row_order_tests.rs"]
mod row_order_tests;

#[cfg(test)]
#[path = "context_full_refresh_tests.rs"]
mod full_refresh_tests;

#[cfg(test)]
#[path = "context_lazy_food_tests.rs"]
mod lazy_food_tests;

#[cfg(test)]
#[path = "context_rich_upgrade_tests.rs"]
mod rich_upgrade_tests;

#[cfg(test)]
#[path = "context_primary_snapshot_tests.rs"]
mod primary_snapshot_tests;

//! Read-only person snapshots and versioned, coordinator-owned derived facts.
//!
//! Capture after lifecycle preparation. Effects must refresh changed people and
//! invalidate changed domains before the next primary or replay evaluation.
//! This context never owns a mutable routine transaction or simulation resource.

use crate::runtime::agent_actions::{needs::NeedAvailability, targets};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use hecs::{Entity, World};
use sim_core::colony::mind::{CompiledMind, LifeStage, RunningChoice};
use sim_core::colony::people::{Drafted, Fleeing, Mood, Sick};
use sim_core::colony::{Carry, Job};
use sim_core::components::{
    Activity, ActivityRest, Agent, AgentId, AgentState, BodyHealth, Child, Cold, Elder, Founding,
    Hunger, Inventory, PorterTrip, Position, ResourceKind, SettlementId, Sleep, Social,
    Temperament, Thirst,
};
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
        let position = world.get::<&Position>(entity).map(|v| *v).map_err(|_| {
            ContextError::MissingComponent {
                agent,
                component: "Position",
            }
        })?;
        let state = world.get::<&AgentState>(entity).map(|v| *v).map_err(|_| {
            ContextError::MissingComponent {
                agent,
                component: "AgentState",
            }
        })?;
        let life_stage = if world.get::<&Child>(entity).is_ok() {
            LifeStage::Child
        } else if world.get::<&Elder>(entity).is_ok() {
            LifeStage::Elder
        } else {
            LifeStage::Adult
        };
        let memory = world.get::<&sim_core::components::Memory>(entity).ok();
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
            hunger: world
                .get::<&Hunger>(entity)
                .ok()
                .map(|v| f64::from(v.value)),
            thirst: world.get::<&Thirst>(entity).ok().map(|v| v.value),
            fatigue: world.get::<&Sleep>(entity).ok().map(|v| v.fatigue),
            social: world.get::<&Social>(entity).ok().map(|v| v.loneliness),
            temperament: world.get::<&Temperament>(entity).ok().map(|v| *v),
            inventory_total: world.get::<&Inventory>(entity).ok().map(|v| v.total()),
            inventory_food: world
                .get::<&Inventory>(entity)
                .ok()
                .map(|v| v.get(ResourceKind::Food)),
            carry: world.get::<&Carry>(entity).ok().map(|v| *v),
            job: world.get::<&Job>(entity).ok().map(|v| *v),
            drafted: world.get::<&Drafted>(entity).is_ok(),
            fleeing: world.get::<&Fleeing>(entity).ok().map(|v| *v),
            cold: world.get::<&Cold>(entity).is_ok(),
            sick: world.get::<&Sick>(entity).is_ok(),
            mood: world.get::<&Mood>(entity).ok().map(|v| *v),
            hp: world.get::<&BodyHealth>(entity).ok().map(|v| v.hp),
            activity: world.get::<&Activity>(entity).ok().map(|v| *v),
            activity_rest: world.get::<&ActivityRest>(entity).ok().map(|v| *v),
            lineage: world
                .get::<&sim_core::components::Lineage>(entity)
                .ok()
                .map(|v| v.key),
            memory_weights,
            memory_causes,
            memory_entries,
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
    dependencies: Vec<(Domain, u64)>,
}
impl<T> Versioned<T> {
    fn fresh(&self, versions: &[u64; DOMAIN_COUNT]) -> bool {
        self.dependencies
            .iter()
            .all(|(domain, version)| versions[*domain as usize] == *version)
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

/// Sorted read-only tick snapshot with explicitly versioned derived observations.
#[derive(Debug)]
pub struct TickContext {
    /// Simulation tick to which all captured observations belong.
    pub tick: u64,
    rows: Vec<PersonSnapshot>,
    versions: [u64; DOMAIN_COUNT],
    derived: BTreeMap<AgentId, BTreeMap<String, Versioned<i64>>>,
    targets: BTreeMap<AgentId, BTreeMap<String, Versioned<Option<TargetCandidate>>>>,
    scopes: BTreeMap<AgentId, TargetScope>,
    running: BTreeMap<AgentId, Option<RunningMetadata>>,
    /// Shared tick-start need-source availability.
    pub availability: NeedAvailability,
    /// Shared tick-start threat identities and tile positions.
    pub threats: Vec<(u32, (u32, u32))>,
    claims: BTreeMap<(u32, u32), u32>,
    gather_cache: RefCell<BTreeMap<GatherCacheKey, GatherCacheValue>>,
}
impl TickContext {
    /// All persons with decision state/position, in stable AgentId order.
    pub fn capture(world: &World, resources: &SimResources) -> Result<Self, ContextError> {
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
        rows.sort_unstable_by_key(|row| row.agent);
        for pair in rows.windows(2) {
            if pair[0].agent == pair[1].agent {
                return Err(ContextError::DuplicateAgent(pair[0].agent));
            }
        }
        for (
            _,
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
                    (row.memory_weights, row.memory_causes) =
                        memory_values(memory, resources.current_tick);
                }
            }
        }
        let mut context = Self {
            tick: resources.current_tick,
            rows,
            versions: [0; DOMAIN_COUNT],
            derived: BTreeMap::new(),
            targets: BTreeMap::new(),
            scopes: BTreeMap::new(),
            availability: NeedAvailability::from_resources(resources),
            threats: crate::runtime::wildlife::flee::threats(resources),
            claims: BTreeMap::new(),
            gather_cache: RefCell::new(BTreeMap::new()),
            running: BTreeMap::new(),
        };
        context.capture_target_scopes(world, resources);
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
        Ok(context)
    }
    /// All captured people in ascending stable AgentId order.
    pub fn rows(&self) -> &[PersonSnapshot] {
        &self.rows
    }
    fn capture_target_scopes(&mut self, world: &World, resources: &SimResources) {
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
                let in_hand = crate::runtime::settlement::food_economy::stock_in_hand(
                    world, resources, **sid,
                );
                let target =
                    crate::runtime::settlement::food_economy::stock_target(world, resources, **sid)
                        .unwrap_or(0.0);
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
        for row in &self.rows {
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
            self.scopes.insert(row.agent, scope);
        }
    }

    /// Look up a captured person; unknown IDs are errors, never fabricated rows.
    pub fn row(&self, agent: AgentId) -> Result<&PersonSnapshot, ContextError> {
        self.rows
            .binary_search_by_key(&agent, |row| row.agent)
            .map(|index| &self.rows[index])
            .map_err(|_| ContextError::UnknownAgent(agent))
    }
    /// Advance one dependency version, invalidating observations stamped against it.
    pub fn invalidate(&mut self, domain: Domain) -> Result<(), ContextError> {
        let version = &mut self.versions[domain as usize];
        *version = version
            .checked_add(1)
            .ok_or(ContextError::VersionOverflow(domain))?;
        Ok(())
    }
    /// None explicitly denotes a fresh/no-running lifecycle; absent means unknown.
    pub fn set_running(
        &mut self,
        agent: AgentId,
        running: Option<RunningChoice>,
        policy: &CompiledMind,
    ) -> Result<(), ContextError> {
        self.row(agent)?;
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
                Ok(RunningMetadata {
                    behaviour: behaviour.name.clone(),
                    target: node.target.clone(),
                    finished: choice.finished,
                })
            })
            .transpose()?;
        self.running.insert(agent, metadata);
        Ok(())
    }
    /// Descriptive prior actual ownership for shadow; independent of its predicted node.
    pub fn set_observed_running(
        &mut self,
        agent: AgentId,
        running: Option<RunningMetadata>,
        policy: &CompiledMind,
    ) -> Result<(), ContextError> {
        self.row(agent)?;
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
        self.running.insert(agent, running);
        Ok(())
    }
    /// Read prior actual ownership; unset history is an error, distinct from explicit None.
    pub fn running(&self, agent: AgentId) -> Result<Option<&RunningMetadata>, ContextError> {
        self.row(agent)?;
        self.running
            .get(&agent)
            .map(Option::as_ref)
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
        let index = self
            .rows
            .binary_search_by_key(&agent, |row| row.agent)
            .map_err(|_| ContextError::UnknownAgent(agent))?;
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
        let index = self
            .rows
            .binary_search_by_key(&agent, |row| row.agent)
            .map_err(|_| ContextError::UnknownAgent(agent))?;
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
    pub fn refresh_person(&mut self, world: &World, agent: AgentId) -> Result<(), ContextError> {
        let index = self
            .rows
            .binary_search_by_key(&agent, |row| row.agent)
            .map_err(|_| ContextError::UnknownAgent(agent))?;
        let updated = PersonSnapshot::capture(world, self.rows[index].entity, agent, self.tick)?;
        self.invalidate(Domain::Person)?;
        self.rows[index] = updated;
        Ok(())
    }
    fn stamp<T>(&self, value: T, domains: &[Domain]) -> Versioned<T> {
        Versioned {
            value,
            dependencies: domains
                .iter()
                .map(|domain| (*domain, self.versions[*domain as usize]))
                .collect(),
        }
    }
    /// Store a factual observation with its current dependency versions; no domains freezes it for this tick.
    pub fn set_derived(
        &mut self,
        agent: AgentId,
        name: &str,
        value: i64,
        domains: &[Domain],
    ) -> Result<(), ContextError> {
        self.row(agent)?;
        let value = self.stamp(value, domains);
        self.derived
            .entry(agent)
            .or_default()
            .insert(name.to_owned(), value);
        Ok(())
    }
    /// Read a factual observation, rejecting missing or invalidated evidence.
    pub fn derived(&self, agent: AgentId, name: &str) -> Result<i64, ContextError> {
        let value = self
            .derived
            .get(&agent)
            .and_then(|values| values.get(name))
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
        self.row(agent)?;
        let value = self.stamp(target, domains);
        self.targets
            .entry(agent)
            .or_default()
            .insert(name.to_owned(), value);
        Ok(())
    }
    /// Read a resolved target; missing/stale evidence differs from a valid absent target.
    pub fn target(
        &self,
        agent: AgentId,
        name: &str,
    ) -> Result<Option<TargetCandidate>, ContextError> {
        let value = self
            .targets
            .get(&agent)
            .and_then(|values| values.get(name))
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
        self.row(agent)?;
        self.invalidate(Domain::Person)?;
        self.scopes.insert(agent, scope);
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
        let value = targets::nearest_gather_tile(
            &person.position,
            &resources.food_tiles,
            targets::GATHER_SCAN_RADIUS,
            scope.local_food,
            &scope.avoid_hearths,
        )
        .is_some();
        self.gather_cache.borrow_mut().insert(key, (version, value));
        Ok(value)
    }
    /// Read the required per-person targeting scope, rejecting an unprepared scope.
    pub fn target_scope(&self, agent: AgentId) -> Result<&TargetScope, ContextError> {
        self.scopes
            .get(&agent)
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
    use crate::runtime::agent_actions::memory_bias::{
        memory_weight_delta, top_contributor_entry, CascadeArm,
    };
    let arms = [
        CascadeArm::Hunger,
        CascadeArm::Thirst,
        CascadeArm::Fatigue,
        CascadeArm::Construction,
        CascadeArm::Social,
        CascadeArm::Combat,
    ];
    (
        arms.map(|arm| memory_weight_delta(memory, arm, tick)),
        arms.map(|arm| top_contributor_entry(memory, arm, tick)),
    )
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

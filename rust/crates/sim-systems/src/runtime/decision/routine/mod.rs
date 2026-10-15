mod lifecycle;
mod places;

use hecs::{Entity, World};
use places::{Occupancy, Places, KINDS};
use sim_core::components::{
    Activity, ActivityKind, ActivityRest, Agent, AgentId, AgentState, Lineage, RoutineObservation,
    RoutineOutcome, SettlementId, TargetKind, Temperament,
};
use sim_core::rng::{keyed_below, STREAM_ROUTINE};
use sim_core::rules::RoutineRule;
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const fn is_activity(state: AgentState) -> bool {
    matches!(
        state,
        AgentState::Seeking {
            target: TargetKind::Activity
        } | AgentState::Consuming {
            target: TargetKind::Activity
        }
    )
}

pub(super) const fn idle_equivalent(state: AgentState, enabled: bool) -> bool {
    matches!(state, AgentState::Idle) || (enabled && is_activity(state))
}

enum Change {
    Activity(Option<Activity>),
    Rest(Option<ActivityRest>),
    Observation(RoutineObservation),
}

pub(super) struct RoutineTick {
    rule: RoutineRule,
    tick: u64,
    seed: u64,
    replicate: u32,
    minute: Option<u64>,
    active: BTreeMap<AgentId, Activity>,
    entities: BTreeMap<AgentId, Entity>,
    keys: BTreeMap<AgentId, u64>,
    members: BTreeMap<AgentId, SettlementId>,
    rests: BTreeMap<AgentId, u64>,
    interrupted: BTreeSet<AgentId>,
    invitations: BTreeSet<AgentId>,
    places: Places,
    occupancy: Occupancy,
    pending: Vec<(AgentId, Entity, Change)>,
}

impl RoutineTick {
    pub(super) fn new(world: &mut World, resources: &SimResources, rule: RoutineRule) -> Self {
        let tick = resources.current_tick;
        let mut pending = Vec::new();
        let mut interrupted = BTreeSet::new();
        for (entity, (agent, activity, state)) in world
            .query::<(&Agent, &Activity, Option<&AgentState>)>()
            .iter()
        {
            if !state.is_some_and(|state| is_activity(*state)) {
                pending.push((agent.id, entity, Change::Activity(None)));
                pending.push((
                    agent.id,
                    entity,
                    Change::Observation(RoutineObservation {
                        tick,
                        kind: Some(activity.kind),
                        outcome: RoutineOutcome::Ended { completed: false },
                    }),
                ));
                interrupted.insert(agent.id);
            }
        }
        let mut rests = BTreeMap::new();
        for (entity, (agent, rest)) in world.query::<(&Agent, &ActivityRest)>().iter() {
            if rest.until_tick <= tick {
                pending.push((agent.id, entity, Change::Rest(None)));
            } else {
                rests.insert(agent.id, rest.until_tick);
            }
        }
        lifecycle::apply(world, pending);
        let mut active_rows: Vec<_> = world
            .query::<(&Agent, &Activity)>()
            .iter()
            .map(|(_, (agent, activity))| (agent.id, *activity))
            .collect();
        active_rows.sort_by_key(|row| row.0);
        let mut occupancy = BTreeMap::new();
        for (_, activity) in &active_rows {
            *occupancy.entry(activity.place).or_default() += 1;
        }
        let mut members = BTreeMap::new();
        let mut settlements: Vec<_> = resources.settlements.iter().collect();
        settlements.sort_by_key(|(id, _)| **id);
        for (&sid, settlement) in settlements {
            for &id in &settlement.member_agents {
                members.entry(id).or_insert(sid);
            }
        }
        Self {
            rule,
            tick,
            seed: resources.world_seed,
            replicate: resources.noise_replicate_for(tick),
            minute: (resources.ticks_per_day != 0).then(|| {
                tick.saturating_add(resources.rules.clock_offset_ticks) % resources.ticks_per_day
            }),
            active: active_rows.into_iter().collect(),
            entities: world
                .query::<&Agent>()
                .iter()
                .map(|(entity, agent)| (agent.id, entity))
                .collect(),
            keys: world
                .query::<(&Agent, &Lineage)>()
                .iter()
                .map(|(_, (agent, lineage))| (agent.id, lineage.key))
                .collect(),
            members,
            rests,
            interrupted,
            invitations: BTreeSet::new(),
            places: places::collect(resources),
            occupancy,
            pending: Vec::new(),
        }
    }

    pub(super) fn activity(&self, id: AgentId) -> Option<Activity> {
        self.active.get(&id).copied()
    }

    pub(super) fn can_enter(&self, id: AgentId) -> bool {
        self.members.contains_key(&id)
            && !self.rests.contains_key(&id)
            && self.keys.get(&id).is_some_and(|key| {
                self.minute.is_some_and(|minute| {
                    minute >= u64::from(self.rule.start_minute(*key))
                        && minute < u64::from(self.rule.day_end_minute)
                })
            })
    }

    pub(super) fn start(
        &mut self,
        id: AgentId,
        entity: Entity,
        child: bool,
        elder: bool,
        temperament: Option<&Temperament>,
    ) -> bool {
        if self.interrupted.contains(&id) {
            self.rest(id, entity, 1);
            return false;
        }
        let Some((&key, &sid)) = self.keys.get(&id).zip(self.members.get(&id)) else {
            return false;
        };
        let neutral = Temperament::from_axes([500; 4]);
        let weights = self
            .rule
            .adjusted_weights(child, elder, temperament.unwrap_or(&neutral));
        let selected = self.draw(&weights, key, 0);
        let mut redrawn = false;
        let choice = selected
            .and_then(|kind| self.place(sid, kind, key).map(|place| (kind, place)))
            .or_else(|| {
                redrawn = true;
                let available = std::array::from_fn(|index| {
                    if self.place(sid, KINDS[index], key).is_some() {
                        weights[index]
                    } else {
                        0
                    }
                });
                self.draw(&available, key, 1)
                    .and_then(|kind| self.place(sid, kind, key).map(|place| (kind, place)))
            });
        let Some((kind, place)) = choice else {
            self.rest(id, entity, 30);
            self.observe(id, entity, selected, RoutineOutcome::NoPlace { redrawn });
            return false;
        };
        let activity = Activity {
            kind,
            settlement: sid,
            place,
            chosen_tick: self.tick,
            arrived_tick: None,
            until_tick: None,
        };
        *self.occupancy.entry(place).or_default() += 1;
        self.active.insert(id, activity);
        self.pending
            .push((id, entity, Change::Activity(Some(activity))));
        self.observe(id, entity, Some(kind), RoutineOutcome::Started { redrawn });
        true
    }

    fn draw(&self, weights: &[u32; 7], key: u64, slot: u32) -> Option<ActivityKind> {
        let total: u64 = weights.iter().map(|weight| u64::from(*weight)).sum();
        if total == 0 {
            return None;
        }
        let mut draw = keyed_below(
            self.seed,
            STREAM_ROUTINE,
            key,
            self.tick,
            slot,
            self.replicate,
            total,
        );
        for (kind, weight) in KINDS.into_iter().zip(weights) {
            if draw < u64::from(*weight) {
                return Some(kind);
            }
            draw -= u64::from(*weight);
        }
        None
    }

    fn place(&self, sid: SettlementId, kind: ActivityKind, key: u64) -> Option<(u32, u32)> {
        places::choose(
            &self.places.get(&sid)?[kind.index()],
            &self.occupancy,
            self.seed,
            key,
            self.tick,
            self.replicate,
        )
    }
}

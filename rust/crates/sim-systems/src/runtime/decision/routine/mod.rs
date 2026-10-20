mod lifecycle;
mod places;

pub(in crate::runtime::decision) use places::PlacesCache;

use hecs::{Entity, World};
use places::{Cursor, Occupancy, Tile, KINDS};
use sim_core::components::{
    Activity, ActivityKind, ActivityRest, Agent, AgentId, AgentState, RoutineObservation,
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

#[derive(Clone, Copy)]
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
    /// Pass 18: every agent's entity, built on the first invitation of a
    /// tick (agents are neither spawned nor despawned during the decision
    /// system). Lineage keys and settlement membership come from the caller.
    entities: Option<BTreeMap<AgentId, Entity>>,
    rests: BTreeMap<AgentId, u64>,
    interrupted: BTreeSet<AgentId>,
    invitations: BTreeSet<AgentId>,
    /// Pass 18: the cross-tick place cache, checked against its inputs on the
    /// first `place()` of a tick (`checked`); none at night.
    places: PlacesCache,
    checked: bool,
    /// Pass 18: this tick's scan positions by (settlement, kind index); a
    /// place depends only on the candidates and `occupancy` (see [`Cursor`]).
    cursors: BTreeMap<(SettlementId, usize), Cursor>,
    occupancy: Occupancy,
    pending: Vec<(AgentId, Entity, Change)>,
}

/// Each settlement member's settlement (settlement id order; the first
/// settlement listing a person wins). The decision system's `member_of` is
/// this same map whenever the firewood or forage rule built it.
pub(super) fn members_of(resources: &SimResources) -> BTreeMap<AgentId, SettlementId> {
    let mut members = BTreeMap::new();
    let mut settlements: Vec<_> = resources.settlements.iter().collect();
    settlements.sort_by_key(|(id, _)| **id);
    for (&sid, settlement) in settlements {
        for &id in &settlement.member_agents {
            members.entry(id).or_insert(sid);
        }
    }
    members
}

impl RoutineTick {
    pub(super) fn new(
        world: &mut World,
        resources: &SimResources,
        rule: RoutineRule,
        places: PlacesCache,
    ) -> Self {
        let tick = resources.current_tick;
        #[cfg(feature = "subprofile")]
        let timer = crate::subprofile::Timer::start();
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
        #[cfg(feature = "subprofile")]
        {
            timer.stop("routine_new_cleanup");
            crate::subprofile::flush_moves("routine_new_moves");
        }
        #[cfg(feature = "subprofile")]
        let timer = crate::subprofile::Timer::start();
        let mut active_rows: Vec<_> = world
            .query::<(&Agent, &Activity)>()
            .iter()
            .map(|(_, (agent, activity))| (agent.id, *activity))
            .collect();
        active_rows.sort_by_key(|row| row.0);
        // Pass 18: the same per-tile counts, built from sorted places in one
        // pass instead of one map insertion per activity.
        let mut held: Vec<Tile> = active_rows
            .iter()
            .map(|(_, activity)| activity.place)
            .collect();
        held.sort_unstable();
        let mut counts: Vec<(Tile, u32)> = Vec::new();
        for place in held {
            match counts.last_mut() {
                Some((last, count)) if *last == place => *count += 1,
                _ => counts.push((place, 1)),
            }
        }
        let occupancy: Occupancy = counts.into_iter().collect();
        #[cfg(feature = "subprofile")]
        timer.stop("routine_new_maps");
        Self {
            rule,
            tick,
            seed: resources.world_seed,
            replicate: resources.noise_replicate_for(tick),
            minute: (resources.ticks_per_day != 0).then(|| {
                tick.saturating_add(resources.rules.clock_offset_ticks) % resources.ticks_per_day
            }),
            active: active_rows.into_iter().collect(),
            entities: None,
            rests,
            interrupted,
            invitations: BTreeSet::new(),
            places,
            checked: false,
            cursors: BTreeMap::new(),
            occupancy,
            pending: Vec::new(),
        }
    }

    pub(super) fn activity(&self, id: AgentId) -> Option<Activity> {
        self.active.get(&id).copied()
    }

    /// `key` is the person's lineage key, `member` its settlement (see
    /// [`members_of`]). Pass 19: the window ends at the person's own
    /// `RoutineRule::end_minute(key, temperament)` (`day_end_minute` without
    /// an end stagger).
    pub(super) fn can_enter(
        &self,
        id: AgentId,
        key: Option<u64>,
        member: Option<SettlementId>,
        temperament: Option<&Temperament>,
    ) -> bool {
        member.is_some()
            && !self.rests.contains_key(&id)
            && key.is_some_and(|key| {
                self.minute.is_some_and(|minute| {
                    minute >= u64::from(self.rule.start_minute(key))
                        && minute < u64::from(self.rule.end_minute(key, temperament))
                })
            })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the caller's row fields, passed instead of per-tick maps"
    )]
    pub(super) fn start(
        &mut self,
        id: AgentId,
        entity: Entity,
        key: Option<u64>,
        member: Option<SettlementId>,
        child: bool,
        elder: bool,
        temperament: Option<&Temperament>,
        resources: &SimResources,
    ) -> bool {
        if self.interrupted.contains(&id) {
            #[cfg(feature = "subprofile")]
            crate::subprofile::count("start_interrupted", 1);
            self.rest(id, entity, 1);
            return false;
        }
        let Some((key, sid)) = key.zip(member) else {
            return false;
        };
        let neutral = Temperament::from_axes([500; 4]);
        let weights = self
            .rule
            .adjusted_weights(child, elder, temperament.unwrap_or(&neutral));
        let selected = self.draw(&weights, key, 0);
        let mut redrawn = false;
        let choice = selected
            .and_then(|kind| self.place(sid, kind, resources).map(|place| (kind, place)))
            .or_else(|| {
                redrawn = true;
                let available = std::array::from_fn(|index| {
                    if self.place(sid, KINDS[index], resources).is_some() {
                        weights[index]
                    } else {
                        0
                    }
                });
                self.draw(&available, key, 1)
                    .and_then(|kind| self.place(sid, kind, resources).map(|place| (kind, place)))
            });
        #[cfg(feature = "subprofile")]
        crate::subprofile::count(
            match (&choice, redrawn) {
                (None, _) => "start_noplace",
                (Some(_), true) => "start_redrawn",
                (Some(_), false) => "start_first_draw",
            },
            1,
        );
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

    /// The free place of `kind` for settlement `sid`: Pass 17's choice
    /// over its candidates, resumed from this tick's [`Cursor`].
    fn place(
        &mut self,
        sid: SettlementId,
        kind: ActivityKind,
        resources: &SimResources,
    ) -> Option<Tile> {
        #[cfg(feature = "subprofile")]
        crate::subprofile::count("place_calls", 1);
        if !self.checked {
            self.places.refresh(resources);
            self.checked = true;
        }
        let lists = self.places.places().get(&sid)?;
        self.cursors
            .entry((sid, kind.index()))
            .or_default()
            .choose(&lists[kind.index()], &self.occupancy)
    }

    /// `tile`'s occupancy fell: every cursor whose list holds it rescans from it.
    fn rewind(&mut self, tile: Tile) {
        let places = self.places.places();
        for ((sid, index), cursor) in &mut self.cursors {
            if let Some(lists) = places.get(sid) {
                cursor.rewind(&lists[*index], tile);
            }
        }
    }
}

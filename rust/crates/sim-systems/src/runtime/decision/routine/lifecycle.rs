use super::{is_activity, Change, RoutineTick};
use hecs::{Entity, World};
use sim_core::components::{
    ActivityRest, AgentId, AgentState, RoutineObservation, RoutineObservationBatch, RoutineOutcome,
    Temperament,
};

pub(super) fn apply(world: &mut World, mut pending: Vec<(AgentId, Entity, Change)>) {
    pending.sort_by_key(|(id, _, _)| *id);
    let mut batches = Vec::new();
    let mut current: Option<(AgentId, Entity, Vec<RoutineObservation>)> = None;
    for (id, entity, change) in &pending {
        if let Change::Observation(observation) = change {
            if current
                .as_ref()
                .is_some_and(|(previous, _, _)| previous != id)
            {
                if let Some(batch) = current.take() {
                    batches.push(batch);
                }
            }
            current
                .get_or_insert_with(|| (*id, *entity, Vec::new()))
                .2
                .push(*observation);
        }
    }
    if let Some(batch) = current {
        batches.push(batch);
    }
    for (_, entity, change) in pending {
        match change {
            Change::Activity(Some(activity)) => {
                #[cfg(feature = "subprofile")]
                note_insert::<sim_core::components::Activity>(world, entity);
                let _ = world.insert_one(entity, activity);
            }
            Change::Activity(None) => {
                let removed = world.remove_one::<sim_core::components::Activity>(entity);
                #[cfg(feature = "subprofile")]
                crate::subprofile::note_move(removed.is_ok());
                let _ = removed;
            }
            Change::Rest(Some(rest)) => {
                #[cfg(feature = "subprofile")]
                note_insert::<ActivityRest>(world, entity);
                let _ = world.insert_one(entity, rest);
            }
            Change::Rest(None) => {
                let removed = world.remove_one::<ActivityRest>(entity);
                #[cfg(feature = "subprofile")]
                crate::subprofile::note_move(removed.is_ok());
                let _ = removed;
            }
            Change::Observation(observation) => {
                #[cfg(feature = "subprofile")]
                note_insert::<RoutineObservation>(world, entity);
                let _ = world.insert_one(entity, observation);
            }
        }
    }
    for (_, entity, events) in batches {
        if events.len() > 1 {
            let tick = events[0].tick;
            #[cfg(feature = "subprofile")]
            note_insert::<RoutineObservationBatch>(world, entity);
            let _ = world.insert_one(entity, RoutineObservationBatch { tick, events });
        }
    }
}

/// Pass 18 (`subprofile` only): an insert moves the entity to another
/// archetype only when it lacks the component (a replacement does not).
#[cfg(feature = "subprofile")]
fn note_insert<T: hecs::Component>(world: &World, entity: Entity) {
    crate::subprofile::note_move(world.contains(entity) && world.get::<&T>(entity).is_err());
}

impl RoutineTick {
    pub(in crate::runtime::decision) fn finish(self, world: &mut World) {
        scope!("routine_finish", { apply(world, self.pending) });
        #[cfg(feature = "subprofile")]
        crate::subprofile::flush_moves("routine_finish_moves");
    }

    pub(in crate::runtime::decision) fn interrupt(&mut self, id: AgentId, entity: Entity) {
        self.end(id, entity, false, false);
        self.interrupted.insert(id);
    }

    pub(in crate::runtime::decision) fn invite(&mut self, id: AgentId) {
        if !self.active.contains_key(&id) {
            return;
        }
        if let Some(entity) = self.entities.get(&id).copied() {
            self.interrupt(id, entity);
            self.invitations.insert(id);
        }
    }

    pub(in crate::runtime::decision) fn take_invitation(&mut self, id: AgentId) -> bool {
        self.invitations.remove(&id)
    }

    pub(in crate::runtime::decision) fn replay(
        &mut self,
        world: &mut World,
    ) -> Vec<(AgentId, Entity)> {
        let mut replay = Vec::new();
        for id in std::mem::take(&mut self.invitations) {
            let Some(entity) = self.entities.get(&id).copied() else {
                continue;
            };
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                if is_activity(*state) {
                    *state = AgentState::Idle;
                    replay.push((id, entity));
                }
            }
        }
        replay
    }

    pub(in crate::runtime::decision) fn deadline(
        &self,
        id: AgentId,
        walking: bool,
    ) -> Option<bool> {
        let activity = self.active.get(&id)?;
        if self.members.get(&id) != Some(&activity.settlement) {
            return Some(false);
        }
        if activity.until_tick.is_some_and(|until| self.tick >= until)
            || self
                .minute
                .is_some_and(|minute| minute >= u64::from(self.rule.day_end_minute))
        {
            return Some(activity.arrived_tick.is_some());
        }
        if walking
            && self.tick.saturating_sub(activity.chosen_tick)
                >= u64::from(self.rule.walk_timeout_minutes)
        {
            return Some(false);
        }
        None
    }

    pub(in crate::runtime::decision) fn end(
        &mut self,
        id: AgentId,
        entity: Entity,
        completed: bool,
        resting: bool,
    ) {
        let Some(activity) = self.active.remove(&id) else {
            return;
        };
        if let Some(count) = self.occupancy.get_mut(&activity.place) {
            *count = count.saturating_sub(1);
        }
        self.pending.push((id, entity, Change::Activity(None)));
        self.observe(
            id,
            entity,
            Some(activity.kind),
            RoutineOutcome::Ended { completed },
        );
        if resting {
            let key = self.keys.get(&id).copied().unwrap_or(id);
            let duration = self
                .rule
                .rest_minutes(self.seed, key, self.tick, self.replicate);
            self.rest(id, entity, u64::from(duration));
        }
    }

    pub(in crate::runtime::decision) fn arrive(
        &mut self,
        id: AgentId,
        entity: Entity,
        temperament: Option<&Temperament>,
    ) {
        if let Some(activity) = self.active.get_mut(&id) {
            let neutral = Temperament::from_axes([500; 4]);
            activity.arrived_tick = Some(self.tick);
            activity.until_tick = Some(self.tick.saturating_add(u64::from(
                self.rule.duration_minutes(temperament.unwrap_or(&neutral)),
            )));
            self.pending
                .push((id, entity, Change::Activity(Some(*activity))));
        }
    }

    pub(super) fn rest(&mut self, id: AgentId, entity: Entity, duration: u64) {
        let until_tick = self.tick.saturating_add(duration);
        self.rests.insert(id, until_tick);
        self.pending
            .push((id, entity, Change::Rest(Some(ActivityRest { until_tick }))));
    }

    pub(super) fn observe(
        &mut self,
        id: AgentId,
        entity: Entity,
        kind: Option<sim_core::components::ActivityKind>,
        outcome: RoutineOutcome,
    ) {
        self.pending.push((
            id,
            entity,
            Change::Observation(RoutineObservation {
                tick: self.tick,
                kind,
                outcome,
            }),
        ));
    }
}

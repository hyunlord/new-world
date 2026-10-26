//! Additive lifecycle entry points; legacy lottery and start remain unchanged.
use super::*;

enum Admission {
    Catalogue,
    ReachableHearth,
}
impl RoutineTick {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime) fn start_selected_kind(
        &mut self,
        id: AgentId,
        entity: Entity,
        key: Option<u64>,
        member: Option<SettlementId>,
        kind: ActivityKind,
        place: Tile,
        resources: &SimResources,
    ) -> bool {
        self.start_selected(
            id,
            entity,
            key,
            member,
            kind,
            place,
            resources,
            Admission::Catalogue,
        )
    }
    /// The caller's registered target provider certifies a reachable lit hearth.
    pub(in crate::runtime) fn start_selected_hearth(
        &mut self,
        id: AgentId,
        entity: Entity,
        key: Option<u64>,
        member: Option<SettlementId>,
        place: Tile,
        resources: &SimResources,
    ) -> bool {
        self.start_selected(
            id,
            entity,
            key,
            member,
            ActivityKind::SitByFire,
            place,
            resources,
            Admission::ReachableHearth,
        )
    }

    /// Record a data-selected failed entry; its diagnostic kind does not select an action.
    pub(in crate::runtime) fn deny_selected_node(
        &mut self,
        id: AgentId,
        entity: Entity,
        child: bool,
        elder: bool,
        temperament: Option<&Temperament>,
        node_random_key: u64,
    ) {
        if self.interrupted.contains(&id) {
            self.rest(id, entity, 1);
            return;
        }
        let neutral = Temperament::from_axes([500; 4]);
        let weights = self
            .rule
            .adjusted_weights(child, elder, temperament.unwrap_or(&neutral));
        let total: u64 = weights.iter().map(|weight| u64::from(*weight)).sum();
        let selected = if total == 0 {
            None
        } else {
            let mut draw = keyed_below(
                self.seed,
                STREAM_ROUTINE ^ node_random_key,
                id,
                self.tick,
                0,
                self.replicate,
                total,
            );
            KINDS.iter().zip(weights).find_map(|(kind, weight)| {
                if draw < u64::from(weight) {
                    Some(*kind)
                } else {
                    draw -= u64::from(weight);
                    None
                }
            })
        };
        self.rest(id, entity, 30);
        self.observe(
            id,
            entity,
            selected,
            RoutineOutcome::NoPlace { redrawn: true },
        );
    }

    /// End a mind-owned activity with a rest draw keyed by person, tick and node.
    /// Legacy `end` retains its original lineage-keyed draw.
    pub(in crate::runtime) fn end_selected_node(
        &mut self,
        id: AgentId,
        entity: Entity,
        completed: bool,
        node_random_key: u64,
    ) {
        if !self.active.contains_key(&id) {
            return;
        }
        self.end(id, entity, completed, None);
        let span = u64::from(
            self.rule
                .rest_minutes_max
                .saturating_sub(self.rule.rest_minutes_min),
        ) + 1;
        let duration = u64::from(self.rule.rest_minutes_min)
            + keyed_below(
                self.seed,
                STREAM_ROUTINE ^ node_random_key,
                id,
                self.tick,
                2,
                self.replicate,
                span,
            );
        self.rest(id, entity, duration);
    }

    /// Commit an explicitly selected activity and place without running the legacy lottery.
    /// Selection owns node-keyed randomness; arrival retains the existing duration effect.
    #[allow(clippy::too_many_arguments)]
    fn start_selected(
        &mut self,
        id: AgentId,
        entity: Entity,
        key: Option<u64>,
        member: Option<SettlementId>,
        kind: ActivityKind,
        place: Tile,
        resources: &SimResources,
        admission: Admission,
    ) -> bool {
        if self.interrupted.contains(&id) {
            self.rest(id, entity, 1);
            return false;
        }
        let Some((_, sid)) = key.zip(member) else {
            return false;
        };
        if !self.checked {
            self.places.refresh(resources);
            self.checked = true;
        }
        let legal = match admission {
            Admission::Catalogue => self
                .places
                .places()
                .get(&sid)
                .is_some_and(|lists| lists[kind.index()].contains(&place)),
            Admission::ReachableHearth => {
                place.0 < resources.tile_grid.width && place.1 < resources.tile_grid.height
            }
        } && self.occupancy.get(&place).copied().unwrap_or(0) < 2;
        if !legal {
            self.rest(id, entity, 30);
            self.observe(
                id,
                entity,
                Some(kind),
                RoutineOutcome::NoPlace { redrawn: false },
            );
            return false;
        }
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
        self.observe(
            id,
            entity,
            Some(kind),
            RoutineOutcome::Started { redrawn: false },
        );
        true
    }
}

#[cfg(test)]
mod denial_tests {
    use super::*;

    #[test]
    fn failed_entry_with_nonzero_weights_records_diagnostic_and_thirty_ticks() {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, AgentState::Idle));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.current_tick = 600;
        resources.rules = sim_core::rules::WorldRules::colony(7);
        let mut rule = resources.rules.agent.routine.expect("routine");
        rule.adult_weights = [1, 0, 0, 0, 0];
        rule.day_start_minute = 0;
        rule.day_end_minute = 1440;
        rule.end_stagger = None;
        let mut tick = RoutineTick::new(&mut world, &resources, rule, PlacesCache::default());
        tick.prepare_preview_places(&resources);
        assert!(tick.can_enter(7, Some(7), Some(1), None));
        assert!(!tick.weighted_place_available(7, Some(7), Some(1), false, false, None));
        tick.deny_selected_node(7, entity, false, false, None, 91);
        tick.finish(&mut world);
        assert_eq!(
            world.get::<&ActivityRest>(entity).expect("rest").until_tick,
            630
        );
        let event = *world
            .get::<&RoutineObservation>(entity)
            .expect("observation");
        assert_eq!(event.kind, Some(ActivityKind::TendFire));
        assert_eq!(event.outcome, RoutineOutcome::NoPlace { redrawn: true });
        assert!(world.get::<&Activity>(entity).is_err());
    }

    #[test]
    fn interrupted_failed_entry_waits_one_tick_without_a_no_place_event() {
        let mut world = World::new();
        let entity = world.spawn((Agent { id: 7 }, AgentState::Idle));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.current_tick = 600;
        let rule = sim_core::rules::WorldRules::colony(7)
            .agent
            .routine
            .expect("routine");
        let mut tick = RoutineTick::new(&mut world, &resources, rule, PlacesCache::default());
        tick.interrupted.insert(7);
        tick.deny_selected_node(7, entity, false, false, None, 91);
        tick.finish(&mut world);
        assert_eq!(
            world.get::<&ActivityRest>(entity).expect("rest").until_tick,
            601
        );
        assert!(world.get::<&RoutineObservation>(entity).is_err());
        assert!(world.get::<&Activity>(entity).is_err());
    }
}

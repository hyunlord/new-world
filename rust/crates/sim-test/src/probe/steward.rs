//! Read-only A8 measurements, emitted only for explicit steward probe runs.
use serde_json::{json, Value};
use sim_core::{
    colony::{story::PressureKind, Command},
    components::{Agent, AgentId, AgentState, Hunger, Position, Thirst},
};
use sim_engine::SimEngine;
use std::collections::BTreeMap;

/// Observer independent of the assistant's decision data and provenance.
#[derive(Default)]
pub struct StewardWatch {
    stockpile_tick: Option<u64>,
    all_beds_tick: Option<u64>,
    winter_started: bool,
    winter_finished: bool,
    cold_before: u64,
    previous_cold: u64,
    cold_first_winter: u64,
    snap_ticks: u64,
    snap_zero_ticks: u64,
    before_threat: BTreeMap<AgentId, Value>,
    losses_during_threat: Vec<Value>,
    flight_origins: BTreeMap<AgentId, (u64, Value)>,
    first_roof_layout: Option<Value>,
}
impl StewardWatch {
    /// Capture pre-tick context only while an actual hunting threat exists.
    pub fn before_tick(&mut self, engine: &SimEngine) {
        self.before_threat.clear();
        let res = &engine.resources;
        let threats = sim_systems::runtime::wildlife::flee::threats(res);
        if threats.is_empty() {
            return;
        }
        for (_, (agent, pos, state, job, hunger, thirst, fleeing)) in engine
            .world
            .query::<(
                &Agent,
                &Position,
                Option<&AgentState>,
                Option<&sim_core::colony::Job>,
                Option<&Hunger>,
                Option<&Thirst>,
                Option<&sim_core::colony::people::Fleeing>,
            )>()
            .iter()
        {
            let tile = (pos.x, pos.y);
            let distance =
                |other: (u32, u32)| tile.0.abs_diff(other.0).max(tile.1.abs_diff(other.1));
            let region = res.nav.region_of(tile);
            let roof = res
                .colony
                .rooms
                .values()
                .filter(|room| room.roofed)
                .flat_map(|room| room.tiles.iter().copied())
                .filter(|&other| res.nav.reaches(region, other))
                .min_by_key(|&other| (distance(other), other.1, other.0));
            self.before_threat.insert(agent.id, json!({"tick":engine.current_tick(), "agent":agent.id,
                "tile":tile,"state":state,"job":job,"hunger":hunger.map(|h|h.value),
                "thirst":thirst.map(|t|t.value),"fleeing":fleeing,
                "roofed":sim_systems::runtime::rooms::roofed(res,tile),
                "nearest_reachable_roof":roof,"roof_distance":roof.map(distance),"threats":threats}));
        }
    }
    /// Observe physical completion and resources after every simulation tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let res = &engine.resources;
        let tick = engine.current_tick();
        if res
            .colony
            .zones
            .values()
            .any(|zone| zone.kind == sim_core::colony::ZoneKind::Stockpile)
        {
            self.stockpile_tick.get_or_insert(tick);
        }
        let live: std::collections::BTreeSet<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
        for (id, context) in &self.before_threat {
            if !live.contains(id) {
                let death = res.chronicle.entries().iter().rev().find(|entry| {
                    entry.kind == sim_core::chronicle::ChronicleKind::Died
                        && entry.subject == Some(*id)
                });
                self.losses_during_threat
                    .push(json!({"observed_after_tick":tick,"death":death,"before":context,"flight_entry":self.flight_origins.get(id).map(|(_,context)|context)}));
            }
        }
        let flights: BTreeMap<_, _> = engine
            .world
            .query::<(&Agent, &sim_core::colony::people::Fleeing)>()
            .iter()
            .map(|(_, (agent, flight))| (agent.id, flight.since))
            .collect();
        self.flight_origins.retain(|id, _| flights.contains_key(id));
        for (id, since) in flights {
            if self
                .flight_origins
                .get(&id)
                .is_none_or(|(old, _)| *old != since)
            {
                if let Some(context) = self.before_threat.get(&id) {
                    self.flight_origins.insert(id, (since, context.clone()));
                }
            }
        }
        self.before_threat.clear();
        if self.first_roof_layout.is_none() && res.colony.rooms.values().any(|room| room.roofed) {
            self.first_roof_layout = Some(layout(engine));
        }
        let people = live.len();
        let beds = res
            .colony
            .beds
            .keys()
            .filter(|t| {
                res.colony
                    .room_of_tile
                    .get(t)
                    .and_then(|id| res.colony.rooms.get(id))
                    .is_some_and(|room| room.roofed && !room.dug && !room.doors.is_empty())
            })
            .count();
        if people > 0 && beds >= people {
            self.all_beds_tick.get_or_insert(tick);
        }
        let winter = sim_systems::runtime::resource_regen::season_at(res)
            .is_some_and(|(season, _)| season == 3);
        if winter && !self.winter_started {
            self.winter_started = true;
            self.cold_before = self.previous_cold;
        }
        if self.winter_started && !self.winter_finished {
            if winter {
                self.cold_first_winter =
                    res.tallies.deaths_exposure.saturating_sub(self.cold_before);
            }
            if !winter
                || sim_systems::runtime::resource_regen::season_at_tick(
                    res,
                    res.current_tick.saturating_add(1),
                )
                .is_some_and(|(season, _)| season != 3)
            {
                self.winter_finished = true;
            }
        }
        self.previous_cold = res.tallies.deaths_exposure;
        if res.colony.story.active_of(PressureKind::ColdSnap).is_some() {
            self.snap_ticks += 1;
            if sim_systems::runtime::story::firewood_have(res) == 0 {
                self.snap_zero_ticks += 1;
            }
        }
    }
    /// Measurements and coverage, never a substituted pass/fail threshold.
    pub fn report(&self, engine: &SimEngine) -> Value {
        let mut actions: BTreeMap<u16, u64> = BTreeMap::new();
        for record in &engine.resources.colony.command_log {
            if let Command::Steward { rule, .. } = record.command {
                if record.result.accepted {
                    *actions.entry(rule).or_default() += 1;
                }
            }
        }
        let alive = engine.world.query::<&Agent>().iter().count();
        json!({"ticks_per_day":engine.resources.ticks_per_day,"stockpile_first_tick":self.stockpile_tick,"walled_beds_for_all_first_tick":self.all_beds_tick,
            "first_winter_observed":self.winter_started,"first_winter_complete":self.winter_finished,
            "first_winter_cold_deaths":self.cold_first_winter,"cold_snap_observed_ticks":self.snap_ticks,
            "cold_snap_firewood_zero_ticks":self.snap_zero_ticks,"alive_end":alive,
            "alive_at_day48":if engine.current_tick()>=48*engine.resources.ticks_per_day {Some(alive>0)}else{None},
            "accepted_actions_by_rule":actions,
            "diagnostics":{"first_roof_layout":self.first_roof_layout,"final_layout":layout(engine),
                "losses_during_threat":self.losses_during_threat}})
    }
}

fn layout(engine: &SimEngine) -> Value {
    let res = &engine.resources;
    let hearths = sim_systems::runtime::jobs::hearth::hearth_tiles(res);
    let workshops: Vec<_> = res.colony.workshops.iter().map(|(id,shop)| {
        let work = shop.work_tile();
        let usable = res.colony.zone_of_tile.contains_key(&work) || hearths.iter().any(|hearth|
            work.0.abs_diff(hearth.0).max(work.1.abs_diff(hearth.1)) <= sim_systems::runtime::jobs::hearth::HEARTH_PILE_REACH);
        json!({"id":id,"kind":shop.kind,"work_tile":work,"work_roofed":sim_systems::runtime::rooms::roofed(res,work),"work_usable_for_fire":usable})
    }).collect();
    let orders: Vec<_> = res
        .colony
        .command_log
        .iter()
        .filter(|record| {
            record.result.accepted && matches!(record.command, Command::Steward { rule: 3, .. })
        })
        .collect();
    json!({"tick":engine.current_tick(),"hearths":hearths,"rooms":res.colony.rooms,
        "stock_tiles":res.colony.zone_of_tile.keys().collect::<Vec<_>>(),"workshops":workshops,"house_orders":orders})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_are_read_only_and_capture_pre_loss_threat_context() {
        let mut engine = crate::colony_engine(29);
        engine.tick();
        let animal = engine
            .resources
            .colony
            .wildlife
            .animals
            .values_mut()
            .next()
            .expect("animal");
        animal.kind = sim_core::colony::wildlife::AnimalKind::Wolf;
        animal.state = sim_core::colony::wildlife::AnimalState::Hunt;
        let before = crate::probe::fingerprint(&engine);
        let mut watch = StewardWatch::default();
        watch.before_tick(&engine);
        assert!(!watch.before_threat.is_empty());
        assert_eq!(before, crate::probe::fingerprint(&engine));
        let entity = engine
            .world
            .query::<&Agent>()
            .iter()
            .next()
            .map(|(entity, _)| entity)
            .expect("person");
        engine
            .world
            .insert_one(
                entity,
                sim_core::colony::people::Fleeing {
                    until: 100,
                    from: 1,
                    to: (1, 1),
                    since: engine.current_tick(),
                },
            )
            .expect("flight fixture");
        watch.after_tick(&engine);
        watch.before_tick(&engine);
        engine.world.despawn(entity).expect("loss fixture");
        let after_loss = crate::probe::fingerprint(&engine);
        watch.after_tick(&engine);
        assert_eq!(watch.losses_during_threat.len(), 1);
        assert!(watch.losses_during_threat[0]["flight_entry"].is_object());
        assert!(watch.losses_during_threat[0]["before"]["tile"].is_array());
        assert!(watch.report(&engine)["diagnostics"]["final_layout"]["stock_tiles"].is_array());
        assert_eq!(after_loss, crate::probe::fingerprint(&engine));
    }
    #[test]
    fn winter_counts_both_boundary_ticks_and_excludes_spring_and_second_year() {
        let mut engine = crate::colony_engine(29);
        let mut watch = StewardWatch::default();
        let res = &mut engine.resources;
        res.rules.clock_offset_ticks = 0;
        let season = u64::from(res.rules.season_days()) * res.ticks_per_day;
        engine.resources.current_tick = 3 * season - 1;
        engine.resources.tallies.deaths_exposure = 2;
        watch.after_tick(&engine);
        engine.resources.current_tick = 3 * season;
        engine.resources.tallies.deaths_exposure = 3;
        watch.after_tick(&engine);
        assert_eq!(watch.cold_first_winter, 1);
        assert!(!watch.winter_finished);
        engine.resources.current_tick = 4 * season - 1;
        engine.resources.tallies.deaths_exposure = 4;
        watch.after_tick(&engine);
        assert_eq!(watch.cold_first_winter, 2);
        assert!(watch.winter_finished);
        engine.resources.current_tick = 4 * season;
        engine.resources.tallies.deaths_exposure = 5;
        watch.after_tick(&engine);
        engine.resources.current_tick = 7 * season;
        engine.resources.tallies.deaths_exposure = 6;
        watch.after_tick(&engine);
        assert_eq!(watch.cold_first_winter, 2);
        assert_eq!(
            watch.report(&engine)["ticks_per_day"],
            engine.resources.ticks_per_day
        );
    }
}

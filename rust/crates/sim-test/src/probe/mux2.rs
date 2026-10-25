//! Opt-in M-UX2 physical housing and exact fatal-predator observations; never writes simulation.
use serde_json::{json, Value};
use sim_core::colony::content::AnimalMechanic;
use sim_core::components::{Agent, AgentId};
use sim_core::rooms::Room;
use sim_engine::{SimEngine, SimResources};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "mux2/tests.rs"]
mod tests;

/// A detected room qualifies only with a physically built cardinal perimeter.
fn walled_house(res: &SimResources, room: &Room) -> bool {
    if !room.roofed || room.dug || room.tiles.is_empty() || room.doors.is_empty() {
        return false;
    }
    let grid = &res.tile_grid;
    let mut actual_door = false;
    for &(x, y) in &room.tiles {
        if x == 0 || y == 0 || x + 1 >= grid.width || y + 1 >= grid.height {
            return false;
        }
        let i = grid.idx(x, y);
        if grid.wall_material[i].is_some() || grid.is_dug(i) {
            return false;
        }
        for tile in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if room.tiles.contains(&tile) {
                continue;
            }
            let boundary = grid.idx(tile.0, tile.1);
            if grid.wall_material[boundary].is_none() {
                return false;
            }
            actual_door |= grid.is_door(boundary);
        }
    }
    actual_door
}

/// Independent post-tick measurements and coverage, with no policy access.
#[derive(Default)]
pub struct Mux2Watch {
    observed_ticks: u64,
    walled_beds_tick: Option<u64>,
    wolf_deaths: Vec<Value>,
    seen_victims: BTreeSet<AgentId>,
    errors: BTreeSet<String>,
}
impl Mux2Watch {
    /// Read completion and resolver-emitted fatal-hit provenance once per completed tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let res = &engine.resources;
        if res.current_tick != self.observed_ticks
            || engine.current_tick() != self.observed_ticks + 1
        {
            self.errors
                .insert("noncontiguous_or_repeated_tick_observation".into());
        }
        self.observed_ticks += 1;
        let live: BTreeSet<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
        let beds: BTreeSet<_> = res
            .colony
            .rooms
            .values()
            .filter(|room| walled_house(res, room))
            .flat_map(|room| room.tiles.iter().copied())
            .filter(|tile| res.colony.beds.contains_key(tile))
            .collect();
        if !live.is_empty() && beds.len() >= live.len() {
            self.walled_beds_tick.get_or_insert(engine.current_tick());
        }
        for event in &res.fatal_animal_attacks {
            if event.tick != res.current_tick
                || live.contains(&event.victim)
                || !self.seen_victims.insert(event.victim)
            {
                self.errors.insert("invalid_fatal_animal_provenance".into());
                continue;
            }
            if event.kind.mechanic() == AnimalMechanic::Predator {
                self.wolf_deaths
                    .push(json!({"tick":event.tick,"victim":event.victim,
                    "attacker":event.attacker,"kind":event.kind.code(),"mechanic":"predator"}));
            }
        }
    }
    /// Versioned metrics; caller separately supplies validated run-input metadata.
    pub fn report(&self, engine: &SimEngine) -> Value {
        json!({"schema_version":1,"observed_ticks":self.observed_ticks,
            "start_tick":0,"end_tick":engine.current_tick(),
            "walled_beds_for_all_first_tick":self.walled_beds_tick,
            "wolf_deaths":self.wolf_deaths.len(),"wolf_death_events":self.wolf_deaths,
            "fatal_animal_events":self.seen_victims.len(),
            "combat_deaths":engine.resources.tallies.deaths_combat,
            "observer_integrity_errors":self.errors})
    }
}

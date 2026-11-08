//! Habitat-gated data populations within AnimalSystem (priority171, interval3).
//! Reads: climate, terrain, navigation, species; writes: wildlife animals; events: none.
use super::{chebyshev, near_herd, new_animal, AnimalKind, SimResources, WildlifeRule};
use sim_core::colony::content::{installed_or_bundled, SpeciesOrigin};
use sim_core::rng::{keyed_below, STREAM_SPECIES_ORIGIN};

pub(super) fn admits(res: &SimResources, origin: &SpeciesOrigin, at: (u32, u32)) -> bool {
    if at.0 >= res.tile_grid.width || at.1 >= res.tile_grid.height {
        return false;
    }
    let index = res.tile_grid.idx(at.0, at.1);
    origin.admits(
        res.world_variety
            .unwrap_or_else(|| sim_core::variety::WorldVariety::draw(res.world_seed))
            .climate,
        res.tile_grid.terrain_type[index],
        res.tile_grid.elevation[index],
    )
}

pub(super) fn births(
    res: &mut SimResources,
    rule: &WildlifeRule,
    hearth: (u32, u32),
    initial: bool,
) {
    let catalog = installed_or_bundled();
    let day = crate::runtime::resource_regen::day_index(res);
    for def in catalog.animals() {
        if def.status != sim_core::colony::knowledge::ContentStatus::Connected {
            continue;
        }
        let Some(population) = &def.population else {
            continue;
        };
        let Some(kind) = catalog.animal_by_name(&def.identity.id) else {
            continue;
        };
        let draw = |slot, tick, bound| {
            keyed_below(
                res.world_seed,
                STREAM_SPECIES_ORIGIN,
                u64::from(def.identity.code),
                tick,
                slot,
                0,
                bound,
            )
        };
        let cap = population.initial.0
            + draw(
                0,
                0,
                u64::from(population.initial.1 - population.initial.0) + 1,
            ) as u32;
        let living: Vec<_> = res
            .colony
            .wildlife
            .animals
            .values()
            .filter(|a| a.kind == kind)
            .collect();
        let count = living.len() as u32;
        if count >= cap
            || (!initial && (day == 0 || !day.is_multiple_of(u64::from(population.respawn_days))))
        {
            continue;
        }
        let home = living.first().and_then(|a| a.home);
        let projected = def.resolve_rule(rule);
        let distance = def
            .grazer
            .as_ref()
            .map_or(0, |_| projected.deer_min_hearth_distance);
        let region = res.nav.region_of(hearth);
        let mut places = Vec::new();
        for y in 0..res.tile_grid.height {
            for x in 0..res.tile_grid.width {
                let at = (x, y);
                if admits(res, &def.origin, at)
                    && res.nav.passable(at)
                    && res.nav.reaches(region, at)
                    && chebyshev(at, hearth) >= distance
                    && !res.colony.trees.contains_key(&at)
                {
                    places.push(at);
                }
            }
        }
        if places.is_empty() {
            continue;
        }
        let centre = home.unwrap_or_else(|| {
            let near: Vec<_> = places
                .iter()
                .copied()
                .filter(|p| chebyshev(*p, hearth) <= rule.deer_herd_max_tiles)
                .collect();
            let pool = if near.is_empty() { &places } else { &near };
            pool[draw(1, 0, pool.len() as u64) as usize]
        });
        let places = near_herd(&places, Some(centre), population.herd_tiles);
        let births = if initial { cap - count } else { 1 };
        let chosen: Vec<_> = (0..births)
            .map(|i| places[draw(2 + i, day, places.len() as u64) as usize])
            .collect();
        for at in chosen {
            if res.colony.wildlife.next_id.max(1).checked_add(1).is_some() {
                new_animal(res, kind, def.hp.wildlife(rule), at, None);
            }
        }
    }
}

pub(super) fn resident_wander(res: &mut SimResources, kind: AnimalKind, id: u32, at: (u32, u32)) {
    let def = kind.definition();
    let mut candidates = Vec::new();
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            let (x, y) = (i64::from(at.0) + dx, i64::from(at.1) + dy);
            if x < 0
                || y < 0
                || x >= i64::from(res.tile_grid.width)
                || y >= i64::from(res.tile_grid.height)
            {
                continue;
            }
            let tile = (x as u32, y as u32);
            if res.nav.can_step(at, tile) && admits(res, &def.origin, tile) {
                candidates.push(tile);
            }
        }
    }
    if candidates.is_empty() {
        return;
    }
    let selected = keyed_below(
        res.world_seed,
        STREAM_SPECIES_ORIGIN,
        u64::from(id),
        res.current_tick,
        100,
        0,
        candidates.len() as u64,
    ) as usize;
    if let Some(animal) = res.colony.wildlife.animals.get_mut(&id) {
        animal.pos = candidates[selected];
        animal.state = sim_core::colony::wildlife::AnimalState::Hunt;
        res.colony.wildlife.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::story::{Pressure, PressureKind};
    use sim_core::colony::wildlife::{Fight, Fighter};
    use sim_core::components::{Agent, BodyHealth, Position};
    use sim_core::material::TerrainType;

    fn fixture() -> (hecs::World, SimResources, AnimalKind) {
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.world_variety = Some(sim_core::variety::WorldVariety {
            climate: 0,
            ..sim_core::variety::WorldVariety::draw(0)
        });
        res.nav.install(sim_core::nav::CostMap::new(8, 8));
        res.tile_grid.terrain_type.fill(TerrainType::Forest);
        let kind = installed_or_bundled()
            .animal_by_name("wild_boar")
            .expect("boar");
        (hecs::World::new(), res, kind)
    }

    #[test]
    fn resident_predator_chase_cannot_step_out_of_its_habitat() {
        let (_, mut res, kind) = fixture();
        super::super::new_animal(&mut res, kind, 30, (2, 2), None);
        let id = *res.colony.wildlife.animals.keys().next().expect("animal");
        assert!(admits(&res, &kind.definition().origin, (2, 2)));
        let index = res.tile_grid.idx(3, 2);
        res.tile_grid.terrain_type[index] = TerrainType::Plain;
        // A precomputed chase step must obey the same origin as idle wandering.
        let animal = res.colony.wildlife.animals.get_mut(&id).expect("animal");
        animal.goal = Some((4, 2));
        animal.path = vec![(3, 2), (4, 2)];
        super::super::wolf_walk(&mut res, id, (4, 2));
        assert_eq!(res.colony.wildlife.animals[&id].pos, (2, 2));
        assert!(res.colony.wildlife.animals[&id].path.is_empty());
    }

    #[test]
    fn resident_bite_and_kill_do_not_borrow_an_unrelated_pack_cause() {
        let (mut world, mut res, kind) = fixture();
        world.spawn((Agent { id: 7 }, Position::new(2, 3), BodyHealth::new()));
        let pressure = PressureKind::role("pack_default");
        res.colony.story.active.push(Pressure {
            kind: pressure,
            start_day: 0,
            end_day: 3,
            began_entry: None,
            deaths_at_start: 0,
            intensity: 2,
            subject: None,
            cost: Default::default(),
        });
        super::super::new_animal(&mut res, kind, 1, (2, 2), None);
        let id = *res.colony.wildlife.animals.keys().next().expect("animal");
        let fight = Fight {
            progress: 0,
            ticks: 1,
            damage: 2,
            hits_left: 1,
            quarrel: None,
        };
        res.colony
            .fights
            .insert((Fighter::Animal(id), Fighter::Person(7)), fight);
        super::super::fight::resolve_fights(&mut world, &mut res);
        assert_eq!(res.colony.wildlife.tallies.bites, 1);
        assert!(!res.colony.story.touched.contains(&(pressure.code(), 7, 1)));
        res.colony
            .fights
            .insert((Fighter::Person(7), Fighter::Animal(id)), fight);
        super::super::fight::resolve_fights(&mut world, &mut res);
        assert!(!res.colony.wildlife.animals.contains_key(&id));
        let (index, entry) = res
            .chronicle
            .entries()
            .iter()
            .enumerate()
            .find(|(_, e)| e.kind == sim_core::chronicle::ChronicleKind::WolfKilled)
            .expect("kill");
        assert_eq!(entry.cause, kind.code());
        assert_eq!(res.chronicle.cause_of(index as u32), None);
    }
}

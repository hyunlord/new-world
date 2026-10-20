//! Pass 19: helpers that append to the chronicle (`SimResources::chronicle`).
//!
//! Every helper only reads the world (`get` and one `Agent` query, which
//! inserts or removes nothing) and writes the chronicle; no simulation state
//! depends on what is written.
use hecs::{Entity, World};
use sim_core::chronicle::{ChronicleEntry, PersonRecord};
use sim_core::components::{Agent, AgentId, BodyHealth, Kin, Life, Sex, Temperament};
use sim_engine::SimResources;

/// The person record built from `entity`'s live components (an absent
/// component leaves its fields `None`).
pub fn person_record(world: &World, entity: Entity) -> PersonRecord {
    let kin = world.get::<&Kin>(entity).ok().map(|kin| *kin);
    PersonRecord {
        female: world.get::<&Sex>(entity).ok().map(|sex| sex.female),
        born_tick: world.get::<&Life>(entity).ok().map(|life| life.born_tick),
        mother: kin.and_then(|kin| kin.mother),
        father: kin.and_then(|kin| kin.father),
        temperament: world
            .get::<&Temperament>(entity)
            .ok()
            .map(|t| [t.ns, t.ha, t.rd, t.p]),
        died_tick: None,
        death_reason: None,
    }
}

/// Whether `entity` is present and not dead (`BodyHealth::is_dead`).
pub fn is_alive(world: &World, entity: Entity) -> bool {
    world.contains(entity)
        && world
            .get::<&BodyHealth>(entity)
            .map_or(true, |health| !health.is_dead())
}

/// The living entity of agent `id`, by a scan (used only when a record is
/// missing or a partner must be found).
pub fn alive_entity(world: &World, id: AgentId) -> Option<Entity> {
    world
        .query::<(&Agent, Option<&BodyHealth>)>()
        .iter()
        .find(|(_, (agent, health))| {
            agent.id == id && health.is_none_or(|health| !health.is_dead())
        })
        .map(|(entity, _)| entity)
}

/// Create `id`'s record from its components if it has none and it is alive.
pub fn ensure(world: &World, resources: &mut SimResources, id: AgentId) {
    resources.chronicle.ensure_person(id, || {
        alive_entity(world, id).map(|entity| person_record(world, entity))
    });
}

/// [`ensure`] when the entity is already known.
pub fn ensure_entity(world: &World, resources: &mut SimResources, id: AgentId, entity: Entity) {
    resources.chronicle.ensure_person(id, || {
        is_alive(world, entity).then(|| person_record(world, entity))
    });
}

/// Ensure the record of every person `entry` names, then append it.
pub fn record(world: &World, resources: &mut SimResources, entry: ChronicleEntry) {
    for id in entry.agents() {
        ensure(world, resources, id);
    }
    resources.chronicle.push(entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::chronicle::ChronicleKind;
    use sim_core::MaterialRegistry;

    #[test]
    fn records_living_people_from_components_and_skips_the_dead() {
        let mut world = World::new();
        let mut resources = SimResources::new(4, 4, MaterialRegistry::new());
        let mother = world.spawn((
            Agent { id: 1 },
            Sex { female: true },
            Life {
                born_tick: -40,
                lifespan_ticks: 900,
            },
            Temperament {
                ns: 1,
                ha: 2,
                rd: 3,
                p: 4,
            },
        ));
        let child = world.spawn((
            Agent { id: 3 },
            Kin {
                mother: Some(1),
                father: Some(2),
                ..Kin::default()
            },
        ));
        let mut dead = BodyHealth::new();
        dead.hp = 0.0;
        world.spawn((Agent { id: 2 }, dead));
        record(
            &world,
            &mut resources,
            ChronicleEntry::new(9, ChronicleKind::Born)
                .subject(3)
                .other(Some(1))
                .third(Some(2)),
        );
        let mother_record = resources.chronicle.person(1).expect("alive mother");
        assert_eq!(mother_record.female, Some(true));
        assert_eq!(mother_record.born_tick, Some(-40));
        assert_eq!(mother_record.temperament, Some([1, 2, 3, 4]));
        let child_record = resources.chronicle.person(3).expect("alive child");
        assert_eq!(
            (child_record.mother, child_record.father),
            (Some(1), Some(2))
        );
        assert_eq!(
            (
                child_record.female,
                child_record.born_tick,
                child_record.temperament
            ),
            (None, None, None)
        );
        assert!(
            resources.chronicle.person(2).is_none(),
            "the dead are recorded at death only"
        );
        assert_eq!(resources.chronicle.len(), 1);
        assert!(is_alive(&world, mother) && is_alive(&world, child));
    }
}

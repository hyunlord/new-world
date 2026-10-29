//! Convert detached old fission provisions once without altering group membership or wood.
use super::{roster, FissionSystem};
use hecs::World;
use sim_engine::SimResources;

/// Rewrite only legacy colony system payloads before installation into the engine.
pub(crate) fn migrate_pending_food(
    world: &World,
    res: &mut SimResources,
    systems: &mut [(String, Vec<u8>)],
) -> Result<(), String> {
    if res.rules.colony.is_none() {
        return Ok(());
    }
    let people = roster(world);
    for (name, bytes) in systems {
        if name != "FissionSystem" {
            continue;
        }
        let mut state: FissionSystem = crate::save::decode(bytes)?;
        for group in state.groups.values_mut() {
            if group.food == 0 {
                continue;
            }
            // A stamped camp holds the pending store; a walking group carries it
            // at its leader. If the leader died, the group's destination is retained.
            let tile = group
                .placed
                .map(|(tile, _)| tile)
                .or_else(|| people.get(&group.leader).map(|row| row.1))
                .unwrap_or(group.site);
            crate::runtime::food::migration::place_pending(res, tile, group.food)?;
            group.food = 0;
        }
        *bytes = crate::save::encode(&state)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::settlement::fission::Group;
    use sim_core::colony::food::FoodOwner;
    use sim_core::components::{Agent, Position};
    use sim_core::rules::WorldRules;
    use sim_core::MaterialRegistry;
    use std::collections::BTreeSet;

    fn old_groups() -> FissionSystem {
        let mut state = FissionSystem::new();
        state.next_id = 9;
        state.groups.insert(
            9,
            Group {
                parent: 1,
                site: (6, 6),
                members: BTreeSet::from([2, 3, 4]),
                leader: 2,
                since: 17,
                food: 7,
                wood: 11,
                placed: None,
                log: 0,
            },
        );
        state
    }

    #[test]
    fn old_pending_food_is_physical_once_and_preserves_group_and_wood() {
        let mut world = World::new();
        world.spawn((Agent { id: 2 }, Position::new(2, 3)));
        let mut res = SimResources::new(8, 8, MaterialRegistry::new());
        res.rules = WorldRules::colony(0);
        let mut state = old_groups();
        let mut systems = vec![("FissionSystem".into(), crate::save::encode(&state).unwrap())];
        migrate_pending_food(&world, &mut res, &mut systems).unwrap();
        state.groups.get_mut(&9).unwrap().food = 0;
        assert_eq!(systems[0].1, crate::save::encode(&state).unwrap());
        assert_eq!(res.colony.ground_items[&(2, 3)].count, 7);
        assert!(res.food.ages.contains_key(&FoodOwner::Ground((2, 3))));
        assert_eq!(res.food.tallies.initial_units, 7);
        assert_eq!(res.food.tallies.migrated.values().sum::<u64>(), 7);
        migrate_pending_food(&world, &mut res, &mut systems).unwrap();
        assert_eq!(res.food.tallies.initial_units, 7);
        crate::runtime::food::validate(&world, &res).unwrap();
    }

    #[test]
    fn placed_group_uses_its_site_and_baseline_payload_is_untouched() {
        let world = World::new();
        let mut res = SimResources::new(8, 8, MaterialRegistry::new());
        let mut state = old_groups();
        state.groups.get_mut(&9).unwrap().placed = Some(((5, 5), 18));
        let original = crate::save::encode(&state).unwrap();
        let mut systems = vec![("FissionSystem".into(), original.clone())];
        migrate_pending_food(&world, &mut res, &mut systems).unwrap();
        assert_eq!(systems[0].1, original);
        assert!(res.colony.ground_items.is_empty());
        res.rules = WorldRules::colony(0);
        migrate_pending_food(&world, &mut res, &mut systems).unwrap();
        assert_eq!(res.colony.ground_items[&(5, 5)].count, 7);
        let migrated: FissionSystem = crate::save::decode(&systems[0].1).unwrap();
        assert_eq!(migrated.groups[&9].wood, 11);
        assert_eq!(migrated.groups[&9].food, 0);
    }
}

//! Retained physical targets expire at real invalidation and action boundaries.
use super::*;
use crate::runtime::food::{items, storage};
use sim_core::colony::{food::FoodOwner, ItemKind, ItemSpec};

#[test]
fn invalidated_food_owner_reselects_without_reviving_or_moving_old_food() {
    for reason in ["depleted", "spoiled", "unreachable"] {
        let mut world = World::new();
        let entity = world.spawn((
            Agent { id: 1 },
            Position::new(2, 2),
            AgentState::Idle,
            Hunger::new(90.0, 0.0),
        ));
        let mut res = SimResources::new(8, 8, MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.nav.install(sim_core::nav::CostMap::new(8, 8));
        let original = FoodOwner::Ground((3, 2));
        let replacement = FoodOwner::Ground((2, 3));
        let spec = ItemSpec::raw(ItemKind::from_name("roots").unwrap(), None);
        items::create_ground(&mut res, (3, 2), spec, 1, 0, items::FoodSource::Harvest).unwrap();
        let mut effects = EffectState::default();
        let mut c = context(
            &mut world,
            &mut res,
            &mut effects,
            entity,
            ActionTarget::Food {
                tile: (3, 2),
                owner: original,
            },
        );
        food::start(&mut c).unwrap();
        match reason {
            "depleted" => {
                storage::expire_count(c.world, c.resources, original, 1).unwrap();
            }
            "spoiled" => {
                c.resources.current_tick = c.resources.rules.calendar.days_to_ticks(100);
                assert_eq!(
                    storage::remaining_days(
                        c.world,
                        c.resources,
                        original,
                        crate::runtime::food::day(c.resources)
                    )
                    .unwrap(),
                    0
                );
            }
            "unreachable" => {
                assert!(c.resources.nav.set_cost((3, 2), 0));
            }
            _ => unreachable!(),
        }
        let day = crate::runtime::food::day(c.resources);
        items::create_ground(
            c.resources,
            (2, 3),
            spec,
            1,
            day,
            items::FoodSource::Harvest,
        )
        .unwrap();
        c.target = ActionTarget::Food {
            tile: (2, 3),
            owner: replacement,
        };
        assert_eq!(food::advance(&mut c), Ok(EffectStatus::Running), "{reason}");
        assert_eq!(
            c.resources.food.active_targets.get(&1),
            Some(&replacement),
            "{reason}"
        );
        assert_eq!(
            c.effects.seek_set.last(),
            Some(&(entity, (2, 3))),
            "{reason}"
        );
        assert_eq!(c.resources.food.tallies.meals, 0);
        if reason != "depleted" {
            assert_eq!(
                items::stack(c.world, c.resources, original).unwrap().count,
                1
            );
            assert_eq!(c.resources.food.ages[&original].born_day, 0);
        }
    }
}

#[test]
fn interruption_and_death_clear_retained_owner_even_without_carried_food() {
    let mut world = World::new();
    let entity = world.spawn((Agent { id: 1 }, Position::new(2, 2), AgentState::Idle));
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    let mut effects = EffectState::default();
    let first = FoodOwner::Ground((3, 2));
    let next = FoodOwner::Ground((2, 3));
    let mut c = context(
        &mut world,
        &mut res,
        &mut effects,
        entity,
        ActionTarget::Food {
            tile: (3, 2),
            owner: first,
        },
    );
    food::start(&mut c).unwrap();
    c.interrupted = true;
    assert_eq!(food::advance(&mut c), Ok(EffectStatus::Finished));
    assert!(!c.resources.food.active_targets.contains_key(&1));
    c.interrupted = false;
    c.target = ActionTarget::Food {
        tile: (2, 3),
        owner: next,
    };
    food::start(&mut c).unwrap();
    assert_eq!(c.resources.food.active_targets.get(&1), Some(&next));
    food::finish(&mut c).unwrap();
    assert!(!c.resources.food.active_targets.contains_key(&1));
    food::start(&mut c).unwrap();
    crate::runtime::food::loss::lose_carried(c.world, c.resources, entity);
    assert!(!c.resources.food.active_targets.contains_key(&1));
}

//! Assignment must respect the real recovery owner before routine cleanup runs.
use super::*;
use sim_core::colony::{
    ItemKind, ItemSpec, ItemStack, Job, JobKind, WorkPriorities, WorkType, Zone, ZoneKind,
};
use sim_core::components::{Activity, ResourceKind};

#[test]
fn assignment_preserves_inflight_recovery_but_allows_healthy_leisure_and_real_fuel() {
    for (injured, deficit, fuel) in [
        (true, false, false),
        (false, false, false),
        (true, true, true),
        (true, true, false),
    ] {
        let (mut engine, entity, id, home) = fixture();
        let others: Vec<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .filter(|(e, _)| *e != entity)
            .map(|(e, _)| e)
            .collect();
        for other in others {
            engine.world.despawn(other).expect("isolated worker");
        }
        engine.resources.current_tick = 1;
        engine.resources.colony.ground_items.clear();
        engine.resources.colony.zones.clear();
        engine.resources.colony.zone_of_tile.clear();
        engine.resources.colony.designations.clear();
        engine.resources.colony.dress_off = true;
        engine
            .resources
            .rules
            .colony
            .as_mut()
            .expect("colony")
            .knowledge = None;
        for settlement in engine.resources.settlements.values_mut() {
            settlement
                .stockpile
                .insert(ResourceKind::Wood, if deficit { 0 } else { 100 });
        }
        let source = (home.0 + 1, home.1);
        let destination = (home.0 + 2, home.1);
        assert!(engine.resources.nav.passable(source));
        assert!(engine.resources.nav.passable(destination));
        let kind = ItemKind::from_name(if fuel { "firewood" } else { "hide" }).expect("item");
        engine
            .resources
            .colony
            .ground_items
            .insert(source, ItemStack::of(ItemSpec::raw(kind, None), 2));
        engine.resources.colony.zones.insert(
            900,
            Zone {
                id: 900,
                kind: ZoneKind::Stockpile,
                tiles: [destination].into(),
            },
        );
        engine
            .resources
            .colony
            .zone_of_tile
            .insert(destination, 900);
        engine.resources.colony.touch(false);
        let mut priorities = WorkPriorities {
            levels: [0; WorkType::ALL.len()],
        };
        priorities.levels[WorkType::Haul.code() as usize] = 1;
        engine
            .world
            .insert_one(entity, priorities)
            .expect("hauling worker");
        engine
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 0.0;
        engine
            .world
            .get::<&mut Thirst>(entity)
            .expect("thirst")
            .value = 0.0;
        engine
            .world
            .get::<&mut BodyHealth>(entity)
            .expect("health")
            .apply_damage(10.0);
        engine
            .world
            .insert_one(entity, Cold)
            .expect("cold admission");
        let source_policy = include_str!("../../../../sim-core/data/colony/mind.ron")
            .replace("warm_refuge_enter", "custom_refuge_admission");
        let mut mind =
            ColonyMindSystem::new(MindProgram::load(&source_policy).expect("custom authored node"));
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("start refuge");
        let before = *engine
            .world
            .get::<&Activity>(entity)
            .expect("in-flight refuge");
        assert!(before.arrived_tick.is_none());
        if !injured {
            let mut health = engine.world.get::<&mut BodyHealth>(entity).expect("health");
            health.hp = health.max_hp;
        }
        let taken_before = engine.resources.colony.tallies.jobs_taken;
        engine.resources.current_tick += 1;
        crate::runtime::jobs::JobAssignSystem::new().tick(&mut engine.world, &mut engine.resources);
        let should_assign = !injured || fuel;
        assert_eq!(
            engine.world.get::<&Job>(entity).is_ok(),
            should_assign,
            "injured={injured} deficit={deficit} fuel={fuel}"
        );
        if should_assign {
            assert!(
                matches!(engine.world.get::<&Job>(entity).expect("physical haul").kind, JobKind::Haul { from, .. } if from == source)
            );
        } else {
            assert_eq!(engine.resources.colony.tallies.jobs_taken, taken_before);
            assert!(engine.resources.colony.reservations.is_empty());
        }
        mind.try_tick(&mut engine.world, &mut engine.resources)
            .expect("mind after assignment");
        if injured && !fuel {
            assert_eq!(
                engine
                    .world
                    .get::<&Activity>(entity)
                    .expect("recovery retained")
                    .chosen_tick,
                before.chosen_tick
            );
            let context = TickContext::capture(&engine.world, &engine.resources).expect("context");
            assert!(refuge::active(context.row(id).expect("resident")));
        }
    }
}

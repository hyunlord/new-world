//! Capacity and in-transit fuel ownership boundaries for refuge targeting.
use super::*;

#[test]
fn refuge_respects_real_reserved_seats_and_spreads_to_another_warm_tile() {
    let (mut engine, _, id, tile) = fixture();
    let sid = *engine
        .resources
        .settlements
        .keys()
        .next()
        .expect("settlement");
    let others: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| a.id != id)
        .take(2)
        .map(|(e, _)| e)
        .collect();
    for entity in others {
        engine
            .world
            .insert_one(
                entity,
                sim_core::components::Activity {
                    kind: sim_core::components::ActivityKind::SitByFire,
                    settlement: sid,
                    place: tile,
                    chosen_tick: 0,
                    arrived_tick: Some(0),
                    until_tick: None,
                },
            )
            .expect("existing occupied seats");
    }
    let context = TickContext::capture(&engine.world, &engine.resources).expect("context");
    assert!(refuge::target(
        context.row(id).expect("person"),
        &context,
        &engine.resources
    )
    .is_none());
    let other = (tile.0 + 1, tile.1);
    assert!(engine.resources.nav.reachable(tile, other));
    engine
        .resources
        .colony
        .rooms
        .get_mut(&900)
        .expect("room")
        .tiles
        .insert(other);
    engine.resources.colony.room_of_tile.insert(other, 900);
    assert_eq!(
        refuge::target(
            context.row(id).expect("person"),
            &context,
            &engine.resources
        )
        .expect("other seat")
        .tile,
        other
    );
}

#[test]
fn fuel_supply_retains_ownership_after_source_pickup_only_for_matching_input() {
    use sim_core::colony::{
        Bill, BillMode, Carry, ItemKind, ItemSpec, Job, JobKind, JobStage, Workshop,
    };
    let (mut engine, entity, id, tile) = fixture();
    let recipe = sim_core::colony::workshop::recipes()
        .iter()
        .find(|r| r.output.0.definition().fuel_units.is_some())
        .expect("fuel recipe");
    let input = recipe.inputs[0].0;
    engine.resources.colony.workshops.insert(
        901,
        Workshop {
            id: 901,
            kind: recipe.workshop,
            top_left: tile,
            size: (1, 1),
            bills: vec![Bill {
                id: 902,
                recipe: recipe.id,
                mode: BillMode::Make(1),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf: vec![],
            active: Some(902),
        },
    );
    for settlement in engine.resources.settlements.values_mut() {
        settlement
            .stockpile
            .insert(sim_core::components::ResourceKind::Wood, 0);
    }
    let source = (tile.0 + 3, tile.1);
    engine.resources.colony.ground_items.remove(&source);
    engine
        .world
        .insert_one(
            entity,
            Job {
                kind: JobKind::Supply {
                    from: source,
                    workshop: 901,
                    to: tile,
                },
                stage: JobStage::ToDest,
                taken_at: 0,
                amount: 1,
                item: input,
                material: None,
                fetch: None,
                best_distance: 3,
                progress_tick: 0,
            },
        )
        .expect("supply");
    engine
        .world
        .insert_one(entity, Carry::of(ItemSpec::raw(input, None), 1))
        .expect("carried input");
    let context = TickContext::capture(&engine.world, &engine.resources).expect("context");
    assert!(refuge::fuel_job(
        context.row(id).expect("supplier"),
        &engine.resources
    ));
    let other_recipe = sim_core::colony::workshop::recipes()
        .iter()
        .find(|r| {
            r.workshop == recipe.workshop
                && r.output.0.definition().fuel_units.is_none()
                && r.inputs.iter().any(|(kind, _)| *kind == input)
        })
        .expect("same-bench nonfuel input");
    let shop = engine
        .resources
        .colony
        .workshops
        .get_mut(&901)
        .expect("workshop");
    shop.bills.push(Bill {
        id: 903,
        recipe: other_recipe.id,
        mode: BillMode::Make(1),
        done: 0,
        suspended: false,
        auto: false,
    });
    shop.active = Some(903);
    assert!(
        !refuge::fuel_job(context.row(id).expect("supplier"), &engine.resources),
        "an inactive fuel bill cannot protect another bill's delivery"
    );
    engine
        .resources
        .colony
        .workshops
        .get_mut(&901)
        .expect("workshop")
        .active = Some(902);
    engine
        .world
        .insert_one(entity, Carry::of(ItemSpec::raw(ItemKind::Flint, None), 1))
        .expect("unrelated input");
    let context = TickContext::capture(&engine.world, &engine.resources).expect("context");
    assert!(!refuge::fuel_job(
        context.row(id).expect("supplier"),
        &engine.resources
    ));
}

#[test]
fn primary_and_rich_refresh_retain_live_thermal_guard_inputs() {
    use sim_core::colony::people::{Apparel, Top};
    let (mut engine, entity, id, _) = fixture();
    let mut context = TickContext::capture(&engine.world, &engine.resources).expect("context");
    let apparel = Apparel {
        top: Top::HideWrap,
        since_day: 0,
        material: None,
        quality: 50,
        wear_remaining_hours: None,
    };
    engine
        .world
        .insert_one(entity, apparel)
        .expect("physical garment");
    engine.world.insert_one(entity, Cold).expect("condition");
    engine
        .world
        .get::<&mut BodyHealth>(entity)
        .expect("health")
        .apply_damage(1.0);
    context
        .refresh_primary_continuation_exclusive(&mut engine.world, id)
        .expect("primary");
    let row = context.row(id).expect("row");
    assert!(row.cold);
    assert!(!row.health_full);
    assert_eq!(row.apparel, Some(apparel));
    engine
        .world
        .remove_one::<Apparel>(entity)
        .expect("remove garment");
    engine
        .world
        .remove_one::<Cold>(entity)
        .expect("remove condition");
    context
        .upgrade_continuation_exclusive(&mut engine.world, id)
        .expect("rich");
    let row = context.row(id).expect("row");
    assert!(!row.cold);
    assert!(!row.health_full);
    assert_eq!(row.apparel, None);
}

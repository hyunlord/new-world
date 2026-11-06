//! Physical garment ownership, hourly wear and authored comfort regressions.
mod colony_support;

use sim_core::colony::people::{Apparel, Top};
use sim_core::colony::{ItemKind, ItemSpec, Job, JobKind, JobStage};
use sim_core::components::Agent;
use sim_engine::RuntimeSystem;
use sim_systems::runtime::jobs::{cancel, hunt, invariants};

fn clothing(kind: ItemKind, hours: Option<u32>) -> ItemSpec {
    ItemSpec {
        kind,
        material: Some(sim_core::colony::materials::OAK),
        quality: 73,
        wear_remaining_hours: hours,
    }
}

fn worn(spec: ItemSpec) -> Apparel {
    Apparel {
        top: Top::of_item(spec.kind).expect("garment"),
        since_day: 0,
        material: spec.material,
        quality: spec.quality,
        wear_remaining_hours: spec.wear_remaining_hours,
    }
}

#[test]
fn equipping_and_death_keep_exact_physical_garments() {
    let mut engine = colony_support::job_world_m2(12, 12);
    let entity = colony_support::worker(&mut engine, 3, 3);
    let id = engine.world.get::<&Agent>(entity).expect("agent").id;
    let old = clothing(ItemKind::HideWrap, Some(17));
    let new = clothing(ItemKind::FurCloak, Some(25));
    engine
        .world
        .insert_one(entity, worn(old))
        .expect("old clothing");
    engine.resources.colony.item_ledger.create(old.kind, 1);
    colony_support::drop_items(&mut engine, (3, 3), new, 1);
    let job = Job {
        kind: JobKind::Equip { from: (3, 3) },
        stage: JobStage::ToSource,
        taken_at: 0,
        amount: 1,
        item: new.kind,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    hunt::equip_step(
        &mut engine.world,
        &mut engine.resources,
        id,
        entity,
        job,
        (3, 3),
    );
    assert_eq!(
        engine.world.get::<&Apparel>(entity).expect("worn").spec(),
        new
    );
    assert!(engine
        .resources
        .colony
        .ground_items
        .values()
        .any(|s| s.spec() == old));
    cancel::on_death(&mut engine.world, &mut engine.resources, entity);
    assert!(engine.world.get::<&Apparel>(entity).is_err());
    assert!(engine
        .resources
        .colony
        .ground_items
        .values()
        .any(|s| s.spec() == new));
    invariants::check(&engine.world, &engine.resources).expect("conserved after death drop");
}

#[test]
fn only_worn_finite_clothing_expires_and_is_consumed_once() {
    let mut engine = colony_support::job_world_m2(12, 12);
    let finite = colony_support::worker(&mut engine, 3, 3);
    let legacy = colony_support::worker(&mut engine, 4, 3);
    let short = clothing(ItemKind::HideWrap, Some(2));
    let lasting = clothing(ItemKind::FurCloak, None);
    for (entity, spec) in [(finite, short), (legacy, lasting)] {
        engine.world.insert_one(entity, worn(spec)).expect("worn");
        engine.resources.colony.item_ledger.create(spec.kind, 1);
    }
    colony_support::drop_items(&mut engine, (6, 6), short, 1);
    let mut fire = sim_systems::runtime::settlement::FireSystem::new();
    engine.resources.current_tick = 60;
    fire.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine
            .world
            .get::<&Apparel>(finite)
            .expect("one hour left")
            .wear_remaining_hours,
        Some(1)
    );
    engine.resources.current_tick = 120;
    fire.tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Apparel>(finite).is_err());
    engine.resources.current_tick = 180;
    fire.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(
        engine
            .world
            .get::<&Apparel>(legacy)
            .expect("legacy unchanged")
            .spec(),
        lasting
    );
    assert_eq!(engine.resources.colony.ground_items[&(6, 6)].spec(), short);
    assert_eq!(
        invariants::ledger_of(&engine.world, &engine.resources, short.kind).consumed,
        1
    );
    invariants::check(&engine.world, &engine.resources).expect("expiry ledger");
}

#[test]
fn worn_authored_comfort_changes_the_real_mood_target() {
    use sim_core::colony::people::Modifier;
    use sim_systems::runtime::mood;
    let mut engine = sim_test::colony_engine(0);
    let (_, entity) = colony_support::adults(&engine)[0];
    let _ = engine.world.remove_one::<Apparel>(entity);
    let rule = engine
        .resources
        .rules
        .colony
        .and_then(|r| r.mood)
        .expect("mood rule");
    let bare = mood::modifiers(&mood::facts_of(&engine.world, &engine.resources, entity));
    let kind = ItemKind::from_name("linen_tunic").expect("tunic");
    let definition = kind.definition().apparel.as_ref().expect("apparel data");
    let spec = clothing(kind, definition.wear_days.map(|days| days * 24));
    engine.world.insert_one(entity, worn(spec)).expect("tunic");
    let dressed = mood::modifiers(&mood::facts_of(&engine.world, &engine.resources, entity));
    assert_ne!(
        definition.comfort_permille, 0,
        "authored example exercises the mechanic"
    );
    assert!(dressed.contains(&(Modifier::ClothingComfort, definition.comfort_permille)));
    assert_eq!(
        i32::from(mood::target(&rule, &dressed)) - i32::from(mood::target(&rule, &bare)),
        definition.comfort_permille
    );
    engine
        .world
        .insert_one(entity, worn(clothing(ItemKind::HideWrap, None)))
        .expect("legacy wrap");
    assert_eq!(
        mood::modifiers(&mood::facts_of(&engine.world, &engine.resources, entity)),
        bare,
        "inert legacy comfort"
    );
}

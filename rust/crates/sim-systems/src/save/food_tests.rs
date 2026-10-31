//! Food sidecar wire boundaries and precisely bounded legacy-system migrations.
use super::*;
use sim_core::colony::food::{FoodAge, FoodState, MealHistory, MealRecord};
use sim_core::colony::ItemKind;

fn legacy12_baseline() -> Vec<u8> {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    let old = LegacyBodyV12 {
        current_tick: engine.current_tick(),
        world: world::capture(&engine.world).expect("world"),
        systems: engine.system_states().expect("systems"),
        steward: std::mem::take(&mut engine.resources.steward),
        content_state: std::mem::take(&mut engine.resources.content_state),
        mind: std::mem::take(&mut engine.resources.mind),
        resources: engine.resources,
    };
    let envelope = Envelope {
        revision: "catalog-v1-fnv1a64:1e4d7fded25139b4:45408".into(),
        payload: encode(&old).expect("format12 body"),
    };
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&12u32.to_le_bytes());
    bytes.extend(encode(&envelope).expect("envelope"));
    bytes
}

#[test]
fn pre_food_baseline_has_empty_sidecar_and_format14_roundtrip() {
    let bytes = legacy12_baseline();
    let mut loaded = load_with(&bytes, |_| {}).expect("known old catalog and body");
    assert_eq!(loaded.resources.food, FoodState::default());
    assert!(loaded.system_names().is_empty());
    let current = save(&mut loaded).expect("format14");
    assert_eq!(&current[8..12], &14u32.to_le_bytes());
    let mut again = load_with(&current, |_| {}).expect("format14 reload");
    assert_eq!(save(&mut again).expect("stable bytes"), current);
}

#[test]
fn current_format_cannot_claim_an_old_catalog_to_bypass_admission() {
    let mut bytes = legacy12_baseline();
    bytes[8..12].copy_from_slice(&14u32.to_le_bytes());
    // Whether catalog definitions have changed yet or not, the old body lacks
    // the final FoodState and must never be accepted as a current save.
    assert!(load_with(&bytes, |_| {}).is_err());
}

#[test]
fn skipped_food_sidecar_does_not_change_resource_wire_bytes() {
    let engine = SimEngine::new(4, 4, MaterialRegistry::new());
    let mut resources = engine.resources;
    let before = encode(&resources).expect("resources");
    resources.food.last_daily_day = Some(17);
    resources.food.version = 42;
    resources.food.tallies.harvested_units = 99;
    assert_eq!(encode(&resources).expect("resources"), before);
}

#[test]
fn known_system_additions_preserve_registration_order() {
    let registered: Vec<_> = [
        "A",
        "FoodDailySystem",
        "B",
        "StewardSystem",
        "StewardDefenceSystem",
        "C",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), Vec::new()))
    .collect();
    let mut saved: Vec<_> = ["A", "B", "C"]
        .into_iter()
        .map(|name| (name.to_owned(), Vec::new()))
        .collect();
    insert_legacy_systems(
        &mut saved,
        &registered,
        LegacySystemAdditions {
            steward: true,
            defence: true,
            food: true,
            diagnosis: true,
        },
    )
    .expect("all known additions in order");
    assert_eq!(saved, registered);
    assert!(insert_legacy_systems(
        &mut saved,
        &registered,
        LegacySystemAdditions {
            steward: false,
            defence: false,
            food: true,
            diagnosis: true,
        }
    )
    .is_err());
}

#[test]
fn food_age_merge_and_bounded_history_do_not_rejuvenate_evidence() {
    let old = FoodAge { born_day: 3 };
    let fresh = FoodAge { born_day: 8 };
    assert_eq!(old.merged(fresh), fresh.merged(old));
    assert_eq!(old.merged(fresh).remaining(9, 6), 0);
    let mut history = MealHistory::default();
    let item = ItemKind::role("grazer_meat");
    for tick in 0..12 {
        history.record(
            MealRecord {
                tick,
                item,
                pieces: 1,
                units: 1,
                cooked: false,
                source_owner: sim_core::colony::food::FoodOwner::Ground((0, 0)),
                born_day: 0,
                storage_lifetime_days: 6,
                source_choice: None,
            },
            &[if tick == 0 { "fruit" } else { "root" }.to_owned()],
        );
    }
    assert_eq!(history.last_meals.len(), 8);
    assert_eq!(history.groups_in(11, 12), 2);
    assert_eq!(history.groups_in(12, 12), 1);
    history.prune(12, 12);
    assert!(!history.last_group_tick.contains_key("fruit"));
}

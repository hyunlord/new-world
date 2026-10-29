//! A completed hunt accounts for blocked meat without inventing retained food.
use super::*;
use sim_core::colony::wildlife::{Animal, AnimalState};
use sim_core::colony::ItemStack;
use sim_core::components::{Agent, Position};
use sim_core::rules::WorldRules;

fn scene(packed: bool) -> (World, SimResources, Entity, WildlifeRule, ItemKind) {
    let mut world = World::new();
    let mut res = SimResources::new(4, 4, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(4, 4));
    let rule = rule(&res).expect("wildlife rule");
    let kind = AnimalKind::from_name("deer").expect("deer");
    let meat = kind.definition().loot.meat.as_ref().expect("meat").item;
    if packed {
        // Every tile is incompatible with meat, but the legitimate hide output
        // can merge. Thus this exercises the food failure, not old nonfood spill.
        let hide = kind.definition().loot.hide.as_ref().expect("hide").item;
        for y in 0..4 {
            for x in 0..4 {
                res.colony
                    .ground_items
                    .insert((x, y), ItemStack::of(ItemSpec::raw(hide, None), 1));
                res.colony.item_ledger.create(hide, 1);
            }
        }
    }
    res.colony.wildlife.animals.insert(
        1,
        Animal {
            id: 1,
            kind,
            hp: 0,
            state: AnimalState::Wander,
            pos: (1, 1),
            pack: None,
            born_tick: 0,
            path: Vec::new(),
            goal: None,
            home: Some((1, 1)),
        },
    );
    let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1)));
    (world, res, entity, rule, meat)
}

#[test]
fn packed_hunt_records_real_yield_as_lost_and_reports_zero_retained_meat() {
    let (mut world, mut res, entity, rule, meat) = scene(true);
    let before = res.colony.ground_items.clone();
    crate::runtime::jobs::invariants::check(&world, &res).expect("valid packed scene");
    assert!(kill(&mut world, &mut res, &rule, 7, entity, 1).is_empty());
    assert!(!res.colony.wildlife.animals.contains_key(&1));
    assert_eq!(res.food.tallies.harvested[&meat], 2);
    assert_eq!(res.food.tallies.lost[&meat], 2);
    assert_eq!(res.food.tallies.harvested_units, 4);
    assert_eq!(res.food.tallies.lost_units, 4);
    assert_eq!(res.food.tallies.spoiled_units, 0);
    assert_eq!(res.colony.item_ledger.created_of(meat), 2);
    assert_eq!(res.colony.item_ledger.consumed_of(meat), 2);
    assert_eq!(res.colony.wildlife.tallies.meat_dropped, 0);
    assert_eq!(res.colony.ground_total(meat), 0);
    assert!(res.food.ages.is_empty());
    assert_eq!(res.colony.ground_items.len(), before.len());
    let hunted = res
        .chronicle
        .entries()
        .iter()
        .find(|entry| entry.kind == ChronicleKind::Hunted)
        .expect("hunt event");
    assert_eq!(hunted.subject, Some(7));
    assert_eq!(hunted.value, 0);
    crate::runtime::jobs::invariants::check(&world, &res).expect("conserved failed yield");
    // A repeated completion cannot book the removed animal twice.
    assert!(kill(&mut world, &mut res, &rule, 7, entity, 1).is_empty());
    assert_eq!(res.food.tallies.lost_units, 4);
}

#[test]
fn ordinary_hunt_keeps_existing_yield_and_records_fresh_owners_once() {
    let (mut world, mut res, entity, rule, meat) = scene(false);
    assert!(!kill(&mut world, &mut res, &rule, 7, entity, 1).is_empty());
    assert!(!res.colony.wildlife.animals.contains_key(&1));
    assert_eq!(res.colony.ground_total(meat), 2);
    assert_eq!(res.colony.wildlife.tallies.meat_dropped, 2);
    assert_eq!(res.food.tallies.harvested_units, 4);
    assert_eq!(res.food.tallies.lost_units, 0);
    assert_eq!(res.colony.item_ledger.created_of(meat), 2);
    assert_eq!(res.colony.item_ledger.consumed_of(meat), 0);
    assert_eq!(res.food.ages.len(), 1);
    assert_eq!(res.food.ages.values().next().expect("age").born_day, 0);
    assert_eq!(
        res.chronicle
            .entries()
            .iter()
            .find(|entry| entry.kind == ChronicleKind::Hunted)
            .expect("hunt event")
            .value,
        2
    );
    crate::runtime::jobs::invariants::check(&world, &res).expect("conserved ordinary yield");
}

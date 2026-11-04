use super::*;
use sim_core::colony::cultivation::{FieldTile, FieldWork};
fn scene() -> SimResources {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    let crop = installed_or_bundled().crop_by_name("emmer").expect("emmer");
    res.cultivation.fields.insert(
        (2, 2),
        FieldTile {
            owner: 1,
            crop,
            phase: FieldPhase::Sown { planted_day: 0 },
            active: false,
            work: FieldWork::Paused,
            tended: false,
        },
    );
    res.cultivation.fertility.insert((2, 2), 800);
    res.cultivation.last_update_day = Some(0);
    res
}
#[test]
fn erased_paused_crop_grows_without_resetting_soil() {
    let mut res = scene();
    res.current_tick = res.ticks_per_day;
    update(&mut res);
    assert!(matches!(
        res.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Growing { growth_days: 1, .. }
    ));
    res.current_tick = res.ticks_per_day * 4;
    update(&mut res);
    assert!(matches!(
        res.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Ripe { .. }
    ));
    assert_eq!(res.cultivation.fertility[&(2, 2)], 800);
}
#[test]
fn frost_fails_once_without_items_or_seed_refund() {
    let mut res = scene();
    res.current_tick = res.ticks_per_day * 18;
    update(&mut res);
    assert_eq!(res.cultivation.fields[&(2, 2)].phase, FieldPhase::Fallow);
    assert_eq!(
        res.chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == sim_core::chronicle::ChronicleKind::CropFailure)
            .count(),
        1
    );
    let before = res.cultivation.clone();
    update(&mut res);
    assert_eq!(res.cultivation, before);
    assert!(res.colony.ground_items.is_empty());
    assert_eq!(res.cultivation.counters.harvests, 0);
}
#[test]
fn fallow_recovers_once_and_window_is_half_open() {
    let mut res = scene();
    res.cultivation.fields.get_mut(&(2, 2)).unwrap().phase =
        FieldPhase::Harvested { harvest_day: 0 };
    res.current_tick = res.ticks_per_day;
    update(&mut res);
    assert_eq!(res.cultivation.fields[&(2, 2)].phase, FieldPhase::Fallow);
    assert_eq!(res.cultivation.fertility[&(2, 2)], 810);
    assert!(sowing_window(&res, res.cultivation.fields[&(2, 2)].crop));
    res.current_tick = res.ticks_per_day * 3;
    assert!(!sowing_window(&res, res.cultivation.fields[&(2, 2)].crop));
}

#[test]
fn covered_tile_rejects_both_planner_and_command_admission() {
    let mut res = scene();
    let tile = (3, 3);
    let idx = res.tile_grid.idx(tile.0, tile.1);
    res.tile_grid.outdoor[idx] = false;
    assert!(!commands::zoneable(&res, tile));
    let crop = res.cultivation.fields[&(2, 2)].crop;
    let result = commands::zone(&mut hecs::World::new(), &mut res, tile, tile, crop);
    assert_eq!(result.reason, Some(sim_core::colony::RejectReason::Blocked));
    assert!(!res.cultivation.fields.contains_key(&tile));
}

#[test]
fn roof_after_sowing_fails_erased_crop_once_without_refund() {
    let mut res = scene();
    let idx = res.tile_grid.idx(2, 2);
    res.tile_grid.outdoor[idx] = false;
    res.current_tick = res.ticks_per_day;
    update(&mut res);
    assert_eq!(res.cultivation.fields[&(2, 2)].phase, FieldPhase::Fallow);
    let failures: Vec<_> = res
        .chronicle
        .entries()
        .iter()
        .filter(|e| e.kind == sim_core::chronicle::ChronicleKind::CropFailure)
        .collect();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].cause, 1);
    assert_eq!(res.cultivation.fertility[&(2, 2)], 810);
    assert!(res.colony.ground_items.is_empty());
    assert_eq!(res.cultivation.counters.harvests, 0);
    res.current_tick += res.ticks_per_day;
    update(&mut res);
    assert_eq!(res.chronicle.entries().len(), 1);
}

#[test]
fn existing_wild_plant_is_preserved_and_blocks_field_admission() {
    let mut res = scene();
    let tile = (3, 3);
    let plant = sim_core::colony::food::PlantState {
        species: installed_or_bundled().plant_by_name("wild_root").unwrap(),
        units_left: 0,
        next_regrow_day: 10,
    };
    res.food.plants.insert(tile, plant);
    assert!(!commands::zoneable(&res, tile));
    let crop = res.cultivation.fields[&(2, 2)].crop;
    let result = commands::zone(&mut hecs::World::new(), &mut res, tile, tile, crop);
    assert_eq!(result.reason, Some(sim_core::colony::RejectReason::Blocked));
    assert_eq!(res.food.plants[&tile], plant);
    assert!(!res.cultivation.fields.contains_key(&tile));
}

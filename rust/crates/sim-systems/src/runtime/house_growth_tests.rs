//! Daily residence observes actual assigned beds and physical houses.
use super::*;
use sim_core::colony::building_site::{BuildCause, BuildingPiece, PiecePhase};
use sim_core::colony::{BlueprintKind, ItemKind, ItemSpec, ItemStack, Occupant, Zone};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;

fn fixture() -> sim_engine::SimEngine {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled");
    let mut engine = sim_engine::SimEngine::new(16, 16, MaterialRegistry::new());
    let res = &mut engine.resources;
    res.rules = WorldRules::colony(0);
    res.ticks_per_day = 1440;
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.formation_tile = (1, 1);
    settlement.add_member_agent(0);
    res.settlements.insert(1, settlement);
    res.building_kinds.insert((3, 3), BuildingKind::Hut);
    res.building_owner.insert((3, 3), 1);
    res.nav.install(sim_core::nav::CostMap::new(16, 16));
    let site = BuildingSite {
        id: 0,
        template: "hut".into(),
        station: None,
        anchor: (2, 2),
        rotation: 0,
        footprint: (3, 3),
        pieces: vec![BuildingPiece {
            id: 0,
            kind: BlueprintKind::Hut,
            tile: (2, 2),
            size: (3, 3),
            phase: PiecePhase::Complete,
        }],
        cause: BuildCause::Command,
        source_record: 0,
        placed_tick: 0,
        status: BuildingStatus::Complete,
        completion_entry: None,
    };
    for y in 2..5 {
        for x in 2..5 {
            res.colony.occupied.insert((x, y), Occupant::Hut((3, 3)));
        }
    }
    res.colony.buildings.sites.insert(0, site);
    res.bed_of.insert(0, (3, 3));
    let mut skills = Skills::default();
    skills.entries[0].level = 1;
    engine.world.spawn((Agent { id: 0 }, skills));
    engine
}

fn day(engine: &mut sim_engine::SimEngine, day: u64) {
    engine.resources.current_tick = day * engine.resources.ticks_per_day;
    HouseGrowthSystem.tick(&mut engine.world, &mut engine.resources);
}

#[test]
fn house_stage_requires_live_residence_goods_and_skill_then_stays_reached() {
    let mut engine = fixture();
    for n in 1..=8 {
        day(&mut engine, n);
    }
    assert_eq!(engine.resources.colony.buildings.house_growth[&0].stage, 1);
    engine.resources.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: [(8, 8)].into_iter().collect(),
        },
    );
    engine.resources.colony.zone_of_tile.insert((8, 8), 1);
    engine
        .resources
        .colony
        .ground_items
        .insert((8, 8), ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 1));
    day(&mut engine, 9);
    assert_eq!(engine.resources.colony.buildings.house_growth[&0].stage, 2);
    engine.resources.colony.ground_items.clear();
    engine.resources.bed_of.clear();
    day(&mut engine, 10);
    assert_eq!(
        engine.resources.colony.buildings.house_growth[&0].occupied_days,
        9
    );
    assert_eq!(engine.resources.colony.buildings.house_growth[&0].stage, 2);
    assert_eq!(
        engine
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|entry| entry.kind == ChronicleKind::HouseStage)
            .count(),
        2
    );
}

#[test]
fn lost_physical_home_and_unassigned_people_never_age_a_house() {
    let mut engine = fixture();
    engine.resources.colony.occupied.remove(&(2, 2));
    day(&mut engine, 1);
    assert!(engine.resources.colony.buildings.house_growth.is_empty());
    engine
        .resources
        .colony
        .occupied
        .insert((2, 2), Occupant::Hut((3, 3)));
    engine.resources.bed_of.clear();
    day(&mut engine, 2);
    assert!(engine.resources.colony.buildings.house_growth.is_empty());
}

#[test]
fn material_wave_uses_authored_era_progress_and_threshold() {
    let mut engine = fixture();
    let catalog = installed_or_bundled();
    let era = &catalog.knowledge_defs().eras.eras[1];
    engine.resources.colony.tech.era_progress = vec![1000, 749];
    assert!(!era_ready(&engine.resources, catalog, &era.id));
    engine.resources.colony.tech.era_progress[1] = 750;
    assert!(era_ready(&engine.resources, catalog, &era.id));
    assert!(!era_ready(&engine.resources, catalog, "unknown-era"));
}

#[test]
fn occupied_upgrade_needs_a_finished_replacement_and_an_empty_house_does_not() {
    let mut engine = fixture();
    let mut settlement = sim_core::components::Settlement::new_with_id(1, 0);
    settlement.add_member_agent(0);
    engine.resources.settlements.insert(1, settlement);
    let catalog = installed_or_bundled();
    let original = engine.resources.colony.buildings.sites[&0].clone();
    assert!(!spare_beds(
        &engine.world,
        &engine.resources,
        catalog,
        &original
    ));
    let mut replacement = original.clone();
    replacement.id = 1;
    replacement.anchor = (8, 2);
    replacement.pieces[0].tile = (8, 2);
    for y in 2..5 {
        for x in 8..11 {
            engine
                .resources
                .colony
                .occupied
                .insert((x, y), Occupant::Hut((9, 3)));
        }
    }
    engine
        .resources
        .building_kinds
        .insert((9, 3), BuildingKind::Hut);
    engine.resources.building_owner.insert((9, 3), 1);
    engine
        .resources
        .colony
        .buildings
        .sites
        .insert(1, replacement);
    assert!(spare_beds(
        &engine.world,
        &engine.resources,
        catalog,
        &original
    ));
    engine
        .resources
        .colony
        .buildings
        .sites
        .get_mut(&1)
        .expect("replacement")
        .status = BuildingStatus::Building;
    assert!(!spare_beds(
        &engine.world,
        &engine.resources,
        catalog,
        &original
    ));
    engine.resources.bed_of.clear();
    assert!(spare_beds(
        &engine.world,
        &engine.resources,
        catalog,
        &original
    ));
}

#[test]
fn distant_stockpile_does_not_age_another_settlements_yard() {
    let mut engine = fixture();
    let mut other = sim_core::components::Settlement::new_with_id(2, 0);
    other.formation_tile = (14, 14);
    engine.resources.settlements.insert(2, other);
    engine.resources.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: [(14, 13)].into_iter().collect(),
        },
    );
    engine.resources.colony.zone_of_tile.insert((14, 13), 1);
    engine.resources.colony.ground_items.insert(
        (14, 13),
        ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 1),
    );
    for n in 1..=8 {
        day(&mut engine, n);
    }
    assert_eq!(engine.resources.colony.buildings.house_growth[&0].stage, 1);
    assert_eq!(stored_goods(&engine.resources).get(&2), Some(&1));
}

#[test]
fn missing_physical_replacement_beds_do_not_count_authored_capacity() {
    let mut engine = fixture();
    let original = engine.resources.colony.buildings.sites[&0].clone();
    let mut replacement = original.clone();
    replacement.id = 1;
    replacement.anchor = (8, 2);
    replacement.pieces[0].tile = (8, 2);
    for y in 2..5 {
        for x in 8..11 {
            engine
                .resources
                .colony
                .occupied
                .insert((x, y), Occupant::Hut((9, 3)));
        }
    }
    engine
        .resources
        .colony
        .buildings
        .sites
        .insert(1, replacement);
    assert!(!spare_beds(
        &engine.world,
        &engine.resources,
        installed_or_bundled(),
        &original
    ));
    engine
        .resources
        .building_kinds
        .insert((9, 3), BuildingKind::Hut);
    engine.resources.building_owner.insert((9, 3), 1);
    assert!(spare_beds(
        &engine.world,
        &engine.resources,
        installed_or_bundled(),
        &original
    ));
    for y in 0..16 {
        engine.resources.nav.set_cost((7, y), 0);
    }
    assert!(!spare_beds(
        &engine.world,
        &engine.resources,
        installed_or_bundled(),
        &original
    ));
}

#[test]
fn upgrade_requires_all_local_materials_after_existing_commitments() {
    let mut engine = fixture();
    let mut other = sim_core::components::Settlement::new_with_id(2, 0);
    other.formation_tile = (14, 14);
    engine.resources.settlements.insert(2, other);
    let needs = [(ItemKind::Log, 12)].into_iter().collect();
    let available = [(ItemKind::Log, 12)].into_iter().collect();
    engine.resources.colony.ground_items.insert(
        (14, 13),
        ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 12),
    );
    assert!(!supplies_ready(&engine.resources, 1, &needs, &available));
    engine.resources.colony.ground_items.insert(
        (1, 2),
        ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 11),
    );
    assert!(!supplies_ready(&engine.resources, 1, &needs, &available));
    engine
        .resources
        .colony
        .ground_items
        .get_mut(&(1, 2))
        .expect("local stock")
        .count = 12;
    assert!(supplies_ready(&engine.resources, 1, &needs, &available));
    assert!(!supplies_ready(
        &engine.resources,
        1,
        &needs,
        &[(ItemKind::Log, 11)].into_iter().collect()
    ));
}

#[test]
fn upgrade_targets_cover_the_house_and_expose_nonorigin_player_vetoes() {
    use crate::runtime::steward::lifecycle::targets;
    use sim_core::colony::{steward::Target, Command};
    let mut engine = fixture();
    let command = Command::UpgradeBuilding { site: 0 };
    let footprint = targets(&engine.resources, &command);
    assert_eq!(footprint.len(), 9);
    assert!(footprint.contains(&Target::Tile(2, 2)));
    assert!(footprint.contains(&Target::Tile(4, 4)));
    engine
        .resources
        .steward
        .vetoes
        .insert(Target::Tile(4, 4), 100);
    assert!(footprint
        .iter()
        .any(|target| engine.resources.steward.vetoed(target, 99)));
    assert!(!footprint
        .iter()
        .any(|target| engine.resources.steward.vetoed(target, 100)));
    assert!(targets(&engine.resources, &Command::UpgradeBuilding { site: 999 }).is_empty());
    engine
        .resources
        .colony
        .buildings
        .sites
        .get_mut(&0)
        .expect("house")
        .anchor = (15, 15);
    assert!(targets(&engine.resources, &command).is_empty());
}

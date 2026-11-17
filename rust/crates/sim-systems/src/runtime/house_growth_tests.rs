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

// Reuse the occupied hut as a finished spare home; construct the upgradeable
// shell through the real template command and physical completion callbacks.
fn ready_upgrade_fixture() -> (sim_engine::SimEngine, u32, u16) {
    use sim_core::colony::{knowledge, steward, ColonySite, Command, MaterialNeeds};
    let mut engine = fixture();
    let catalog = installed_or_bundled();
    let person = engine
        .world
        .query::<&Agent>()
        .iter()
        .next()
        .expect("resident")
        .0;
    let mut known = knowledge::Knowledge::default();
    for node in &catalog.knowledge_defs().nodes {
        known.set(node.id, 1000, knowledge::Source::Seeded, 0);
    }
    engine
        .world
        .insert_one(person, known)
        .expect("known upgrade");
    engine.resources.colony.tech.era_progress =
        vec![1000; catalog.knowledge_defs().eras.eras.len()];
    engine
        .resources
        .building_kinds
        .insert((1, 1), BuildingKind::Hearth);
    engine.resources.colony.buildings.next_site_id = 1;
    engine.resources.colony.buildings.next_piece_id = 1;
    let site = engine.resources.colony.buildings.next_site_id;
    let placed = crate::commands::apply_and_log(
        &mut engine.world,
        &mut engine.resources,
        Command::BuildTemplate {
            id: "log_house".into(),
            anchor: (8, 8),
            rotation: 0,
            station: None,
        },
        0,
    );
    assert!(placed.accepted, "{placed:?}");
    for _ in 0..4 {
        let active: Vec<_> = engine
            .world
            .query::<(&ColonySite, &MaterialNeeds)>()
            .iter()
            .map(|(entity, (site, needs))| (site.top_left, site.kind.code(), entity, *needs))
            .collect();
        let mut active = active;
        active.sort_by_key(|(tile, code, _, _)| (*tile, *code));
        for (_, _, entity, mut needs) in active {
            for line in &mut needs.lines {
                line.delivered = line.needed;
                engine
                    .resources
                    .colony
                    .item_ledger
                    .create(line.kind, u64::from(line.delivered));
            }
            engine
                .world
                .insert_one(entity, needs)
                .expect("delivered fixture inputs");
            crate::runtime::jobs::space::complete_site(
                &mut engine.world,
                &mut engine.resources,
                entity,
            );
        }
        super::super::building_templates::advance(&mut engine.world, &mut engine.resources);
        if engine.resources.colony.buildings.sites[&site].status == BuildingStatus::Complete {
            break;
        }
    }
    assert!(super::super::building_templates::is_intact(
        &engine.resources,
        &engine.resources.colony.buildings.sites[&site]
    ));
    let needs = super::super::building_templates::upgrade::material_requirements(
        &engine.world,
        &engine.resources,
        site,
    )
    .expect("eligible exact upgrade cost");
    for (index, (item, count)) in needs.into_iter().enumerate() {
        engine.resources.colony.ground_items.insert(
            (2 + index as u32, 1),
            ItemStack::of(
                ItemSpec::raw(item, None),
                u16::try_from(count).expect("small cost"),
            ),
        );
        engine
            .resources
            .colony
            .item_ledger
            .create(item, u64::from(count));
    }
    engine.resources.steward.enabled = true;
    engine.resources.current_tick = 360;
    let rule = steward::rules()
        .expect("steward rules")
        .rules
        .iter()
        .find(|rule| {
            matches!(
                rule.action,
                steward::Action::House {
                    completion_role: Some(sim_core::rooms::RoomRole::Bedroom),
                    ..
                }
            )
        })
        .expect("bedroom housing rule")
        .id;
    assert_eq!(
        upgrade_candidate(&engine.world, &engine.resources),
        Some(site)
    );
    (engine, site, rule)
}

#[test]
fn steward_upgrade_dispatch_respects_scope_cooldown_and_interior_veto_and_replays() {
    use crate::runtime::steward::StewardSystem;
    use sim_core::colony::{
        steward::{StewardScope, Target},
        Command,
    };
    let mut accepted = None;
    for (scope, cooldown, veto, should_upgrade) in [
        (StewardScope::Food, false, false, false),
        (StewardScope::Housing, true, false, false),
        (StewardScope::Housing, false, true, false),
        (StewardScope::Housing, false, false, true),
        (StewardScope::All, false, false, true),
    ] {
        let (mut engine, site, rule) = ready_upgrade_fixture();
        engine.resources.steward.scope = scope;
        let now = engine.resources.current_tick;
        if cooldown {
            engine.resources.steward.cooldowns.insert(rule, now + 1);
        }
        if veto {
            engine
                .resources
                .steward
                .vetoes
                .insert(Target::Tile(9, 9), now + 1);
        }
        // The house remains physically/materially eligible in every guarded case.
        assert_eq!(
            upgrade_candidate(&engine.world, &engine.resources),
            Some(site)
        );
        let before = engine.resources.colony.command_log.len();
        StewardSystem.tick(&mut engine.world, &mut engine.resources);
        let upgrades: Vec<_> = engine.resources.colony.command_log[before..].iter().filter(|record|
            matches!(&record.command, Command::Steward { command, .. }
                if matches!(command.as_ref(), Command::UpgradeBuilding { site: target } if *target == site)))
            .collect();
        let waves: Vec<_> = engine
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|entry| entry.kind == ChronicleKind::MaterialWave && entry.cause == site)
            .collect();
        assert_eq!(
            upgrades.len(),
            usize::from(should_upgrade),
            "scope={scope:?} cooldown={cooldown} veto={veto}"
        );
        assert_eq!(waves.len(), usize::from(should_upgrade));
        if should_upgrade {
            assert!(upgrades[0].result.accepted);
            assert!(
                matches!(&upgrades[0].command, Command::Steward { rule: actual, .. } if *actual == rule)
            );
            assert_eq!(
                engine.resources.colony.buildings.sites[&site].status,
                BuildingStatus::Building
            );
            assert_eq!(
                engine.resources.colony.buildings.sites[&site].cause,
                BuildCause::Steward(u32::from(rule))
            );
            assert!(engine.resources.steward.cooldowns[&rule] > now);
            assert!(engine
                .resources
                .steward
                .owned
                .iter()
                .any(|owned| owned.rule == rule
                    && owned.command == Command::UpgradeBuilding { site }
                    && owned.targets.contains(&Target::Tile(9, 9))));
            if scope == StewardScope::Housing {
                accepted = Some((
                    engine.resources.colony.command_log[before..].to_vec(),
                    waves[0].clone(),
                    engine.resources.colony.buildings.sites[&site].clone(),
                ));
            }
        } else {
            assert_eq!(
                engine.resources.colony.buildings.sites[&site].template,
                "log_house"
            );
            assert_eq!(
                engine.resources.colony.buildings.sites[&site].status,
                BuildingStatus::Complete
            );
        }
    }
    let (records, wave, site_after) = accepted.expect("live housing upgrade");
    let (mut replay, site, _) = ready_upgrade_fixture();
    replay.resources.steward.scope = StewardScope::Housing;
    replay.resources.steward.replay = true;
    let before = replay.resources.colony.command_log.len();
    // Replay the complete source phase, including shared stockpile/fuel rules,
    // so the upgrade keeps its exact command-log provenance index.
    replay.resources.steward.pending = records
        .iter()
        .map(|record| (record.tick, record.command.clone()))
        .collect();
    StewardSystem.tick(&mut replay.world, &mut replay.resources);
    assert_eq!(replay.resources.colony.buildings.sites[&site], site_after);
    assert_eq!(replay.resources.colony.command_log[before..], records);
    assert_eq!(
        replay
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|entry| entry.kind == ChronicleKind::MaterialWave)
            .collect::<Vec<_>>(),
        vec![&wave]
    );
}

//! Dwelling template choice uses real planner rules and exact physical commitments.
use super::*;
use sim_core::colony::{
    ItemKind, ItemSpec, ItemStack, Job, JobKind, JobStage, MaterialNeeds, ReserveKey,
};

fn fixture() -> (sim_engine::SimEngine, Action, Tile, Tile) {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let mut engine = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(4),
        sim_core::rules::WorldRules::colony(4),
    );
    let rules = sim_core::colony::steward::rules().expect("rules");
    for command in plan(&engine.world, &engine.resources, &rules.rules[0]) {
        assert!(
            crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                .accepted
        );
    }
    let hearth = hearth_tiles(&engine.resources)[0];
    engine
        .resources
        .tile_grid
        .terrain_type
        .fill(sim_core::TerrainType::Plain);
    engine.resources.colony.trees.clear();
    engine.resources.colony.nodes.clear();
    engine.resources.colony.ground_items.clear();
    engine.resources.colony.zone_of_tile.clear();
    engine.resources.colony.zones.clear();
    for (_, pos) in engine.world.query::<&mut Position>().iter() {
        pos.x = hearth.0;
        pos.y = hearth.1;
    }
    let people: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in people {
        let mut known = Knowledge::default();
        for node in &knowledge::defs().nodes {
            known.set(node.id, 1000, knowledge::Source::Discovered, 0);
        }
        engine.world.insert_one(entity, known).expect("knowledge");
    }
    let anchor = candidates(&engine.resources, hearth)
        .into_iter()
        .find(|&(x, y)| {
            distance((x, y), hearth) > 3
                && x + 4 < engine.resources.tile_grid.width
                && y + 4 < engine.resources.tile_grid.height
                && (y..y + 4).all(|yy| {
                    (x..x + 4).all(|xx| {
                        !crate::runtime::jobs::items::under_building(&engine.resources, (xx, yy))
                    })
                })
        })
        .expect("clear envelope");
    (engine, rules.rules[2].action.clone(), hearth, anchor)
}
fn supply(engine: &mut sim_engine::SimEngine, item: ItemKind, count: u16) {
    let tile = (1, 1);
    engine
        .resources
        .colony
        .ground_items
        .insert(tile, ItemStack::of(ItemSpec::raw(item, None), count));
}
fn zone(engine: &mut sim_engine::SimEngine, anchor: Tile, size: Tile) {
    engine.resources.colony.buildings.housing_zones.insert(
        41,
        (anchor.1..anchor.1 + size.1)
            .flat_map(|y| (anchor.0..anchor.0 + size.0).map(move |x| (x, y)))
            .collect(),
    );
}

#[test]
fn affordable_four_square_falls_back_to_known_hut_in_three_square_zone() {
    let (mut engine, action, hearth, anchor) = fixture();
    supply(&mut engine, ItemKind::Log, 50);
    zone(&mut engine, anchor, (3, 3));
    let result = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(
        matches!(result.commands.as_slice(),[Command::BuildTemplate{id,anchor:chosen,station:None,..}] if id=="hut" && *chosen==anchor)
    );
    assert_eq!(result.reserved.len(), 9);
    assert!(result
        .reserved
        .iter()
        .all(|tile| engine.resources.colony.buildings.housing_zones[&41].contains(tile)));
}

#[test]
fn four_square_zone_selects_highest_known_affordable_dwelling() {
    let (mut engine, action, hearth, anchor) = fixture();
    supply(&mut engine, ItemKind::Log, 50); // No stone: stone_house remains unaffordable.
    zone(&mut engine, anchor, (4, 4));
    let result = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(
        matches!(result.commands.as_slice(),[Command::BuildTemplate{id,anchor:chosen,..}] if id=="log_house" && *chosen==anchor)
    );
    assert_eq!(result.covered.len(), 16);
    assert_eq!(
        result.reserved.len(),
        16,
        "all declared floor/bed layers reserve the envelope"
    );
}

#[test]
fn physical_hut_capacity_prevents_repeat_housing_and_pending_wood_demand() {
    let (mut engine, action, hearth, anchor) = fixture();
    let agents = engine.world.query::<&Agent>().iter().count() as u32;
    let capacity = engine.resources.rules.colony.expect("colony").hut_beds;
    assert!(capacity > 0);
    for index in 0..agents.div_ceil(capacity) {
        engine.resources.building_kinds.insert(
            (anchor.0 + index, anchor.1),
            sim_core::components::BuildingKind::Hut,
        );
    }
    supply(&mut engine, ItemKind::Log, 50);
    assert!(
        house(&engine.world, &engine.resources, hearth, &action, false)
            .reserved
            .is_empty()
    );
    assert_eq!(
        pending_shelter_logs(&engine.world, &engine.resources, &[ItemKind::Log]),
        0
    );
}

#[test]
fn zero_stock_retains_original_estimator_until_material_acquisition() {
    let (mut engine, _, _, _) = fixture();
    let catalog = sim_core::colony::content::installed_or_bundled();
    assert!(template_selection::ranked(&engine.world, &engine.resources, catalog).is_empty());
    assert!(
        pending_shelter_logs(&engine.world, &engine.resources, &[ItemKind::Log]) > 0,
        "no affordability/acquisition cycle"
    );
    supply(&mut engine, ItemKind::Log, 50);
    assert!(
        template_selection::ranked(&engine.world, &engine.resources, catalog)
            .iter()
            .any(|template| template.id == "log_house")
    );
}

#[test]
fn promised_delivery_is_not_subtracted_twice_from_free_material() {
    let (mut engine, _, _, anchor) = fixture();
    supply(&mut engine, ItemKind::Log, 30);
    let from = (2, 1);
    engine
        .resources
        .colony
        .ground_items
        .insert(from, ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 5));
    engine
        .resources
        .colony
        .reservations
        .insert(ReserveKey::Stack(from), 0);
    let kind = sim_core::colony::BlueprintKind::Hut;
    let site = ColonySite {
        kind,
        top_left: anchor,
        size: (1, 1),
    };
    engine
        .world
        .spawn((site, MaterialNeeds::single(ItemKind::Log, 5)));
    engine.world.spawn((Job {
        kind: JobKind::Deliver {
            from,
            site: site.centre(),
        },
        stage: JobStage::ToSource,
        taken_at: 0,
        amount: 5,
        item: ItemKind::Log,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    },));
    assert_eq!(
        template_selection::available(&engine.world, &engine.resources, ItemKind::Log),
        30
    );
}

#[test]
fn higher_tier_tries_all_housing_geometry_before_lower_tier() {
    let (mut engine, action, hearth, anchor) = fixture();
    supply(&mut engine, ItemKind::Log, 50);
    zone(&mut engine, anchor, (3, 3));
    let other = (anchor.0 + 6, anchor.1);
    assert!(other.0 + 4 < engine.resources.tile_grid.width);
    engine.resources.colony.buildings.housing_zones.insert(
        42,
        (other.1..other.1 + 4)
            .flat_map(|y| (other.0..other.0 + 4).map(move |x| (x, y)))
            .collect(),
    );
    let result = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(matches!(result.commands.as_slice(),
        [Command::BuildTemplate { id, anchor: chosen, .. }]
        if id == "log_house" && *chosen == other));
}

#[test]
fn pending_and_replacing_reserve_once_active_and_cancelled_do_not_reserve_again() {
    use sim_core::colony::building_site::{
        BuildCause, BuildingPiece, BuildingSite, BuildingStatus, PiecePhase,
    };
    let (mut engine, _, _, anchor) = fixture();
    supply(&mut engine, ItemKind::Log, 50);
    let kind = sim_core::colony::BlueprintKind::Hut;
    let rule = engine.resources.rules.colony.expect("colony");
    let log_cost: u32 = kind
        .plan(&rule)
        .1
        .used()
        .filter(|need| need.kind == ItemKind::Log)
        .map(|need| u32::from(need.needed))
        .sum();
    assert!(log_cost > 0 && log_cost < 50);
    let mut site = BuildingSite {
        id: 0,
        template: "hut".into(),
        station: None,
        anchor,
        rotation: 0,
        footprint: kind.plan(&rule).0,
        pieces: vec![BuildingPiece {
            id: 0,
            kind,
            tile: anchor,
            size: kind.plan(&rule).0,
            phase: PiecePhase::Pending,
        }],
        cause: BuildCause::Steward(2),
        source_record: 0,
        placed_tick: 0,
        status: BuildingStatus::Building,
        completion_entry: None,
    };
    for (phase, status, expected) in [
        (PiecePhase::Pending, BuildingStatus::Building, 50 - log_cost),
        (
            PiecePhase::Replacing,
            BuildingStatus::Building,
            50 - log_cost,
        ),
        (PiecePhase::Active, BuildingStatus::Building, 50),
        (PiecePhase::Complete, BuildingStatus::Complete, 50),
        (PiecePhase::Cancelled, BuildingStatus::Cancelled, 50),
    ] {
        site.pieces[0].phase = phase;
        site.status = status;
        engine
            .resources
            .colony
            .buildings
            .sites
            .insert(0, site.clone());
        assert_eq!(
            template_selection::available(&engine.world, &engine.resources, ItemKind::Log),
            expected,
            "phase {phase:?}, status {status:?}"
        );
    }
}

#[test]
fn ranked_candidates_are_connected_known_and_stably_ordered() {
    let (mut engine, _, _, _) = fixture();
    supply(&mut engine, ItemKind::Log, 50);
    let catalog = sim_core::colony::content::installed_or_bundled();
    let ranked = template_selection::ranked(&engine.world, &engine.resources, catalog);
    assert!(!ranked.is_empty());
    assert!(ranked
        .iter()
        .all(|template| template.status == sim_core::colony::knowledge::ContentStatus::Connected));
    assert!(ranked.windows(2).all(|pair| pair[0].tier > pair[1].tier
        || (pair[0].tier == pair[1].tier && pair[0].id < pair[1].id)));
    for (_, known) in engine.world.query::<&mut Knowledge>().iter() {
        *known = Knowledge::default();
    }
    let unknown = template_selection::ranked(&engine.world, &engine.resources, catalog);
    assert!(unknown
        .iter()
        .all(|template| template.requires_knowledge.is_empty()));
}

#[test]
fn accepted_template_keeps_full_pending_reservation_until_player_cancellation() {
    let (mut engine, action, hearth, anchor) = fixture();
    supply(&mut engine, ItemKind::Log, 50);
    zone(&mut engine, anchor, (4, 4));
    let planned = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(
        matches!(planned.commands.as_slice(), [Command::BuildTemplate { id, .. }] if id == "log_house")
    );
    for command in planned.commands {
        assert!(crate::commands::submit(&mut engine, command).accepted);
    }
    let site = engine
        .resources
        .colony
        .buildings
        .sites
        .values()
        .next()
        .expect("accepted template");
    assert!(site
        .pieces
        .iter()
        .any(|piece| piece.phase == sim_core::colony::building_site::PiecePhase::Pending));
    let expected: BTreeSet<_> = (anchor.1..anchor.1 + 4)
        .flat_map(|y| (anchor.0..anchor.0 + 4).map(move |x| (x, y)))
        .collect();
    assert_eq!(template_housing::pending_tiles(&engine.resources), expected);
    assert!(expected.is_subset(&reserved_house_tiles(
        &engine.world,
        &engine.resources,
        hearth
    )));
    assert!(expected.is_subset(&reserved_field_house_tiles(
        &engine.world,
        &engine.resources,
        hearth
    )));
    assert!(
        crate::commands::submit(
            &mut engine,
            Command::CancelArea {
                x0: anchor.0,
                y0: anchor.1,
                x1: anchor.0 + 3,
                y1: anchor.1 + 3,
            }
        )
        .accepted
    );
    assert!(
        template_housing::pending_tiles(&engine.resources).is_empty(),
        "ordinary cancellation releases deferred layers"
    );
    assert!(engine
        .resources
        .colony
        .buildings
        .sites
        .values()
        .all(|site| site.completion_entry.is_none()));
}

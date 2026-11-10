//! Sequential chain stages, operating on the same live production engine.
mod history;
mod mud_bricks;
use super::spine_support::*;
use sim_core::chronicle::CauseRef;
use sim_core::colony::building_site::BuildCause;
use sim_core::colony::steward::StewardScope;
use sim_core::colony::Command;
use sim_engine::SimEngine;

pub fn grow_housing_zone(engine: &mut SimEngine) -> u32 {
    harvest_logs(engine);
    let rules = sim_core::colony::steward::rules().expect("validated steward rules");
    // Housing also enables All: establish its stockpile before choosing an envelope.
    let storage = rules
        .rules
        .iter()
        .find(|r| {
            matches!(
                r.action,
                sim_core::colony::steward::Action::Stockpile { .. }
            )
        })
        .expect("ordinary stockpile rule");
    for order in
        sim_systems::runtime::steward::planning::plan(&engine.world, &engine.resources, storage)
    {
        command(engine, order);
    }
    let rule = rules
        .rules
        .iter()
        .find(|rule| {
            rule.scope == StewardScope::Housing
                && matches!(rule.action, sim_core::colony::steward::Action::House { .. })
        })
        .expect("actual dwelling rule");
    let planned =
        sim_systems::runtime::steward::planning::plan(&engine.world, &engine.resources, rule);
    println!(
        "SPINE_HOUSING_PREFLIGHT tick={} plan={planned:?}",
        engine.current_tick()
    );
    let anchor = planned
        .iter()
        .find_map(|command| match command {
            Command::BuildTemplate { id, anchor, .. } if id == "log_house" => Some(*anchor),
            _ => None,
        })
        .expect("existing planner admits an affordable log-house envelope before zone designation");
    let zone = engine.resources.colony.next_zone_id;
    command(
        engine,
        Command::AddHousingZone {
            x0: anchor.0,
            y0: anchor.1,
            x1: anchor.0 + 3,
            y1: anchor.1 + 3,
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.housing_zones[&zone].len(),
        16
    );
    assert!(
        (anchor.1..anchor.1 + 4).all(|y| (anchor.0..anchor.0 + 4)
            .all(|x| { !engine.resources.colony.zone_of_tile.contains_key(&(x, y)) })),
        "housing must never masquerade as stockpile storage"
    );
    command(
        engine,
        Command::SetSteward {
            on: true,
            scope: StewardScope::Housing,
        },
    );
    until(engine, "steward template in housing zone", |engine| {
        engine
            .resources
            .colony
            .buildings
            .sites
            .values()
            .any(|site| site.cause == BuildCause::Zone(zone))
    });
    let site = engine
        .resources
        .colony
        .buildings
        .sites
        .values()
        .find(|site| site.cause == BuildCause::Zone(zone))
        .expect("actual zone order")
        .clone();
    assert_eq!(
        site.template, "log_house",
        "known affordable tier2 beats hut; no stone was supplied"
    );
    assert!(
        matches!(&engine.resources.colony.command_log[site.source_record as usize].command,
        Command::Steward { command, .. } if matches!(command.as_ref(), Command::BuildTemplate { .. })),
        "recorded source must be the real steward planner"
    );
    until(engine, "zone house complete", |engine| {
        complete(engine, site.id)
    });
    // A real detached designation gives native selection an unobstructed world tile.
    let tile = ui_fixture::free_tile(engine);
    command(
        engine,
        Command::AddHousingZone {
            x0: tile.0,
            y0: tile.1,
            x1: tile.0,
            y1: tile.1,
        },
    );
    checkpoint(engine, "03zonegrown", Some(site.id));
    let complete = &engine.resources.colony.buildings.sites[&site.id];
    let entry = complete.completion_entry.expect("one zone completion");
    assert_eq!(
        engine.resources.chronicle.entries()[entry as usize].cause,
        1
    );
    assert_eq!(
        engine.resources.chronicle.cause_of(entry),
        Some(CauseRef::Building {
            site: site.id,
            record: site.source_record,
            origin: BuildCause::Zone(zone),
        })
    );
    assert!(engine
        .resources
        .colony
        .rooms
        .values()
        .any(|room| room.dwelling_kind.as_deref() == Some("log_house")));
    command(
        engine,
        Command::SetSteward {
            on: false,
            scope: StewardScope::Housing,
        },
    );
    site.id
}

pub fn upgrade_house(engine: &mut SimEngine, id: u32) {
    let original = engine.resources.colony.buildings.sites[&id].clone();
    let floors = engine.resources.tile_grid.floor_material.clone();
    let beds = engine.resources.colony.beds.clone();
    mud_bricks::craft(engine);
    sim_systems::runtime::building_templates::upgrade::check(&engine.world, &engine.resources, id)
        .expect("authoritative read-only upgrade admission");
    let needs = sim_systems::runtime::building_templates::upgrade::material_requirements(
        &engine.world,
        &engine.resources,
        id,
    )
    .expect("changed wall costs");
    assert_eq!(
        needs.get(&sim_core::colony::ItemKind::from_name("mud_brick").expect("brick")),
        Some(&33)
    );
    assert!(
        !needs.contains_key(&sim_core::colony::ItemKind::Log),
        "retained doors, floor and beds must not be priced again"
    );
    checkpoint(engine, "03-ready-upgrade", Some(id));
    command(engine, Command::UpgradeBuilding { site: id });
    sim_systems::runtime::building_templates::validate_state_at_boundary(
        &engine.world,
        &engine.resources,
        engine.current_tick(),
    )
    .expect("real Replacing generation");
    let partial = sim_systems::save::save(engine).expect("save upgrading walls");
    let mut restored = sim_systems::save::load(&partial).expect("load upgrading walls");
    sim_systems::runtime::building_templates::validate_state_at_boundary(
        &restored.world,
        &restored.resources,
        restored.current_tick(),
    )
    .expect("restored replacement provenance and old-wall witnesses");
    until(engine, "mud-brick house", |engine| complete(engine, id));
    until(&mut restored, "restored mud-brick house", |engine| {
        complete(engine, id)
    });
    assert_eq!(
        engine.current_tick(),
        restored.current_tick(),
        "same upgrade completion tick"
    );
    assert_eq!(
        sim_systems::save::save(engine).expect("original upgrade bytes"),
        sim_systems::save::save(&mut restored).expect("restored upgrade bytes"),
        "exact full-state continuation from Replacing"
    );
    until(engine, "actual assigned template home", |engine| {
        ui_fixture::home(engine).is_some()
    });
    checkpoint(engine, "04upgraded", Some(id));
    assert_eq!(
        engine.resources.colony.buildings.sites[&id].template,
        "mud_brick_house"
    );
    assert_eq!(engine.resources.colony.beds, beds);
    assert_eq!(engine.resources.tile_grid.floor_material, floors);
    history::check(engine, id, &original);
}

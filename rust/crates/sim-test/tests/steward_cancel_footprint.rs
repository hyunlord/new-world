//! D24/D25: footprint cancellation and bounded target capture preserve ordinary player intent.
use sim_core::colony::steward::{StewardScope, Target};
use sim_core::colony::{BlueprintKind, Command};
use sim_engine::SimEngine;
use sim_systems::commands::{check_blueprint, submit};

fn workshop(engine: &mut SimEngine, automatic: bool) -> (u32, u32) {
    let (x, y) = (1..engine.resources.tile_grid.height)
        .flat_map(|y| (1..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .find(|&(x, y)| check_blueprint(&engine.resources, BlueprintKind::WoodBench, x, y).is_ok())
        .expect("legal two-cell workshop footprint");
    let command = Command::PlaceBlueprint {
        kind: BlueprintKind::WoodBench,
        x,
        y,
    };
    let command = if automatic {
        Command::Steward {
            rule: 4,
            command: Box::new(command),
        }
    } else {
        command
    };
    assert!(submit(engine, command).accepted);
    (x, y)
}
fn cancel_second(x: u32, y: u32) -> Command {
    Command::CancelArea {
        x0: x + 1,
        y0: y,
        x1: x + 1,
        y1: y,
    }
}
#[test]
fn non_origin_cancellation_protects_entire_workshop_and_rule_for_two_days() {
    let mut engine = sim_test::colony_engine(29);
    assert!(
        submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            }
        )
        .accepted
    );
    let (x, y) = workshop(&mut engine, true);
    assert!(submit(&mut engine, cancel_second(x, y)).accepted);
    assert!(!engine.resources.colony.occupied.contains_key(&(x, y)));
    assert!(!engine.resources.colony.occupied.contains_key(&(x + 1, y)));
    let deadline = engine.current_tick() + 2 * engine.resources.ticks_per_day;
    assert_eq!(engine.resources.steward.cooldowns[&4], deadline);
    assert!(engine
        .resources
        .steward
        .vetoed(&Target::Tile(x + 1, y), deadline - 1));
    assert!(!engine
        .resources
        .steward
        .vetoed(&Target::Tile(x + 1, y), deadline));
    let before = engine.resources.steward.clone();
    assert!(!submit(&mut engine, cancel_second(x, y)).accepted);
    assert_eq!(
        before, engine.resources.steward,
        "rejected repeat must not extend protection"
    );
    assert!(
        !submit(
            &mut engine,
            Command::PlaceBlueprint {
                kind: BlueprintKind::WoodBench,
                x: u32::MAX,
                y,
            }
        )
        .accepted
    );
    assert_eq!(
        before, engine.resources.steward,
        "invalid footprint must not create protection"
    );
}
#[test]
fn old_origin_only_provenance_still_protects_non_origin_cancellation() {
    let mut engine = sim_test::colony_engine(29);
    let (x, y) = workshop(&mut engine, true);
    engine.resources.steward.owned[0].targets = vec![Target::Tile(x, y)];
    assert!(submit(&mut engine, cancel_second(x, y)).accepted);
    assert_eq!(
        engine.resources.steward.cooldowns[&4],
        engine.current_tick() + 2 * engine.resources.ticks_per_day
    );
}
#[test]
fn ordinary_off_workshop_cancellation_does_not_create_steward_state() {
    let mut engine = sim_test::colony_engine(29);
    let before = engine.resources.steward.clone();
    let (x, y) = workshop(&mut engine, false);
    assert!(submit(&mut engine, cancel_second(x, y)).accepted);
    assert_eq!(before, engine.resources.steward);
}

fn rectangle_commands(x0: u32, y0: u32, x1: u32, y1: u32) -> [Command; 3] {
    [
        Command::AddStockpile { x0, y0, x1, y1 },
        Command::AddHuntZone { x0, y0, x1, y1 },
        Command::DesignateQuarry { x0, y0, x1, y1 },
    ]
}

#[test]
fn rectangle_targets_clip_bounded_outside_cells_in_ordinary_order() {
    let engine = SimEngine::new(4, 3, sim_core::material::MaterialRegistry::new());
    let expected = vec![
        Target::Tile(2, 1),
        Target::Tile(3, 1),
        Target::Tile(2, 2),
        Target::Tile(3, 2),
    ];
    // Only sixteen raw cells: safe to run against the defective implementation.
    for command in rectangle_commands(2, 1, 5, 4) {
        assert_eq!(
            sim_systems::runtime::steward::lifecycle::targets(&engine.resources, &command),
            expected
        );
    }
}

#[test]
fn huge_reversed_and_outside_rectangles_match_ordinary_acceptance_without_off_state() {
    for (x0, y0, x1, y1, count) in [
        (0, 0, u32::MAX, u32::MAX, 12),
        (u32::MAX, u32::MAX, 0, 0, 12),
        (u32::MAX - 1, u32::MAX - 1, u32::MAX, u32::MAX, 0),
        (5, 4, 2, 1, 4),
    ] {
        for command in rectangle_commands(x0, y0, x1, y1) {
            let make = || {
                let mut engine = SimEngine::new(4, 3, sim_core::material::MaterialRegistry::new());
                engine.resources.rules = sim_core::rules::WorldRules::colony(0);
                engine
                    .resources
                    .tile_grid
                    .terrain_type
                    .fill(sim_core::TerrainType::Plain);
                sim_systems::runtime::nav::ensure(&mut engine.resources);
                engine
            };
            let mut off = make();
            let mut on = make();
            assert!(
                submit(
                    &mut on,
                    Command::SetSteward {
                        on: true,
                        scope: StewardScope::All
                    }
                )
                .accepted
            );
            let targets =
                sim_systems::runtime::steward::lifecycle::targets(&on.resources, &command);
            assert_eq!(targets.len(), count);
            assert!(targets
                .iter()
                .all(|target| matches!(target, Target::Tile(x, y) if *x < 4 && *y < 3)));
            let before_off = off.resources.steward.clone();
            let before_on = on.resources.steward.clone();
            let ordinary = submit(&mut off, command.clone());
            if matches!(command, Command::AddStockpile { .. }) && count > 0 {
                assert!(
                    ordinary.accepted,
                    "plain small-map stockpile must be accepted"
                );
            }
            assert_eq!(submit(&mut on, command.clone()), ordinary, "{command:?}");
            assert_eq!(off.resources.steward, before_off);
            if !ordinary.accepted {
                assert_eq!(on.resources.steward, before_on);
            }
            assert_eq!(on.resources.colony.zones, off.resources.colony.zones);
            assert_eq!(
                on.resources.colony.wildlife.zones,
                off.resources.colony.wildlife.zones
            );
            assert_eq!(
                on.resources.colony.designations,
                off.resources.colony.designations
            );
        }
    }
}

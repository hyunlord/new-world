//! Steward command provenance, scope, cooldown, migration and replay invariants.
use sim_core::colony::steward::{parse_rules, StewardScope};
use sim_core::colony::Command;
use sim_systems::commands::submit;

#[test]
fn steward_is_off_in_probe_world_and_stockpile_is_an_ordinary_logged_command() {
    let mut engine = sim_test::colony_engine(0);
    assert!(!engine.resources.steward.enabled);
    let result = submit(
        &mut engine,
        Command::SetSteward {
            on: true,
            scope: StewardScope::All,
        },
    );
    assert!(result.accepted);
    engine.tick();
    assert!(!engine.resources.colony.zones.is_empty());
    assert!(engine
        .resources
        .colony
        .command_log
        .iter()
        .any(|r| matches!(r.command, Command::Steward { .. })));
    assert!(
        sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources).is_ok()
    );
}

#[test]
fn steward_data_rejects_unknown_version_and_fields() {
    assert!(parse_rules("(schema_version:999,rules:[])").is_err());
    assert!(parse_rules("(schema_version:1,rules:[],unknown:true)").is_err());
}

#[test]
fn player_cancellation_blocks_the_rule_for_two_days_and_scope_is_saved() {
    let mut engine = sim_test::colony_engine(19);
    submit(
        &mut engine,
        Command::SetSteward {
            on: true,
            scope: StewardScope::Food,
        },
    );
    engine.tick();
    let order = engine
        .resources
        .colony
        .command_log
        .iter()
        .find_map(|r| match &r.command {
            Command::Steward { rule, command }
                if matches!(**command, Command::AddStockpile { .. }) =>
            {
                Some((*rule, (**command).clone()))
            }
            _ => None,
        })
        .expect("stockpile order");
    let Command::AddStockpile { x0, y0, x1, y1 } = order.1 else {
        panic!("stockpile");
    };
    assert!(submit(&mut engine, Command::RemoveZoneArea { x0, y0, x1, y1 }).accepted);
    assert!(engine.resources.steward.cooldowns[&order.0] >= 2880);
    for _ in 0..360 {
        engine.tick();
    }
    assert!(engine.resources.colony.zones.is_empty());
    assert!(!engine.resources.colony.command_log.iter().any(|r| matches!(&r.command,Command::Steward{command,..} if matches!(**command,Command::PlaceBlueprint{kind:sim_core::colony::BlueprintKind::Bedroll,..}))));
    let bytes = sim_systems::save::save(&mut engine).expect("save");
    let mut loaded = sim_systems::save::load(&bytes).expect("load");
    assert_eq!(engine.resources.steward, loaded.resources.steward);
    assert_eq!(
        bytes,
        sim_systems::save::save(&mut loaded).expect("stable resave")
    );
    for _ in 0..360 {
        engine.tick();
        loaded.tick();
    }
    assert_eq!(
        sim_systems::save::save(&mut engine).expect("continued"),
        sim_systems::save::save(&mut loaded).expect("continued load")
    );
}

#[test]
fn source_commands_replay_after_assignment_without_duplicate_planning() {
    let mut original = sim_test::colony_engine(29);
    submit(
        &mut original,
        Command::SetSteward {
            on: true,
            scope: StewardScope::All,
        },
    );
    for _ in 0..720 {
        original.tick();
    }
    let log = original.resources.colony.command_log.clone();
    let mut replay = sim_test::colony_engine(29);
    replay.resources.steward.replay = true;
    for tick in 0..720 {
        for record in log.iter().filter(|r| r.tick == tick) {
            if matches!(record.command, Command::Steward { .. }) {
                let json = sim_test::probe::colony::command_json(tick, &record.command);
                assert_eq!(json["source"], "Steward");
                assert_eq!(
                    sim_test::probe::colony::resolve(&replay, &json, &mut Vec::new())
                        .expect("source parse"),
                    vec![record.command.clone()]
                );
                replay
                    .resources
                    .steward
                    .pending
                    .push((tick, record.command.clone()));
            } else {
                assert_eq!(submit(&mut replay, record.command.clone()), record.result);
            }
        }
        replay.tick();
    }
    replay.resources.steward.replay = false;
    assert_eq!(
        original.resources.colony.command_log,
        replay.resources.colony.command_log
    );
    assert_eq!(original.resources.steward, replay.resources.steward);
    assert_eq!(
        sim_systems::save::save(&mut original).expect("live"),
        sim_systems::save::save(&mut replay).expect("replay")
    );
}

#[test]
fn rule_data_accepts_a_new_identity_without_code_changes() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("explicit content boot before typed rule parsing");
    let text = include_str!("../../sim-core/data/colony/steward.ron");
    let changed = text
        .replace("id: 1,", "id: 71,")
        .replace("base_cells: 6", "base_cells: 7");
    let data = parse_rules(&changed).expect("data-only rule variant");
    assert_eq!(data.rules[0].id, 71);
    assert_eq!(data.rules.len(), 7);
}

#[test]
#[ignore = "requires the pre-change v6 fixtures supplied through MUX_V6_FIXTURES"]
fn real_v6_saves_migrate_off_and_continue_identically_after_v7_resave() {
    let directory = std::env::var("MUX_V6_FIXTURES").expect("fixture directory");
    for name in ["v6-new.save", "v6-orders.save"] {
        let bytes =
            std::fs::read(std::path::Path::new(&directory).join(name)).expect("real v6 bytes");
        let mut old = sim_systems::save::load(&bytes).expect("migrate v6");
        assert!(!old.resources.steward.enabled);
        assert_eq!(old.resources.steward, Default::default());
        if name == "v6-orders.save" {
            assert!(!old.resources.colony.command_log.is_empty());
        }
        let v7 = sim_systems::save::save(&mut old).expect("resave v7");
        let mut loaded = sim_systems::save::load(&v7).expect("load v7");
        for _ in 0..360 {
            old.tick();
            loaded.tick();
        }
        assert_eq!(
            sim_systems::save::save(&mut old).expect("continue migrated"),
            sim_systems::save::save(&mut loaded).expect("continue v7")
        );
    }
}

#[test]
fn bill_edits_made_while_off_remain_player_owned_after_enabling() {
    use sim_core::colony::{Bill, BillMode, BlueprintKind, RecipeKind, Workshop};
    for suspended in [false, true] {
        let mut engine = sim_test::colony_engine(29);
        engine.resources.colony.workshops.insert(
            99,
            Workshop {
                id: 99,
                kind: BlueprintKind::WoodBench,
                top_left: (1, 1),
                size: (2, 1),
                bills: vec![Bill {
                    id: 99,
                    recipe: RecipeKind::SplitFirewood,
                    mode: BillMode::Maintain(12),
                    done: 0,
                    suspended: false,
                    auto: true,
                }],
                shelf: Vec::new(),
                active: None,
            },
        );
        let command = if suspended {
            Command::SetBillSuspended {
                workshop: 99,
                bill: 99,
                suspended: true,
            }
        } else {
            Command::SetBillMode {
                workshop: 99,
                bill: 99,
                mode: BillMode::Maintain(7),
            }
        };
        assert!(submit(&mut engine, command).accepted);
        assert_eq!(
            engine.resources.steward,
            Default::default(),
            "OFF commands must not create assistant state"
        );
        let before = engine.resources.colony.workshops[&99].bills[0];
        submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All,
            },
        );
        for _ in 0..361 {
            engine.tick();
        }
        let bill = engine.resources.colony.workshops[&99]
            .bill(99)
            .expect("player bill ID retained");
        assert_eq!(bill.mode, before.mode);
        assert_eq!(bill.suspended, before.suspended);
        assert!(!engine.resources.colony.command_log.iter().any(|record|matches!(&record.command,Command::Steward{command,..} if matches!(**command,Command::AddBill{workshop:99,recipe:RecipeKind::SplitFirewood,..}))));
    }
}

#[test]
fn rejected_player_commands_do_not_change_steward_protection() {
    use sim_core::colony::{BillMode, RecipeKind};
    let mut engine = sim_test::colony_engine(29);
    submit(
        &mut engine,
        Command::SetSteward {
            on: true,
            scope: StewardScope::All,
        },
    );
    engine.tick();
    let before = engine.resources.steward.clone();
    assert!(
        !submit(
            &mut engine,
            Command::AddBill {
                workshop: u32::MAX,
                recipe: RecipeKind::SplitFirewood,
                mode: BillMode::Maintain(1)
            }
        )
        .accepted
    );
    assert_eq!(
        before, engine.resources.steward,
        "rejected bill must not create a permanent veto"
    );
    let (&tile, _) = engine
        .resources
        .colony
        .zone_of_tile
        .iter()
        .next()
        .expect("stockpile");
    assert!(
        !submit(
            &mut engine,
            Command::CancelArea {
                x0: tile.0,
                y0: tile.1,
                x1: tile.0,
                y1: tile.1
            }
        )
        .accepted
    );
    assert_eq!(
        before, engine.resources.steward,
        "failed cancellation must not pause historical source rules"
    );
}

#[test]
fn cancellation_is_protected_until_the_full_two_day_deadline_then_expires() {
    use sim_engine::RuntimeSystem;
    let mut engine = sim_test::colony_engine(29);
    submit(
        &mut engine,
        Command::SetSteward {
            on: true,
            scope: StewardScope::Food,
        },
    );
    engine.tick();
    let zone = engine.resources.colony.zones.values().next().expect("zone");
    let x0 = zone.tiles.iter().map(|t| t.0).min().expect("x");
    let x1 = zone.tiles.iter().map(|t| t.0).max().expect("x");
    let y0 = zone.tiles.iter().map(|t| t.1).min().expect("y");
    let y1 = zone.tiles.iter().map(|t| t.1).max().expect("y");
    let cancelled_at = engine.current_tick();
    assert!(submit(&mut engine, Command::RemoveZoneArea { x0, y0, x1, y1 }).accepted);
    let deadline = cancelled_at + 2 * engine.resources.ticks_per_day;
    assert_eq!(engine.resources.steward.cooldowns[&1], deadline);
    let mut system = sim_systems::runtime::steward::StewardSystem;
    for at in [cancelled_at + 360, deadline - 1] {
        engine.resources.current_tick = at;
        system.tick(&mut engine.world, &mut engine.resources);
        assert!(
            engine.resources.colony.zones.is_empty(),
            "protected at {at}"
        );
    }
    engine.resources.current_tick = deadline;
    system.tick(&mut engine.world, &mut engine.resources);
    assert!(
        !engine.resources.colony.zones.is_empty(),
        "eligible when two full days have elapsed"
    );
}

#[path = "colony_steward_housing/construction.rs"]
mod construction;

fn assert_completed_bed_assignment(engine: &mut sim_engine::SimEngine) {
    let start = engine.current_tick();
    let baseline = sim_systems::save::save(engine).expect("before completion");
    for _ in 0..1440 {
        engine.tick();
    }
    assert!(
        engine
            .resources
            .colony
            .command_log
            .iter()
            .any(|record| matches!(&record.command,
        Command::Steward { command, .. } if matches!(**command,Command::SetRoomRole{..}))
                && record.result.accepted),
        "ordinary House role completion"
    );
    assert_eq!(
        engine.resources.bed_of.len(),
        engine
            .world
            .query::<&sim_core::components::Agent>()
            .iter()
            .count()
    );
    assert!(
        engine
            .resources
            .bed_of
            .values()
            .all(|&tile| sim_systems::runtime::rooms::roofed_as(
                &engine.resources,
                tile,
                sim_core::rooms::RoomRole::Bedroom
            )),
        "actual BedAssignmentSystem prefers the enclosed beds"
    );
    println!(
        "D27 role proof: tick={} assigned_roofed_bedroom_beds={}",
        engine.current_tick(),
        engine.resources.bed_of.len()
    );
    assert_role_completion_replay_and_player_intent(engine, &baseline, start);
}

fn assert_role_completion_replay_and_player_intent(
    engine: &mut sim_engine::SimEngine,
    baseline: &[u8],
    start: u64,
) {
    let mut replay = sim_systems::save::load(baseline).expect("replay baseline");
    replay.resources.steward.replay = true;
    let end = engine.current_tick();
    for tick in start..end {
        for r in engine
            .resources
            .colony
            .command_log
            .iter()
            .filter(|r| r.tick == tick)
        {
            if matches!(r.command, Command::Steward { .. }) {
                let json = sim_test::probe::colony::command_json(tick, &r.command);
                assert_eq!(
                    sim_test::probe::colony::resolve(&replay, &json, &mut Vec::new())
                        .expect("source role parse"),
                    vec![r.command.clone()]
                );
                replay
                    .resources
                    .steward
                    .pending
                    .push((tick, r.command.clone()));
            } else {
                assert_eq!(submit(&mut replay, r.command.clone()), r.result);
            }
        }
        replay.tick();
    }
    replay.resources.steward.replay = false;
    assert_eq!(
        sim_systems::save::save(engine).expect("live"),
        sim_systems::save::save(&mut replay).expect("replay")
    );
    assert_eq!(engine.resources.colony.command_log.iter().filter(|r| matches!(&r.command,Command::Steward{command,..} if matches!(**command,Command::SetRoomRole{..}))).count(),1,"completion is not duplicated");
    for scope in [StewardScope::Food, StewardScope::Housing] {
        let mut gated = sim_systems::save::load(baseline).expect("gated baseline");
        assert!(submit(&mut gated, Command::SetSteward { on: true, scope }).accepted);
        if scope == StewardScope::Housing {
            gated.resources.steward.cooldowns.insert(3, start + 1440);
        }
        for _ in 0..1440 {
            gated.tick();
        }
        assert!(!gated.resources.colony.command_log.iter().any(|r| matches!(&r.command,Command::Steward{command,..} if matches!(**command,Command::SetRoomRole{..}))),"scope or cooldown gates completion");
        if scope == StewardScope::Housing {
            for _ in 0..360 {
                gated.tick();
            }
            assert!(gated.resources.colony.command_log.iter().any(|r| matches!(&r.command,Command::Steward{command,..} if matches!(**command,Command::SetRoomRole{..}))));
        }
    }
    use sim_core::rooms::RoomRole;
    for (role, clear_after) in [
        (None, false),
        (Some(RoomRole::Workshop), false),
        (Some(RoomRole::Bedroom), false),
        (Some(RoomRole::Workshop), true),
    ] {
        let mut manual = sim_systems::save::load(baseline).expect("manual baseline");
        assert!(
            submit(
                &mut manual,
                Command::SetSteward {
                    on: false,
                    scope: StewardScope::All
                }
            )
            .accepted
        );
        let room = *manual.resources.colony.rooms.keys().next().expect("room");
        let result = submit(&mut manual, Command::SetRoomRole { room, role });
        assert!(result.accepted || result.reason == Some(sim_core::colony::RejectReason::Nothing));
        if clear_after {
            assert!(submit(&mut manual, Command::SetRoomRole { room, role: None }).accepted);
        }
        let bytes = sim_systems::save::save(&mut manual).expect("manual save");
        let mut restored = sim_systems::save::load(&bytes).expect("manual load");
        assert!(
            submit(
                &mut restored,
                Command::SetSteward {
                    on: true,
                    scope: StewardScope::Housing
                }
            )
            .accepted
        );
        for _ in 0..1440 {
            restored.tick();
        }
        assert!(!restored.resources.colony.command_log.iter().any(|r|matches!(&r.command,Command::Steward{command,..} if matches!(**command,Command::SetRoomRole{..}))),"manual role including Auto persists");
    }
}

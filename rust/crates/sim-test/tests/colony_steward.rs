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
    // The cancelled stockpile rule must not re-fire within the cooldown; the food
    // plan's independent single-tile zones (rule 8 storage expansion) are allowed.
    assert!(!engine.resources.colony.command_log.iter().any(|r| {
        r.tick > 0
            && r.result.accepted
            && matches!(&r.command, Command::Steward { rule, command }
                if *rule == order.0
                    && matches!(**command, Command::AddStockpile { .. }))
    }));
    assert!(!engine.resources.colony.command_log.iter().any(|r| matches!(&r.command,
        Command::Steward { command, .. } if match &**command {
            Command::PlaceBlueprint { kind: sim_core::colony::BlueprintKind::Bedroll, .. } => true,
            Command::BuildTemplate { id, .. } => sim_core::colony::content::installed_or_bundled()
                .building_templates().get(id).is_some_and(|template| template.category == sim_core::colony::building_template::BuildingCategory::Dwelling),
            _ => false,
        })), "Food scope must not request either legacy beds or template dwellings");
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
    assert_eq!(data.rules.len(), 10);
    let mut expected = parse_rules(text).expect("bundled rules");
    expected.rules[0].id = 71;
    expected.rules[0].action = sim_core::colony::steward::Action::Stockpile {
        base_cells: 7,
        divisor: 2,
    };
    assert_eq!(
        data, expected,
        "data edits preserve every other rule and field"
    );
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
    assert_template_completions(engine);
    assert_eq!(
        engine.resources.bed_of.len(),
        engine
            .world
            .query::<&sim_core::components::Agent>()
            .iter()
            .count()
    );
    let capacities = construction::physical_capacity(engine);
    let mut assigned = std::collections::BTreeMap::new();
    for (_, agent) in engine.world.query::<&sim_core::components::Agent>().iter() {
        let tile = engine
            .resources
            .bed_of
            .get(&agent.id)
            .expect("each living agent has a bed");
        *assigned.entry(*tile).or_insert(0usize) += 1;
    }
    for (tile, count) in assigned {
        assert!(capacities.get(&tile).is_some_and(|capacity| count <= *capacity),
            "assignment {tile:?} count={count} must fit a physically intact reachable Hut or enclosed bedroom bed: {capacities:?}");
    }
    println!(
        "physical housing proof: tick={} assigned={} capacities={capacities:?}",
        engine.current_tick(),
        engine.resources.bed_of.len()
    );
    assert_role_completion_replay_and_player_intent(engine, &baseline, start);
}

fn assert_template_completions(engine: &sim_engine::SimEngine) {
    use sim_core::{
        chronicle::{CauseRef, ChronicleKind},
        colony::{content::installed_or_bundled, BuildingStatus},
    };
    let catalog = installed_or_bundled().building_templates();
    let completed: Vec<_> = engine
        .resources
        .colony
        .buildings
        .sites
        .values()
        .filter(|site| {
            site.status == BuildingStatus::Complete
                && catalog.get(&site.template).is_some_and(|template| {
                    template.category
                        == sim_core::colony::building_template::BuildingCategory::Dwelling
                })
        })
        .collect();
    assert!(
        !completed.is_empty(),
        "real template construction must finish"
    );
    for site in completed {
        assert!(sim_systems::runtime::building_templates::is_intact(
            &engine.resources,
            site
        ));
        let index = site.completion_entry.expect("completion chronicle");
        let entry = engine
            .resources
            .chronicle
            .entry(index)
            .expect("retained completion");
        assert_eq!(entry.kind, ChronicleKind::BuildingComplete);
        assert_eq!(
            entry.value,
            catalog
                .buildings
                .iter()
                .position(|template| template.id == site.template)
                .expect("known template") as i64
        );
        let expected = CauseRef::Building {
            site: site.id,
            record: site.source_record,
            origin: site.cause,
        };
        assert_eq!(engine.resources.chronicle.cause_of(index), Some(expected));
        assert_eq!(
            engine
                .resources
                .chronicle
                .causes()
                .filter(|(_, cause)| *cause == expected)
                .count(),
            1,
            "one completion per site generation"
        );
        let source = &engine.resources.colony.command_log[site.source_record as usize];
        assert!(
            source.result.accepted
                && matches!(&source.command, Command::Steward { rule: 3, command } if matches!(&**command, Command::BuildTemplate { id, .. } if *id == site.template))
        );
    }
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
    assert_template_completions(engine);
    assert_template_completions(&replay);
    // Legacy role-completion scope/cooldown dispatch is independently covered by
    // completion_tests::legacy_role_completion_dispatch_respects_scope_and_cooldown.
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
        let intent_tick = manual.current_tick();
        let result = submit(&mut manual, Command::SetRoomRole { room, role });
        assert!(result.accepted || result.reason == Some(sim_core::colony::RejectReason::Nothing));
        if clear_after {
            assert!(submit(&mut manual, Command::SetRoomRole { room, role: None }).accepted);
        }
        let expected_overrides = manual.resources.colony.room_overrides.clone();
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
        assert_eq!(
            restored.resources.colony.room_overrides, expected_overrides,
            "saved player override including cleared Auto survives stewardship"
        );
        let actual = &restored.resources.colony.rooms[&room];
        let expected_role = if clear_after { None } else { role }
            .unwrap_or_else(|| sim_systems::runtime::rooms::auto_role(&restored.resources, actual));
        assert_eq!(
            actual.role, expected_role,
            "actual room honors saved explicit or Auto role"
        );
        assert!(!restored.resources.colony.command_log.iter().any(|r| r.tick >= intent_tick && matches!(&r.command,
            Command::Steward { command, .. } if matches!(**command, Command::SetRoomRole { room: target, .. } if target == room))),
            "steward must not overwrite the player's selected Room after explicit or Auto intent");
    }
}

/// Stage 1 keeps survival tools immediate and separates seasonal fuel production.
#[test]
fn tools_rule_fires_immediately_without_grace() {
    let mut engine = sim_test::colony_engine(0);
    submit(
        &mut engine,
        Command::SetSteward {
            on: true,
            scope: StewardScope::All,
        },
    );
    // Tick through the first steward interval (360 ticks).
    for _ in 0..360 {
        engine.tick();
    }
    // Rule 4 covers tools without a grace window.
    let production_fired = engine
        .resources
        .colony
        .command_log
        .iter()
        .any(|r| matches!(&r.command, Command::Steward { rule, .. } if *rule == 4));
    assert!(
        production_fired,
        "tools production must fire within the first steward interval"
    );
}

#[test]
fn teaching_is_player_requested_while_legacy_rule_remains_readable() {
    use sim_core::colony::steward::{rules, Action, Condition};

    let _engine = sim_test::colony_engine(0);
    let current = rules().expect("bundled policy");
    assert!(current.rules.iter().all(|rule| {
        rule.id != 6
            && rule.condition != Condition::KnowledgeRisk
            && !matches!(rule.action, Action::Teach)
    }));

    // Removing default automation must not remove the retained rule schema or IDs.
    let legacy = parse_rules(include_str!("fixtures/m6b_base_steward.ron"))
        .expect("published BASE policy remains readable");
    let teaching = legacy
        .rules
        .iter()
        .find(|rule| rule.id == 6)
        .expect("historical teaching rule");
    assert_eq!(teaching.condition, Condition::KnowledgeRisk);
    assert_eq!(teaching.action, Action::Teach);
}

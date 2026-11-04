//! M-UX2 automatic defence: real runtime registration, player precedence and safety.
mod colony_support;
use colony_support::adults;
use sim_core::colony::people::{DraftOrder, Drafted};
use sim_core::colony::steward::StewardScope;
use sim_core::colony::{Command, ItemKind, ItemSpec, Tool};
use sim_core::components::{BodyHealth, Position};
use sim_systems::commands::submit;

fn armed_colony() -> sim_engine::SimEngine {
    let mut engine = sim_test::colony_engine(0);
    engine.tick();
    for (_, entity) in adults(&engine) {
        engine
            .world
            .insert_one(
                entity,
                Tool {
                    item: ItemSpec::raw(ItemKind::Spear, None),
                    uses: 0,
                },
            )
            .expect("arm fixture");
        engine
            .resources
            .colony
            .item_ledger
            .create(ItemKind::Spear, 1);
    }
    engine
}
fn incoming(engine: &mut sim_engine::SimEngine) {
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let at = (
        hearth
            .0
            .saturating_add(20)
            .min(engine.resources.tile_grid.width - 1),
        hearth.1,
    );
    sim_systems::runtime::wildlife::spawn_pack(&mut engine.resources, at, 4, 42);
    assert!(
        submit(
            engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            }
        )
        .accepted
    );
}

#[test]
fn incoming_pack_drafts_four_armed_adults_before_the_next_slow_planning_tick() {
    // Given armed adults and a newly spawned approaching pack after tick zero.
    let mut engine = armed_colony();
    incoming(&mut engine);
    // When the ordinary runtime advances one tick.
    engine.tick();
    // Then fast defence is registered and sends exactly four adults to attack.
    let orders: Vec<_> = engine
        .world
        .query::<&Drafted>()
        .iter()
        .map(|(_, d)| d.order)
        .collect();
    assert_eq!(orders.len(), 4);
    assert!(orders
        .iter()
        .all(|order| matches!(order, DraftOrder::Attack(_))));
}

#[test]
fn explicit_manual_undraft_while_off_protects_the_person_when_steward_is_enabled() {
    // Given a player explicitly leaves an already undrafted adult free while OFF.
    let mut engine = armed_colony();
    let (id, entity) = adults(&engine)[0];
    let result = submit(
        &mut engine,
        Command::SetDrafted {
            agent: id,
            drafted: false,
        },
    );
    // When defence is enabled and the pack approaches.
    incoming(&mut engine);
    engine.tick();
    // Then the explicit choice is accepted and cannot be overwritten by automation.
    assert!(
        result.accepted,
        "an explicit no-op is still a manual choice"
    );
    assert!(engine.world.get::<&Drafted>(entity).is_err());
    assert_eq!(engine.world.query::<&Drafted>().iter().count(), 4);
}

#[test]
fn defence_excludes_low_health_and_ranks_weapon_health_then_distance() {
    // Given two ineligible low-health adults and healthy armed alternatives.
    let mut engine = armed_colony();
    let people = adults(&engine);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    for (index, (_, entity)) in people.iter().enumerate() {
        engine
            .world
            .get::<&mut BodyHealth>(*entity)
            .expect("health")
            .hp = if index < 2 { 49.0 } else { 100.0 };
        let mut position = engine
            .world
            .get::<&mut Position>(*entity)
            .expect("position");
        position.x = hearth.0 + (index as u32 % 3);
        position.y = hearth.1;
    }
    incoming(&mut engine);
    // When one ordinary tick runs.
    engine.tick();
    // Then no low-health person is drafted and the cap is respected.
    assert!(people
        .iter()
        .take(2)
        .all(|(_, entity)| engine.world.get::<&Drafted>(*entity).is_err()));
    assert_eq!(
        engine.world.query::<&Drafted>().iter().count(),
        4.min(people.len() - 2)
    );
}

fn defence_tick(engine: &mut sim_engine::SimEngine, tick: u64) {
    use sim_engine::RuntimeSystem;
    engine.resources.current_tick = tick;
    sim_systems::runtime::steward::defence::StewardDefenceSystem
        .tick(&mut engine.world, &mut engine.resources);
}

#[test]
fn manual_protection_expires_at_exactly_one_day_and_manual_drafts_stay_owned() {
    let mut engine = armed_colony();
    let (id, entity) = adults(&engine)[0];
    assert!(
        submit(
            &mut engine,
            Command::SetDrafted {
                agent: id,
                drafted: false
            }
        )
        .accepted
    );
    let until = engine.resources.steward.defence.manual_until[&id];
    incoming(&mut engine);
    defence_tick(&mut engine, until - 1);
    assert!(engine.world.get::<&Drafted>(entity).is_err());
    // Free a slot without creating another player veto for this protected adult.
    let owned = engine.resources.steward.defence.owned_ids.clone();
    for person in owned {
        assert!(
            sim_systems::commands::apply_command(
                &mut engine.world,
                &mut engine.resources,
                Command::SetDrafted {
                    agent: person,
                    drafted: false
                }
            )
            .accepted
        );
    }
    assert!(engine
        .resources
        .steward
        .defence
        .manual_until
        .iter()
        .all(|(person, deadline)| *person == id || *deadline > until));
    for animal in engine
        .resources
        .colony
        .wildlife
        .animals
        .values_mut()
        .filter(|a| a.pack.is_some())
    {
        animal.pack = Some(43);
    }
    defence_tick(&mut engine, until);
    assert!(
        !engine
            .resources
            .steward
            .defence
            .manual_until
            .contains_key(&id),
        "protection expires exactly at its exclusive deadline"
    );
    assert!(engine.world.get::<&Drafted>(entity).is_ok());
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::SetDrafted {
                agent: id,
                drafted: true
            }
        )
        .accepted
    );
    assert!(!engine.resources.steward.defence.owned_ids.contains(&id));
    assert!(
        sim_systems::commands::apply_command(
            &mut engine.world,
            &mut engine.resources,
            Command::SetStewardDefence { on: false }
        )
        .accepted
    );
    defence_tick(&mut engine, until + 1);
    assert!(
        engine.world.get::<&Drafted>(entity).is_ok(),
        "manual draft survives automatic release"
    );
}

#[test]
fn quiet_sixty_ticks_releases_distant_pack_then_rearms_when_it_enters_radius() {
    let mut engine = armed_colony();
    incoming(&mut engine);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    for animal in engine
        .resources
        .colony
        .wildlife
        .animals
        .values_mut()
        .filter(|a| a.pack.is_some())
    {
        animal.pos = (hearth.0 + 25, hearth.1);
    }
    defence_tick(&mut engine, 10);
    assert_eq!(engine.resources.steward.defence.owned_ids.len(), 4);
    defence_tick(&mut engine, 69);
    assert_eq!(engine.resources.steward.defence.owned_ids.len(), 4);
    defence_tick(&mut engine, 70);
    assert!(engine.resources.steward.defence.owned_ids.is_empty());
    defence_tick(&mut engine, 71);
    assert!(
        engine.resources.steward.defence.owned_ids.is_empty(),
        "same distant pack stays retired"
    );
    for animal in engine
        .resources
        .colony
        .wildlife
        .animals
        .values_mut()
        .filter(|a| a.pack.is_some())
    {
        animal.pos = (hearth.0 + 24, hearth.1);
    }
    defence_tick(&mut engine, 72);
    assert_eq!(engine.resources.steward.defence.owned_ids.len(), 4);
    for animal in engine
        .resources
        .colony
        .wildlife
        .animals
        .values_mut()
        .filter(|a| a.pack.is_some())
    {
        animal.state = sim_core::colony::wildlife::AnimalState::Retreat;
    }
    defence_tick(&mut engine, 73);
    assert!(engine.resources.steward.defence.owned_ids.is_empty());
}

#[test]
fn independent_toggle_and_chronicle_report_actual_batch_without_replacing_sequence() {
    let mut engine = armed_colony();
    incoming(&mut engine);
    let sequence = engine.resources.steward.sequence;
    assert!(submit(&mut engine, Command::SetStewardDefence { on: false }).accepted);
    defence_tick(&mut engine, 10);
    assert!(engine.resources.steward.enabled);
    assert!(engine.resources.steward.defence.owned_ids.is_empty());
    assert!(submit(&mut engine, Command::SetStewardDefence { on: true }).accepted);
    defence_tick(&mut engine, 11);
    assert_eq!(engine.resources.steward.sequence, sequence);
    assert_eq!(engine.resources.steward.defence.last_count, 4);
    let events: Vec<_> = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|e| e.kind == sim_core::chronicle::ChronicleKind::StewardDefence)
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!((events[0].cause, events[0].value), (7, 4));
}

#[test]
fn defence_save_roundtrip_keeps_owned_people_manual_veto_and_quiet_deadline() {
    let mut engine = armed_colony();
    let (id, _) = adults(&engine)[0];
    assert!(
        submit(
            &mut engine,
            Command::SetDrafted {
                agent: id,
                drafted: false
            }
        )
        .accepted
    );
    incoming(&mut engine);
    engine.tick();
    let state = engine.resources.steward.clone();
    let bytes = sim_systems::save::save(&mut engine).expect("save defence");
    let mut restored = sim_systems::save::load(&bytes).expect("load defence");
    assert_eq!(restored.resources.steward, state);
    engine.tick();
    restored.tick();
    assert_eq!(
        sim_systems::save::save(&mut engine).expect("continued"),
        sim_systems::save::save(&mut restored).expect("restored continuation")
    );
}

#[test]
fn only_weapon_capable_adults_are_selected_and_one_loose_weapon_is_not_overclaimed() {
    let mut engine = armed_colony();
    for (_, entity) in adults(&engine) {
        engine
            .world
            .remove_one::<Tool>(entity)
            .expect("remove fixture weapon");
    }
    engine.resources.colony.ground_items.retain(|_, stack| {
        stack
            .kind
            .definition()
            .weapon
            .as_ref()
            .and_then(|w| w.draft_rank)
            .is_none()
    });
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    assert!(engine.resources.steward.defence.owned_ids.is_empty());
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let tile = (hearth.0 + 1, hearth.1);
    engine.resources.colony.ground_items.insert(
        tile,
        sim_core::colony::ItemStack::of(ItemSpec::raw(ItemKind::Spear, None), 1),
    );
    for animal in engine
        .resources
        .colony
        .wildlife
        .animals
        .values_mut()
        .filter(|a| a.pack.is_some())
    {
        animal.pack = Some(43);
    }
    defence_tick(&mut engine, 11);
    assert_eq!(engine.resources.steward.defence.owned_ids.len(), 1);
    let drafts: Vec<_> = engine
        .world
        .query::<&Drafted>()
        .iter()
        .map(|(_, d)| d.arming)
        .collect();
    assert_eq!(drafts, vec![Some(tile)]);
    defence_tick(&mut engine, 12);
    assert_eq!(
        engine.resources.steward.defence.owned_ids.len(),
        1,
        "already promised weapon is unavailable"
    );
}

#[test]
fn deterministic_rank_prefers_weapon_then_precise_health_distance_and_id() {
    let mut engine = armed_colony();
    let people = adults(&engine);
    assert!(people.len() >= 6);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    for (index, (_, entity)) in people.iter().enumerate() {
        let mut hp = engine
            .world
            .get::<&mut BodyHealth>(*entity)
            .expect("health");
        hp.hp = if index < 6 { 70.0 } else { 49.0 };
        drop(hp);
        let mut pos = engine
            .world
            .get::<&mut Position>(*entity)
            .expect("position");
        pos.x = hearth.0 + 1;
        pos.y = hearth.1;
    }
    // A healthier nearby axe loses to every eligible spear.
    engine
        .world
        .insert_one(
            people[0].1,
            Tool {
                item: ItemSpec::raw(ItemKind::StoneAxe, None),
                uses: 0,
            },
        )
        .expect("axe");
    engine
        .world
        .get::<&mut BodyHealth>(people[0].1)
        .expect("health")
        .hp = 100.0;
    // Fractional health is retained in rank; distance precedes id only after health.
    engine
        .world
        .get::<&mut BodyHealth>(people[1].1)
        .expect("health")
        .hp = 70.5;
    engine
        .world
        .get::<&mut Position>(people[1].1)
        .expect("position")
        .x = hearth.0 + 10;
    engine
        .world
        .get::<&mut Position>(people[5].1)
        .expect("position")
        .x = hearth.0 + 2;
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    let selected = &engine.resources.steward.defence.owned_ids;
    assert_eq!(
        selected.iter().copied().collect::<Vec<_>>(),
        people[1..5].iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
}

#[test]
fn fast_source_replay_matches_recruitment_and_release_without_duplicate_planning() {
    let mut live = armed_colony();
    let mut replay = armed_colony();
    incoming(&mut live);
    incoming(&mut replay);
    replay.resources.steward.replay = true;
    for engine in [&mut live, &mut replay] {
        let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
        for animal in engine
            .resources
            .colony
            .wildlife
            .animals
            .values_mut()
            .filter(|a| a.pack.is_some())
        {
            animal.pos = (hearth.0 + 25, hearth.1);
        }
    }
    for tick in [10, 69, 70, 71] {
        defence_tick(&mut live, tick);
        replay.resources.steward.pending.extend(
            live.resources
                .colony
                .command_log
                .iter()
                .filter(|record| {
                    record.tick == tick && matches!(record.command, Command::Steward { .. })
                })
                .map(|record| (record.tick, record.command.clone())),
        );
        defence_tick(&mut replay, tick);
        let mut state = replay.resources.steward.clone();
        state.replay = false;
        assert_eq!(live.resources.steward, state, "state tick {tick}");
        assert_eq!(
            live.resources.chronicle.entries(),
            replay.resources.chronicle.entries(),
            "chronicle tick {tick}"
        );
        assert_eq!(
            live.resources.colony.command_log, replay.resources.colony.command_log,
            "log tick {tick}"
        );
    }
}

#[test]
fn ordinary_critical_need_undraft_survives_without_replacement_recruitment() {
    use sim_engine::RuntimeSystem;
    let mut engine = armed_colony();
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    let id = *engine
        .resources
        .steward
        .defence
        .owned_ids
        .iter()
        .next()
        .expect("defender");
    let entity = adults(&engine)
        .into_iter()
        .find(|(person, _)| *person == id)
        .expect("person")
        .1;
    engine
        .world
        .get::<&mut sim_core::components::Hunger>(entity)
        .expect("hunger")
        .value = 100.0;
    engine.resources.current_tick = 11;
    sim_systems::runtime::wildlife::DraftSystem.tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.get::<&Drafted>(entity).is_err());
    defence_tick(&mut engine, 11);
    assert!(engine.world.get::<&Drafted>(entity).is_err());
    assert_eq!(engine.resources.steward.defence.owned_ids.len(), 3);
    defence_tick(&mut engine, 12);
    assert_eq!(
        engine.resources.steward.defence.owned_ids.len(),
        3,
        "response does not refill replacements"
    );
}

#[test]
fn armed_children_and_rejected_manual_orders_do_not_gain_defence_ownership() {
    let mut engine = armed_colony();
    let (id, entity) = adults(&engine)[0];
    engine
        .world
        .insert_one(entity, sim_core::components::Child { born_tick: 0 })
        .expect("child fixture");
    assert!(
        !submit(
            &mut engine,
            Command::SetDrafted {
                agent: id,
                drafted: true
            }
        )
        .accepted
    );
    assert!(!engine
        .resources
        .steward
        .defence
        .manual_until
        .contains_key(&id));
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    assert!(engine.world.get::<&Drafted>(entity).is_err());
    assert!(!engine.resources.steward.defence.owned_ids.contains(&id));
}

#[test]
fn distinct_loose_weapon_stacks_arm_a_full_batch_without_double_claims() {
    let mut engine = armed_colony();
    let people = adults(&engine);
    for (_, entity) in &people {
        engine
            .world
            .remove_one::<Tool>(*entity)
            .expect("remove fixture weapon");
    }
    engine.resources.colony.ground_items.retain(|_, stack| {
        stack
            .kind
            .definition()
            .weapon
            .as_ref()
            .and_then(|w| w.draft_rank)
            .is_none()
    });
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let region = engine.resources.nav.region_of((hearth.0 + 1, hearth.1));
    let tiles: Vec<_> = (hearth.1.saturating_sub(2)..=hearth.1 + 2)
        .flat_map(|y| (hearth.0.saturating_sub(2)..=hearth.0 + 2).map(move |x| (x, y)))
        .filter(|tile| {
            engine.resources.nav.passable(*tile) && engine.resources.nav.reaches(region, *tile)
        })
        .take(4)
        .collect();
    assert_eq!(tiles.len(), 4);
    for tile in tiles {
        engine.resources.colony.ground_items.insert(
            tile,
            sim_core::colony::ItemStack::of(ItemSpec::raw(ItemKind::Spear, None), 1),
        );
    }
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    let destinations: Vec<_> = engine
        .world
        .query::<&Drafted>()
        .iter()
        .filter_map(|(_, d)| d.arming)
        .collect();
    assert_eq!(destinations.len(), 4);
    assert_eq!(
        destinations
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
}

#[test]
fn asymmetric_weapon_reach_reassigns_flexible_adult_so_four_can_defend() {
    use sim_core::nav::{CostMap, COST_PLAIN};
    let mut engine = armed_colony();
    let people = adults(&engine);
    assert!(people.len() >= 4);
    let mut map = CostMap::new(
        engine.resources.tile_grid.width,
        engine.resources.tile_grid.height,
    );
    for y in 0..engine.resources.tile_grid.height {
        for x in 0..engine.resources.tile_grid.width {
            map.set((x, y), COST_PLAIN);
        }
    }
    engine.resources.nav.install(map);
    engine
        .resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .wildlife
        .as_mut()
        .expect("wildlife")
        .arm_reach = 12;
    for (index, (_, entity)) in people.iter().enumerate() {
        engine
            .world
            .get::<&mut BodyHealth>(*entity)
            .expect("health")
            .hp = match index {
            0 => 100.0,
            1 => 90.0,
            2 | 3 => 100.0,
            _ => 49.0,
        };
    }
    let flexible = people[0].1;
    let constrained = people[1].1;
    engine
        .world
        .remove_one::<Tool>(flexible)
        .expect("unarm flexible");
    engine
        .world
        .remove_one::<Tool>(constrained)
        .expect("unarm constrained");
    for (entity, x) in [(flexible, 24), (constrained, 12)] {
        let mut pos = engine.world.get::<&mut Position>(entity).expect("position");
        pos.x = x;
        pos.y = 24;
    }
    engine.resources.colony.ground_items.clear();
    for tile in [(24, 24), (36, 24)] {
        engine.resources.colony.ground_items.insert(
            tile,
            sim_core::colony::ItemStack::of(ItemSpec::raw(ItemKind::Spear, None), 1),
        );
    }
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    assert_eq!(
        engine.resources.steward.defence.owned_ids.len(),
        4,
        "flexible adult must not monopolize the constrained adult's only reachable spear"
    );
    assert_eq!(
        engine
            .world
            .get::<&Drafted>(flexible)
            .expect("flexible defender")
            .arming,
        Some((36, 24))
    );
    assert_eq!(
        engine
            .world
            .get::<&Drafted>(constrained)
            .expect("constrained defender")
            .arming,
        Some((24, 24))
    );
}

#[test]
fn exported_replay_file_preserves_both_ordinary_and_fast_source_phases() {
    let mut original = armed_colony();
    incoming(&mut original);
    let start = original.current_tick();
    let until = start + 720;
    while original.current_tick() < until {
        original.tick();
    }
    let records = original.resources.colony.command_log.clone();
    let rows: Vec<_> = records
        .iter()
        .map(|record| sim_test::probe::colony::command_json(record.tick, &record.command))
        .collect();
    let file = std::env::temp_dir().join(format!(
        "worldsim-defence-source-phases-{}.json",
        std::process::id()
    ));
    std::fs::write(
        &file,
        serde_json::to_vec(&rows).expect("encode replay file"),
    )
    .expect("write replay file");
    let loaded: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&file).expect("read replay file"))
            .expect("decode replay file");
    std::fs::remove_file(file).expect("remove temporary replay file");
    let phases: std::collections::BTreeSet<_> = loaded
        .iter()
        .filter_map(|row| row["phase"].as_str())
        .collect();
    assert_eq!(
        phases,
        std::collections::BTreeSet::from(["after_assignment", "before_flight"])
    );

    let mut replay = armed_colony();
    incoming(&mut replay);
    // Keep the same world fixture, then let the exported player command restore policy.
    replay.resources.steward = Default::default();
    replay.resources.steward.replay = true;
    replay.resources.colony.command_log.clear();
    let mut consumed = 0;
    while replay.current_tick() < until {
        let tick = replay.current_tick();
        for row in loaded
            .iter()
            .filter(|row| row["tick"].as_u64() == Some(tick))
        {
            let commands = sim_test::probe::colony::resolve(&replay, row, &mut Vec::new())
                .expect("resolve exported command");
            assert_eq!(commands, vec![records[consumed].command.clone()]);
            if row["source"] == "Steward" {
                let mut wrong_phase = row.clone();
                wrong_phase["phase"] = serde_json::json!(if row["phase"] == "before_flight" {
                    "after_assignment"
                } else {
                    "before_flight"
                });
                assert!(
                    sim_test::probe::colony::resolve(&replay, &wrong_phase, &mut Vec::new())
                        .is_err()
                );
            }
            for command in commands {
                if matches!(command, Command::Steward { .. }) {
                    replay.resources.steward.pending.push((tick, command));
                } else {
                    assert_eq!(submit(&mut replay, command), records[consumed].result);
                }
            }
            consumed += 1;
        }
        replay.tick();
    }
    assert_eq!(consumed, records.len(), "every exported row replays once");
    assert!(replay.resources.steward.pending.is_empty());
    replay.resources.steward.replay = false;
    assert_eq!(original.resources.steward, replay.resources.steward);
    assert_eq!(
        original.resources.colony.command_log,
        replay.resources.colony.command_log
    );
    assert_eq!(
        original.resources.chronicle.entries(),
        replay.resources.chronicle.entries()
    );
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(&original),
        sim_test::probe::colony::colony_fingerprint(&replay)
    );
}

#[test]
fn final_team_prefers_four_spears_over_reassignment_to_three_spears_and_an_axe() {
    use sim_core::nav::{CostMap, COST_PLAIN};
    let mut engine = armed_colony();
    let people = adults(&engine);
    assert!(people.len() >= 5);
    let mut map = CostMap::new(
        engine.resources.tile_grid.width,
        engine.resources.tile_grid.height,
    );
    for y in 0..engine.resources.tile_grid.height {
        for x in 0..engine.resources.tile_grid.width {
            map.set((x, y), COST_PLAIN);
        }
    }
    engine.resources.nav.install(map);
    engine
        .resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .wildlife
        .as_mut()
        .expect("wildlife")
        .arm_reach = 12;
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    for (index, (_, entity)) in people.iter().enumerate() {
        engine
            .world
            .get::<&mut BodyHealth>(*entity)
            .expect("health")
            .hp = match index {
            0 | 3 | 4 => 100.0,
            1 => 90.0,
            2 => 80.0,
            _ => 49.0,
        };
        let mut pos = engine
            .world
            .get::<&mut Position>(*entity)
            .expect("position");
        pos.x = hearth.0;
        pos.y = hearth.1;
    }
    for (entity, x) in [(people[0].1, 24), (people[1].1, 12)] {
        engine.world.remove_one::<Tool>(entity).expect("unarm");
        let mut pos = engine.world.get::<&mut Position>(entity).expect("position");
        pos.x = x;
        pos.y = 24;
    }
    engine.resources.colony.ground_items.clear();
    for (tile, kind) in [((24, 24), ItemKind::Spear), ((36, 24), ItemKind::StoneAxe)] {
        engine.resources.colony.ground_items.insert(
            tile,
            sim_core::colony::ItemStack::of(ItemSpec::raw(kind, None), 1),
        );
    }
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    assert_eq!(engine.resources.steward.defence.owned_ids.len(), 4);
    assert!(
        engine
            .resources
            .steward
            .defence
            .owned_ids
            .contains(&people[2].0),
        "held spear outranks a proposal that downgrades another defender to an axe"
    );
    assert!(!engine
        .resources
        .steward
        .defence
        .owned_ids
        .contains(&people[1].0));
    assert_eq!(
        engine
            .world
            .get::<&Drafted>(people[0].1)
            .expect("flexible defender")
            .arming,
        Some((24, 24))
    );
}

#[test]
fn free_farther_spear_prevents_downgrading_a_constrained_defender_to_an_axe() {
    use sim_core::nav::{CostMap, COST_PLAIN};
    let mut engine = armed_colony();
    let people = adults(&engine);
    assert!(people.len() >= 4);
    let mut map = CostMap::new(
        engine.resources.tile_grid.width,
        engine.resources.tile_grid.height,
    );
    for y in 0..engine.resources.tile_grid.height {
        for x in 0..engine.resources.tile_grid.width {
            map.set((x, y), COST_PLAIN);
        }
    }
    engine.resources.nav.install(map);
    engine
        .resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .wildlife
        .as_mut()
        .expect("wildlife")
        .arm_reach = 12;
    for (index, (_, entity)) in people.iter().enumerate() {
        engine
            .world
            .get::<&mut BodyHealth>(*entity)
            .expect("health")
            .hp = match index {
            0 => 100.0,
            1 => 90.0,
            2 | 3 => 100.0,
            _ => 49.0,
        };
    }
    let flexible = people[0].1;
    let constrained = people[1].1;
    engine
        .world
        .remove_one::<Tool>(flexible)
        .expect("unarm flexible");
    engine
        .world
        .remove_one::<Tool>(constrained)
        .expect("unarm constrained");
    for (entity, x) in [(flexible, 24), (constrained, 12)] {
        let mut pos = engine.world.get::<&mut Position>(entity).expect("position");
        pos.x = x;
        pos.y = 24;
    }
    engine.resources.colony.ground_items.clear();
    for tile in [(24, 24), (36, 24)] {
        engine.resources.colony.ground_items.insert(
            tile,
            sim_core::colony::ItemStack::of(ItemSpec::raw(ItemKind::Spear, None), 1),
        );
    }
    // B can also reach this axe. A must use the free farther spear instead
    // of moving B off its only spear merely to take the nearer stack.
    engine.resources.colony.ground_items.insert(
        (12, 24),
        sim_core::colony::ItemStack::of(ItemSpec::raw(ItemKind::StoneAxe, None), 1),
    );
    incoming(&mut engine);
    defence_tick(&mut engine, 10);
    assert_eq!(
        engine.resources.steward.defence.owned_ids.len(),
        4,
        "all four weapon-capable adults must still be selected"
    );
    assert_eq!(
        engine
            .world
            .get::<&Drafted>(flexible)
            .expect("flexible defender")
            .arming,
        Some((36, 24))
    );
    assert_eq!(
        engine
            .world
            .get::<&Drafted>(constrained)
            .expect("constrained defender")
            .arming,
        Some((24, 24))
    );
}

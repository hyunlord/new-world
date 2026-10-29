//! MUX bulk UI reads: ordered facts, authoritative accounting and observer purity.

use sim_bridge::ffi::collect_ui_facts;
use sim_core::colony::{Designation, ItemKind, ItemSpec, ItemStack, ReserveKey};
use sim_core::components::{Agent, AgentState, BodyHealth, Child, Kin, Position, TargetKind};

#[test]
fn people_facts_keep_ids_health_sleep_and_household_links_in_agent_order() {
    // Given an otherwise empty colony with deliberately reversed insertion order.
    let mut engine = sim_test::colony_engine(0);
    engine.world.clear();
    engine.world.spawn((
        Agent { id: 9 },
        Position { x: 7, y: 8 },
        AgentState::Consuming {
            target: TargetKind::Sleep,
        },
        BodyHealth {
            hp: 49.0,
            max_hp: 100.0,
        },
        Child { born_tick: 0 },
        Kin {
            mother: Some(2),
            ..Kin::default()
        },
    ));
    engine
        .world
        .spawn((Agent { id: 2 }, Position { x: 3, y: 4 }, AgentState::Idle));
    engine.resources.bed_of.insert(2, (5, 6));
    let before = sim_test::probe::fingerprint(&engine);
    // When the observer is called repeatedly.
    let facts = collect_ui_facts(&engine.world, &engine.resources);
    assert_eq!(facts, collect_ui_facts(&engine.world, &engine.resources));
    // Then rows retain identity, precise health and actual sleep/bed facts without writes.
    assert_eq!(
        facts.people.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![2, 9]
    );
    assert_eq!(facts.people[0].bed, Some((5, 6)));
    assert!(facts.people[0].idle);
    assert_eq!(facts.people[1].hp, Some(49.0));
    assert!(facts.people[1].sleeping && !facts.people[1].roofed);
    assert_eq!(facts.people[1].mother, Some(2));
    assert_eq!(facts.people[1].born_tick, Some(0));
    assert_eq!(before, sim_test::probe::fingerprint(&engine));
}

#[test]
fn household_ids_use_authoritative_groups_for_siblings_and_outsiders() {
    use sim_core::components::Settlement;
    let mut engine = sim_test::colony_engine(0);
    engine.world.clear();
    engine.resources.settlements.clear();
    let mut settlement = Settlement::new_with_id(8, 0);
    for id in [20, 2, 10, 1, 0] {
        settlement.add_member_agent(id);
    }
    engine.resources.settlements.insert(8, settlement);
    for (id, partner) in [(20, 10), (10, 20)] {
        engine.world.spawn((
            Agent { id },
            Position { x: 3, y: 4 },
            Kin {
                partner: Some(partner),
                ..Kin::default()
            },
        ));
    }
    // Two infants share a family and its one assigned bed; a nonmember does not.
    for id in [99, 2, 1] {
        engine.world.spawn((
            Agent { id },
            Position { x: 3, y: 4 },
            Child { born_tick: 0 },
            Kin {
                mother: Some(10),
                father: Some(20),
                ..Kin::default()
            },
        ));
    }
    engine.world.spawn((
        Agent { id: 0 },
        Position { x: 3, y: 4 },
        BodyHealth {
            hp: 0.0,
            max_hp: 100.0,
        },
    ));
    engine.resources.bed_of.insert(10, (5, 6));
    let before = sim_test::probe::fingerprint(&engine);
    let facts = collect_ui_facts(&engine.world, &engine.resources);
    assert_eq!(
        facts
            .people
            .iter()
            .map(|p| (p.id, p.household_id))
            .collect::<Vec<_>>(),
        vec![(1, 1), (2, 1), (10, 1), (20, 1), (99, 99)]
    );
    assert_eq!(
        facts
            .people
            .iter()
            .filter(|p| p.household_id == 1 && p.child)
            .count(),
        2
    );
    assert_eq!(
        facts
            .people
            .iter()
            .filter(|p| p.household_id == 1 && p.bed.is_some())
            .count(),
        1
    );
    assert_eq!(facts, collect_ui_facts(&engine.world, &engine.resources));
    assert_eq!(before, sim_test::probe::fingerprint(&engine));
}

#[test]
fn stacks_preserve_fifo_allocation_order_and_reservations_without_invented_ages() {
    // Given lots of unequal ages and ground stacks in non-coordinate insertion order.
    let mut engine = sim_test::colony_engine(0);
    engine.resources.colony.ground_items.clear();
    for (tile, count) in [((8, 2), 7), ((2, 8), 3)] {
        engine.resources.colony.ground_items.insert(
            tile,
            ItemStack::of(ItemSpec::raw(ItemKind::Meat, None), count),
        );
    }
    engine
        .resources
        .colony
        .reservations
        .insert(ReserveKey::Stack((2, 8)), 90);
    engine.resources.colony.wildlife.meat_lots = vec![(1, 2), (3, 8)];
    // When collecting raw spoilage inputs.
    let facts = collect_ui_facts(&engine.world, &engine.resources);
    // Then native (x,y) order and global lots are exact, not inferred stack ages.
    assert_eq!(
        facts.stacks.iter().map(|s| s.tile).collect::<Vec<_>>(),
        vec![(2, 8), (8, 2)]
    );
    assert_eq!(facts.meat_lots, vec![(1, 2), (3, 8)]);
    assert!(facts.stacks[0].reserved && !facts.stacks[1].reserved);
    assert_eq!(facts.stacks[0].kind, ItemKind::Meat.code());
    assert_eq!(facts.stacks[1].count, 7);
}

#[test]
fn unreachable_facts_exclude_expired_and_removed_orders() {
    // Given one active mark, one removed mark and one expired avoidance record.
    let mut engine = sim_test::colony_engine(0);
    engine.resources.current_tick = 100;
    engine
        .resources
        .colony
        .designations
        .insert((3, 4), Designation::Chop);
    engine.resources.colony.unreachable.insert((1, (3, 4)), 101);
    engine.resources.colony.unreachable.insert((2, (3, 4)), 100);
    engine.resources.colony.unreachable.insert((3, (5, 6)), 101);
    // When exposing current order causes.
    let facts = collect_ui_facts(&engine.world, &engine.resources);
    // Then only the still-live unexpired target remains.
    assert_eq!(facts.unreachable.len(), 1);
    assert_eq!(
        (facts.unreachable[0].agent, facts.unreachable[0].tile),
        (1, (3, 4))
    );
}

#[test]
fn baseline_world_has_no_colony_ui_facts() {
    // Given the feature-off baseline world.
    let engine = sim_test::baseline_engine(0);
    // When collecting the colony UI surface.
    let facts = collect_ui_facts(&engine.world, &engine.resources);
    // Then no colony causes are invented.
    assert!(facts.people.is_empty() && facts.settlements.is_empty() && facts.stacks.is_empty());
    assert!(facts.unreachable.is_empty() && facts.meat_lots.is_empty());
}

#[test]
fn economy_counts_member_carried_food_and_preserves_fractional_daily_need() {
    use sim_core::colony::food::FoodOwner;
    use sim_core::colony::{ItemKind, ItemSpec, Zone, ZoneKind};
    use sim_core::components::{Hunger, Settlement};
    use sim_systems::runtime::food::{self, items};
    // Given two members with exact binary hunger rates and a nonmember carrying food.
    let mut engine = sim_test::colony_engine(0);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources)
        .expect("worldgen hearth exists before settlement formation");
    let region = engine.resources.nav.region_of(hearth);
    assert_ne!(region, 0, "hearth has a reachable nav region");
    let home = (0..engine.resources.tile_grid.height)
        .flat_map(|y| (0..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .filter(|&tile| {
            engine.resources.nav.region_of(tile) == region
                && sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, tile)
        })
        .min_by_key(|&(x, y)| (x.abs_diff(hearth.0).max(y.abs_diff(hearth.1)), y, x))
        .expect("reachable camp tile can hold the fixture stock");
    engine.world.clear();
    engine.resources.settlements.clear();
    engine.resources.colony.ground_items.clear();
    engine.resources.food.ages.clear();
    engine.resources.colony.zones.clear();
    engine.resources.ticks_per_day = 1440;
    let mut settlement = Settlement::new_with_id(8, 0);
    settlement.formation_tile = home;
    settlement.add_member_agent(1);
    settlement.add_member_agent(2);
    engine.resources.settlements.insert(8, settlement);
    engine.resources.colony.zones.insert(
        1,
        Zone {
            id: 1,
            kind: ZoneKind::Stockpile,
            tiles: std::collections::BTreeSet::from([home]),
        },
    );
    engine.resources.colony.touch(false);
    let day = food::day(&engine.resources);
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    items::create_ground(
        &mut engine.resources,
        home,
        roots,
        25,
        day,
        items::FoodSource::Debug,
    )
    .expect("fresh stock");
    for (id, rate, count) in [(1, 0.125, 3), (2, 0.015625, 2), (3, 0.125, 9)] {
        engine.world.spawn((Agent { id }, Hunger::new(0.0, rate)));
        assert_eq!(
            items::transfer(
                &mut engine.world,
                &mut engine.resources,
                FoodOwner::Ground(home),
                FoodOwner::Carried(id),
                count,
                day
            )
            .expect("carry fresh food"),
            count
        );
    }
    // When the real accounting is exposed.
    let facts = collect_ui_facts(&engine.world, &engine.resources);
    // Then 11+3+2 stock and (0.125+0.015625)*1440/30 daily need stay exact.
    assert_eq!(facts.settlements.len(), 1);
    assert_eq!(facts.settlements[0].food, 16);
    assert_eq!(facts.settlements[0].food_need, 6.75);
    assert_eq!(facts.settlements[0].tile, home);
}

#[test]
fn observer_does_not_change_future_ticks_or_command_log() {
    // Given two identical live colonies, with the observer used only on one.
    let mut observed = sim_test::colony_engine(2);
    let mut control = sim_test::colony_engine(2);
    // When both run the same ticks.
    for _ in 0..120 {
        let _ = collect_ui_facts(&observed.world, &observed.resources);
        observed.tick();
        control.tick();
    }
    // Then simulation identity and order history remain identical.
    assert_eq!(
        sim_test::probe::fingerprint(&observed),
        sim_test::probe::fingerprint(&control)
    );
    assert_eq!(
        observed.resources.colony.command_log,
        control.resources.colony.command_log
    );
}

#[test]
fn steward_scope_boundary_rejects_without_mutation_and_valid_changes_are_logged() {
    use sim_bridge::ffi::{collect_steward, submit_set_steward};
    // Given an untouched OFF colony.
    let mut engine = sim_test::colony_engine(0);
    assert!(!collect_steward(&engine.resources).enabled);
    let before = engine.resources.colony.command_log.len();
    // When invalid raw scope values cross the bridge boundary.
    for scope in [-1, 3, 256, i64::MAX] {
        assert!(!submit_set_steward(&mut engine, true, scope).accepted);
    }
    // Then invalid values neither change state nor manufacture an ordinary command.
    assert!(!collect_steward(&engine.resources).enabled);
    assert_eq!(engine.resources.colony.command_log.len(), before);
    // When the actual supported scope is selected, the normal command path records it.
    assert!(submit_set_steward(&mut engine, true, 1).accepted);
    let state = collect_steward(&engine.resources);
    assert!(state.enabled);
    assert_eq!(state.scope, 1);
    assert_eq!(engine.resources.colony.command_log.len(), before + 1);
    assert_eq!(state.sequence, 0);
    assert!(state.accepted_rules.is_empty());
    assert_eq!(state.labels.len(), 8);
    assert_eq!(state.labels[0], (1, "MUX_STEWARD_RULE_1".to_string()));
    assert_eq!(state.labels[5], (6, "MUX_STEWARD_RULE_6".to_string()));
    assert_eq!(state.labels[6], (7, "MUX_STEWARD_RULE_7".to_string()));
    assert_eq!(state.labels[7], (8, "M6A_STEWARD_FOOD".to_string()));
}

#[test]
fn first_winter_completion_uses_retained_death_dates_not_cumulative_deaths() {
    use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
    // Given a colony at the first calendar year boundary.
    let mut engine = sim_test::colony_engine(0);
    assert!(engine.resources.rules.seasons.is_some());
    let boundary = engine
        .resources
        .rules
        .calendar
        .ticks_per_year()
        .saturating_sub(engine.resources.rules.clock_offset_ticks);
    engine.resources.current_tick = boundary.saturating_sub(1);
    assert!(!collect_ui_facts(&engine.world, &engine.resources).first_winter_all_alive);
    engine.resources.current_tick = boundary;
    // When a death is recorded after the completed first year.
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(boundary, ChronicleKind::Died).subject(99));
    // Then the completed goal stays checked, but an earlier death disqualifies it.
    assert!(collect_ui_facts(&engine.world, &engine.resources).first_winter_all_alive);
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(boundary - 1, ChronicleKind::Died).subject(100));
    assert!(!collect_ui_facts(&engine.world, &engine.resources).first_winter_all_alive);
}

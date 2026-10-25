//! Controlled destination geometry, capacity and live reservation boundaries.
use super::*;
use colony_support::drop_items;
use sim_core::colony::ItemSpec;
use sim_core::TerrainType;
use sim_systems::runtime::jobs::items::{chebyshev, room_on, stack_max};

pub(super) fn add_zone(f: &mut Fixture, at: (u32, u32)) {
    assert!(
        submit(
            &mut f.engine,
            Command::AddStockpile {
                x0: at.0,
                y0: at.1,
                x1: at.0,
                y1: at.1,
            }
        )
        .accepted
    );
}

pub(super) fn next_kill(f: &mut Fixture, hunter: Entity) {
    let before = f.engine.resources.colony.wildlife.tallies.deer_killed;
    let created = f.engine.resources.colony.item_ledger.created_of(meat());
    let hides = f
        .engine
        .resources
        .colony
        .item_ledger
        .created_of(ItemKind::Hide);
    let mut observed = false;
    for _ in 0..240 {
        tick_checked(&mut f.engine);
        observed |= f
            .engine
            .world
            .get::<&Job>(hunter)
            .is_ok_and(|j| matches!(j.kind, JobKind::Hunt { .. }));
        if f.engine.resources.colony.wildlife.tallies.deer_killed > before {
            assert!(observed, "ordinary Hunt observed");
            assert_eq!(
                f.engine.resources.colony.wildlife.tallies.deer_killed,
                before + 1
            );
            assert_eq!(
                f.engine.resources.colony.item_ledger.created_of(meat()),
                created + 2
            );
            assert_eq!(
                f.engine
                    .resources
                    .colony
                    .item_ledger
                    .created_of(ItemKind::Hide),
                hides + 1
            );
            return;
        }
    }
    panic!("controlled hunt did not finish");
}

pub(super) fn destination(f: &Fixture, hunter: Entity) -> (u32, u32) {
    let j = *f.engine.world.get::<&Job>(hunter).expect("real haul");
    let JobKind::Haul { to, .. } = j.kind else {
        panic!("not a haul: {j:?}")
    };
    assert_eq!(j.item, meat());
    to
}

pub(super) fn finish_haul(f: &mut Fixture, hunter: Entity, to: (u32, u32)) {
    for _ in 0..180 {
        tick_checked(&mut f.engine);
        if f.engine.world.get::<&Job>(hunter).is_err() {
            break;
        }
    }
    assert!(f.engine.world.get::<&Job>(hunter).is_err());
    assert!(f.engine.world.get::<&Carry>(hunter).is_err());
    let p = f
        .engine
        .world
        .get::<&Position>(hunter)
        .expect("delivered position");
    assert_eq!((p.x, p.y), to);
    let id = f.engine.world.get::<&Agent>(hunter).expect("agent").id;
    assert!(!f
        .engine
        .resources
        .colony
        .reservations
        .values()
        .any(|owner| *owner == id));
}

fn keep_cells(f: &mut Fixture, anchor: (u32, u32), cells: &[(u32, u32)]) {
    for y in anchor.1.saturating_sub(2)..=(anchor.1 + 2).min(31) {
        for x in anchor.0.saturating_sub(2)..=(anchor.0 + 2).min(31) {
            if (x, y) != anchor && !cells.contains(&(x, y)) {
                f.engine
                    .resources
                    .building_kinds
                    .insert((x, y), BuildingKind::Shelter);
            }
        }
    }
    for &cell in cells {
        assert!(room_on(&f.engine.resources, cell, ItemSpec::raw(meat(), None)) > 0);
    }
}

fn second_hunter(f: &mut Fixture, source: (u32, u32)) -> Entity {
    let entity = worker(&mut f.engine, source.0 - 1, source.1);
    let mut known = Knowledge::default();
    known.set(
        knowledge::id("tracking").expect("tracking"),
        1000,
        Source::Seeded,
        0,
    );
    f.engine.world.insert_one(entity, known).expect("knowledge");
    let id = f.engine.world.get::<&Agent>(entity).expect("agent").id;
    assert!(f
        .engine
        .resources
        .settlements
        .get_mut(&1)
        .expect("settlement")
        .add_member_agent(id));
    let animal = wildlife::spawn_deer(&mut f.engine.resources, source).expect("second deer");
    assert!(submit(&mut f.engine, Command::DesignateHunt { animal }).accepted);
    entity
}

#[test]
fn full_and_incompatible_stockpiles_select_hearth_without_changing_the_stacks() {
    for kind in [meat(), ItemKind::Stone] {
        let mut f = fixture();
        add_zone(&mut f, STOCKPILE);
        let count = if kind == meat() {
            stack_max(&f.engine.resources, kind)
        } else {
            1
        };
        drop_items(&mut f.engine, STOCKPILE, ItemSpec::raw(kind, None), count);
        if kind == meat() {
            f.engine
                .resources
                .colony
                .wildlife
                .meat_lots
                .push((0, u32::from(count)));
        }
        let original = f.engine.resources.colony.ground_items[&STOCKPILE];
        assert_eq!(original.count, count);
        assert_eq!(
            room_on(&f.engine.resources, STOCKPILE, ItemSpec::raw(meat(), None)),
            0
        );
        let hunter = f.hunter;
        next_kill(&mut f, hunter);
        let to = destination(&f, hunter);
        assert!(!f.engine.resources.colony.zone_of_tile.contains_key(&to));
        assert!(chebyshev(to, HEARTH) <= 2);
        finish_haul(&mut f, hunter, to);
        assert_eq!(f.engine.resources.colony.ground_items[&STOCKPILE], original);
        assert_eq!(ground(&f, ItemKind::Hide), 1);
    }
}

#[test]
fn unreachable_but_legal_stockpile_selects_hearth() {
    let mut f = fixture();
    let island = (28, 26);
    add_zone(&mut f, island);
    for y in 25..=27 {
        for x in 27..=29 {
            if (x, y) == island {
                continue;
            }
            let index = f.engine.resources.tile_grid.idx(x, y);
            f.engine.resources.tile_grid.terrain_type[index] = TerrainType::River;
            sim_systems::runtime::nav::refresh(&mut f.engine.resources, (x, y));
        }
    }
    assert!(room_on(&f.engine.resources, island, ItemSpec::raw(meat(), None)) > 0);
    assert!(!f.engine.resources.nav.reachable(SOURCE, island));
    kill(&mut f);
    let to = haul(&f).1;
    assert!(chebyshev(to, HEARTH) <= 2);
    let hunter = f.hunter;
    finish_haul(&mut f, hunter, to);
    assert!(!f.engine.resources.colony.ground_items.contains_key(&island));
}

#[test]
fn fallback_ties_use_source_distance_then_y_then_x() {
    for (source, anchor, candidates, expected) in [
        ((10, 10), (14, 10), vec![(12, 8), (12, 9)], (12, 8)),
        ((14, 6), (14, 10), vec![(12, 8), (13, 8)], (12, 8)),
        ((10, 10), (14, 10), vec![(12, 10), (16, 8)], (12, 10)),
    ] {
        let mut f = fixture_at(anchor, source, true);
        keep_cells(&mut f, anchor, &candidates);
        let hunter = f.hunter;
        next_kill(&mut f, hunter);
        assert_eq!(destination(&f, hunter), expected);
        finish_haul(&mut f, hunter, expected);
    }
}

#[test]
fn radius_two_corner_is_eligible_but_radius_three_is_not() {
    for corner in [(10, 12), (11, 10)] {
        let mut f = fixture();
        keep_cells(&mut f, HEARTH, &[corner]);
        kill(&mut f);
        if corner == (10, 12) {
            assert_eq!(haul(&f).1, corner);
            let hunter = f.hunter;
            finish_haul(&mut f, hunter, corner);
        } else {
            assert_released(&f);
            assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_hauls, 0);
        }
    }
}

#[test]
fn fallback_excludes_source_even_when_source_has_merge_room() {
    let source = (10, 10);
    let chosen = (6, 12);
    let mut f = fixture_at(HEARTH, source, true);
    keep_cells(&mut f, HEARTH, &[source, chosen]);
    let hunter = f.hunter;
    next_kill(&mut f, hunter);
    assert!(room_on(&f.engine.resources, source, ItemSpec::raw(meat(), None)) > 0);
    assert_eq!(destination(&f, hunter), chosen);
    finish_haul(&mut f, hunter, chosen);
}

#[test]
fn edge_hearth_candidates_are_clipped_to_the_map() {
    for (anchor, source, chosen) in [((0, 0), (5, 5), (2, 2)), ((31, 31), (26, 26), (29, 29))] {
        let mut f = fixture_at(anchor, source, true);
        keep_cells(&mut f, anchor, &[chosen]);
        let hunter = f.hunter;
        next_kill(&mut f, hunter);
        assert_eq!(destination(&f, hunter), chosen);
        finish_haul(&mut f, hunter, chosen);
    }
}

#[test]
fn fallback_merges_only_free_capacity_and_preserves_source_remainder() {
    let chosen = (10, 10);
    let mut f = fixture();
    keep_cells(&mut f, HEARTH, &[chosen]);
    let maximum = stack_max(&f.engine.resources, meat());
    assert!(maximum > 1);
    drop_items(
        &mut f.engine,
        chosen,
        ItemSpec::raw(meat(), None),
        maximum - 1,
    );
    f.engine
        .resources
        .colony
        .wildlife
        .meat_lots
        .push((0, u32::from(maximum - 1)));
    let hunter = f.hunter;
    next_kill(&mut f, hunter);
    assert_eq!(destination(&f, hunter), chosen);
    assert_eq!(job(&f).expect("haul").amount, 1);
    let mut carried = false;
    for _ in 0..60 {
        tick_checked(&mut f.engine);
        if let Ok(carry) = f.engine.world.get::<&Carry>(hunter) {
            assert_eq!(carry.count, 1);
            assert_eq!(f.engine.resources.colony.ground_items[&SOURCE].count, 1);
            carried = true;
            break;
        }
    }
    assert!(carried);
    finish_haul(&mut f, hunter, chosen);
    assert_eq!(
        f.engine.resources.colony.ground_items[&chosen].count,
        maximum
    );
    assert_eq!(ground(&f, meat()), u64::from(maximum) + 1);
    assert_eq!(ground(&f, ItemKind::Hide), 1);
    assert_eq!(
        f.engine
            .resources
            .colony
            .wildlife
            .meat_lots
            .iter()
            .map(|(_, n)| *n)
            .sum::<u32>(),
        u32::from(maximum) + 1
    );
}

#[test]
fn incompatible_meat_quality_bucket_is_not_a_fallback_merge_target() {
    let mut f = fixture();
    let blocked = (10, 10);
    let chosen = (10, 11);
    keep_cells(&mut f, HEARTH, &[blocked, chosen]);
    let spec = ItemSpec {
        kind: meat(),
        material: None,
        quality: 10,
    };
    assert!(!spec.merges_with(&ItemSpec::raw(meat(), None)));
    drop_items(&mut f.engine, blocked, spec, 1);
    f.engine.resources.colony.wildlife.meat_lots.push((0, 1));
    let hunter = f.hunter;
    next_kill(&mut f, hunter);
    assert_eq!(destination(&f, hunter), chosen);
    finish_haul(&mut f, hunter, chosen);
    assert_eq!(
        f.engine.resources.colony.ground_items[&blocked].spec(),
        spec
    );
}

#[test]
fn reserved_stockpile_belongs_to_real_haul_and_second_hunter_uses_hearth() {
    let mut f = fixture_at((4, 4), (28, 22), true);
    let stockpile = (2, 28);
    add_zone(&mut f, stockpile);
    let first = f.hunter;
    next_kill(&mut f, first);
    assert_eq!(destination(&f, first), stockpile);
    let second = second_hunter(&mut f, (26, 22));
    next_kill(&mut f, second);
    assert_eq!(
        f.engine
            .resources
            .colony
            .reservations
            .get(&ReserveKey::Cell(stockpile)),
        Some(&f.id)
    );
    assert_eq!(destination(&f, first), stockpile);
    let to = destination(&f, second);
    assert!(chebyshev(to, (4, 4)) <= 2);
    assert_ne!(to, stockpile);
    invariants::check(&f.engine.world, &f.engine.resources).expect("real reservation owners");
    finish_haul(&mut f, second, to);
    for _ in 0..180 {
        if ground(&f, meat()) == 4 {
            break;
        }
        tick_checked(&mut f.engine);
    }
    // First may take a subsequent ordinary hide haul; all meat must be on ground.
    assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_hauls, 2);
    assert_eq!(ground(&f, meat()), 4);
}

#[test]
fn two_hunters_choose_distinct_unreserved_cells_or_leave_second_loot() {
    for cells in [vec![(6, 4), (6, 5)], vec![(6, 4)]] {
        let mut f = fixture_at((4, 4), (28, 22), true);
        keep_cells(&mut f, (4, 4), &cells);
        let first = f.hunter;
        next_kill(&mut f, first);
        let first_to = destination(&f, first);
        let second = second_hunter(&mut f, (26, 22));
        next_kill(&mut f, second);
        assert_eq!(
            f.engine
                .resources
                .colony
                .reservations
                .get(&ReserveKey::Cell(first_to)),
            Some(&f.id)
        );
        if cells.len() == 2 {
            let second_to = destination(&f, second);
            assert_ne!(first_to, second_to);
            finish_haul(&mut f, second, second_to);
            assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_hauls, 2);
        } else {
            assert!(f.engine.world.get::<&Job>(second).is_err());
            assert_eq!(f.engine.resources.colony.ground_items[&(26, 22)].count, 2);
            assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_hauls, 1);
            finish_haul(&mut f, first, first_to);
        }
        assert_eq!(ground(&f, meat()), 4);
        assert_eq!(ground(&f, ItemKind::Hide), 2);
        invariants::check(&f.engine.world, &f.engine.resources).expect("both hunts conserved");
    }
}

#[test]
fn another_actual_hearth_keeps_anchor_fallback_live_until_last_building_is_removed() {
    for remove_last in [false, true] {
        let mut f = fixture();
        let other = (2, 28);
        f.engine
            .resources
            .building_kinds
            .insert(other, BuildingKind::Hearth);
        let mut settlement = Settlement::new_with_id(2, 0);
        settlement.formation_tile = other;
        f.engine.resources.settlements.insert(2, settlement);
        kill(&mut f);
        let chosen = haul(&f).1;
        assert!(chebyshev(chosen, HEARTH) <= 2);
        pickup(&mut f);
        f.engine.resources.building_kinds.remove(&HEARTH);
        assert!(sim_systems::worldgen::colony::hearth(&f.engine.resources).is_some());
        assert_eq!(
            hearth::hearth_tiles(&f.engine.resources),
            vec![HEARTH, other]
        );
        if remove_last {
            f.engine.resources.building_kinds.remove(&other);
            tick_checked(&mut f.engine);
            assert_released(&f);
            assert_eq!(
                f.engine
                    .resources
                    .colony
                    .tallies
                    .cancels
                    .get(&CancelReason::TargetGone),
                Some(&1)
            );
        } else {
            let hunter = f.hunter;
            finish_haul(&mut f, hunter, chosen);
            assert!(!f
                .engine
                .resources
                .colony
                .tallies
                .cancels
                .contains_key(&CancelReason::TargetGone));
        }
        assert_eq!(ground(&f, meat()), 2);
        assert_eq!(food(&f), 0);
    }
}

#[test]
fn global_destination_order_does_not_follow_settlement_id_order() {
    let mut f = fixture_at((2, 2), SOURCE, true);
    let mut closer = Settlement::new_with_id(2, 0);
    closer.formation_tile = HEARTH;
    f.engine.resources.settlements.insert(2, closer);
    keep_cells(&mut f, (2, 2), &[(4, 4)]);
    keep_cells(&mut f, HEARTH, &[(10, 10)]);
    kill(&mut f);
    assert_eq!(haul(&f).1, (10, 10));
    let hunter = f.hunter;
    finish_haul(&mut f, hunter, (10, 10));
}

#[test]
fn losing_anchor_coverage_cancels_both_haul_stages_with_building_hearth_retained() {
    for carrying in [false, true] {
        let mut f = fixture();
        kill(&mut f);
        let (from, to, amount) = haul(&f);
        if carrying {
            pickup(&mut f);
            assert_eq!(
                f.engine.world.get::<&Carry>(f.hunter).expect("load").count,
                amount
            );
        }
        let source_before = f.engine.resources.colony.ground_items.get(&from).copied();
        let lots_before = f.engine.resources.colony.wildlife.meat_lots.clone();
        let food_before = food(&f);
        // Controlled predicate mutation, not a player command or normal anchor movement.
        f.engine
            .resources
            .settlements
            .get_mut(&1)
            .expect("settlement")
            .formation_tile = (28, 28);
        assert_eq!(
            f.engine.resources.building_kinds.get(&HEARTH),
            Some(&BuildingKind::Hearth)
        );
        assert!(sim_systems::worldgen::colony::hearth(&f.engine.resources).is_some());
        assert!(hearth::hearth_tiles(&f.engine.resources)
            .iter()
            .all(|anchor| chebyshev(to, *anchor) > hearth::HEARTH_PILE_REACH));
        assert!(!f.engine.resources.colony.zone_of_tile.contains_key(&to));
        tick_checked(&mut f.engine);
        assert_released(&f);
        assert_eq!(
            f.engine
                .resources
                .colony
                .tallies
                .cancels
                .get(&CancelReason::TargetGone),
            Some(&1)
        );
        assert_eq!(ground(&f, meat()), 2);
        assert_eq!(ground(&f, ItemKind::Hide), 1);
        assert_eq!(f.engine.resources.colony.wildlife.meat_lots, lots_before);
        assert_eq!(food_before, 0);
        assert_eq!(food(&f), food_before);
        assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_eaten, 0);
        if carrying {
            let p = f
                .engine
                .world
                .get::<&Position>(f.hunter)
                .expect("cancel position");
            assert!(
                f.engine
                    .resources
                    .colony
                    .ground_items
                    .iter()
                    .any(|(tile, stack)| {
                        stack.kind == meat()
                            && stack.count == amount
                            && chebyshev(*tile, (p.x, p.y)) <= 1
                    }),
                "load drops near the actual carrier rather than the invalid destination"
            );
        } else {
            assert_eq!(
                f.engine.resources.colony.ground_items.get(&from).copied(),
                source_before
            );
        }
    }
}

//! Frozen Bills reference equivalence tests for planner-only performance changes.
use super::*;

// Frozen pre-optimization Bills branch: eager reservation and uncached ranking.
fn reference(world: &World, res: &SimResources, rule: &Rule) -> Vec<Command> {
    let Some(hearth) = hearth_tiles(res).first().copied() else {
        return Vec::new();
    };
    match &rule.action {
        Action::Bills {
            recipes,
            winter_recipe,
            days,
        } => {
            let reserved_house = sim_core::colony::steward::rules()
                .ok()
                .and_then(|data| {
                    data.rules.iter().find(|rule| {
                        matches!(rule.action, Action::House { .. })
                            && res.steward.scope.permits(rule.scope)
                    })
                })
                .map(|rule| house(world, res, hearth, &rule.action, true).reserved)
                .unwrap_or_default();
            let mut wanted = recipes.clone();
            wanted.push((
                *winter_recipe,
                u16::try_from(u32::from(*days).saturating_mul(winter_daily_fuel(res)))
                    .unwrap_or(u16::MAX),
            ));
            let mut out = super::super::production::inputs(world, res, hearth, &wanted);
            let mut pending = BTreeSet::new();
            for (recipe, target) in wanted {
                let Some(def) = recipe.recipe() else {
                    continue;
                };
                if !crate::runtime::knowledge::anyone_practises(
                    world,
                    knowledge::gates().build(def.workshop),
                ) || !crate::runtime::knowledge::anyone_practises(
                    world,
                    knowledge::gates().recipe(recipe),
                ) {
                    continue;
                }
                if let Some((&id, shop)) = res
                    .colony
                    .workshops
                    .iter()
                    .find(|(_, s)| def.supports_workshop(s.kind))
                {
                    if shop.bills.iter().any(|b| {
                        b.recipe == recipe
                            && (!b.auto
                                || super::super::production::player_edited_bill(res, id, b.id))
                    }) || res
                        .steward
                        .vetoed(&Target::Bill(id, recipe.code()), res.current_tick)
                    {
                        continue;
                    }
                    out.push(Command::AddBill {
                        workshop: id,
                        recipe,
                        mode: BillMode::Maintain(target),
                    });
                } else if !world
                    .query::<&ColonySite>()
                    .iter()
                    .any(|(_, s)| def.supports_workshop(s.kind))
                    && pending.insert(def.workshop)
                {
                    let mut locations = candidates(res, hearth);
                    if let Some(colony_rule) = res.rules.colony {
                        let (size, _, _) = def.workshop.plan(&colony_rule);
                        locations.sort_by_key(|&(x, y)| {
                            let work = sim_core::colony::workshop::work_tile_of((x, y), size);
                            let usable = res.colony.zone_of_tile.contains_key(&work)
                                || distance(work, hearth)
                                    <= crate::runtime::jobs::hearth::HEARTH_PILE_REACH;
                            (
                                !usable,
                                !crate::runtime::rooms::roofed(res, work),
                                distance(work, hearth),
                                distance((x, y), hearth),
                                y,
                                x,
                            )
                        });
                    }
                    if let Some((x, y)) = locations.into_iter().find(|&t| {
                        let avoids_house = res.rules.colony.is_some_and(|rule| {
                            let (size, _, _) = def.workshop.plan(&rule);
                            !reserved_house
                                .contains(&sim_core::colony::workshop::work_tile_of(t, size))
                                && (t.1..t.1 + size.1).all(|y| {
                                    (t.0..t.0 + size.0).all(|x| !reserved_house.contains(&(x, y)))
                                })
                        });
                        avoids_house
                            && permitted(res, t)
                            && check_blueprint(res, def.workshop, t.0, t.1).is_ok()
                    }) {
                        // One missing workshop per interval avoids overlapping speculative footprints.
                        out.push(Command::PlaceBlueprint {
                            kind: def.workshop,
                            x,
                            y,
                        });
                        break;
                    }
                }
            }
            out
        }
        _ => panic!("Bills fixture"),
    }
}

#[test]
fn cached_and_lazy_bills_match_frozen_reference_in_generated_worlds() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let rules = sim_core::colony::steward::rules().expect("rules");
    let bills = rules
        .rules
        .iter()
        .find(|rule| matches!(rule.action, Action::Bills { .. }))
        .expect("Bills rule");
    let stockpile = rules
        .rules
        .iter()
        .find(|rule| matches!(rule.action, Action::Stockpile { .. }))
        .expect("stockpile rule");
    let house_rule = rules
        .rules
        .iter()
        .find(|rule| matches!(rule.action, Action::House { .. }))
        .expect("house rule");
    for seed in [0, 1, 6, 7] {
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(seed),
            sim_core::rules::WorldRules::colony(seed),
        );
        for phase in 0..4 {
            match phase {
                1 => {
                    for command in plan(&engine.world, &engine.resources, stockpile) {
                        assert!(
                            crate::commands::apply_command(
                                &mut engine.world,
                                &mut engine.resources,
                                command
                            )
                            .accepted
                        );
                    }
                }
                2 => {
                    let Action::House { bed, .. } = house_rule.action else {
                        panic!("house action")
                    };
                    engine.resources.colony.space_sites.insert((1, 1), bed);
                }
                3 => {
                    let entities: Vec<_> = engine
                        .world
                        .query::<&Agent>()
                        .iter()
                        .map(|(entity, _)| entity)
                        .collect();
                    for entity in entities {
                        engine
                            .world
                            .insert_one(entity, Knowledge::default())
                            .expect("remove knowledge");
                    }
                }
                _ => {}
            }
            let before = reference(&engine.world, &engine.resources, bills);
            let after = plan(&engine.world, &engine.resources, bills);
            assert_eq!(after, before, "seed={seed} phase={phase}");
        }
    }
}

#[test]
fn direct_workshop_candidates_preserve_the_complete_previous_order() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let rules = sim_core::colony::steward::rules().expect("rules");
    let stock_rule = rules
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::Stockpile { .. }))
        .expect("stock");
    for seed in [0, 1, 6, 7] {
        let mut e = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(seed),
            sim_core::rules::WorldRules::colony(seed),
        );
        let home = hearth_tiles(&e.resources)[0];
        e.resources.colony.rooms.insert(
            999,
            sim_core::rooms::Room {
                id: 999,
                tiles: BTreeSet::from([home]),
                doors: BTreeSet::new(),
                roofed: true,
                role: sim_core::rooms::RoomRole::Unassigned,
                dug: false,
            },
        );
        e.resources.colony.room_of_tile.insert(home, 999);
        for phase in 0..2 {
            if phase == 1 {
                for command in plan(&e.world, &e.resources, stock_rule) {
                    assert!(
                        crate::commands::apply_command(&mut e.world, &mut e.resources, command)
                            .accepted
                    );
                }
            }
            for hearth in [
                home,
                (1, 1),
                (
                    e.resources.tile_grid.width - 2,
                    e.resources.tile_grid.height - 2,
                ),
            ] {
                for size in [(1, 1), (2, 2), (3, 4)] {
                    let rank = |&(x, y): &Tile| {
                        let work = sim_core::colony::workshop::work_tile_of((x, y), size);
                        let usable = e.resources.colony.zone_of_tile.contains_key(&work)
                            || distance(work, hearth)
                                <= crate::runtime::jobs::hearth::HEARTH_PILE_REACH;
                        (
                            !usable,
                            !crate::runtime::rooms::roofed(&e.resources, work),
                            distance(work, hearth),
                            distance((x, y), hearth),
                            y,
                            x,
                        )
                    };
                    let mut old = candidates(&e.resources, hearth);
                    old.sort_by_cached_key(rank);
                    let mut new = interior_tiles(&e.resources);
                    new.sort_by_cached_key(rank);
                    assert_eq!(
                        bills_locations(
                            &e.resources,
                            hearth,
                            Some(size),
                            interior_tiles(&e.resources)
                        )
                        .collect::<Vec<_>>(),
                        old
                    );
                    assert_eq!(
                        new, old,
                        "seed={seed} phase={phase} hearth={hearth:?} size={size:?}"
                    );
                }
            }
        }
    }
}

fn frozen_bill_order(
    res: &SimResources,
    hearth: Tile,
    size: Option<Tile>,
    mut tiles: Vec<Tile>,
) -> Vec<Tile> {
    if let Some(size) = size {
        tiles.sort_by_cached_key(|&(x, y)| {
            let work = sim_core::colony::workshop::work_tile_of((x, y), size);
            let usable = res.colony.zone_of_tile.contains_key(&work)
                || distance(work, hearth) <= crate::runtime::jobs::hearth::HEARTH_PILE_REACH;
            (
                !usable,
                !crate::runtime::rooms::roofed(res, work),
                distance(work, hearth),
                distance((x, y), hearth),
                y,
                x,
            )
        });
    }
    tiles
}

#[test]
fn bills_order_preserves_stable_duplicates_fallback_and_every_search_prefix() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let e = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(0),
        sim_core::rules::WorldRules::colony(0),
    );
    let home = hearth_tiles(&e.resources)[0];
    for tiles in [
        vec![],
        vec![home],
        vec![(4, 5), (2, 3), (4, 5), home, (3, 2), (2, 3)],
    ] {
        for size in [None, Some((1, 1)), Some((3, 4))] {
            let expected = frozen_bill_order(&e.resources, home, size, tiles.clone());
            assert_eq!(
                bills_locations(&e.resources, home, size, tiles.clone()).collect::<Vec<_>>(),
                expected
            );
            for accepted_index in 0..=tiles.len() {
                let mut visited = Vec::new();
                let selected =
                    bills_locations(&e.resources, home, size, tiles.clone()).find(|&tile| {
                        visited.push(tile);
                        visited.len() - 1 == accepted_index
                    });
                assert_eq!(visited, expected[..(accepted_index + 1).min(tiles.len())]);
                assert_eq!(selected, expected.get(accepted_index).copied());
            }
        }
    }
}

#[test]
fn bills_order_preserves_real_validation_prefix_and_independent_nav_stats() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let rules = sim_core::colony::steward::rules().expect("rules");
    let kind = rules
        .rules
        .iter()
        .find_map(|r| match &r.action {
            Action::Bills { recipes, .. } => recipes
                .iter()
                .find_map(|(r, _)| r.recipe().map(|d| d.workshop)),
            _ => None,
        })
        .expect("workshop");
    for seed in [0, 1, 6, 7] {
        for reserved_count in [0, 20] {
            let run = |optimized| {
                let mut e = crate::worldgen::build_world_with(
                    &crate::worldgen::WorldGenConfig::colony(seed),
                    sim_core::rules::WorldRules::colony(seed),
                );
                let res = &mut e.resources;
                let home = hearth_tiles(res)[0];
                let size = Some(kind.plan(&res.rules.colony.expect("colony")).0);
                let tiles = interior_tiles(res);
                let expected = frozen_bill_order(res, home, size, tiles.clone());
                let reserved: BTreeSet<_> = expected.iter().take(reserved_count).copied().collect();
                // Independent installations guarantee neither run prewarms the other's regions.
                let map = crate::runtime::nav::build(res);
                res.nav.install(map);
                *res.nav.stats.borrow_mut() = sim_core::nav::NavStats::default();
                let before = *res.nav.stats.borrow();
                let mut visited = Vec::new();
                let mut checks = Vec::new();
                let mut accepts = |&t: &Tile| {
                    visited.push(t);
                    let avoids = size.is_some_and(|size| {
                        !reserved.contains(&sim_core::colony::workshop::work_tile_of(t, size))
                            && (t.1..t.1 + size.1)
                                .all(|y| (t.0..t.0 + size.0).all(|x| !reserved.contains(&(x, y))))
                    });
                    avoids && permitted(res, t) && {
                        checks.push(t);
                        check_blueprint(res, kind, t.0, t.1).is_ok()
                    }
                };
                let selected = if optimized {
                    bills_locations(res, home, size, tiles).find(&mut accepts)
                } else {
                    expected.into_iter().find(&mut accepts)
                };
                let after = *res.nav.stats.borrow();
                (selected, visited, checks, before, after)
            };
            assert_eq!(
                run(true),
                run(false),
                "seed={seed} reserved={reserved_count}"
            );
        }
    }
}

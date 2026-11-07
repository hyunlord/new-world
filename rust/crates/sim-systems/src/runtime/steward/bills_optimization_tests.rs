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
            ..
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
    let mut legacy_bills = bills.clone();
    if let Action::Bills { days, fuel, .. } = &mut legacy_bills.action {
        *days = 12;
        *fuel = None;
    }
    let bills = &legacy_bills;
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
                dwelling_kind: None,
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

fn tool_progression_fixture(player_basic: bool) -> (sim_engine::SimEngine, Rule, u32, u32) {
    use sim_core::colony::{
        knowledge::{KnowledgeEntry, Source},
        RecipeKind, Workshop,
    };
    let mut e = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(1),
        sim_core::rules::WorldRules::colony(1),
    );
    let fine = RecipeKind::from_name("knap_fine_blade").expect("fine");
    let basic = RecipeKind::from_name("knap_blade").expect("basic");
    let pressure = knowledge::defs()
        .id_of("pressure_flaking")
        .expect("pressure");
    let mut rule = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find(|rule| rule.id == 4)
        .expect("tools")
        .clone();
    let Action::Bills {
        recipes,
        days,
        fuel,
        ..
    } = &mut rule.action
    else {
        panic!("Bills")
    };
    *recipes = vec![(fine, 4), (basic, 4)];
    *days = 0;
    *fuel = None;
    let entities: Vec<_> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    let known = Knowledge::from_entries(
        knowledge::defs()
            .nodes
            .iter()
            .filter(|node| node.id != pressure)
            .map(|node| KnowledgeEntry::new(node.id, Source::Seeded, 1000, 0))
            .collect(),
    );
    for &entity in &entities {
        e.world
            .insert_one(entity, known.clone())
            .expect("knowledge");
    }
    let shop = 900;
    let at = hearth_tiles(&e.resources)[0];
    e.resources.colony.workshops.insert(
        shop,
        Workshop {
            id: shop,
            kind: basic.recipe().expect("recipe").workshop,
            top_left: at,
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    let add = Command::AddBill {
        workshop: shop,
        recipe: basic,
        mode: BillMode::Maintain(4),
    };
    let command = if player_basic {
        add
    } else {
        Command::Steward {
            rule: rule.id,
            command: Box::new(add),
        }
    };
    assert!(crate::commands::apply_and_log(&mut e.world, &mut e.resources, command, 0).accepted);
    assert!(!plan(&e.world, &e.resources, &rule)
        .iter()
        .any(|command| matches!(command, Command::AddBill { recipe, .. } if *recipe == fine)));
    e.world
        .get::<&mut Knowledge>(entities[0])
        .expect("holder")
        .raise_to(pressure, 500, Source::Discovered, 0);
    let additions = plan(&e.world, &e.resources, &rule);
    let add_fine = additions.into_iter().find(|command|
        matches!(command, Command::AddBill { workshop, recipe, .. } if *workshop == shop && *recipe == fine))
        .expect("fine admitted after discovery");
    assert!(
        crate::commands::apply_and_log(
            &mut e.world,
            &mut e.resources,
            Command::Steward {
                rule: rule.id,
                command: Box::new(add_fine)
            },
            1
        )
        .accepted
    );
    let bills = &e.resources.colony.workshops[&shop].bills;
    assert_eq!(
        bills.iter().map(|bill| bill.recipe).collect::<Vec<_>>(),
        vec![basic, fine]
    );
    let fine_id = bills[1].id;
    (e, rule, shop, fine_id)
}

#[test]
fn tool_progression_reconciles_owned_order_after_delayed_discovery() {
    let (mut e, rule, shop, fine) = tool_progression_fixture(false);
    let before = e.resources.colony.workshops[&shop].bills.clone();
    let promotion = Command::MoveBill {
        workshop: shop,
        bill: fine,
        up: true,
    };
    let commands = plan(&e.world, &e.resources, &rule);
    assert!(
        commands.contains(&promotion),
        "missing authored preference: {commands:?}"
    );
    assert!(
        crate::commands::apply_and_log(
            &mut e.world,
            &mut e.resources,
            Command::Steward {
                rule: rule.id,
                command: Box::new(promotion)
            },
            2
        )
        .accepted
    );
    let after = &e.resources.colony.workshops[&shop].bills;
    assert_eq!(
        after,
        &vec![before[1], before[0]],
        "move retains bill identities and all counters"
    );
    assert!(!plan(&e.world, &e.resources, &rule)
        .iter()
        .any(|command| matches!(command, Command::MoveBill { .. })));
}

#[test]
fn tool_progression_preserves_both_neighbours_manual_order_and_vetoes() {
    for edit_fine in [false, true] {
        let (mut e, rule, shop, fine) = tool_progression_fixture(false);
        let basic = e.resources.colony.workshops[&shop].bills[0].id;
        for up in [true, false] {
            let (bill, up) = if edit_fine { (fine, up) } else { (basic, !up) };
            assert!(
                crate::commands::apply_and_log(
                    &mut e.world,
                    &mut e.resources,
                    Command::MoveBill {
                        workshop: shop,
                        bill,
                        up
                    },
                    2
                )
                .accepted
            );
        }
        assert!(!plan(&e.world, &e.resources, &rule)
            .iter()
            .any(|command| matches!(command, Command::MoveBill { .. })));
    }
    for veto_fine in [false, true] {
        let (mut e, rule, shop, _) = tool_progression_fixture(false);
        let recipe = e.resources.colony.workshops[&shop].bills[usize::from(veto_fine)].recipe;
        e.resources
            .steward
            .vetoes
            .insert(Target::Bill(shop, recipe.code()), 1000);
        assert!(!plan(&e.world, &e.resources, &rule)
            .iter()
            .any(|command| matches!(command, Command::MoveBill { .. })));
    }
    let (e, rule, _, _) = tool_progression_fixture(true);
    assert!(!plan(&e.world, &e.resources, &rule)
        .iter()
        .any(|command| matches!(command, Command::MoveBill { .. })));
}

#[test]
fn tool_progression_leaves_inflight_work_and_suspended_bills_untouched() {
    use sim_core::colony::{ItemKind, Job, JobKind, JobStage};
    for selected in 0..3 {
        let (mut e, rule, shop, fine) = tool_progression_fixture(false);
        let basic = e.resources.colony.workshops[&shop].bills[0].id;
        let at = e.resources.colony.workshops[&shop].work_tile();
        let kind = match selected {
            0 => JobKind::Craft {
                workshop: shop,
                bill: basic,
                at,
            },
            1 => JobKind::Craft {
                workshop: shop,
                bill: fine,
                at,
            },
            _ => JobKind::Supply {
                workshop: shop,
                from: at,
                to: at,
            },
        };
        let job = Job {
            kind,
            stage: JobStage::Working { ticks_left: 30 },
            taken_at: 0,
            amount: 1,
            item: ItemKind::from_name("flint").expect("flint"),
            material: None,
            fetch: None,
            best_distance: 0,
            progress_tick: 0,
        };
        let entity = e.world.spawn((job,));
        assert!(!plan(&e.world, &e.resources, &rule)
            .iter()
            .any(|command| matches!(command, Command::MoveBill { .. })));
        assert_eq!(*e.world.get::<&Job>(entity).expect("still working"), job);
    }
    for index in 0..2 {
        let (mut e, rule, shop, _) = tool_progression_fixture(false);
        e.resources
            .colony
            .workshops
            .get_mut(&shop)
            .expect("shop")
            .bills[index]
            .suspended = true;
        assert!(!plan(&e.world, &e.resources, &rule)
            .iter()
            .any(|command| matches!(command, Command::MoveBill { .. })));
    }
}

fn outside_axe_target_fixture(player: bool) -> (sim_engine::SimEngine, Rule, u32, u32) {
    use sim_core::colony::{ItemSpec, RecipeKind, Workshop};
    let (mut e, mut rule, _, _) = tool_progression_fixture(false);
    let recipe = RecipeKind::from_name("haft_axe").expect("axe recipe");
    let def = recipe.recipe().expect("definition");
    if let Action::Bills { recipes, .. } = &mut rule.action {
        *recipes = vec![(recipe, 2)];
    }
    let shop = 901;
    let home = hearth_tiles(&e.resources)[0];
    e.resources.colony.workshops.insert(
        shop,
        Workshop {
            id: shop,
            kind: def.workshop,
            top_left: home,
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    let add = Command::AddBill {
        workshop: shop,
        recipe,
        mode: BillMode::Maintain(2),
    };
    let command = if player {
        add
    } else {
        Command::Steward {
            rule: rule.id,
            command: Box::new(add),
        }
    };
    assert!(crate::commands::apply_and_log(&mut e.world, &mut e.resources, command, 2).accepted);
    let bill = e.resources.colony.workshops[&shop].bills[0].id;
    e.resources.colony.rooms.insert(
        990,
        sim_core::rooms::Room {
            dwelling_kind: None,
            id: 990,
            tiles: BTreeSet::from([home]),
            doors: BTreeSet::new(),
            roofed: true,
            role: sim_core::rooms::RoomRole::Storehouse,
            dug: false,
        },
    );
    e.resources.colony.room_of_tile.insert(home, 990);
    crate::runtime::food::items::create_ground(
        &mut e.resources,
        (5, 5),
        ItemSpec::raw(def.output.0, None),
        2,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("physical outside axes");
    assert_eq!(
        crate::runtime::jobs::bills::maintain_count(&e.world, &e.resources, def.output.0),
        2
    );
    assert_eq!(
        crate::runtime::jobs::bills::bill_count(&e.world, &e.resources, def.output.0),
        0
    );
    (e, rule, shop, bill)
}

#[test]
fn owned_tool_target_counts_axes_waiting_outside_store() {
    let (mut e, rule, shop, bill) = outside_axe_target_fixture(false);
    let commands = plan(&e.world, &e.resources, &rule);
    let stop = Command::SetBillMode {
        workshop: shop,
        bill,
        mode: BillMode::Maintain(0),
    };
    assert!(
        commands.contains(&stop),
        "two actual axes fulfill the target; commands={commands:?}"
    );
    assert!(
        crate::commands::apply_and_log(
            &mut e.world,
            &mut e.resources,
            Command::Steward {
                rule: rule.id,
                command: Box::new(stop)
            },
            3
        )
        .accepted
    );
    assert!(!crate::runtime::jobs::bills::runnable(
        &e.world,
        &e.resources,
        &e.resources.colony.workshops[&shop],
        0
    ));
}

#[test]
fn outside_tool_target_preserves_player_and_suspended_bills() {
    for player in [true, false] {
        let (mut e, rule, shop, bill) = outside_axe_target_fixture(player);
        if !player {
            assert!(
                crate::commands::apply_and_log(
                    &mut e.world,
                    &mut e.resources,
                    Command::Steward {
                        rule: rule.id,
                        command: Box::new(Command::SetBillSuspended {
                            workshop: shop,
                            bill,
                            suspended: true,
                        })
                    },
                    3
                )
                .accepted
            );
        }
        assert!(!plan(&e.world, &e.resources, &rule).iter().any(
            |command| matches!(command, Command::SetBillMode { workshop, bill: id, .. }
                if *workshop == shop && *id == bill)
        ));
    }
}

#[test]
fn mixed_tool_target_never_promotes_existing_surplus_to_a_new_target() {
    let (mut e, rule, shop, bill) = outside_axe_target_fixture(false);
    let home = hearth_tiles(&e.resources)[0];
    let axe = sim_core::colony::ItemKind::from_name("stone_axe").expect("axe");
    crate::runtime::food::items::create_ground(
        &mut e.resources,
        home,
        sim_core::colony::ItemSpec::raw(axe, None),
        4,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .expect("actual covered surplus");
    let covered = e
        .resources
        .colony
        .ground_items
        .iter()
        .find(|(_, stack)| stack.kind == axe && stack.count == 4)
        .map(|(tile, _)| *tile)
        .expect("new surplus stack");
    e.resources
        .colony
        .rooms
        .get_mut(&990)
        .expect("store")
        .tiles
        .insert(covered);
    e.resources.colony.room_of_tile.insert(covered, 990);
    assert_eq!(maintain_count(&e.world, &e.resources, axe), 6);
    assert_eq!(
        crate::runtime::jobs::bills::bill_count(&e.world, &e.resources, axe),
        4
    );
    let commands = plan(&e.world, &e.resources, &rule);
    assert!(
        commands.contains(&Command::SetBillMode {
            workshop: shop,
            bill,
            mode: BillMode::Maintain(0),
        }),
        "outside axes already satisfy target; covered surplus must drain: {commands:?}"
    );
}

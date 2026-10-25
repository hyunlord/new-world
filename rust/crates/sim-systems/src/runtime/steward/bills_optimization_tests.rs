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

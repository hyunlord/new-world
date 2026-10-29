//! Frozen production-input command oracle for call-local preparation.
use super::*;

fn reference(
    world: &World,
    res: &SimResources,
    hearth: (u32, u32),
    recipes: &[(RecipeKind, u16)],
) -> Vec<Command> {
    let Some(colony_rule) = res.rules.colony else {
        return Vec::new();
    };
    let mut need: BTreeMap<ItemKind, u32> = BTreeMap::new();
    let mut workshops = std::collections::BTreeSet::new();
    for &(recipe, target) in recipes {
        let Some(recipe) = recipe.recipe() else {
            continue;
        };
        if !crate::runtime::knowledge::anyone_practises(
            world,
            sim_core::colony::knowledge::gates().recipe(recipe.id),
        ) {
            continue;
        }
        let crafts = u32::from(target).div_ceil(u32::from(recipe.output.1).max(1));
        for (kind, count) in recipe.needs() {
            *need.entry(kind).or_default() += u32::from(count).saturating_mul(crafts);
        }
        if workshops.insert(recipe.workshop)
            && !res
                .colony
                .workshops
                .values()
                .any(|s| recipe.supports_workshop(s.kind))
            && !world
                .query::<&ColonySite>()
                .iter()
                .any(|(_, s)| recipe.supports_workshop(s.kind))
        {
            let (_, materials, _) = recipe.workshop.plan(&colony_rule);
            for line in materials.used() {
                *need.entry(line.kind).or_default() += u32::from(line.needed);
            }
        }
    }
    for (_, materials) in world.query::<&MaterialNeeds>().iter() {
        for line in materials.used() {
            *need.entry(line.kind).or_default() +=
                u32::from(line.needed.saturating_sub(line.delivered));
        }
    }
    let mut commands = Vec::new();
    for (kind, wanted) in need {
        let marked: u32 = res
            .colony
            .nodes
            .iter()
            .filter(|(tile, node)| {
                node.kind.yields() == kind && res.colony.designations.contains_key(tile)
            })
            .map(|(_, node)| u32::from(node.left))
            .sum();
        let mut deficit =
            wanted.saturating_sub(maintain_count(world, res, kind).saturating_add(marked));
        let mut nodes: Vec<_> = res
            .colony
            .nodes
            .iter()
            .filter(|(tile, node)| {
                node.kind.yields() == kind
                    && !res.colony.designations.contains_key(tile)
                    && !res
                        .steward
                        .vetoed(&Target::Tile(tile.0, tile.1), res.current_tick)
                    && crate::commands::camp_reaches(res, **tile)
            })
            .collect();
        nodes.sort_by_key(|(tile, _)| {
            (
                tile.0.abs_diff(hearth.0).max(tile.1.abs_diff(hearth.1)),
                tile.1,
                tile.0,
            )
        });
        for (&(x, y), node) in nodes {
            if deficit == 0 {
                break;
            }
            commands.push(Command::DesignateQuarry {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            });
            deficit = deficit.saturating_sub(u32::from(node.left));
        }
    }
    commands
}

#[test]
fn prepared_inputs_match_frozen_commands_across_navigation_and_supply_states() {
    use sim_core::colony::{steward::Action, Designation, ItemStack};
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let data = sim_core::colony::steward::rules().expect("rules");
    let recipes = data
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::Bills {
                recipes,
                winter_recipe,
                ..
            } => {
                let mut wanted = recipes.clone();
                wanted.push((*winter_recipe, 100));
                Some(wanted)
            }
            _ => None,
        })
        .expect("Bills rule");
    let mut nonempty = 0;
    let mut empty = 0;
    for seed in [0, 1, 6, 7] {
        for nav_state in 0..6 {
            let mut e = crate::worldgen::build_world_with(
                &crate::worldgen::WorldGenConfig::colony(seed),
                sim_core::rules::WorldRules::colony(seed),
            );
            let hearth = crate::worldgen::colony::hearth(&e.resources).expect("hearth");
            e.resources.nav = sim_core::nav::NavState::default();
            if nav_state != 0 {
                crate::runtime::nav::ensure(&mut e.resources);
            }
            match nav_state {
                2 => {
                    e.resources.nav.set_cost(hearth, sim_core::nav::IMPASSABLE);
                    assert_eq!(e.resources.nav.region_of(hearth), 0);
                }
                3 => e.resources.building_kinds.clear(),
                4 => e.resources.rules.colony.as_mut().expect("colony").space = None,
                5 => e.resources.rules.colony = None,
                _ => {}
            }
            let nodes: Vec<_> = e.resources.colony.nodes.keys().copied().collect();
            for supply_state in 0..4 {
                e.resources.colony.designations.clear();
                e.resources.steward.vetoes.clear();
                if supply_state == 1 {
                    for &tile in &nodes {
                        e.resources
                            .colony
                            .designations
                            .insert(tile, Designation::Quarry);
                    }
                } else if supply_state == 2 {
                    for (i, &tile) in nodes.iter().enumerate() {
                        if i % 3 == 0 {
                            // The original accepts any designation as marked supply.
                            e.resources
                                .colony
                                .designations
                                .insert(tile, Designation::Chop);
                        } else if i % 3 == 1 {
                            e.resources
                                .steward
                                .vetoes
                                .insert(Target::Tile(tile.0, tile.1), u64::MAX);
                        }
                    }
                } else if supply_state == 3 {
                    let kinds: std::collections::BTreeSet<_> = e
                        .resources
                        .colony
                        .nodes
                        .values()
                        .map(|node| node.kind.yields())
                        .collect();
                    for (i, kind) in kinds.into_iter().enumerate() {
                        e.resources.colony.ground_items.insert(
                            (i as u32, 0),
                            ItemStack {
                                kind,
                                count: u16::MAX,
                                quality: 50,
                                material: None,
                            },
                        );
                        e.world.spawn((MaterialNeeds::single(kind, 120),));
                    }
                }
                let expected = reference(&e.world, &e.resources, hearth, &recipes);
                nonempty += usize::from(!expected.is_empty());
                empty += usize::from(expected.is_empty());
                assert_eq!(
                    inputs(&e.world, &e.resources, hearth, &recipes),
                    expected,
                    "seed={seed} nav={nav_state} supply={supply_state}"
                );
            }
        }
    }
    assert!(nonempty > 0, "oracle must exercise actual quarry commands");
    assert!(
        empty > 0,
        "oracle must exercise satisfied or disabled demand"
    );
}

#[test]
fn inputs_preserve_independent_dirty_nav_cache_effects() {
    use sim_core::colony::{Designation, ItemStack};
    use sim_core::nav::{CostMap, NavStats, COST_DOOR};
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    for scenario in 0..7 {
        let run = |optimized: bool| {
            let mut e = crate::worldgen::build_world_with(
                &crate::worldgen::WorldGenConfig::colony(0),
                sim_core::rules::WorldRules::colony(0),
            );
            let hearth = crate::worldgen::colony::hearth(&e.resources).expect("hearth");
            // This fixture isolates quarry planning's lazy navigation effect.
            // Clay discovery has its own command/knowledge integration fixture.
            let mut node = *e.resources.colony.nodes.values().next().expect("node");
            node.kind = sim_core::colony::NodeKind::Boulder;
            let kind = node.kind.yields();
            let tile = (hearth.0 + 1, hearth.1);
            e.resources.colony.nodes.clear();
            e.resources.colony.designations.clear();
            e.resources.colony.ground_items.clear();
            e.resources.steward.vetoes.clear();
            e.resources.colony.nodes.insert(tile, node);
            let mut world = World::new();
            let mut practitioner = sim_core::colony::knowledge::Knowledge::default();
            practitioner.set(
                sim_core::colony::knowledge::id("stone_quarrying").expect("quarry knowledge"),
                1000,
                sim_core::colony::knowledge::Source::Seeded,
                0,
            );
            world.spawn((practitioner,));
            if scenario != 0 {
                world.spawn((MaterialNeeds::single(kind, 10),));
            }
            match scenario {
                1 => e.resources.colony.nodes.clear(),
                2 => {
                    e.resources
                        .colony
                        .designations
                        .insert(tile, Designation::Quarry);
                }
                3 => {
                    e.resources
                        .steward
                        .vetoes
                        .insert(Target::Tile(tile.0, tile.1), u64::MAX);
                }
                4 => {
                    let other = e.resources.colony.nodes.values().next().expect("node").kind;
                    let different = [
                        sim_core::colony::NodeKind::Boulder,
                        sim_core::colony::NodeKind::FlintOutcrop,
                        sim_core::colony::NodeKind::ClayBank,
                    ]
                    .into_iter()
                    .find(|k| k.yields() != other.yields())
                    .expect("different output");
                    e.resources.colony.nodes.get_mut(&tile).expect("node").kind = different;
                }
                5 => {
                    e.resources.colony.ground_items.insert(
                        hearth,
                        ItemStack {
                            kind,
                            count: 100,
                            quality: 50,
                            material: None,
                        },
                    );
                }
                _ => {}
            }
            let grid = &e.resources.tile_grid;
            e.resources
                .nav
                .install(CostMap::new(grid.width, grid.height));
            assert_ne!(e.resources.nav.region_of(hearth), 0);
            assert!(e.resources.nav.set_cost((0, 0), COST_DOOR));
            *e.resources.nav.stats.borrow_mut() = NavStats::default();
            let before = *e.resources.nav.stats.borrow();
            let commands = if optimized {
                inputs(&world, &e.resources, hearth, &[])
            } else {
                reference(&world, &e.resources, hearth, &[])
            };
            let after = *e.resources.nav.stats.borrow();
            let region = e.resources.nav.region_of(hearth);
            let observed = *e.resources.nav.stats.borrow();
            (commands, before, after, region, observed)
        };
        let expected = run(false);
        let actual = run(true);
        assert_eq!(actual, expected, "scenario={scenario}");
        assert_eq!(expected.2.region_builds, u64::from(scenario >= 5));
        assert_eq!(expected.4.region_builds, 1);
        if scenario == 6 {
            assert!(!expected.0.is_empty());
        }
    }
}

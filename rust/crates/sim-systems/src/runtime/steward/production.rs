//! Production bootstrap inputs, derived from registered workshop and recipe requirements.
use crate::runtime::jobs::bills::maintain_count;
use hecs::World;
use sim_core::colony::{steward::Target, ColonySite, Command, ItemKind, MaterialNeeds, RecipeKind};
use sim_engine::SimResources;
use std::collections::BTreeMap;
#[path = "production_profile.rs"]
mod profile;
use profile::{InputProfile, Phase};

/// Quarry only the nearest unmarked nodes needed by planned production.
pub(super) fn inputs(
    world: &World,
    res: &SimResources,
    hearth: (u32, u32),
    recipes: &[(RecipeKind, u16)],
) -> Vec<Command> {
    inputs_with_discovery(world, res, hearth, recipes, BTreeMap::new())
}

/// Food production may request authored discovery materials before a recipe is known.
pub(super) fn inputs_with_discovery(
    world: &World,
    res: &SimResources,
    hearth: (u32, u32),
    recipes: &[(RecipeKind, u16)],
    discovery: BTreeMap<ItemKind, u32>,
) -> Vec<Command> {
    let mut profile = InputProfile::new(res.current_tick, recipes.len(), res.colony.nodes.len());
    let profiling = profile.enabled();
    let Some(colony_rule) = res.rules.colony else {
        return Vec::new();
    };
    let mut need: BTreeMap<ItemKind, u32> = discovery;
    let mut workshops = std::collections::BTreeSet::new();
    profile.phase(Phase::Recipes);
    for &(recipe, target) in recipes {
        let Some(recipe) = recipe.recipe() else {
            continue;
        };
        profile.phase(Phase::Knowledge);
        let known = crate::runtime::knowledge::anyone_practises(
            world,
            sim_core::colony::knowledge::gates().recipe(recipe.id),
        );
        profile.phase(Phase::Recipes);
        if !known {
            continue;
        }
        profile.known();
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
    profile.phase(Phase::Materials);
    for (_, materials) in world.query::<&MaterialNeeds>().iter() {
        profile.material_entity();
        for line in materials.used() {
            profile.material_line();
            *need.entry(line.kind).or_default() +=
                u32::from(line.needed.saturating_sub(line.delivered));
        }
    }
    profile.phase(Phase::Other);
    let mut camp_region: Option<Option<u32>> = None;
    let mut camp_reaches = |tile| {
        let region = camp_region.get_or_insert_with(|| {
            if colony_rule.space.is_some() && res.nav.is_built() {
                crate::worldgen::colony::hearth(res)
                    .map(|hearth| res.nav.region_of(hearth))
                    .filter(|&region| region != 0)
            } else {
                None
            }
        });
        region.is_none_or(|region| res.nav.reaches(region, tile))
    };
    let mut commands = Vec::new();
    for (kind, wanted) in need {
        profile.begin_kind(kind, wanted, &res.nav);
        profile.phase(Phase::Marked);
        let marked: u32 = res
            .colony
            .nodes
            .iter()
            .filter(|(tile, node)| {
                node.kind.yields() == kind && res.colony.designations.contains_key(tile)
            })
            .map(|(_, node)| u32::from(node.left))
            .sum();
        profile.phase(Phase::Count);
        let available = maintain_count(world, res, kind);
        let mut deficit = wanted.saturating_sub(available.saturating_add(marked));
        profile.demand(marked, available, deficit);
        profile.phase(Phase::Filter);
        let mut scanned = 0usize;
        let mut reaches = 0usize;
        let mut eligible = res.colony.nodes.iter().filter(|(tile, node)| {
            if profiling {
                scanned += 1;
            }
            node.kind.yields() == kind
                && !res.colony.designations.contains_key(tile)
                && !res
                    .steward
                    .vetoed(&Target::Tile(tile.0, tile.1), res.current_tick)
        });
        if deficit == 0 {
            // The old discarded scan could build lazy nav regions on its first eligible node.
            if let Some((&tile, _)) = eligible.next() {
                if profiling {
                    reaches += 1;
                }
                camp_reaches(tile);
            }
            profile.finish_kind([scanned, reaches, 0, 0], &res.nav);
            continue;
        }
        let mut nodes: Vec<_> = eligible
            .filter(|(tile, _)| {
                if profiling {
                    reaches += 1;
                }
                camp_reaches(**tile)
            })
            .collect();
        let candidates = nodes.len();
        profile.phase(Phase::Sort);
        nodes.sort_by_key(|(tile, _)| {
            (
                tile.0.abs_diff(hearth.0).max(tile.1.abs_diff(hearth.1)),
                tile.1,
                tile.0,
            )
        });
        profile.phase(Phase::Emit);
        let emitted_before = commands.len();
        for (&(x, y), node) in nodes {
            if deficit == 0 {
                break;
            }
            let clay = node.kind == sim_core::colony::NodeKind::ClayBank;
            let designation = if clay {
                sim_core::colony::Designation::ClayPit
            } else {
                sim_core::colony::Designation::Quarry
            };
            if !crate::runtime::knowledge::anyone_practises(
                world,
                sim_core::colony::knowledge::gates().designation(designation),
            ) {
                continue;
            }
            commands.push(if clay {
                Command::DesignateClay {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                }
            } else {
                Command::DesignateQuarry {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                }
            });
            deficit = deficit.saturating_sub(u32::from(node.left));
        }
        profile.finish_kind(
            [
                scanned,
                reaches,
                candidates,
                commands.len() - emitted_before,
            ],
            &res.nav,
        );
    }
    commands
}

/// Old OFF sessions retain player ownership in the ordinary command log.
pub(super) fn player_edited_bill(res: &SimResources, workshop: u32, bill: u32) -> bool {
    res.colony.command_log.iter().any(|record| {
        record.result.accepted
            && match record.command {
                Command::SetBillMode {
                    workshop: w,
                    bill: b,
                    ..
                }
                | Command::SetBillSuspended {
                    workshop: w,
                    bill: b,
                    ..
                }
                | Command::MoveBill {
                    workshop: w,
                    bill: b,
                    ..
                } => w == workshop && b == bill,
                _ => false,
            }
    })
}

#[cfg(test)]
#[path = "production_optimization_tests.rs"]
mod optimization_tests;

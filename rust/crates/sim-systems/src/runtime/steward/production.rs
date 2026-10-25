//! Production bootstrap inputs, derived from registered workshop and recipe requirements.
use crate::runtime::jobs::bills::maintain_count;
use hecs::World;
use sim_core::colony::{steward::Target, ColonySite, Command, ItemKind, MaterialNeeds, RecipeKind};
use sim_engine::SimResources;
use std::collections::BTreeMap;

/// Quarry only the nearest unmarked nodes needed by planned production.
pub(super) fn inputs(
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

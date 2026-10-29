//! Small standing cooking bills share one daily food budget and respect player edits.
use super::FoodSummary;
use hecs::World;
use sim_core::colony::knowledge::{self, Knowledge};
use sim_core::colony::steward::{FoodPlan, Target};
use sim_core::colony::{BillMode, ColonySite, Command, RecipeKind};
use sim_engine::SimResources;

pub(super) fn plan(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> Vec<Command> {
    let mut available = Vec::new();
    let practice = world
        .query::<&Knowledge>()
        .iter()
        .fold(0, |mask, (_, knowledge)| mask | knowledge.practice());
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        if !recipe
            .inputs
            .iter()
            .all(|(item, count)| facts.pieces.get(item).copied().unwrap_or(0) >= u64::from(*count))
        {
            continue;
        }
        if recipe.food_requirements.duration_days > 0 && !facts.preparing {
            continue;
        }
        if !crate::runtime::knowledge::anyone_practises(world, knowledge::gates().recipe(kind)) {
            continue;
        }
        let usable = res.colony.workshops.values().any(|shop| {
            recipe.supports_workshop(shop.kind)
                && crate::runtime::jobs::food_recipe_supplyable(res, shop, recipe, practice)
        });
        if usable {
            available.push(kind);
        }
    }
    let total = facts
        .daily_milli
        .saturating_mul(u64::from(policy.cooked_days_milli))
        .div_ceil(1_000_000);
    let share = total.div_ceil(u64::try_from(available.len()).unwrap_or(u64::MAX).max(1));
    let mut wanted = policy.vessel_recipes.clone();
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        let units = recipe
            .output
            .0
            .definition()
            .food
            .as_ref()
            .map_or(1, |food| food.units.max(1));
        let target = if available.contains(&kind) {
            u16::try_from(share.div_ceil(u64::from(units))).unwrap_or(u16::MAX)
        } else {
            0
        };
        wanted.push((kind, target));
    }
    let hearth = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()
        .unwrap_or((0, 0));
    let mut commands = super::super::production::inputs_with_discovery(
        world,
        res,
        hearth,
        &policy.vessel_recipes,
        super::dependencies::discovery_materials(world, policy),
    );
    for (kind, mut target) in wanted {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        if !crate::runtime::knowledge::anyone_practises(world, knowledge::gates().recipe(kind)) {
            target = 0;
        }
        if recipe.output.0.definition().pot_capacity > 0 {
            let deployed = res
                .food
                .pots
                .values()
                .filter(|pot| pot.vessel.kind == recipe.output.0)
                .count();
            target = target.saturating_sub(u16::try_from(deployed).unwrap_or(u16::MAX));
        }
        if let Some((&id, shop)) = res
            .colony
            .workshops
            .iter()
            .find(|(_, shop)| recipe.supports_workshop(shop.kind))
        {
            if res
                .steward
                .vetoed(&Target::Bill(id, kind.code()), res.current_tick)
            {
                continue;
            }
            if let Some(bill) = shop.bills.iter().find(|bill| bill.recipe == kind) {
                if !(bill.auto || steward_owns_recipe(res, id, kind))
                    || super::super::production::player_edited_bill(res, id, bill.id)
                {
                    continue;
                }
                if target == 0 {
                    if !bill.suspended {
                        commands.push(Command::SetBillSuspended {
                            workshop: id,
                            bill: bill.id,
                            suspended: true,
                        });
                    }
                } else {
                    if bill.suspended {
                        commands.push(Command::SetBillSuspended {
                            workshop: id,
                            bill: bill.id,
                            suspended: false,
                        });
                    }
                    if bill.mode != BillMode::Maintain(target) {
                        commands.push(Command::SetBillMode {
                            workshop: id,
                            bill: bill.id,
                            mode: BillMode::Maintain(target),
                        });
                    }
                }
            } else if target > 0 {
                commands.push(Command::AddBill {
                    workshop: id,
                    recipe: kind,
                    mode: BillMode::Maintain(target),
                });
            }
        } else if target > 0 || recipe.food_requirements.duration_days > 0 && facts.preparing {
            if let Some(command) = workshop(world, res, kind) {
                commands.push(command);
                break;
            }
        }
    }
    commands
}

fn workshop(world: &World, res: &SimResources, kind: RecipeKind) -> Option<Command> {
    let recipe = kind.recipe()?;
    if world
        .query::<&ColonySite>()
        .iter()
        .any(|(_, site)| recipe.supports_workshop(site.kind))
        || !crate::runtime::knowledge::anyone_practises(
            world,
            knowledge::gates().build(recipe.workshop),
        )
    {
        return None;
    }
    let hearth = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()?;
    // Candidate order is deterministic; the existing blueprint validator owns geometry.
    for radius in 0u32..res.tile_grid.width.max(res.tile_grid.height) {
        for y in hearth.1.saturating_sub(radius)
            ..=hearth
                .1
                .saturating_add(radius)
                .min(res.tile_grid.height.saturating_sub(1))
        {
            for x in hearth.0.saturating_sub(radius)
                ..=hearth
                    .0
                    .saturating_add(radius)
                    .min(res.tile_grid.width.saturating_sub(1))
            {
                if x.abs_diff(hearth.0).max(y.abs_diff(hearth.1)) != radius {
                    continue;
                }
                if res.steward.vetoed(&Target::Tile(x, y), res.current_tick) {
                    continue;
                }
                if crate::commands::camp_reaches(res, (x, y))
                    && crate::commands::check_blueprint(res, recipe.workshop, x, y).is_ok()
                {
                    return Some(Command::PlaceBlueprint {
                        kind: recipe.workshop,
                        x,
                        y,
                    });
                }
            }
        }
    }
    None
}

/// `Bill.auto` belongs to legacy engine bills; ordinary steward AddBill uses
/// the same false value as player commands. Accepted ownership is separate.
fn steward_owns_recipe(res: &SimResources, workshop: u32, recipe: RecipeKind) -> bool {
    res.steward.owned.iter().any(|owned| {
        matches!(&owned.command,
        Command::AddBill { workshop: owned_workshop, recipe: owned_recipe, .. }
            if *owned_workshop == workshop && *owned_recipe == recipe)
    })
}

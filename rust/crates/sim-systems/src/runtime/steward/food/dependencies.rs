//! Recipe prerequisites become ordinary material requests, never synthetic knowledge.
use hecs::World;
use sim_core::colony::knowledge::{self, Cond, Knowledge};
use sim_core::colony::steward::FoodPlan;
use sim_core::colony::ItemKind;
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn discovery_materials(world: &World, policy: &FoodPlan) -> BTreeMap<ItemKind, u32> {
    let practice = world
        .query::<&Knowledge>()
        .iter()
        .fold(0, |mask, (_, knowledge)| mask | knowledge.practice());
    let mut pending = Vec::new();
    for &(recipe, _) in &policy.vessel_recipes {
        let mask = knowledge::gates().recipe(recipe);
        pending.extend(
            knowledge::defs()
                .nodes
                .iter()
                .filter(|node| mask & knowledge::bit(node.id) != 0)
                .map(|node| node.id),
        );
    }
    let mut seen = BTreeSet::new();
    let mut demand: BTreeMap<ItemKind, u32> = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) || practice & knowledge::bit(id) != 0 {
            continue;
        }
        let Some(node) = knowledge::defs().get(id) else {
            continue;
        };
        pending.extend(node.prereqs.iter().copied());
        for condition in &node.discover {
            match *condition {
                Cond::Stockpile(kind, count) => {
                    demand
                        .entry(kind)
                        .and_modify(|n| *n = (*n).max(u32::from(count)))
                        .or_insert(u32::from(count));
                }
                Cond::Knows(other, _) => pending.push(other),
                _ => {}
            }
        }
    }
    demand
}

/// Hold one reachable physical vessel for each cooking workshop still needing one.
/// Once hauled to the shelf, spare vessels remain eligible for storage deployment.
pub(super) fn reserved_cooking_pots(res: &SimResources, policy: &FoodPlan) -> BTreeSet<(u32, u32)> {
    let mut held = BTreeSet::new();
    for shop in res.colony.workshops.values() {
        let needs_pot = policy
            .recipes
            .iter()
            .filter_map(|kind| kind.recipe())
            .any(|recipe| {
                recipe.supports_workshop(shop.kind)
                    && (recipe.food_requirements.requires_pot
                        || recipe.food_requirements.pot_alternative)
            });
        let local = shop
            .shelf
            .iter()
            .any(|stack| stack.count > 0 && stack.kind.definition().pot_capacity > 0)
            || res.food.pots.values().any(|pot| {
                pot.tile == shop.work_tile()
                    || (pot.tile.0 >= shop.top_left.0
                        && pot.tile.1 >= shop.top_left.1
                        && pot.tile.0 < shop.top_left.0.saturating_add(shop.size.0)
                        && pot.tile.1 < shop.top_left.1.saturating_add(shop.size.1))
            });
        if !needs_pot || local {
            continue;
        }
        let vessel = res
            .colony
            .ground_items
            .iter()
            .filter(|(tile, stack)| {
                stack.count > 0
                    && stack.kind.definition().pot_capacity > 0
                    && !held.contains(*tile)
                    && (!res.nav.is_built()
                        || res.nav.reaches(res.nav.region_of(shop.work_tile()), **tile))
            })
            .min_by_key(|(tile, _)| {
                (
                    tile.0
                        .abs_diff(shop.work_tile().0)
                        .max(tile.1.abs_diff(shop.work_tile().1)),
                    tile.1,
                    tile.0,
                )
            });
        if let Some((&tile, _)) = vessel {
            held.insert(tile);
        }
    }
    held
}

/// One missing recipe batch is enough to request an ingredient harvest. This
/// bounds pre-cooking stock without treating inedible grain as satiety reserves.
pub(super) fn ingredient_deficits(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &super::FoodSummary,
) -> BTreeMap<ItemKind, u64> {
    let practice = world
        .query::<&Knowledge>()
        .iter()
        .fold(0, |mask, (_, knowledge)| mask | knowledge.practice());
    let mut deficits: BTreeMap<ItemKind, u64> = BTreeMap::new();
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        if recipe.food_requirements.duration_days > 0 && !facts.preparing {
            continue;
        }
        if !crate::runtime::knowledge::anyone_practises(world, knowledge::gates().recipe(kind))
            || !res.colony.workshops.values().any(|shop| {
                recipe.supports_workshop(shop.kind)
                    && crate::runtime::jobs::food_recipe_supplyable(res, shop, recipe, practice)
            })
        {
            continue;
        }
        for &(item, count) in &recipe.inputs {
            let missing =
                u64::from(count).saturating_sub(facts.pieces.get(&item).copied().unwrap_or(0));
            deficits
                .entry(item)
                .and_modify(|n| *n = (*n).max(missing))
                .or_insert(missing);
        }
    }
    deficits
}

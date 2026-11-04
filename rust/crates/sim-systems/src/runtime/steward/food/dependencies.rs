//! Recipe prerequisites become ordinary material requests, never synthetic knowledge.
use hecs::World;
use sim_core::colony::knowledge::{self, Cond, Knowledge};
use sim_core::colony::steward::FoodPlan;
use sim_core::colony::{ItemKind, Job, JobKind, JobStage};
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
    if let Some(crop) = policy
        .cultivation
        .as_ref()
        .and_then(|plan| sim_core::colony::content::installed_or_bundled().crop(plan.crop))
    {
        pending.extend(crop.knowledge);
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

/// Procure only residual authored cooking demand; preservation keeps its winter budget.
pub(super) fn ingredient_deficits(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &super::FoodSummary,
) -> BTreeMap<ItemKind, u64> {
    let mut reserved = seed_inputs(world, res, policy, facts);
    if facts.usable_units.saturating_mul(1000) < facts.daily_milli {
        for (item, count) in &mut reserved {
            if item.definition().food.is_some() {
                *count = 0;
            }
        }
    }
    super::cooking::priority_targets(world, res, policy, facts, &reserved, true).missing
}

/// Assigned output already counts toward bills; its still-shelved input cannot count twice.
pub(super) fn unconsumed_craft_inputs(
    world: &World,
    res: &SimResources,
) -> BTreeMap<ItemKind, u64> {
    let mut inputs: BTreeMap<ItemKind, u64> = BTreeMap::new();
    for (_, job) in world.query::<&Job>().iter() {
        let JobKind::Craft { workshop, bill, .. } = job.kind else {
            continue;
        };
        if !matches!(job.stage, JobStage::ToSource) {
            continue;
        }
        let Some(recipe) = res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|shop| shop.bill(bill))
            .and_then(|bill| bill.recipe.recipe())
        else {
            continue;
        };
        for &(kind, amount) in &recipe.inputs {
            *inputs.entry(kind).or_default() += u64::from(amount);
        }
    }
    inputs
}

/// Authored conversion producing the configured crop's physical sowing item.
pub(super) fn seed_recipe(policy: &FoodPlan) -> Option<sim_core::colony::RecipeKind> {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let crop = catalog.crop(policy.cultivation.as_ref()?.crop)?;
    catalog.recipe_ids().find(|kind| {
        kind.recipe()
            .is_some_and(|recipe| recipe.output.0 == crop.seed_item)
    })
}

/// Physical inputs held for one seed reserve, plus unresolved discovery material floors.
pub(super) fn seed_inputs(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &super::FoodSummary,
) -> BTreeMap<ItemKind, u64> {
    let Some((seed, target)) = super::seed_reserve(res, policy, facts) else {
        return BTreeMap::new();
    };
    let mut reserved: BTreeMap<_, _> = discovery_materials(world, policy)
        .into_iter()
        .map(|(item, count)| (item, u64::from(count)))
        .collect();
    let Some(recipe) = seed_recipe(policy).and_then(|kind| kind.recipe()) else {
        let amount = reserved.entry(seed).or_default();
        *amount = amount.saturating_add(target);
        return reserved;
    };
    let missing = target.saturating_sub(seed_count(world, res, seed));
    if missing == 0 {
        return reserved;
    }
    let batches = missing.div_ceil(u64::from(recipe.output.1).max(1));
    for &(item, count) in &recipe.inputs {
        let amount = reserved.entry(item).or_default();
        *amount = amount.saturating_add(batches.saturating_mul(u64::from(count)));
    }
    reserved
}

/// Physical pieces across every owner, including cultivation carriers and workshop shelves.
pub(super) fn physical_pieces(world: &World, res: &SimResources, kind: ItemKind) -> u64 {
    let ground = res
        .colony
        .ground_items
        .values()
        .filter(|stack| stack.kind == kind)
        .map(|stack| u64::from(stack.count))
        .sum::<u64>();
    let carried = world
        .query::<&sim_core::colony::Carry>()
        .iter()
        .filter(|(_, carry)| carry.kind == kind)
        .map(|(_, carry)| u64::from(carry.count))
        .sum::<u64>();
    let stored = res
        .colony
        .workshops
        .values()
        .flat_map(|shop| shop.shelf.iter())
        .chain(res.food.pots.values().flat_map(|pot| pot.contents.iter()))
        .filter(|stack| stack.kind == kind)
        .map(|stack| u64::from(stack.count))
        .sum::<u64>();
    ground.saturating_add(carried).saturating_add(stored)
}

/// Bootstrap count: physical seed plus each assigned conversion output exactly once.
pub(super) fn seed_count(world: &World, res: &SimResources, seed: ItemKind) -> u64 {
    let making = world
        .query::<&Job>()
        .iter()
        .filter_map(|(_, job)| {
            let JobKind::Craft { workshop, bill, .. } = job.kind else {
                return None;
            };
            let recipe = res
                .colony
                .workshops
                .get(&workshop)?
                .bill(bill)?
                .recipe
                .recipe()?;
            (recipe.output.0 == seed).then_some(u64::from(recipe.output.1))
        })
        .sum::<u64>();
    physical_pieces(world, res, seed).saturating_add(making)
}

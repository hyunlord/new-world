//! One physical reserve deficit shared by preservation bills and ingredient requests.
use super::FoodSummary;
use crate::runtime::food::{items, storage};
use hecs::World;
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::knowledge::{self, Knowledge};
use sim_core::colony::steward::{FoodPlan, Target};
use sim_core::colony::{ItemKind, Job, JobKind, JobStage, RecipeKind};
use sim_core::components::{Agent, Position};
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct Budget {
    pub targets: BTreeMap<RecipeKind, u16>,
    pub workshops: BTreeMap<RecipeKind, u32>,
    pub inputs: BTreeMap<ItemKind, u64>,
}

pub(super) fn budget(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> Budget {
    let mut out = Budget::default();
    if !facts.preparing {
        return out;
    }
    let practice = world
        .query::<&Knowledge>()
        .iter()
        .fold(0, |mask, (_, k)| mask | k.practice());
    let mut outputs = BTreeSet::new();
    let recipes: Vec<_> = policy
        .recipes
        .iter()
        .filter_map(|&kind| {
            let recipe = kind.recipe()?;
            let days = recipe.food_requirements.duration_days;
            let food = recipe.output.0.definition().food.as_ref()?;
            if days == 0
                || days > facts.winter_in
                || !food.raw_edible
                || !super::summary::covers_winter(
                    food.lifetime_days(FoodStorageClass::Stockpile),
                    facts.winter_in - days,
                    policy.winter_keep_days,
                )
                || !knowledge_ready(world, kind)
            {
                return None;
            }
            let shop = res.colony.workshops.values().find(|shop| {
                recipe.supports_workshop(shop.kind)
                    && crate::runtime::jobs::food_recipe_supplyable(res, shop, recipe, practice)
                    && !res
                        .steward
                        .vetoed(&Target::Bill(shop.id, kind.code()), res.current_tick)
                    && shop
                        .bills
                        .iter()
                        .find(|bill| bill.recipe == kind)
                        .is_none_or(|bill| {
                            (bill.auto || super::cooking::steward_owns_recipe(res, shop.id, kind))
                                && !super::super::production::player_edited_bill(
                                    res, shop.id, bill.id,
                                )
                        })
            })?;
            // One target per physical output prevents duplicate recipes counting the same stock twice.
            outputs
                .insert(recipe.output.0)
                .then_some((kind, recipe, shop.id))
        })
        .collect();
    if recipes.is_empty() {
        return out;
    }
    let reserve = facts
        .daily_milli
        .saturating_mul(u64::from(facts.winter_days))
        .div_ceil(1000)
        .max(super::winter_need(facts.adults));
    let commitments = commitments(world, res, policy, facts, &outputs);
    let deficit = reserve
        .saturating_sub(facts.winter_units)
        .saturating_sub(commitments);
    let count = u64::try_from(recipes.len()).unwrap_or(u64::MAX);
    for (index, (kind, recipe, workshop)) in recipes.into_iter().enumerate() {
        let units = recipe
            .output
            .0
            .definition()
            .food
            .as_ref()
            .map_or(1, |food| food.units.max(1));
        let share =
            deficit / count + u64::from(u64::try_from(index).unwrap_or(u64::MAX) < deficit % count);
        let pieces = share.div_ceil(u64::from(units));
        let existing = crate::runtime::jobs::bills::bill_count(world, res, recipe.output.0);
        let target = u16::try_from(u64::from(existing).saturating_add(pieces)).unwrap_or(u16::MAX);
        out.targets.insert(kind, target);
        out.workshops.insert(kind, workshop);
        let batches = pieces.div_ceil(u64::from(recipe.output.1.max(1)));
        for &(item, amount) in &recipe.inputs {
            let entry = out.inputs.entry(item).or_default();
            *entry = entry.saturating_add(batches.saturating_mul(u64::from(amount)));
        }
    }
    out
}

fn knowledge_ready(world: &World, kind: RecipeKind) -> bool {
    crate::runtime::knowledge::anyone_practises(world, knowledge::gates().recipe(kind))
}

fn commitments(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
    outputs: &BTreeSet<ItemKind>,
) -> u64 {
    let day = crate::runtime::food::day(res);
    let winter_start = day.saturating_add(facts.winter_in);
    let winter_end = winter_start.saturating_add(policy.winter_keep_days);
    let positions: BTreeMap<_, _> = world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (agent, pos))| (agent.id, (pos.x, pos.y)))
        .collect();
    let mut units = 0_u64;
    for (&owner, age) in &res.food.ages {
        if storage::class(res, owner) != FoodStorageClass::Ground {
            continue;
        }
        let Some(stack) = items::stack(world, res, owner) else {
            continue;
        };
        if !outputs.contains(&stack.kind) {
            continue;
        }
        let tile = match owner {
            FoodOwner::Ground(tile) => Some(tile),
            FoodOwner::Carried(id) => positions.get(&id).copied(),
            FoodOwner::Shelf { workshop, .. } => res
                .colony
                .workshops
                .get(&workshop)
                .map(|shop| shop.work_tile()),
            FoodOwner::Pot { .. } => None,
        };
        if !tile.is_some_and(|tile| crate::commands::camp_reaches(res, tile))
            || !storage::remaining_days(world, res, owner, day).is_ok_and(|remaining| remaining > 0)
        {
            continue;
        }
        let Some(food) = &stack.kind.definition().food else {
            continue;
        };
        if age
            .born_day
            .saturating_add(food.lifetime_days(FoodStorageClass::Stockpile))
            >= winter_end
        {
            units = units.saturating_add(u64::from(stack.count) * u64::from(food.units));
        }
    }
    for batch in &res.food.pending_drying {
        if !res.colony.workshops.contains_key(&batch.workshop)
            || !crate::commands::camp_reaches(res, batch.tile)
        {
            continue;
        }
        let Some(food) = &batch.output.kind.definition().food else {
            continue;
        };
        let Ok(born) = u32::try_from(crate::runtime::resource_regen::day_index_at(
            res,
            batch.due_tick,
        )) else {
            continue;
        };
        // Overdue blocked output already ages in its actual drop class, like drying::advance.
        let still_fresh = born > day
            || born.saturating_add(
                food.lifetime_days(storage::class(res, FoodOwner::Ground(batch.tile))),
            ) > day;
        if food.raw_edible
            && still_fresh
            && born <= winter_start
            && born.saturating_add(food.lifetime_days(FoodStorageClass::Stockpile)) >= winter_end
        {
            units = units.saturating_add(u64::from(batch.count) * u64::from(food.units));
        }
    }
    for (_, job) in world.query::<&Job>().iter() {
        let (JobKind::Craft { workshop, bill, .. }, JobStage::Working { ticks_left }) =
            (job.kind, job.stage)
        else {
            continue;
        };
        let Some(shop) = res.colony.workshops.get(&workshop) else {
            continue;
        };
        let Some(recipe) = shop.bill(bill).and_then(|bill| bill.recipe.recipe()) else {
            continue;
        };
        if recipe.food_requirements.duration_days == 0
            || !crate::commands::camp_reaches(res, shop.work_tile())
        {
            continue;
        }
        let Some(food) = &recipe.output.0.definition().food else {
            continue;
        };
        let due = res
            .current_tick
            .saturating_add(u64::from(ticks_left))
            .saturating_add(
                u64::from(recipe.food_requirements.duration_days).saturating_mul(res.ticks_per_day),
            );
        let Ok(born) = u32::try_from(crate::runtime::resource_regen::day_index_at(res, due)) else {
            continue;
        };
        if food.raw_edible
            && born <= winter_start
            && born.saturating_add(food.lifetime_days(FoodStorageClass::Stockpile)) >= winter_end
        {
            units = units.saturating_add(u64::from(recipe.output.1) * u64::from(food.units));
        }
    }
    units
}

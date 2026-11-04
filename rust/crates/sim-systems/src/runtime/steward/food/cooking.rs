//! Small standing cooking bills share one daily food budget and respect player edits.
use super::FoodSummary;
use hecs::World;
use sim_core::colony::knowledge::{self, Knowledge};
use sim_core::colony::steward::{FoodPlan, Target};
use sim_core::colony::{BillMode, ColonySite, Command, Job, JobKind, RecipeKind};
use sim_engine::SimResources;

/// One adult must supply both the primary gate and this shop's food requirements.
pub(super) fn has_practitioner(
    world: &World,
    res: &SimResources,
    recipe: &sim_core::colony::workshop::Recipe,
    accepts: impl Fn(u32) -> bool,
) -> bool {
    let gated = crate::runtime::knowledge::rule(res).is_some();
    let primary = knowledge::gates().recipe(recipe.id);
    world
        .query::<(&sim_core::components::Agent, Option<&Knowledge>)>()
        .without::<&sim_core::components::Child>()
        .iter()
        .any(|(_, (_, known))| {
            let practice = if gated {
                known.map_or(0, |known| known.practice())
            } else {
                u32::MAX
            };
            primary & !practice == 0 && accepts(practice)
        })
}

pub(super) fn edible_units(item: sim_core::colony::ItemKind, raw_only: bool) -> u64 {
    item.definition()
        .food
        .as_ref()
        .filter(|food| !raw_only || food.raw_edible)
        .map_or(0, |food| u64::from(food.units))
}

// None is portable public stock; Some(shop) is fresh input already on that shelf.
pub(super) type CookingInputs =
    std::collections::BTreeMap<(Option<u32>, sim_core::colony::ItemKind), u64>;

fn cooking_inputs(world: &World, res: &SimResources) -> CookingInputs {
    use crate::runtime::food::{items, storage};
    use sim_core::colony::food::FoodOwner;
    use sim_core::components::{Agent, Position};
    let positions = world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (agent, position))| (agent.id, (position.x, position.y)))
        .collect();
    let day = crate::runtime::food::day(res);
    let mut inputs = CookingInputs::new();
    for owner in res.food.ages.keys().copied() {
        let Some(stack) = items::stack(world, res, owner) else {
            continue;
        };
        if !super::summary::reachable(&positions, res, owner)
            || !storage::remaining_days(world, res, owner, day).is_ok_and(|left| left > 0)
        {
            continue;
        }
        let local = match owner {
            FoodOwner::Shelf { workshop, .. } => Some(workshop),
            _ if storage::portable(res, owner, stack) => None,
            _ => continue,
        };
        *inputs.entry((local, stack.kind)).or_default() += u64::from(stack.count);
    }
    inputs
}

pub(super) fn input_count(
    inputs: &CookingInputs,
    shop: u32,
    item: sim_core::colony::ItemKind,
) -> u64 {
    inputs.get(&(None, item)).copied().unwrap_or(0)
        + inputs.get(&(Some(shop), item)).copied().unwrap_or(0)
}

pub(super) fn consume_inputs(
    inputs: &mut CookingInputs,
    shop: u32,
    item: sim_core::colony::ItemKind,
    mut count: u64,
) {
    for key in [(Some(shop), item), (None, item)] {
        if let Some(available) = inputs.get_mut(&key) {
            let taken = (*available).min(count);
            *available -= taken;
            count -= taken;
        }
    }
}

/// Available food ingredients after actual workshop-local Craft commitments.
pub(super) fn free_inputs(world: &World, res: &SimResources) -> CookingInputs {
    let mut inputs = cooking_inputs(world, res);
    for (_, job) in world.query::<&Job>().iter() {
        let JobKind::Craft { workshop, bill, .. } = job.kind else {
            continue;
        };
        if !matches!(job.stage, sim_core::colony::JobStage::ToSource) {
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
        for (item, count) in &recipe.inputs {
            if let Some(available) = inputs.get_mut(&(Some(workshop), *item)) {
                *available = available.saturating_sub(u64::from(*count));
            }
        }
    }
    inputs
}

/// Allocate one ingredient once, using only shelves whose shop can run this recipe.
pub(super) fn take_recipe_input(
    world: &World,
    res: &SimResources,
    inputs: &mut CookingInputs,
    recipe: &sim_core::colony::workshop::Recipe,
    item: sim_core::colony::ItemKind,
    mut needed: u64,
) -> u64 {
    let requested = needed;
    // Match plan's first compatible workshop; another shelf cannot supply its bill.
    let Some(shop) = res
        .colony
        .workshops
        .values()
        .find(|shop| recipe.supports_workshop(shop.kind))
    else {
        return 0;
    };
    if !has_practitioner(world, res, recipe, |practice| {
        crate::runtime::jobs::food_recipe_supplyable(res, shop, recipe, practice)
    }) {
        return 0;
    }
    if let Some(local) = inputs.get_mut(&(Some(shop.id), item)) {
        let taken = (*local).min(needed);
        *local -= taken;
        needed -= taken;
    }
    if let Some(public) = inputs.get_mut(&(None, item)) {
        let taken = (*public).min(needed);
        *public -= taken;
        needed -= taken;
    }
    requested - needed
}

fn available_recipes(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    seed_reserve: &std::collections::BTreeMap<sim_core::colony::ItemKind, u64>,
) -> Vec<RecipeKind> {
    let inputs = cooking_inputs(world, res);
    let mut available = Vec::new();
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        let assigned = assigned_crafts(world, res, kind);
        if recipe.food_requirements.duration_days > 0 {
            continue;
        }
        if !crate::runtime::knowledge::anyone_practises(world, knowledge::gates().recipe(kind)) {
            continue;
        }
        let usable = res.colony.workshops.values().any(|shop| {
            recipe.supports_workshop(shop.kind)
                && (assigned > 0
                    || recipe.inputs.iter().all(|(item, count)| {
                        input_count(&inputs, shop.id, *item)
                            .saturating_sub(seed_reserve.get(item).copied().unwrap_or(0))
                            >= u64::from(*count)
                    }))
                && has_practitioner(world, res, recipe, |practice| {
                    crate::runtime::jobs::food_recipe_supplyable(res, shop, recipe, practice)
                })
        });
        if usable || assigned > 0 {
            available.push(kind);
        }
    }
    available
}

/// Remaining net edible capacity after fresh food and each accepted output once.
pub(super) fn food_headroom(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> u64 {
    use sim_core::colony::JobStage;
    let (immediate, durable) = super::harvest::targets(policy, facts);
    let emergency = facts.usable_units.saturating_mul(1000) < facts.daily_milli;
    // Reserve accepted edible outputs once, independent of overlapping food
    // horizons. Future drying is a commitment, never immediately usable food.
    let mut committed: u64 = res
        .food
        .pending_drying
        .iter()
        .filter(|batch| !emergency || batch.due_tick <= res.current_tick)
        .map(|batch| u64::from(batch.count) * edible_units(batch.output.kind, true))
        .sum();
    for (_, job) in world.query::<&Job>().iter() {
        let JobKind::Craft { workshop, bill, .. } = job.kind else {
            continue;
        };
        let Some(recipe) = res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|shop| shop.bill(bill))
            .and_then(|bill| bill.recipe.recipe())
        else {
            continue;
        };
        if emergency && recipe.food_requirements.duration_days > 0 {
            continue;
        }
        let output = u64::from(recipe.output.1) * edible_units(recipe.output.0, true);
        let raw: u64 = recipe
            .inputs
            .iter()
            .map(|(item, count)| u64::from(*count) * edible_units(*item, true))
            .sum();
        // ToSource inputs still exist in usable food; Working inputs have been
        // consumed. Finished passive crafts live only in pending_drying above.
        let contribution = match job.stage {
            JobStage::ToSource => output.saturating_sub(raw),
            JobStage::Working { .. } => output,
            _ => 0,
        };
        committed = committed.saturating_add(contribution);
    }
    immediate
        .max(durable)
        .saturating_sub(facts.usable_units)
        .saturating_sub(committed)
}

pub(super) struct Allocation {
    pub targets: std::collections::BTreeMap<RecipeKind, u16>,
    pub missing: std::collections::BTreeMap<sim_core::colony::ItemKind, u64>,
    pub preservation: super::preservation::Budget,
}

// Physical cooking, then preservation, then missing-input cooking share one capacity.
pub(super) fn priority_targets(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
    reserved: &std::collections::BTreeMap<sim_core::colony::ItemKind, u64>,
    procure: bool,
) -> Allocation {
    use sim_core::colony::ItemKind;
    let available = available_recipes(world, res, policy, reserved);
    let total = facts
        .daily_milli
        .saturating_mul(u64::from(policy.cooked_days_milli))
        .div_ceil(1_000_000);
    // Match the executor: all physical cooked stock, assigned crafts and drying.
    // Each output kind appears once, including meals outside this recipe policy.
    let occupied: u64 = ItemKind::all()
        .filter(|kind| {
            kind.definition()
                .food
                .as_ref()
                .is_some_and(|food| food.cooked && food.raw_edible)
        })
        .map(|kind| {
            u64::from(crate::runtime::jobs::bills::bill_count(world, res, kind))
                * edible_units(kind, false)
        })
        .sum();
    let mut room = total.saturating_sub(occupied);
    let mut headroom = food_headroom(world, res, policy, facts);
    let mut inputs = free_inputs(world, res);
    let mut deductions = reserved.clone();
    for ((_, item), count) in &mut inputs {
        let deduction = deductions.entry(*item).or_default();
        let taken = (*count).min(*deduction);
        *count -= taken;
        *deduction -= taken;
    }
    let mut targets = std::collections::BTreeMap::new();
    let mut allocated = false;
    let mut outputs = std::collections::BTreeMap::new();
    let mut missing = std::collections::BTreeMap::new();
    let mut preservation = super::preservation::Budget::default();
    let emergency = facts.usable_units.saturating_mul(1000) < facts.daily_milli;
    let mut physical_targets = None;
    for project in [false, true] {
        if project {
            if !emergency {
                preservation = super::preservation::budget(
                    world,
                    res,
                    policy,
                    facts,
                    &mut inputs,
                    &mut headroom,
                );
                if !procure {
                    break;
                }
            } else if !procure {
                // Both callers reserve the same urgent missing-input meals, but
                // only procurement may turn hypothetical inputs into demand.
                physical_targets = Some(targets.clone());
            }
        }
        let mut visited = std::collections::BTreeSet::new();
        for &kind in &policy.recipes {
            if !visited.insert(kind) {
                continue;
            }
            if !project && !available.contains(&kind) {
                continue;
            }
            let Some(recipe) = kind.recipe() else {
                continue;
            };
            if recipe.food_requirements.duration_days > 0 {
                continue;
            }
            let Some(shop) = res
                .colony
                .workshops
                .values()
                .find(|shop| recipe.supports_workshop(shop.kind))
            else {
                continue;
            };
            if res
                .steward
                .vetoed(&Target::Bill(shop.id, kind.code()), res.current_tick)
                || shop
                    .bills
                    .iter()
                    .find(|bill| bill.recipe == kind)
                    .is_some_and(|bill| {
                        !(bill.auto || steward_owns_recipe(res, shop.id, kind))
                            || super::super::production::player_edited_bill(res, shop.id, bill.id)
                    })
                || !has_practitioner(world, res, recipe, |practice| {
                    crate::runtime::jobs::food_recipe_supplyable(res, shop, recipe, practice)
                })
            {
                continue;
            }
            let item_units = edible_units(recipe.output.0, true);
            let output = u64::from(recipe.output.1) * item_units;
            if output == 0
                || outputs
                    .get(&recipe.output.0)
                    .is_some_and(|chosen| *chosen != kind)
            {
                continue;
            }
            let counted = u64::from(crate::runtime::jobs::bills::bill_count(
                world,
                res,
                recipe.output.0,
            ));
            let raw: u64 = recipe
                .inputs
                .iter()
                .map(|(item, count)| u64::from(*count) * edible_units(*item, true))
                .sum();
            let net = output.saturating_sub(raw);
            let capacity = if net == 0 {
                room
            } else {
                headroom.saturating_mul(output) / net
            };
            let allowance = room.min(capacity);
            let physical = recipe
                .inputs
                .iter()
                .map(|(item, count)| {
                    input_count(&inputs, shop.id, *item) / u64::from(*count).max(1)
                })
                .min()
                .unwrap_or(0);
            let physical = if project { u64::MAX } else { physical };
            let mut batches = (allowance / output).min(physical);
            let charge = |batches: u64| {
                let physical_raw: u64 = recipe
                    .inputs
                    .iter()
                    .map(|(item, count)| {
                        (batches * u64::from(*count)).min(input_count(&inputs, shop.id, *item))
                            * edible_units(*item, true)
                    })
                    .sum();
                (batches * output).saturating_sub(physical_raw)
            };
            if project {
                let (mut low, mut high) = (0, batches);
                while low < high {
                    let mid = low + (high - low).div_ceil(2);
                    if charge(mid) <= headroom {
                        low = mid;
                    } else {
                        high = mid - 1;
                    }
                }
                batches = low;
            }
            // Preserve the small-budget Maintain(2) meal: one indivisible batch may
            // cross a positive shortage, never one rounding allowance per recipe.
            if batches == 0
                && allowance > 0
                && physical > 0
                && !allocated
                && (!project || charge(1) == 0 || headroom > 0)
            {
                batches = 1;
            }
            let extra = allowance.min(batches.saturating_mul(output));
            let previous = targets
                .get(&kind)
                .copied()
                .map(u64::from)
                .unwrap_or(counted);
            let target = previous.saturating_add(extra.div_ceil(item_units));
            targets.insert(kind, u16::try_from(target).unwrap_or(u16::MAX));
            if batches > 0 {
                outputs.insert(recipe.output.0, kind);
                allocated = true;
                room = room.saturating_sub(batches * output);
                headroom = headroom.saturating_sub(if project {
                    charge(batches)
                } else {
                    batches * net
                });
                for (item, count) in &recipe.inputs {
                    let needed = batches * u64::from(*count);
                    let deficit = needed.saturating_sub(input_count(&inputs, shop.id, *item));
                    *missing.entry(*item).or_default() += deficit;
                    consume_inputs(&mut inputs, shop.id, *item, needed);
                }
            }
        }
    }
    if emergency {
        preservation =
            super::preservation::budget(world, res, policy, facts, &mut inputs, &mut headroom);
    }
    for (&item, &amount) in &preservation.missing {
        *missing.entry(item).or_default() += amount;
    }
    if let Some(physical) = physical_targets {
        targets = physical;
    }
    Allocation {
        targets,
        missing,
        preservation,
    }
}

/// Net new edible units backed by active bills and uncommitted physical inputs.
pub(super) fn convertible_immediate(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> u64 {
    let mut inputs = free_inputs(world, res);
    let reserved = super::dependencies::cooking_reserve(world, res, policy, facts);
    let targets = priority_targets(world, res, policy, facts, &reserved, false).targets;
    let mut deductions = reserved;
    for ((_, item), count) in &mut inputs {
        let deduction = deductions.entry(*item).or_default();
        let taken = (*count).min(*deduction);
        *count -= taken;
        *deduction -= taken;
    }
    let mut credited = 0_u64;
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        if recipe.food_requirements.duration_days > 0 {
            continue;
        }
        let target = u64::from(targets.get(&kind).copied().unwrap_or(0));
        let made = u64::from(crate::runtime::jobs::bills::bill_count(
            world,
            res,
            recipe.output.0,
        ));
        let desired = target
            .saturating_sub(made)
            .div_ceil(u64::from(recipe.output.1).max(1));
        let mut remaining_desired = desired;
        for (id, shop) in &res.colony.workshops {
            if !recipe.supports_workshop(shop.kind)
                || !has_practitioner(world, res, recipe, |practice| {
                    crate::runtime::jobs::food_recipe_permitted(res, shop, recipe, practice)
                })
                || res
                    .steward
                    .vetoed(&Target::Bill(*id, kind.code()), res.current_tick)
            {
                continue;
            }
            let mut batches = 0_u64;
            for bill in shop
                .bills
                .iter()
                .filter(|bill| bill.recipe == kind && !bill.suspended)
            {
                if !(bill.auto || steward_owns_recipe(res, *id, kind))
                    || super::super::production::player_edited_bill(res, *id, bill.id)
                {
                    continue;
                }
                let remaining = match bill.mode {
                    BillMode::Make(_) => u64::from(bill.remaining().unwrap_or(0))
                        .saturating_sub(assigned_crafts(world, res, kind)),
                    BillMode::Maintain(count) => u64::from(count)
                        .saturating_sub(made)
                        .div_ceil(u64::from(recipe.output.1).max(1)),
                };
                batches = batches.saturating_add(remaining);
            }
            batches = batches.min(remaining_desired);
            for (item, count) in &recipe.inputs {
                batches = batches.min(input_count(&inputs, *id, *item) / u64::from(*count).max(1));
            }
            let consumed: u64 = recipe
                .inputs
                .iter()
                .map(|(item, count)| u64::from(*count) * edible_units(*item, true))
                .sum();
            let net = (u64::from(recipe.output.1) * edible_units(recipe.output.0, false))
                .saturating_sub(consumed);
            for (item, count) in &recipe.inputs {
                consume_inputs(&mut inputs, *id, *item, batches * u64::from(*count));
            }
            credited = credited.saturating_add(batches * net);
            remaining_desired -= batches;
        }
    }
    credited
}

pub(super) fn plan(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> Vec<Command> {
    let seed_reserve = super::dependencies::cooking_reserve(world, res, policy, facts);
    let emergency = facts.usable_units.saturating_mul(1000) < facts.daily_milli;
    let bootstrap = super::dependencies::seed_recipe(policy);
    let mut bootstrap_recipes = policy.vessel_recipes.clone();
    if let Some(kind) = bootstrap {
        if let Some((_, target)) = super::seed_reserve(res, policy, facts) {
            let competing_food_input = kind.recipe().is_some_and(|recipe| {
                recipe
                    .inputs
                    .iter()
                    .any(|(item, _)| item.definition().food.is_some())
            });
            let target = if emergency && competing_food_input {
                0
            } else {
                target
            };
            bootstrap_recipes.push((kind, u16::try_from(target).unwrap_or(u16::MAX)));
        }
    }
    let allocation = priority_targets(world, res, policy, facts, &seed_reserve, false);
    let targets = allocation.targets;
    let preservation = allocation.preservation;
    let mut wanted = bootstrap_recipes.clone();
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        let target = if recipe.food_requirements.duration_days > 0 {
            preservation.targets.get(&kind).copied().unwrap_or(0)
        } else {
            targets.get(&kind).copied().unwrap_or(0)
        };
        wanted.push((kind, target));
    }
    let hearth = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()
        .unwrap_or((0, 0));
    let material_recipes: Vec<_> = bootstrap_recipes
        .iter()
        .copied()
        .filter(|(kind, target)| Some(*kind) != bootstrap || *target > 0)
        .collect();
    let mut commands = super::super::production::inputs_with_discovery(
        world,
        res,
        hearth,
        &material_recipes,
        super::dependencies::discovery_materials(world, policy),
    );
    let mut seed_budget = free_inputs(world, res);
    let mut protected = seed_reserve.clone();
    for ((_, item), count) in &mut seed_budget {
        let reserved = protected.entry(*item).or_default();
        let taken = (*count).min(*reserved);
        *count -= taken;
        *reserved -= taken;
    }
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
        if policy
            .vessel_recipes
            .iter()
            .any(|(vessel, _)| *vessel == kind)
        {
            let physical = crate::runtime::jobs::bills::maintain_count(world, res, recipe.output.0);
            let counted = crate::runtime::jobs::bills::bill_count(world, res, recipe.output.0);
            // Covered-store counting omits outside vessels, not physical production.
            // Keep the original cap even when already-counted stock exceeds it.
            target = target.saturating_sub(
                u16::try_from(physical.saturating_sub(counted)).unwrap_or(u16::MAX),
            );
        }
        if let Some((&id, shop)) = res.colony.workshops.iter().find(|(id, shop)| {
            preservation.workshops.get(&kind).map_or_else(
                || recipe.supports_workshop(shop.kind),
                |selected| *id == selected,
            )
        }) {
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
                let mode = if Some(kind) == bootstrap {
                    bootstrap_mode(world, res, policy, id, kind, target)
                } else {
                    seed_limited_mode(
                        world,
                        res,
                        (id, kind),
                        target,
                        &mut seed_budget,
                        &seed_reserve,
                    )
                };
                if mode == BillMode::Make(0)
                    || target == 0 && !matches!(mode, BillMode::Make(n) if n > 0)
                {
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
                    let unchanged = match mode {
                        BillMode::Make(batches) => bill.remaining() == Some(u32::from(batches)),
                        BillMode::Maintain(_) => bill.mode == mode,
                    };
                    if !unchanged {
                        commands.push(Command::SetBillMode {
                            workshop: id,
                            bill: bill.id,
                            mode,
                        });
                    }
                }
            } else if target > 0 {
                let mode = if Some(kind) == bootstrap {
                    bootstrap_mode(world, res, policy, id, kind, target)
                } else {
                    seed_limited_mode(
                        world,
                        res,
                        (id, kind),
                        target,
                        &mut seed_budget,
                        &seed_reserve,
                    )
                };
                if mode == BillMode::Make(0) {
                    continue;
                }
                commands.push(Command::AddBill {
                    workshop: id,
                    recipe: kind,
                    mode,
                });
            }
        } else if target > 0 || recipe.food_requirements.duration_days > 0 && facts.preparing {
            if let Some(command) = workshop(world, res, kind) {
                commands.push(command);
                if Some(kind) != bootstrap {
                    break;
                }
            }
        }
    }
    if let Some(command) =
        super::super::production::preferred_bill_move(world, res, &policy.recipes)
    {
        commands.push(command);
    }
    commands
}

// Finite crafts cannot reopen when cooked output is eaten before the next plan.
fn seed_limited_mode(
    world: &World,
    res: &SimResources,
    (shop, kind): (u32, RecipeKind),
    target: u16,
    budget: &mut CookingInputs,
    reserved: &std::collections::BTreeMap<sim_core::colony::ItemKind, u64>,
) -> BillMode {
    let Some(recipe) = kind.recipe() else {
        return BillMode::Maintain(target);
    };
    let limited: Vec<_> = recipe
        .inputs
        .iter()
        .filter(|(item, _)| reserved.contains_key(item))
        .collect();
    if limited.is_empty() {
        return BillMode::Maintain(target);
    }
    let counted = crate::runtime::jobs::bills::bill_count(world, res, recipe.output.0);
    let needed = u64::from(u32::from(target).saturating_sub(counted))
        .div_ceil(u64::from(recipe.output.1).max(1));
    let batches = limited.iter().fold(needed, |limit, (item, count)| {
        limit.min(input_count(budget, shop, *item) / u64::from(*count).max(1))
    });
    for (item, count) in limited {
        consume_inputs(
            budget,
            shop,
            *item,
            batches.saturating_mul(u64::from(*count)),
        );
    }
    let assigned = assigned_crafts(world, res, kind);
    BillMode::Make(u16::try_from(batches.saturating_add(assigned)).unwrap_or(u16::MAX))
}

fn bootstrap_mode(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    shop: u32,
    kind: RecipeKind,
    target: u16,
) -> BillMode {
    let Some(recipe) = kind.recipe() else {
        return BillMode::Make(0);
    };
    let counted = super::dependencies::seed_count(world, res, recipe.output.0);
    let needed = u64::from(target)
        .saturating_sub(counted)
        .div_ceil(u64::from(recipe.output.1).max(1));
    let discovery = super::dependencies::discovery_materials(world, policy);
    let inputs = free_inputs(world, res);
    let committed = super::dependencies::unconsumed_craft_inputs(world, res);
    let batches = recipe.inputs.iter().fold(needed, |limit, (item, count)| {
        let physical = if item.definition().food.is_some() {
            input_count(&inputs, shop, *item)
        } else {
            super::dependencies::physical_pieces(world, res, *item)
                .saturating_sub(committed.get(item).copied().unwrap_or(0))
        };
        let free = physical.saturating_sub(u64::from(discovery.get(item).copied().unwrap_or(0)));
        limit.min(free / u64::from(*count).max(1))
    });
    BillMode::Make(
        u16::try_from(batches.saturating_add(assigned_crafts(world, res, kind)))
            .unwrap_or(u16::MAX),
    )
}

fn assigned_crafts(world: &World, res: &SimResources, kind: RecipeKind) -> u64 {
    world
        .query::<&Job>()
        .iter()
        .filter(|(_, job)| {
            let JobKind::Craft { workshop, bill, .. } = job.kind else {
                return false;
            };
            res.colony
                .workshops
                .get(&workshop)
                .and_then(|shop| shop.bill(bill))
                .is_some_and(|bill| bill.recipe == kind)
        })
        .count() as u64
}

fn workshop(world: &World, res: &SimResources, kind: RecipeKind) -> Option<Command> {
    let recipe = kind.recipe()?;
    let size = recipe.workshop.plan(&res.rules.colony?).0;
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
                let work = sim_core::colony::workshop::work_tile_of((x, y), size);
                let occupied = crate::runtime::cultivation::occupies(res, work)
                    || (y..y.saturating_add(size.1)).any(|fy| {
                        (x..x.saturating_add(size.0))
                            .any(|fx| crate::runtime::cultivation::occupies(res, (fx, fy)))
                    });
                if !occupied
                    && crate::commands::camp_reaches(res, (x, y))
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
pub(super) fn steward_owns_recipe(res: &SimResources, workshop: u32, recipe: RecipeKind) -> bool {
    res.steward.owned.iter().any(|owned| {
        matches!(&owned.command,
        Command::AddBill { workshop: owned_workshop, recipe: owned_recipe, .. }
            if *owned_workshop == workshop && *owned_recipe == recipe)
    })
}

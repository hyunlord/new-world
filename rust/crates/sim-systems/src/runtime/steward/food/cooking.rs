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

/// Share the existing daily cooked budget as whole output items per recipe.
pub(super) fn daily_target(
    policy: &FoodPlan,
    facts: &FoodSummary,
    recipe_count: usize,
    kind: RecipeKind,
) -> u16 {
    let total = facts
        .daily_milli
        .saturating_mul(u64::from(policy.cooked_days_milli))
        .div_ceil(1_000_000);
    let (immediate, durable) = super::harvest::targets(policy, facts);
    let reserve = immediate.max(durable);
    // Maintain targets already include finished and assigned cooked output.
    // Credit only raw edible stock here, avoiding a second subtraction of meals.
    let raw = facts.usable_units.saturating_sub(facts.cooked_units);
    let Some(recipe) = kind.recipe() else {
        return 0;
    };
    let output = recipe.output.0;
    let output_units = u64::from(recipe.output.1) * edible_units(output, false);
    let consumed_units: u64 = recipe
        .inputs
        .iter()
        .map(|(item, count)| u64::from(*count) * edible_units(*item, true))
        .sum();
    let net = output_units.saturating_sub(consumed_units);
    let headroom = reserve.saturating_sub(raw);
    let capacity = if net == 0 {
        total
    } else {
        headroom.saturating_mul(output_units) / net
    };
    let total = total.min(capacity);
    let share = total.div_ceil(u64::try_from(recipe_count).unwrap_or(u64::MAX).max(1));
    let units = output
        .definition()
        .food
        .as_ref()
        .map_or(1, |food| food.units.max(1));
    u16::try_from(share.div_ceil(u64::from(units))).unwrap_or(u16::MAX)
}

fn edible_units(item: sim_core::colony::ItemKind, raw_only: bool) -> u64 {
    item.definition()
        .food
        .as_ref()
        .filter(|food| !raw_only || food.raw_edible)
        .map_or(0, |food| u64::from(food.units))
}

// None is portable public stock; Some(shop) is fresh input already on that shelf.
type CookingInputs = std::collections::BTreeMap<(Option<u32>, sim_core::colony::ItemKind), u64>;

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

fn input_count(inputs: &CookingInputs, shop: u32, item: sim_core::colony::ItemKind) -> u64 {
    inputs.get(&(None, item)).copied().unwrap_or(0)
        + inputs.get(&(Some(shop), item)).copied().unwrap_or(0)
}

fn consume_inputs(
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

/// Net new edible units backed by active bills and uncommitted physical inputs.
pub(super) fn convertible_immediate(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> u64 {
    let mut inputs = cooking_inputs(world, res);
    let reserved = super::dependencies::seed_inputs(world, res, policy, facts);
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
    let emergency = facts.usable_units.saturating_mul(1000) < facts.daily_milli;
    let mut deductions = if emergency {
        std::collections::BTreeMap::new()
    } else {
        reserved
    };
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
        let target = u64::from(daily_target(policy, facts, policy.recipes.len(), kind));
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
    let preservation = super::preservation::budget(world, res, policy, facts);
    let mut seed_reserve = super::dependencies::seed_inputs(world, res, policy, facts);
    let emergency = facts.usable_units.saturating_mul(1000) < facts.daily_milli;
    if emergency {
        // Grain is not raw-edible: protecting future seed must not forbid today's meal.
        for (item, reserved) in &mut seed_reserve {
            if item.definition().food.is_some() {
                *reserved = 0;
            }
        }
    }
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
    let mut wanted = bootstrap_recipes.clone();
    for &kind in &policy.recipes {
        let Some(recipe) = kind.recipe() else {
            continue;
        };
        let target = if recipe.food_requirements.duration_days > 0 {
            preservation.targets.get(&kind).copied().unwrap_or(0)
        } else if available.contains(&kind) {
            daily_target(policy, facts, available.len(), kind)
        } else {
            0
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
    let committed = super::dependencies::unconsumed_craft_inputs(world, res);
    let mut seed_budget: std::collections::BTreeMap<_, _> = seed_reserve
        .iter()
        .map(|(&item, &reserved)| {
            (
                item,
                facts
                    .pieces
                    .get(&item)
                    .copied()
                    .unwrap_or(0)
                    .saturating_sub(reserved)
                    .saturating_sub(committed.get(&item).copied().unwrap_or(0)),
            )
        })
        .collect();
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
                    bootstrap_mode(world, res, policy, facts, kind, target)
                } else {
                    seed_limited_mode(world, res, kind, target, &mut seed_budget)
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
                    bootstrap_mode(world, res, policy, facts, kind, target)
                } else {
                    seed_limited_mode(world, res, kind, target, &mut seed_budget)
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
    kind: RecipeKind,
    target: u16,
    budget: &mut std::collections::BTreeMap<sim_core::colony::ItemKind, u64>,
) -> BillMode {
    let Some(recipe) = kind.recipe() else {
        return BillMode::Maintain(target);
    };
    let limited: Vec<_> = recipe
        .inputs
        .iter()
        .filter(|(item, _)| budget.contains_key(item))
        .collect();
    if limited.is_empty() {
        return BillMode::Maintain(target);
    }
    let counted = crate::runtime::jobs::bills::bill_count(world, res, recipe.output.0);
    let needed = u64::from(u32::from(target).saturating_sub(counted))
        .div_ceil(u64::from(recipe.output.1).max(1));
    let batches = limited.iter().fold(needed, |limit, (item, count)| {
        limit.min(budget.get(item).copied().unwrap_or(0) / u64::from(*count).max(1))
    });
    for (item, count) in limited {
        if let Some(available) = budget.get_mut(item) {
            *available = available.saturating_sub(batches.saturating_mul(u64::from(*count)));
        }
    }
    let assigned = assigned_crafts(world, res, kind);
    BillMode::Make(u16::try_from(batches.saturating_add(assigned)).unwrap_or(u16::MAX))
}

fn bootstrap_mode(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
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
    let committed = super::dependencies::unconsumed_craft_inputs(world, res);
    let batches = recipe.inputs.iter().fold(needed, |limit, (item, count)| {
        let physical = if item.definition().food.is_some() {
            facts.pieces.get(item).copied().unwrap_or(0)
        } else {
            super::dependencies::physical_pieces(world, res, *item)
        };
        let free = physical
            .saturating_sub(u64::from(discovery.get(item).copied().unwrap_or(0)))
            .saturating_sub(committed.get(item).copied().unwrap_or(0));
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

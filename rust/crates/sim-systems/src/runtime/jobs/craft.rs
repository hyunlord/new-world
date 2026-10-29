//! The craft job's two ends (M2 goal 3.4–3.7, §6): start (inputs consumed,
//! work ticks from the skill) and finish (quality drawn, output created at
//! the work tile, the bill counted, the skill trained).
//!
//! Inputs are consumed when the craft **starts** (ledger `craft_start`), and
//! the output is created when it **ends** (`craft_end`); a craft cancelled in
//! between loses its inputs (counted in `ColonyTallies::crafts_lost`; ADR 0019).
//! The output's material is the first input's (a blade's flint; an axe's
//! blade). Called only from `JobExecuteSystem`, in `AgentId` order.
use hecs::{Entity, World};
use sim_core::colony::skills::{
    craft_quality, material_term, quality_bonus, scaled_ticks, temperament_quality,
    temperament_ticks,
};
use sim_core::colony::{BillId, ItemKind, ItemSpec, Job, JobKind, SkillKind, Skills, WorkshopId};
use sim_core::components::{AgentId, Temperament};
use sim_engine::SimResources;

use super::items::put_items;

/// The level of `skill` of `entity` (0 without `Skills`).
pub fn level_of(world: &World, entity: Entity, skill: SkillKind) -> u8 {
    world
        .get::<&Skills>(entity)
        .map_or(0, |skills| skills.level(skill))
}

/// Add `ticks` of completed work to `entity`'s `skill`, adding `Skills` at
/// the first training (a structural insert, made by the caller's sorted loop).
pub fn train(world: &mut World, entity: Entity, skill: SkillKind, ticks: u32) {
    if let Ok(mut skills) = world.get::<&mut Skills>(entity) {
        skills.train(skill, ticks);
        return;
    }
    let mut skills = Skills::default();
    skills.train(skill, ticks);
    let _ = world.insert_one(entity, skills);
}

/// Craft ticks in a roofed workshop room, permille (M3 goal 3.3).
pub const WORKSHOP_ROOM_PERMILLE: u32 = 900;

/// Food preparation cannot progress after its actual heat or vessel disappears.
pub(super) fn can_continue(world: &World, res: &SimResources, entity: Entity, job: &Job) -> bool {
    let JobKind::Craft { workshop, bill, .. } = job.kind else {
        return true;
    };
    let Some(shop) = res.colony.workshops.get(&workshop) else {
        return false;
    };
    let Some(recipe) = shop.bill(bill).and_then(|bill| bill.recipe.recipe()) else {
        return false;
    };
    let practice = if res
        .rules
        .colony
        .is_some_and(|rule| rule.knowledge.is_some())
    {
        world
            .get::<&sim_core::colony::knowledge::Knowledge>(entity)
            .map_or(0, |knowledge| knowledge.practice())
    } else {
        u32::MAX
    };
    super::craft_food::permitted(res, shop, recipe, practice)
}

/// Start a craft at the work tile: take one craft's inputs off the shelf,
/// record them consumed and return the work ticks; `None` when the bill is
/// gone or done or the shelf lacks an input (nothing is taken then).
pub fn start(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    job: &mut Job,
    workshop: WorkshopId,
    bill: BillId,
) -> Option<u32> {
    // M3 (goal 3.3, 8.5): a workshop in a roofed workshop room works at 900 ‰ ticks.
    let in_room = res.rules.colony.is_some_and(|r| r.space.is_some())
        && res.colony.workshops.get(&workshop).is_some_and(|shop| {
            crate::runtime::rooms::roofed_as(
                res,
                shop.top_left,
                sim_core::rooms::RoomRole::Workshop,
            )
        });
    let shop = res.colony.workshops.get(&workshop)?;
    let found = *shop.bill(bill)?;
    let recipe = found.recipe.recipe()?;
    if found.remaining() == Some(0) || !recipe.supports_workshop(shop.kind) {
        return None;
    }
    let practice = if res
        .rules
        .colony
        .is_some_and(|rule| rule.knowledge.is_some())
    {
        world
            .get::<&sim_core::colony::knowledge::Knowledge>(entity)
            .map_or(0, |k| k.practice())
    } else {
        u32::MAX
    };
    if !super::craft_food::permitted(res, shop, recipe, practice) {
        return None;
    }
    let cutter = recipe.shelf_tool(shop);
    if recipe.tool.is_some() && cutter.is_none() {
        return None;
    }
    if !recipe
        .needs_with_tool(cutter)
        .iter()
        .all(|&(kind, n)| shop.shelved(kind) >= n)
    {
        return None;
    }
    let material = super::craft_food::consume(world, res, workshop, recipe)?;
    res.colony.item_ledger.craft_start(&recipe.inputs);
    res.colony.touch(false);
    if let Some(cutter) = cutter {
        wear_knife(world, res, entity, (workshop, cutter));
    }
    job.material = material;
    let level = level_of(world, entity, recipe.skill);
    let temperament = world.get::<&Temperament>(entity).ok().map(|t| *t);
    let mut ticks = scaled_ticks(recipe.work_ticks, level);
    if in_room {
        ticks = (ticks * WORKSHOP_ROOM_PERMILLE + 500) / 1000;
    }
    let ticks = ticks
        .saturating_sub(temperament_ticks(temperament.as_ref()))
        .max(1);
    // M5 (ADR 0027, 0030): a cold snap outdoors, illness and mood.
    let at = res
        .colony
        .workshops
        .get(&workshop)
        .map_or((0, 0), |s| s.top_left);
    Some(crate::runtime::story::scale_work(
        world, res, entity, at, ticks,
    ))
}

/// Whether `bill` at `workshop` wants another craft now (M3, DECISIONS D13): it
/// is the workshop's active bill, not done, not suspended, its `Maintain`
/// count below target, and every input of one craft is on the shelf.
pub fn next_ready(world: &World, res: &SimResources, workshop: WorkshopId, bill: BillId) -> bool {
    let Some(shop) = res.colony.workshops.get(&workshop) else {
        return false;
    };
    let Some(found) = shop
        .bill(bill)
        .filter(|b| shop.active == Some(b.id) && !b.suspended)
    else {
        return false;
    };
    let Some(recipe) = found.recipe.recipe() else {
        return false;
    };
    if found.remaining() == Some(0) || !recipe.supports_workshop(shop.kind) {
        return false;
    }
    if let sim_core::colony::BillMode::Maintain(n) = found.mode {
        if super::bills::bill_count(world, res, recipe.output.0) >= u32::from(n) {
            return false;
        }
    }
    let cutter = recipe.shelf_tool(shop);
    if recipe.tool.is_some() && cutter.is_none() {
        return false;
    }
    recipe
        .needs_with_tool(cutter)
        .iter()
        .all(|&(kind, n)| shop.shelved(kind) >= n)
}

/// Wear the selected reusable cutter, consuming and reporting its actual identity.
fn wear_knife(
    world: &World,
    res: &mut SimResources,
    entity: Entity,
    selected: (WorkshopId, ItemKind),
) {
    let (workshop, cutter) = selected;
    let catalog = sim_core::colony::content::installed_or_bundled();
    let Some(definition) = catalog.item(cutter).and_then(|item| item.cutter.as_ref()) else {
        return;
    };
    let default_cutter = catalog.item_role("preferred_cutter");
    let previous = res
        .content_state
        .workshop_cutters
        .get(&workshop)
        .copied()
        .or(default_cutter);
    if previous != Some(cutter) {
        res.colony.knife_wear.remove(&workshop);
    }
    if Some(cutter) == default_cutter {
        res.content_state.workshop_cutters.remove(&workshop);
    } else {
        res.content_state.workshop_cutters.insert(workshop, cutter);
    }
    let limit = match definition.wear_rule.as_deref() {
        Some("knife_uses") => res
            .rules
            .colony
            .and_then(|c| c.wildlife)
            .map_or(definition.wear_default, |wildlife| wildlife.knife_uses),
        _ => definition.wear_default,
    }
    .max(1);
    let uses = {
        let wear = res.colony.knife_wear.entry(workshop).or_insert(0);
        *wear += 1;
        *wear
    };
    res.colony.wildlife.tallies.knife_uses += 1;
    if uses < limit {
        return;
    }
    res.colony.knife_wear.remove(&workshop);
    let Some(shop) = res.colony.workshops.get_mut(&workshop) else {
        return;
    };
    let Some(stack) = shop
        .shelf
        .iter_mut()
        .find(|s| s.kind == cutter && s.count > 0)
    else {
        return;
    };
    stack.count -= 1;
    shop.shelf.retain(|s| s.count > 0);
    res.colony.item_ledger.consume(cutter, 1);
    res.colony.tallies.tools_broken += 1;
    if definition.break_tally.as_deref() == Some("knives_broken") {
        res.colony.wildlife.tallies.knives_broken += 1;
    }
    let mut entry = sim_core::chronicle::ChronicleEntry::new(
        res.current_tick,
        sim_core::chronicle::ChronicleKind::ToolBroke,
    )
    .cause(cutter.code())
    .value(i64::from(uses));
    if let Ok(agent) = world.get::<&sim_core::components::Agent>(entity) {
        entry = entry.subject(agent.id);
    }
    res.chronicle.push(entry);
}

/// Finish the craft `job` (a `JobKind::Craft`): draw its quality, create the
/// output at the work tile, count the bill and train the crafter's skill by
/// the ticks worked.
pub fn finish_craft(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    job: &Job,
) {
    let JobKind::Craft { workshop, bill, at } = job.kind else {
        return;
    };
    let Some(recipe) = res
        .colony
        .workshops
        .get(&workshop)
        .and_then(|shop| shop.bill(bill))
        .and_then(|b| b.recipe.recipe())
    else {
        return;
    };
    let level = level_of(world, entity, recipe.skill);
    let temperament = world.get::<&Temperament>(entity).ok().map(|t| *t);
    let bonus = quality_bonus(res.world_seed, id, bill, res.current_tick);
    let hardness = job
        .material
        .and_then(|m| res.colony.materials.stats(m))
        .map(|stats| stats.hardness_percent());
    let quality = craft_quality(
        level,
        bonus + recipe.quality_add,
        material_term(recipe.id, hardness),
        temperament_quality(temperament.as_ref()),
    );
    // M4 (goal 3.2, ADR 0024): what the crafter knows caps the quality
    // (`QUAL_CAP` of the lowest proficiency among the recipe's gating nodes).
    let gate = sim_core::colony::knowledge::gates().recipe(recipe.id);
    let gated = crate::runtime::knowledge::rule(res).is_some() && gate != 0;
    let quality = if gated {
        let p = crate::runtime::knowledge::proficiency_of(world, entity, gate);
        quality.min(sim_core::colony::knowledge::quality_cap(p))
    } else {
        quality
    };
    let (kind, n) = recipe.output;
    let spec = ItemSpec {
        kind,
        material: job.material,
        quality,
    };
    if recipe.food_requirements.duration_days > 0 {
        res.food
            .pending_drying
            .push(sim_core::colony::food::DryingBatch {
                workshop,
                bill,
                tile: at,
                output: spec,
                count: n,
                due_tick: res.current_tick.saturating_add(
                    res.rules
                        .calendar
                        .days_to_ticks(u64::from(recipe.food_requirements.duration_days)),
                ),
            });
        res.food.touch();
    } else if kind.definition().food.is_some() {
        let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
        if crate::runtime::food::items::create_ground(
            res,
            at,
            spec,
            n,
            day,
            crate::runtime::food::items::FoodSource::Craft,
        )
        .is_err()
        {
            res.food
                .pending_drying
                .push(sim_core::colony::food::DryingBatch {
                    workshop,
                    bill,
                    tile: at,
                    output: spec,
                    count: n,
                    due_tick: res.current_tick,
                });
            res.food.touch();
        } else {
            res.colony.item_ledger.craft_end(&[(kind, n)]);
        }
    } else {
        res.colony.item_ledger.craft_end(&[(kind, n)]);
        put_items(res, at, spec, n);
    }
    if let Some(found) = res
        .colony
        .workshops
        .get_mut(&workshop)
        .and_then(|shop| shop.bills.iter_mut().find(|b| b.id == bill))
    {
        found.done += 1;
    }
    res.colony.tallies.crafts_done += 1;
    let entry = res
        .colony
        .tallies
        .craft_quality
        .entry((id, recipe.id.code()))
        .or_insert((0, 0));
    entry.0 += 1;
    entry.1 += u64::from(quality);
    let ticks = scaled_ticks(recipe.work_ticks, level);
    train(world, entity, recipe.skill, ticks);
    if gated {
        crate::runtime::knowledge::used(world, res, entity, gate);
    }
}

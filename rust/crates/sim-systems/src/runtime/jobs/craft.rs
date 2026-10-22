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
use sim_core::colony::{BillId, ItemSpec, Job, JobKind, SkillKind, Skills, WorkshopId};
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

/// Start a craft at the work tile: take one craft's inputs off the shelf,
/// record them consumed and return the work ticks; `None` when the bill is
/// gone or done or the shelf lacks an input (nothing is taken then).
pub fn start(
    world: &World,
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
    let shop = res.colony.workshops.get_mut(&workshop)?;
    let found = *shop.bill(bill)?;
    let recipe = found.recipe.recipe()?;
    if found.remaining() == Some(0) {
        return None;
    }
    if !recipe
        .inputs
        .iter()
        .all(|&(kind, n)| shop.shelved(kind) >= n)
    {
        return None;
    }
    let mut material = None;
    for (i, &(kind, n)) in recipe.inputs.iter().enumerate() {
        let mut left = n;
        for stack in shop.shelf.iter_mut().filter(|s| s.kind == kind) {
            if left == 0 {
                break;
            }
            if i == 0 && material.is_none() {
                material = stack.material;
            }
            let take = stack.count.min(left);
            stack.count -= take;
            left -= take;
        }
        shop.shelf.retain(|s| s.count > 0);
    }
    res.colony.item_ledger.craft_start(&recipe.inputs);
    res.colony.touch(false);
    job.material = material;
    let level = level_of(world, entity, recipe.skill);
    let temperament = world.get::<&Temperament>(entity).ok().map(|t| *t);
    let mut ticks = scaled_ticks(recipe.work_ticks, level);
    if in_room {
        ticks = (ticks * WORKSHOP_ROOM_PERMILLE + 500) / 1000;
    }
    Some(
        ticks
            .saturating_sub(temperament_ticks(temperament.as_ref()))
            .max(1),
    )
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
    if found.remaining() == Some(0) {
        return false;
    }
    if let sim_core::colony::BillMode::Maintain(n) = found.mode {
        if super::bills::bill_count(world, res, recipe.output.0) >= u32::from(n) {
            return false;
        }
    }
    recipe
        .inputs
        .iter()
        .all(|&(kind, n)| shop.shelved(kind) >= n)
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
        bonus,
        material_term(recipe.id, hardness),
        temperament_quality(temperament.as_ref()),
    );
    let (kind, n) = recipe.output;
    let spec = ItemSpec {
        kind,
        material: job.material,
        quality,
    };
    res.colony.item_ledger.craft_end(&[(kind, n)]);
    put_items(res, at, spec, n);
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
}

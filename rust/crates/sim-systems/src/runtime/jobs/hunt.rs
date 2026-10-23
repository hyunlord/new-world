//! M5 hunting and dressing jobs (ADR 0028, 0029): the `Hunt` job's choice and steps, weapons, the kill's meat,
//! hide and knife use, and the `Equip` job's putting on a top.
//!
//! A hunt is offered to a practitioner of `tracking` (the `Designation::Hunt`
//! gate) for the nearest huntable deer (marked, or inside a hunt zone) that is
//! reachable and not reserved (`ReserveKey::Animal`). A hunter without a tool
//! first fetches the nearest spear (else a flint blade, a knife) from a
//! stockpile within `axe_reach`. The hunter walks to the deer's current tile;
//! within `hunt_reach` it attacks: `hunt_attack_ticks` scaled by its hunting
//! level (and the work-speed factors), then one keyed hit roll
//! (`STREAM_WILDLIFE`) at 400–800 ‰ by level (a spear doubles it, capped);
//! a hit does the weapon's damage (a spear wears one use). A deer at 0 hp
//! drops meat (3 with a knife held — one knife use — else 2) and a hide on
//! its tile; the meat joins the spoilage lots; chronicle `Hunted`.
use hecs::{Entity, World};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::knowledge::gates;
use sim_core::colony::people::{Apparel, Top};
use sim_core::colony::wildlife::AnimalKind;
use sim_core::colony::{
    CancelReason, Designation, ItemKind, ItemSpec, Job, JobKind, JobStage, ReserveKey, SkillKind,
    Tool,
};
use sim_core::components::{AgentId, AgentState, Lineage, SeekTarget, TargetKind};
use sim_core::rng::{keyed_below, STREAM_WILDLIFE};
use sim_core::rules::WildlifeRule;
use sim_engine::SimResources;

use super::cancel::{cancel, finish};
use super::craft;
use super::items::{chebyshev, put_items, take_items};

/// The wildlife rule in force (`None` outside the M5 colony).
pub fn rule(res: &SimResources) -> Option<WildlifeRule> {
    res.rules.colony.and_then(|c| c.wildlife)
}

const SEEK_JOB: AgentState = AgentState::Seeking {
    target: TargetKind::Job,
};
const WORK_JOB: AgentState = AgentState::Consuming {
    target: TargetKind::Job,
};

fn set_state(world: &mut World, entity: Entity, next: AgentState) {
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = next;
    }
}

fn put_job(world: &mut World, entity: Entity, job: Job) {
    if let Ok(mut held) = world.get::<&mut Job>(entity) {
        *held = job;
    }
}

/// The deer `practice` may hunt from `pos` in nav region `region` within
/// `radius`, not reserved — the most wounded first, then the nearest:
/// `(animal, its tile)`.
pub fn pick(
    res: &SimResources,
    practice: u32,
    region: u32,
    pos: (u32, u32),
    radius: u32,
) -> Option<(u32, (u32, u32))> {
    rule(res)?;
    if gates().designation(Designation::Hunt) & !practice != 0 {
        return None;
    }
    let w = &res.colony.wildlife;
    w.deer()
        .filter(|a| w.huntable(a))
        .filter(|a| {
            !res.colony
                .reservations
                .contains_key(&ReserveKey::Animal(a.id))
        })
        .filter(|a| chebyshev(pos, a.pos) <= radius && res.nav.reaches(region, a.pos))
        // A wounded deer first (finish what was started; DECISIONS D8), then the nearest.
        .min_by_key(|a| (a.hp, chebyshev(pos, a.pos), a.pos.1, a.pos.0, a.id))
        .map(|a| (a.id, a.pos))
}

/// The nearest stockpile stack of `kinds` (in preference order: a kind earlier
/// in the list wins over a nearer later one) within `reach` of `pos`, free and
/// reachable from it.
pub fn nearest_stack(
    res: &SimResources,
    pos: (u32, u32),
    reach: u32,
    kinds: &[ItemKind],
    zoned: bool,
) -> Option<(u32, u32)> {
    let region = res.nav.region_of(pos);
    let colony = &res.colony;
    kinds.iter().find_map(|kind| {
        colony
            .ground_items
            .iter()
            .filter(|(tile, stack)| {
                stack.kind == *kind
                    && stack.count > 0
                    && chebyshev(pos, **tile) <= reach
                    && (!zoned || colony.zone_of_tile.contains_key(*tile))
                    && !colony.reservations.contains_key(&ReserveKey::Stack(**tile))
                    && res.nav.reaches(region, **tile)
            })
            .min_by_key(|(tile, _)| (chebyshev(pos, **tile), tile.1, tile.0))
            .map(|(tile, _)| *tile)
    })
}

/// The tool a hunter fetches (`held` = the kind it holds): a stocked spear unless it
/// holds one (swapping any other tool); with no tool and no spear, a knife.
pub fn hunting_fetch(
    res: &SimResources,
    pos: (u32, u32),
    reach: u32,
    held: Option<ItemKind>,
) -> Option<(u32, u32)> {
    match held {
        Some(ItemKind::Spear) => None,
        Some(_) => nearest_stack(res, pos, reach, &[ItemKind::Spear], true),
        None => nearest_stack(
            res,
            pos,
            reach,
            &[ItemKind::Spear, ItemKind::FlintBlade],
            true,
        ),
    }
}

/// Take one item of the stack at `tile` as `entity`'s held tool; a tool already
/// held is left on the worker's tile (a hunter swapping an axe for a spear).
pub fn pick_up_tool(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    tile: (u32, u32),
    pos: (u32, u32),
) {
    let kind = res.colony.ground_items.get(&tile).map(|s| s.kind);
    res.colony.reservations.remove(&ReserveKey::Stack(tile));
    let Some(kind) = kind else {
        return;
    };
    let (taken, spec) = take_items(res, tile, kind, 1);
    if taken == 1 {
        if let Ok(old) = world.get::<&Tool>(entity).map(|t| t.item) {
            put_items(res, pos, old, 1);
        }
        let _ = world.insert_one(
            entity,
            Tool {
                item: spec,
                uses: 0,
            },
        );
    }
}

/// The damage of `entity`'s held weapon (goal 3.2: hands 4, stone axe 8, ground axe 10, spear 12).
pub fn weapon_damage(world: &World, rule: &WildlifeRule, entity: Entity) -> u16 {
    match world.get::<&Tool>(entity).map(|t| t.item.kind) {
        Ok(ItemKind::Spear) => rule.spear_damage,
        Ok(ItemKind::GroundAxe) => rule.ground_axe_damage,
        Ok(ItemKind::StoneAxe) => rule.axe_damage,
        _ => rule.hand_damage,
    }
}

/// One use of `entity`'s held tool of `kind` (a spear's attack, a knife's cut):
/// at `limit` uses it breaks — used up (ledger consumed), chronicle `ToolBroke`.
/// Returns whether a tool of that kind was used.
pub fn wear_tool(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    kind: ItemKind,
    limit: u32,
) -> bool {
    let Ok(mut tool) = world.get::<&Tool>(entity).map(|t| *t) else {
        return false;
    };
    if tool.item.kind != kind {
        return false;
    }
    tool.uses += 1;
    if limit > 0 && tool.uses >= limit {
        let _ = world.remove_one::<Tool>(entity);
        res.colony.item_ledger.consume(kind, 1);
        res.colony.tallies.tools_broken += 1;
        match kind {
            ItemKind::Spear => res.colony.wildlife.tallies.spears_broken += 1,
            ItemKind::FlintBlade => res.colony.wildlife.tallies.knives_broken += 1,
            _ => {}
        }
        res.chronicle.push(
            ChronicleEntry::new(res.current_tick, ChronicleKind::ToolBroke)
                .subject(id)
                .cause(kind.code())
                .value(i64::from(tool.uses)),
        );
    } else if let Ok(mut held) = world.get::<&mut Tool>(entity) {
        *held = tool;
    }
    true
}

/// A hunter's hit chance, ‰: 400 at level 0 to 800 at 20 (goal 3.2); a spear doubles it, capped.
pub fn hit_permille(rule: &WildlifeRule, level: u8, spear: bool) -> u32 {
    let (lo, hi) = rule.hunt_hit_permille;
    let base = lo + (hi.saturating_sub(lo)) * u32::from(level.min(20)) / 20;
    if spear {
        (base * 2).min(rule.spear_hit_cap_permille)
    } else {
        base
    }
}

/// One tick of a `Hunt` job (called by `JobExecuteSystem` in `AgentId` order).
pub fn step(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    mut job: Job,
    pos: (u32, u32),
) {
    let Some(rule) = rule(res) else {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    };
    let JobKind::Hunt { animal } = job.kind else {
        return;
    };
    let Some(deer) = res.colony.wildlife.animals.get(&animal).map(|a| a.pos) else {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    };
    // The tool stop.
    if let (Some(at), JobStage::ToSource) = (job.fetch, job.stage) {
        if !res.nav.arrived(pos, at) {
            super::execute::walk_job(world, res, id, entity, &mut job, pos, at);
            return;
        }
        pick_up_tool(world, res, entity, at, pos);
        job.fetch = None;
        job.best_distance = chebyshev(pos, deer);
        job.progress_tick = res.current_tick;
    }
    let near = chebyshev(pos, deer) <= rule.hunt_reach;
    match job.stage {
        JobStage::ToSource | JobStage::ToDest => {
            if !near {
                let _ = world.insert_one(entity, SeekTarget::new(deer));
                super::execute::walk_job(world, res, id, entity, &mut job, pos, deer);
                return;
            }
            let level = craft::level_of(world, entity, SkillKind::Hunting);
            let ticks = sim_core::colony::skills::scaled_ticks(rule.hunt_attack_ticks, level);
            let ticks = crate::runtime::story::scale_work(world, res, entity, pos, ticks);
            job.amount = ticks.min(u32::from(u16::MAX)) as u16;
            job.stage = JobStage::Working { ticks_left: ticks };
            let _ = world.remove_one::<SeekTarget>(entity);
            set_state(world, entity, WORK_JOB);
            put_job(world, entity, job);
        }
        JobStage::Working { ticks_left } => {
            if !near {
                job.stage = JobStage::ToSource;
                job.best_distance = chebyshev(pos, deer);
                job.progress_tick = res.current_tick;
                set_state(world, entity, SEEK_JOB);
                let _ = world.insert_one(entity, SeekTarget::new(deer));
                put_job(world, entity, job);
                return;
            }
            if ticks_left > 1 {
                job.stage = JobStage::Working {
                    ticks_left: ticks_left - 1,
                };
                put_job(world, entity, job);
                return;
            }
            craft::train(world, entity, SkillKind::Hunting, u32::from(job.amount));
            if attack(world, res, &rule, id, entity, animal) {
                kill(world, res, &rule, id, entity, animal);
                finish(world, res, entity);
                return;
            }
            job.stage = JobStage::ToSource;
            put_job(world, entity, job);
        }
    }
}

/// One hunting attack on deer `animal`; `true` when it fell.
fn attack(
    world: &mut World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    id: AgentId,
    entity: Entity,
    animal: u32,
) -> bool {
    let level = craft::level_of(world, entity, SkillKind::Hunting);
    let spear = world
        .get::<&Tool>(entity)
        .is_ok_and(|t| t.item.kind == ItemKind::Spear);
    let chance = hit_permille(rule, level, spear);
    let key = world.get::<&Lineage>(entity).map_or(id, |l| l.key);
    res.colony.wildlife.tallies.hunt_attacks += 1;
    // Every attack, hit or miss, is tracking used (it fades after 6 unused days).
    crate::runtime::knowledge::used(world, res, entity, gates().designation(Designation::Hunt));
    let roll = keyed_below(
        res.world_seed,
        STREAM_WILDLIFE,
        key,
        res.current_tick,
        animal,
        0,
        1000,
    );
    if roll >= u64::from(chance) {
        return false;
    }
    res.colony.wildlife.tallies.hunt_hits += 1;
    let damage = weapon_damage(world, rule, entity);
    if spear {
        wear_tool(world, res, id, entity, ItemKind::Spear, rule.spear_uses);
    }
    let Some(deer) = res.colony.wildlife.animals.get_mut(&animal) else {
        return false;
    };
    deer.hp = deer.hp.saturating_sub(damage);
    res.colony.wildlife.version += 1;
    deer.hp == 0
}

/// The deer fell: meat (a knife held: one use, more meat) and a hide on its tile.
fn kill(
    world: &mut World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    id: AgentId,
    entity: Entity,
    animal: u32,
) {
    let Some(deer) = res.colony.wildlife.animals.remove(&animal) else {
        return;
    };
    res.colony.wildlife.marks.remove(&animal);
    let knife = wear_tool(
        world,
        res,
        id,
        entity,
        ItemKind::FlintBlade,
        rule.knife_uses,
    );
    if knife {
        res.colony.wildlife.tallies.knife_uses += 1;
    }
    let meat = if knife {
        rule.meat_with_knife
    } else {
        rule.meat_without_knife
    };
    drop_carcass(res, deer.pos, meat, ItemKind::Hide);
    let w = &mut res.colony.wildlife;
    w.tallies.deer_killed += 1;
    w.version += 1;
    res.chronicle.push(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Hunted)
            .subject(id)
            .cause(AnimalKind::Deer.code())
            .value(i64::from(meat)),
    );
    crate::runtime::knowledge::used(world, res, entity, gates().designation(Designation::Hunt));
}

/// Put a carcass's meat and hide on `tile` (ledger created; the meat joins today's lot).
pub fn drop_carcass(res: &mut SimResources, tile: (u32, u32), meat: u16, hide: ItemKind) {
    if meat > 0 {
        res.colony
            .item_ledger
            .create(ItemKind::Meat, u64::from(meat));
        put_items(res, tile, ItemSpec::raw(ItemKind::Meat, None), meat);
        let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
        let lots = &mut res.colony.wildlife.meat_lots;
        match lots.last_mut() {
            Some((d, n)) if *d == day => *n += u32::from(meat),
            _ => lots.push((day, u32::from(meat))),
        }
        res.colony.wildlife.tallies.meat_dropped += u64::from(meat);
    }
    res.colony.item_ledger.create(hide, 1);
    put_items(res, tile, ItemSpec::raw(hide, None), 1);
    res.colony.wildlife.tallies.hides_dropped += 1;
}

/// One tick of an `Equip` job: walk to the stack, take one top and wear it (the
/// old top, if any, is left on the ground).
pub fn equip_step(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    mut job: Job,
    pos: (u32, u32),
) {
    let JobKind::Equip { from } = job.kind else {
        return;
    };
    if !res.nav.arrived(pos, from) {
        super::execute::walk_job(world, res, id, entity, &mut job, pos, from);
        return;
    }
    let kind = res.colony.ground_items.get(&from).map(|s| s.kind);
    res.colony.reservations.remove(&ReserveKey::Stack(from));
    let Some(top) = kind.and_then(Top::of_item) else {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    };
    let (taken, _) = take_items(res, from, top.item(), 1);
    if taken == 1 {
        let old = world.get::<&Apparel>(entity).ok().map(|a| a.top);
        if let Some(old) = old {
            put_items(res, pos, ItemSpec::raw(old.item(), None), 1);
        }
        let day = crate::runtime::resource_regen::day_index(res).min(u64::from(u16::MAX)) as u16;
        let _ = world.insert_one(
            entity,
            Apparel {
                top,
                since_day: day,
            },
        );
        res.colony.wildlife.version += 1;
    }
    finish(world, res, entity);
}

/// How warm a top is (0 = the loincloth).
const fn warmth(top: Option<Top>) -> u8 {
    match top {
        None => 0,
        Some(Top::HideWrap) => 1,
        Some(Top::FurCloak) => 2,
    }
}

/// The dressing pass (ADR 0029; `JobAssignSystem`, every 60 ticks while the
/// standing order is on): each free top on a stockpile cell, warmest first,
/// goes to the person who most needs one — highest `Hardiness` (the lowest
/// cold tolerance), then `AgentId` — among the adults free this pass who wear
/// something colder and can reach it (a busy person is served at a later pass;
/// DECISIONS D10). One `Equip` job a stack a pass (the stack is reserved).
pub fn dress(
    world: &World,
    res: &mut SimResources,
    available: &[(AgentId, Entity, (u32, u32))],
    assigned: &mut std::collections::BTreeSet<AgentId>,
    taken: &mut Vec<(Entity, Job, (u32, u32))>,
) {
    use sim_core::components::{Agent, Child, Hardiness};
    let tick = res.current_tick;
    let mut tops: Vec<(std::cmp::Reverse<u8>, u32, u32, Top)> = res
        .colony
        .ground_items
        .iter()
        .filter(|(t, s)| {
            s.count > 0
                && res.colony.zone_of_tile.contains_key(*t)
                && !res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Stack(**t))
        })
        .filter_map(|(t, s)| {
            Top::of_item(s.kind).map(|top| (std::cmp::Reverse(warmth(Some(top))), t.1, t.0, top))
        })
        .collect();
    if tops.is_empty() {
        return;
    }
    tops.sort();
    let mut wearers: Vec<(std::cmp::Reverse<u16>, AgentId, u8)> = world
        .query::<(&Agent, Option<&Hardiness>, Option<&Apparel>)>()
        .without::<&Child>()
        .iter()
        .map(|(_, (a, h, ap))| {
            (
                std::cmp::Reverse(h.map_or(1000, |h| h.permille)),
                a.id,
                warmth(ap.map(|x| x.top)),
            )
        })
        .collect();
    wearers.sort();
    let mut served: std::collections::BTreeSet<AgentId> = std::collections::BTreeSet::new();
    for (_, y, x, top) in tops {
        let from = (x, y);
        let Some((id, entity, pos)) = wearers.iter().find_map(|&(_, id, now)| {
            if now >= warmth(Some(top)) || served.contains(&id) || assigned.contains(&id) {
                return None;
            }
            let &(_, entity, pos) = available.iter().find(|row| row.0 == id)?;
            res.nav
                .reaches(res.nav.region_of(pos), from)
                .then_some((id, entity, pos))
        }) else {
            continue;
        };
        served.insert(id);
        let job = Job {
            kind: JobKind::Equip { from },
            stage: JobStage::ToSource,
            taken_at: tick,
            amount: 1,
            item: top.item(),
            material: None,
            fetch: None,
            best_distance: chebyshev(pos, from),
            progress_tick: tick,
        };
        for key in job.keys() {
            super::cancel::reserve(res, key, id);
        }
        res.colony.tallies.jobs_taken += 1;
        assigned.insert(id);
        taken.push((entity, job, from));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_chance_follows_the_level_and_a_spear_doubles_it() {
        let rule = WildlifeRule::M5;
        assert_eq!(hit_permille(&rule, 0, false), 400);
        assert_eq!(hit_permille(&rule, 10, false), 600);
        assert_eq!(hit_permille(&rule, 20, false), 800);
        assert_eq!(hit_permille(&rule, 25, false), 800);
        assert_eq!(hit_permille(&rule, 0, true), 800);
        assert_eq!(hit_permille(&rule, 10, true), 950, "capped");
    }
}

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
//! M5.5 (`WildlifeRule::hunter_hauls`, ADR 0028 amendment): the kill turns the
//! hunt into a `Haul` of up to `hunter_meat` meat by the hunter to the nearest
//! stockpile cell with room ([`haul_home`]); the hide and any meat left are
//! noted in `Wildlife::carcasses` and hauled before other loose stacks.
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
    w.animals
        .values()
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

/// Candidate weapons ordered by authored preference, ties by stable item code.
pub fn draft_weapons() -> Vec<ItemKind> {
    let mut kinds: Vec<_> = ItemKind::all()
        .filter_map(|kind| {
            Some((
                std::cmp::Reverse(kind.definition().weapon.as_ref()?.draft_rank?),
                kind,
            ))
        })
        .collect();
    kinds.sort_unstable();
    kinds.into_iter().map(|(_, kind)| kind).collect()
}

fn hunting_tools(include_cutters: bool) -> Vec<ItemKind> {
    let mut ranked: Vec<_> = ItemKind::all()
        .filter_map(|kind| {
            kind.definition()
                .weapon
                .as_ref()
                .and_then(|w| w.hunt_rank)
                .map(|rank| (std::cmp::Reverse(rank), kind))
        })
        .collect();
    ranked.sort_unstable();
    let mut kinds: Vec<_> = ranked.into_iter().map(|(_, kind)| kind).collect();
    if include_cutters {
        for kind in ItemKind::all().filter(|kind| kind.definition().cutter.is_some()) {
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
    }
    kinds
}

fn wildlife_wear(rule: &WildlifeRule, selector: Option<&str>, default: u32) -> u32 {
    match selector {
        Some("spear_uses") => rule.spear_uses,
        Some("knife_uses") => rule.knife_uses,
        _ => default,
    }
}

/// Wear the actual held weapon when its capability declares combat wear.
pub fn wear_weapon(
    world: &mut World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    id: AgentId,
    entity: Entity,
) -> bool {
    let Ok(kind) = world.get::<&Tool>(entity).map(|t| t.item.kind) else {
        return false;
    };
    let Some(w) = kind.definition().weapon.as_ref() else {
        return false;
    };
    // Axes wear on chopping; only weapon-declared wear is applied to blows.
    if w.wear_rule.is_none() && w.wear_default == 0 {
        return false;
    }
    wear_tool(
        world,
        res,
        id,
        entity,
        kind,
        wildlife_wear(rule, w.wear_rule.as_deref(), w.wear_default),
    )
}

fn wear_cutter(
    world: &mut World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    id: AgentId,
    entity: Entity,
) -> bool {
    let Ok(kind) = world.get::<&Tool>(entity).map(|t| t.item.kind) else {
        return false;
    };
    let Some(c) = kind.definition().cutter.as_ref() else {
        return false;
    };
    wear_tool(
        world,
        res,
        id,
        entity,
        kind,
        wildlife_wear(rule, c.wear_rule.as_deref(), c.wear_default),
    )
}

/// The tool a hunter fetches (`held` = the kind it holds): a stocked spear unless it
/// holds one (swapping any other tool); with no tool and no spear, a knife.
pub fn hunting_fetch(
    res: &SimResources,
    pos: (u32, u32),
    reach: u32,
    held: Option<ItemKind>,
) -> Option<(u32, u32)> {
    if held.is_some_and(|kind| {
        kind.definition()
            .weapon
            .as_ref()
            .is_some_and(|w| w.hunt_rank.is_some())
    }) {
        return None;
    }
    nearest_stack(res, pos, reach, &hunting_tools(held.is_none()), true)
}

/// Take one item of the stack at `tile` as `entity`'s held tool when it is one of
/// `kinds`; a tool already held is left on the worker's tile (a hunter swapping an
/// axe for a spear). `owner` is the taker's reservation of the stack (a hunter's
/// fetch), released here; a stack another person has reserved is left alone (a
/// drafted person, who reserves nothing, passes `None`; M5 code review B1).
pub fn pick_up_tool(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    tile: (u32, u32),
    pos: (u32, u32),
    owner: Option<AgentId>,
    kinds: &[ItemKind],
) {
    let key = ReserveKey::Stack(tile);
    match res.colony.reservations.get(&key).copied() {
        Some(holder) if Some(holder) == owner => {
            res.colony.reservations.remove(&key);
        }
        Some(_) => return,
        None => {}
    }
    let Some(kind) = res
        .colony
        .ground_items
        .get(&tile)
        .map(|s| s.kind)
        .filter(|k| kinds.contains(k))
    else {
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
    let Ok(tool) = world.get::<&Tool>(entity) else {
        return rule.hand_damage;
    };
    let Some(weapon) = tool.item.kind.definition().weapon.as_ref() else {
        return rule.hand_damage;
    };
    match weapon.damage_rule.as_deref() {
        Some("spear_damage") => rule.spear_damage,
        Some("ground_axe_damage") => rule.ground_axe_damage,
        Some("axe_damage") => rule.axe_damage,
        _ => weapon.damage,
    }
}

/// The damage of `entity`'s blows in a fight: the held weapon's ([`weapon_damage`]), scaled to
/// `flee_defence_permille` while the person flees (M5.5, ADR 0028 amendment; at least 1).
pub fn defence_damage(world: &World, rule: &WildlifeRule, entity: Entity) -> u16 {
    let full = weapon_damage(world, rule, entity);
    if world
        .get::<&sim_core::colony::people::Fleeing>(entity)
        .is_ok()
    {
        (u32::from(full) * rule.flee_defence_permille / 1000).max(1) as u16
    } else {
        full
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
        let def = kind.definition();
        let tally = def
            .weapon
            .as_ref()
            .and_then(|w| w.break_tally.as_deref())
            .or_else(|| def.cutter.as_ref().and_then(|c| c.break_tally.as_deref()));
        match tally {
            Some("spears_broken") => res.colony.wildlife.tallies.spears_broken += 1,
            Some("knives_broken") => res.colony.wildlife.tallies.knives_broken += 1,
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
    let Some(deer) = res
        .colony
        .wildlife
        .animals
        .get(&animal)
        .filter(|a| a.kind.definition().huntable)
        .map(|a| a.pos)
    else {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    };
    // The tool stop.
    if let (Some(at), JobStage::ToSource) = (job.fetch, job.stage) {
        if !res.nav.arrived(pos, at) {
            super::execute::walk_job(world, res, id, entity, &mut job, pos, at);
            return;
        }
        pick_up_tool(world, res, entity, at, pos, Some(id), &hunting_tools(true));
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
                let meat = kill(world, res, &rule, id, entity, animal);
                // M5.5: the hunter carries the meat home before any other job.
                let hauled = rule.hunter_hauls
                    && meat
                        .first()
                        .is_some_and(|&from| haul_home(world, res, &rule, id, entity, from, pos));
                if !hauled {
                    finish(world, res, entity);
                }
                return;
            }
            // A miss: back to stalking (seeking, so the job layer keeps the job; review m1).
            job.stage = JobStage::ToSource;
            job.best_distance = chebyshev(pos, deer);
            job.progress_tick = res.current_tick;
            set_state(world, entity, SEEK_JOB);
            let _ = world.insert_one(entity, SeekTarget::new(deer));
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
    let spear = world.get::<&Tool>(entity).is_ok_and(|t| {
        t.item
            .kind
            .definition()
            .weapon
            .as_ref()
            .is_some_and(|w| w.hunt_hit_bonus)
    });
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
    wear_weapon(world, res, rule, id, entity);
    let Some(deer) = res.colony.wildlife.animals.get_mut(&animal) else {
        return false;
    };
    deer.hp = deer.hp.saturating_sub(damage);
    res.colony.wildlife.version += 1;
    deer.hp == 0
}

/// The deer fell: meat (a knife held: one use, more meat) and a hide on its tile. Returns the meat's tiles.
fn kill(
    world: &mut World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    id: AgentId,
    entity: Entity,
    animal: u32,
) -> Vec<(u32, u32)> {
    let Some(deer) = res.colony.wildlife.animals.remove(&animal) else {
        return Vec::new();
    };
    res.colony.wildlife.marks.remove(&animal);
    let knife = wear_cutter(world, res, rule, id, entity);
    if knife {
        res.colony.wildlife.tallies.knife_uses += 1;
    }
    let (meat, meat_tiles) = drop_animal_loot(res, deer.pos, deer.kind, rule, knife);
    let w = &mut res.colony.wildlife;
    match deer.kind.mechanic() {
        sim_core::colony::content::AnimalMechanic::Grazer => w.tallies.deer_killed += 1,
        sim_core::colony::content::AnimalMechanic::Predator => w.tallies.wolves_killed += 1,
    }
    w.version += 1;
    res.chronicle.push(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Hunted)
            .subject(id)
            .cause(deer.kind.code())
            .value(i64::from(meat)),
    );
    crate::runtime::knowledge::used(world, res, entity, gates().designation(Designation::Hunt));
    meat_tiles
}

/// Drop the killed species' declared loot, resolving selectors against this world.
/// Meat identity is restricted by catalog validation to the existing spoilage role.
pub fn drop_animal_loot(
    res: &mut SimResources,
    tile: (u32, u32),
    kind: AnimalKind,
    rule: &WildlifeRule,
    cutter: bool,
) -> (u16, Vec<(u32, u32)>) {
    let loot = &kind.definition().loot;
    let meat = loot.meat.as_ref().map(|m| {
        let amount = if cutter {
            m.with_cutter.wildlife(rule)
        } else {
            m.without_cutter.wildlife(rule)
        };
        (m.item, amount)
    });
    let hide = loot
        .hide
        .as_ref()
        .filter(|h| h.enabled.wildlife(rule))
        .map(|h| (h.item, h.count));
    let amount = meat.map_or(0, |(_, amount)| amount);
    (amount, drop_loot(res, tile, meat, hide))
}

/// Compatibility entry for fixtures using the bundled meat lifecycle and one hide.
pub fn drop_carcass(
    res: &mut SimResources,
    tile: (u32, u32),
    meat: u16,
    hide: ItemKind,
) -> Vec<(u32, u32)> {
    drop_loot(
        res,
        tile,
        Some((ItemKind::role("grazer_meat"), meat)),
        Some((hide, 1)),
    )
}

fn drop_loot(
    res: &mut SimResources,
    tile: (u32, u32),
    meat: Option<(ItemKind, u16)>,
    hide: Option<(ItemKind, u16)>,
) -> Vec<(u32, u32)> {
    let mut meat_tiles = Vec::new();
    if let Some((kind, meat)) = meat.filter(|(_, count)| *count > 0) {
        res.colony.item_ledger.create(kind, u64::from(meat));
        meat_tiles = put_items(res, tile, ItemSpec::raw(kind, None), meat);
        let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
        let lots = &mut res.colony.wildlife.meat_lots;
        match lots.last_mut() {
            Some((d, n)) if *d == day => *n += u32::from(meat),
            _ => lots.push((day, u32::from(meat))),
        }
        res.colony.wildlife.tallies.meat_dropped += u64::from(meat);
    }
    let mut hide_tiles = Vec::new();
    if let Some((kind, count)) = hide.filter(|(_, count)| *count > 0) {
        res.colony.item_ledger.create(kind, u64::from(count));
        hide_tiles = put_items(res, tile, ItemSpec::raw(kind, None), count);
        res.colony.wildlife.tallies.hides_dropped += u64::from(count);
    }
    if rule(res).is_some_and(|r| r.hunter_hauls) {
        let carcasses = &mut res.colony.wildlife.carcasses;
        carcasses.extend(meat_tiles.iter().copied());
        carcasses.extend(hide_tiles);
    }
    meat_tiles
}

/// M5.5 (ADR 0028 amendment): right after the kill the hunter carries the meat home — the hunt becomes a `Haul` of
/// up to `hunter_meat` meat from `from` to the nearest stockpile cell with room for it (`(distance, y, x)` from the
/// meat, not reserved, reachable from the hunter). Returns `false` (the hunt just finishes) when there is no meat
/// there or no such cell. The hunt counts as done and the haul as taken.
fn haul_home(
    world: &mut World,
    res: &mut SimResources,
    rule: &WildlifeRule,
    id: AgentId,
    entity: Entity,
    from: (u32, u32),
    pos: (u32, u32),
) -> bool {
    let Some(stack) = res.colony.ground_items.get(&from).filter(|s| s.count > 0) else {
        return false;
    };
    // The stack the kill dropped there (`drop_carcass` returns the tiles its meat went to).
    let (spec, count) = (stack.spec(), stack.count);
    let region = res.nav.region_of(pos);
    let Some(to) = res
        .colony
        .zone_of_tile
        .keys()
        .copied()
        .filter(|c| {
            super::items::room_on(res, *c, spec) > 0
                && !res.colony.reservations.contains_key(&ReserveKey::Cell(*c))
                && res.nav.reaches(region, *c)
        })
        .min_by_key(|c| (chebyshev(from, *c), c.1, c.0))
    else {
        return false;
    };
    let amount = rule
        .hunter_meat
        .min(count)
        .min(super::items::room_on(res, to, spec))
        .min(spec.kind.default_carry().max(1));
    if amount == 0 {
        return false;
    }
    super::cancel::release_all(res, id);
    res.colony.tallies.jobs_done += 1;
    let now = res.current_tick;
    let job = Job {
        kind: JobKind::Haul { from, to },
        stage: JobStage::ToSource,
        taken_at: now,
        amount,
        item: spec.kind,
        material: None,
        fetch: None,
        best_distance: chebyshev(pos, from),
        progress_tick: now,
    };
    for key in job.keys() {
        super::cancel::reserve(res, key, id);
    }
    res.colony.tallies.jobs_taken += 1;
    res.colony.wildlife.tallies.meat_hauls += 1;
    put_job(world, entity, job);
    set_state(world, entity, SEEK_JOB);
    let _ = world.insert_one(entity, SeekTarget::new(from));
    true
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
fn warmth(top: Option<Top>) -> u16 {
    top.map_or(0, Top::warmth_rank)
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
    let mut tops: Vec<(std::cmp::Reverse<u16>, u32, u32, Top)> = res
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
    let mut wearers: Vec<(std::cmp::Reverse<u16>, AgentId, u16)> = world
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

#[cfg(test)]
mod item_capability_tests {
    use super::*;

    #[test]
    fn historical_weapon_preferences_and_rule_damage_are_preserved() {
        assert_eq!(
            draft_weapons(),
            vec![ItemKind::Spear, ItemKind::GroundAxe, ItemKind::StoneAxe]
        );
        assert_eq!(hunting_tools(false), vec![ItemKind::Spear]);
        assert_eq!(
            hunting_tools(true),
            vec![ItemKind::Spear, ItemKind::FlintBlade]
        );
        let mut world = World::new();
        let mut rule = WildlifeRule::M5;
        rule.spear_damage = 71;
        rule.ground_axe_damage = 53;
        rule.axe_damage = 29;
        rule.hand_damage = 7;
        for (kind, damage) in [
            (ItemKind::Spear, 71),
            (ItemKind::GroundAxe, 53),
            (ItemKind::StoneAxe, 29),
            (ItemKind::FlintBlade, 7),
        ] {
            let entity = world.spawn((Tool {
                item: ItemSpec::raw(kind, None),
                uses: 0,
            },));
            assert_eq!(weapon_damage(&world, &rule, entity), damage);
        }
        let empty = world.spawn(());
        assert_eq!(weapon_damage(&world, &rule, empty), 7);
    }
}

//! JobExecuteSystem: moves each held job along — arrive, chop, pick up, drop, deliver, hand a builder to the V7 site flow.
//!
//! Priority: 127 (`JOB_EXECUTE_PRIORITY`). Interval: 1 tick. Gate: `WorldRules.colony`.
//! Reads: `Agent`, `Job`, `AgentState`, `Position`, `Carry`; `SimResources.colony` (trees, designations, ground items, zones), `rules.colony`, `current_tick`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`)
//! Writes: `Job`, `Carry`, `AgentState`, `SeekTarget`, `MaterialNeeds` (delivered); `SimResources.colony` (trees, trees_version, designations, ground items, reservations, item_ledger, overlay_version, tallies)
//! Events: none
//!
//! Runs after movement (120), the job assignment (124) and the decision seam
//! (125), so it sees this tick's position and every early end. Workers are
//! handled in ascending `AgentId` (goal 7.3):
//! - a worker whose state is no longer its job's (`Seeking`/`Consuming
//!   { Job }`, a builder's `Consuming { ConstructionSite }`) was moved by
//!   another system (a finished site, combat, fission): a builder whose site
//!   is gone has finished (counted done; its new state and target stand);
//!   anyone else is released (`Interrupted`);
//! - a vanished target (tree felled, stack taken, cell unzoned, site
//!   cancelled) cancels the job (`TargetGone`);
//! - arrival: a tree counts as reached on it or 8-adjacent; a stack, cell or
//!   site only on the tile itself, or from any 8-neighbour when the tile is
//!   impassable (M3: a wall site);
//! - a walking job without progress for `stuck_ticks` is cancelled (`Stuck`);
//!   progress is a shorter Chebyshev distance or, on a stored path (M3), fewer
//!   steps left, and a re-planned path restarts the count;
//! - M3: a walking job whose goal the move found unreachable (`nav.blocked`)
//!   is cancelled (`Unreachable`) and the target is avoided by that worker for
//!   `SpaceRule::unreachable_cooldown_days` (1 day without the rule).
//!
//! Under debug assertions the 7.6 invariants are checked after every tick.
use hecs::{Entity, World};
use sim_core::colony::materials::OAK;
use sim_core::colony::{
    CancelReason, Carry, Designation, ItemKind, ItemSpec, ItemStack, Job, JobKind, JobStage,
    MaterialNeeds, ReserveKey,
};
use sim_core::components::{Agent, AgentId, AgentState, Position, SeekTarget, TargetKind};
use sim_core::rules::ColonyRule;
use sim_engine::{RuntimeSystem, SimResources};

use super::cancel::{cancel, finish, finish_in_place, release_job};
use super::craft;
use super::items::{chebyshev, colony_sites, put_items, take_items};
use sim_core::colony::skills::{axe_permille, scaled_ticks};
use sim_core::colony::{quality_bucket, SkillKind, Tool};

/// Priority of [`JobExecuteSystem`].
pub const JOB_EXECUTE_PRIORITY: u32 = 127;

/// See the module header.
#[derive(Debug, Default)]
pub struct JobExecuteSystem;

impl JobExecuteSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

const SEEK_JOB: AgentState = AgentState::Seeking {
    target: TargetKind::Job,
};
const WORK_JOB: AgentState = AgentState::Consuming {
    target: TargetKind::Job,
};
const BUILD: AgentState = AgentState::Consuming {
    target: TargetKind::ConstructionSite,
};

fn state_of(world: &World, entity: Entity) -> AgentState {
    world
        .get::<&AgentState>(entity)
        .map(|state| *state)
        .unwrap_or_default()
}

fn set_state(world: &mut World, entity: Entity, next: AgentState) {
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = next;
    }
}

/// The state a worker in `stage` of `kind` must be in (`space`: a Build of an
/// M3 space site, which the job layer works itself).
fn expected_state(kind: JobKind, stage: JobStage, space: bool) -> AgentState {
    match (kind, stage) {
        (JobKind::Build { .. }, JobStage::Working { .. }) if !space => BUILD,
        (_, JobStage::Working { .. }) => WORK_JOB,
        _ => SEEK_JOB,
    }
}

fn site_needs(world: &World, site: (u32, u32)) -> Option<(Entity, MaterialNeeds)> {
    colony_sites(world)
        .get(&site)
        .map(|view| (view.entity, view.needs))
}

/// Whether the job's target still exists.
fn target_exists(
    world: &World,
    res: &SimResources,
    id: AgentId,
    entity: Entity,
    job: &Job,
    pot_sources: &super::pot_supply::SourceIndex,
) -> bool {
    let colony = &res.colony;
    let stack = |tile: (u32, u32)| {
        colony
            .ground_items
            .get(&tile)
            .is_some_and(|stack| stack.kind == job.item && stack.count > 0)
    };
    let hearth_destination = |from, to, spec| {
        world.get::<&Position>(entity).ok().is_some_and(|pos| {
            super::hunt::hearth_haul_destination(res, id, from, to, (pos.x, pos.y), spec)
        })
    };
    match (job.kind, job.stage) {
        (JobKind::Chop { tree }, _) => {
            colony.trees.contains_key(&tree) && colony.designations.contains_key(&tree)
        }
        (JobKind::Haul { from, to }, JobStage::ToSource) => {
            stack(from)
                && (colony.zone_of_tile.contains_key(&to)
                    || colony
                        .ground_items
                        .get(&from)
                        .is_some_and(|source| hearth_destination(from, to, source.spec())))
        }
        (JobKind::Haul { from, to }, _) => {
            colony.zone_of_tile.contains_key(&to)
                || world.get::<&Carry>(entity).ok().is_some_and(|carry| {
                    carry.count > 0
                        && carry.kind == job.item
                        && hearth_destination(from, to, carry.spec())
                })
        }
        (JobKind::Deliver { from, site }, JobStage::ToSource) => {
            stack(from) && site_needs(world, site).is_some()
        }
        (JobKind::Deliver { site, .. }, _) | (JobKind::Build { site }, _) => {
            site_needs(world, site).is_some()
        }
        (JobKind::Quarry { node }, _) => {
            colony.nodes.contains_key(&node)
                && colony.designations.get(&node) == Some(&Designation::Quarry)
        }
        (JobKind::Harvest { plant }, _) => super::harvest::target_exists(res, plant),
        (JobKind::Supply { from, workshop, .. }, JobStage::ToSource) => {
            pot_sources.exists(res, from, job.item) && colony.workshops.contains_key(&workshop)
        }
        (JobKind::Supply { workshop, .. }, _) => colony.workshops.contains_key(&workshop),
        // Gathering: the spot holds wood (or the gatherer already carries some);
        // the drop always exists (items spill around a full or lost cell).
        (JobKind::Gather { spot, .. }, JobStage::ToSource) => {
            res.wood_tiles.get(&spot).is_some_and(|u| *u > 0)
        }
        (JobKind::Gather { .. }, _) => true,
        (JobKind::Craft { workshop, bill, .. }, _) => colony
            .workshops
            .get(&workshop)
            .is_some_and(|shop| shop.bill(bill).is_some()),
        (JobKind::Dig { tile }, _) => {
            colony.designations.get(&tile) == Some(&Designation::Dig)
                && super::space::diggable(res, tile)
        }
        (JobKind::Deconstruct { tile }, _) => {
            colony.designations.get(&tile) == Some(&Designation::Deconstruct)
                && super::space::built_at(res, tile).is_some()
        }
        // Clearing a site: the stack is there; the drop always exists (spill).
        (JobKind::Clear { from, .. }, JobStage::ToSource) => stack(from),
        (JobKind::Clear { .. }, _) => true,
        (JobKind::DigClay { node }, _) => {
            colony.nodes.contains_key(&node)
                && colony.designations.get(&node) == Some(&Designation::ClayPit)
        }
        // Apprenticeship: `teach::step` checks the order and the partner.
        (JobKind::Teach { .. } | JobKind::Learn { .. }, _) => true,
        // M5: the hunted deer is alive; the top is still on its stack.
        (JobKind::Hunt { animal }, _) => colony.wildlife.animals.contains_key(&animal),
        (JobKind::Equip { from }, JobStage::ToSource) => {
            colony.ground_items.get(&from).is_some_and(|s| {
                sim_core::colony::people::Top::of_item(s.kind).is_some() && s.count > 0
            })
        }
        (JobKind::Equip { .. }, _) => true,
        // M5.5: the caravan still trades at the hearth (and, walking to the source, the stack is there).
        (JobKind::Trade { from, .. }, JobStage::ToSource) => {
            stack(from) && colony.neighbours.trading_caravan().is_some()
        }
        (JobKind::Trade { .. }, _) => colony.neighbours.trading_caravan().is_some(),
    }
}

/// One worker's tick. `job` is its current job (copied out of the world).
fn step(
    world: &mut World,
    res: &mut SimResources,
    rule: &ColonyRule,
    id: AgentId,
    entity: Entity,
    mut job: Job,
    pot_sources: &super::pot_supply::SourceIndex,
) {
    let tick = res.current_tick;
    // A partner's step this tick may already have ended this job (M4 apprenticeship).
    if world.get::<&Job>(entity).is_err() {
        return;
    }
    let state = state_of(world, entity);
    let space =
        matches!(job.kind, JobKind::Build { site } if res.colony.space_sites.contains_key(&site));
    if state != expected_state(job.kind, job.stage, space) {
        let built =
            matches!(job.kind, JobKind::Build { site } if site_needs(world, site).is_none());
        if built && matches!(job.stage, JobStage::Working { .. }) {
            finish_in_place(world, res, entity);
        } else {
            release_job(world, res, entity, CancelReason::Interrupted);
        }
        return;
    }
    if !target_exists(world, res, id, entity, &job, pot_sources) {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    }
    let Some(pos) = world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)) else {
        return;
    };
    // M3: the move could not find a way to this stage's goal.
    let goal = match job.stage {
        JobStage::ToSource => Some(job.fetch.unwrap_or(job.kind.source())),
        JobStage::ToDest => job.kind.dest(),
        JobStage::Working { .. } => None,
    };
    if let Some(goal) = goal.filter(|g| res.nav.blocked.get(&id) == Some(g)) {
        let days = rule
            .space
            .map_or(1, |space| space.unreachable_cooldown_days);
        let until = tick + res.rules.calendar.days_to_ticks(days);
        res.colony.unreachable.insert((id, goal), until);
        cancel(world, res, entity, CancelReason::Unreachable);
        return;
    }
    // M5: hunting and dressing run in `hunt`.
    match job.kind {
        JobKind::Hunt { .. } => {
            super::hunt::step(world, res, id, entity, job, pos);
            return;
        }
        JobKind::Equip { .. } => {
            super::hunt::equip_step(world, res, id, entity, job, pos);
            return;
        }
        _ => {}
    }
    // M4: apprenticeship jobs walk to the meeting tile, then `teach::step` runs them.
    if matches!(job.kind, JobKind::Teach { .. } | JobKind::Learn { .. }) {
        let at = job.kind.source();
        if job.stage == JobStage::ToSource && chebyshev(pos, at) > 1 {
            walk(world, res, id, entity, &mut job, pos, at);
            return;
        }
        if let Some(next) = super::teach::step(world, res, entity, job, pos) {
            put_job(world, entity, next);
        }
        return;
    }
    match job.stage {
        JobStage::ToSource => {
            // M2: a chopper first stops at its axe (goal 3.7).
            if let Some(axe) = job.fetch {
                if !res.nav.arrived(pos, axe) {
                    walk(world, res, id, entity, &mut job, pos, axe);
                    return;
                }
                pick_up_axe(world, res, entity, &mut job, axe, pos);
                put_job(world, entity, job);
                return;
            }
            let source = job.kind.source();
            let reached = match job.kind {
                JobKind::Chop { tree: near }
                | JobKind::Harvest { plant: near }
                | JobKind::Quarry { node: near }
                | JobKind::Gather { spot: near, .. }
                | JobKind::Dig { tile: near }
                | JobKind::DigClay { node: near }
                | JobKind::Deconstruct { tile: near } => chebyshev(pos, near) <= 1,
                _ => res.nav.arrived(pos, source),
            };
            if !reached {
                walk(world, res, id, entity, &mut job, pos, source);
                return;
            }
            match job.kind {
                JobKind::Chop { .. } => {
                    let ticks = work_ticks(world, rule, entity, rule.chop_ticks, SkillKind::Chop);
                    let ticks =
                        crate::runtime::story::scale_work(world, res, entity, source, ticks);
                    job.amount = ticks.min(u32::from(u16::MAX)) as u16;
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Harvest { plant } => {
                    let practice = if rule.knowledge.is_some() {
                        world
                            .get::<&sim_core::colony::knowledge::Knowledge>(entity)
                            .map_or(0, |k| k.practice())
                    } else {
                        u32::MAX
                    };
                    let ticks = super::harvest::work_ticks(res, plant, practice);
                    let ticks =
                        crate::runtime::story::scale_work(world, res, entity, source, ticks);
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Quarry { .. } => {
                    let base = rule.economy.map_or(120, |e| e.quarry_ticks);
                    let ticks = work_ticks(world, rule, entity, base, SkillKind::Quarry);
                    let ticks =
                        crate::runtime::story::scale_work(world, res, entity, source, ticks);
                    job.amount = ticks.min(u32::from(u16::MAX)) as u16;
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Build { site } => {
                    job.stage = JobStage::Working { ticks_left: 0 };
                    set_state(world, entity, if space { WORK_JOB } else { BUILD });
                    // M4: building a gated blueprint uses its knowledge (stamped as work starts).
                    let kind = colony_sites(world).get(&site).map(|v| v.site.kind);
                    if let Some(kind) = kind {
                        let mask = sim_core::colony::knowledge::gates().build(kind);
                        crate::runtime::knowledge::used(world, res, entity, mask);
                    }
                }
                JobKind::DigClay { .. } => {
                    let ticks = work_ticks(world, rule, entity, CLAY_TICKS, SkillKind::Mining);
                    let ticks =
                        crate::runtime::story::scale_work(world, res, entity, source, ticks);
                    job.amount = ticks.min(u32::from(u16::MAX)) as u16;
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Dig { .. } => {
                    let base = rule.space.map_or(150, |s| s.dig_ticks);
                    let ticks = work_ticks(world, rule, entity, base, SkillKind::Mining);
                    let ticks =
                        crate::runtime::story::scale_work(world, res, entity, source, ticks);
                    job.amount = ticks.min(u32::from(u16::MAX)) as u16;
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Deconstruct { tile } => {
                    let ticks = super::space::built_at(res, tile)
                        .map_or(1, |b| (b.plan(rule).2 / 2).max(1));
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Gather { .. } => {
                    job.stage = JobStage::Working {
                        ticks_left: GATHER_BATCH_TICKS,
                    };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Craft { workshop, bill, .. } => {
                    let Some(ticks) = craft::start(world, res, entity, &mut job, workshop, bill)
                    else {
                        cancel(world, res, entity, CancelReason::TargetGone);
                        return;
                    };
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                // Handled before the stage match (M4 apprenticeship, M5 hunting and dressing).
                JobKind::Teach { .. }
                | JobKind::Learn { .. }
                | JobKind::Hunt { .. }
                | JobKind::Equip { .. } => {}
                JobKind::Haul { from, .. }
                | JobKind::Deliver { from, .. }
                | JobKind::Supply { from, .. }
                | JobKind::Clear { from, .. }
                | JobKind::Trade { from, .. } => {
                    let (taken, spec) = super::food_transfers::pickup(
                        world, res, entity, from, job.item, job.amount,
                    );
                    res.colony.reservations.remove(&ReserveKey::Stack(from));
                    if taken == 0 {
                        cancel(world, res, entity, CancelReason::TargetGone);
                        return;
                    }
                    // M5.5: owed items stay owed until they leave the stack (review F1: never sold twice).
                    if matches!(job.kind, JobKind::Trade { .. }) {
                        crate::runtime::neighbours::trade::take_owed(res, from, spec.kind, taken);
                    }
                    job.amount = taken;
                    job.stage = JobStage::ToDest;
                    let dest = job.kind.dest().unwrap_or(pos);
                    job.best_distance = chebyshev(pos, dest);
                    job.progress_tick = tick;
                    if spec.kind.definition().food.is_some() {
                        let _ = world.insert_one(entity, SeekTarget::new(dest));
                    } else {
                        let _ =
                            world.insert(entity, (Carry::of(spec, taken), SeekTarget::new(dest)));
                    }
                }
            }
            put_job(world, entity, job);
            // A worker already standing at the destination drops at once.
            if job.stage == JobStage::ToDest
                && job.kind.dest().is_some_and(|d| res.nav.arrived(pos, d))
            {
                arrive(world, res, id, entity, &job, pos);
            }
        }
        JobStage::Working { ticks_left } => {
            if !craft::can_continue(world, res, entity, &job) {
                return;
            }
            if let JobKind::Build { site } = job.kind {
                // A builder: ConstructionSystem does the work; it trains Build (M2).
                if rule.economy.is_some() {
                    craft::train(world, entity, SkillKind::Build, 1);
                }
                // M3: a space site is worked from beside it, by the job layer.
                if space {
                    build_space(world, res, entity, site);
                }
                return;
            }
            if ticks_left > 1 {
                job.stage = JobStage::Working {
                    ticks_left: ticks_left - 1,
                };
                put_job(world, entity, job);
                return;
            }
            match job.kind {
                JobKind::Harvest { plant } => {
                    let mask = super::harvest::knowledge_mask(res, plant);
                    let practiced = world
                        .get::<&sim_core::colony::knowledge::Knowledge>(entity)
                        .map_or(0, |knowledge| knowledge.practice() & mask);
                    let Some(placements) =
                        super::harvest::complete_observed_placements(res, plant, id, job.taken_at)
                    else {
                        cancel(world, res, entity, CancelReason::TargetGone);
                        return;
                    };
                    crate::runtime::knowledge::used(world, res, entity, practiced);
                    if super::harvest_haul::start(world, res, entity, &placements) {
                        return;
                    }
                }
                JobKind::Chop { tree } => {
                    if !fell(res, tree) {
                        cancel(world, res, entity, CancelReason::TargetGone);
                        return;
                    }
                    if rule.economy.is_some() {
                        craft::train(world, entity, SkillKind::Chop, u32::from(job.amount));
                        use_axe(world, res, rule, id, entity, pos);
                    }
                }
                JobKind::Quarry { node } => {
                    quarry_batch(res, rule, node, pos);
                    craft::train(world, entity, SkillKind::Quarry, u32::from(job.amount));
                    let mask =
                        sim_core::colony::knowledge::gates().designation(Designation::Quarry);
                    crate::runtime::knowledge::used(world, res, entity, mask);
                }
                JobKind::DigClay { node } => {
                    clay_batch(res, node, pos);
                    craft::train(world, entity, SkillKind::Mining, u32::from(job.amount));
                    let mask =
                        sim_core::colony::knowledge::gates().designation(Designation::ClayPit);
                    crate::runtime::knowledge::used(world, res, entity, mask);
                }
                JobKind::Craft { workshop, bill, at } => {
                    craft::finish_craft(world, res, id, entity, &job);
                    // M3 (DECISIONS D13): the crafter stays at the bench for the
                    // bill's next craft when its inputs are already on the shelf.
                    if rule.space.is_some() && craft::next_ready(world, res, workshop, bill) {
                        res.colony.tallies.jobs_done += 1;
                        job.kind = JobKind::Craft { workshop, bill, at };
                        job.stage = JobStage::ToSource;
                        job.material = None;
                        job.taken_at = tick;
                        job.progress_tick = tick;
                        job.best_distance = 0;
                        put_job(world, entity, job);
                        set_state(world, entity, SEEK_JOB);
                        return;
                    }
                }
                JobKind::Gather { .. } => {
                    if gather_batch(world, res, rule, entity, &mut job, pos) {
                        put_job(world, entity, job);
                        return;
                    }
                }
                JobKind::Dig { tile } => {
                    super::space::dig(res, tile);
                    craft::train(world, entity, SkillKind::Mining, u32::from(job.amount));
                    let mask = sim_core::colony::knowledge::gates().designation(Designation::Dig);
                    crate::runtime::knowledge::used(world, res, entity, mask);
                }
                JobKind::Deconstruct { tile } => {
                    super::space::deconstruct(res, tile);
                }
                _ => {}
            }
            finish(world, res, entity);
        }
        JobStage::ToDest => {
            let dest = job.kind.dest().unwrap_or(pos);
            if res.nav.arrived(pos, dest) {
                arrive(world, res, id, entity, &job, pos);
            } else {
                walk(world, res, id, entity, &mut job, pos, dest);
            }
        }
    }
}

/// One tick of work on the space site at `site` (M3): its progress grows by
/// one; at the end the tile is built (`space::complete_site`) and the job ends.
fn build_space(world: &mut World, res: &mut SimResources, entity: Entity, site: (u32, u32)) {
    let Some((site_entity, _)) = site_needs(world, site) else {
        return;
    };
    let done = world
        .get::<&mut sim_core::components::ConstructionSite>(site_entity)
        .map(|mut c| c.advance())
        .unwrap_or(false);
    if done {
        super::space::complete_site(world, res, site_entity);
        finish(world, res, entity);
    }
}

/// Work ticks of a job of `base` ticks for `entity` (M2): scaled by its level
/// of `skill`, and for chopping by its axe (`AXE_PERMILLE` of the axe's quality
/// bucket). Without the economy (M1 rules) the base ticks.
fn work_ticks(
    world: &World,
    rule: &ColonyRule,
    entity: Entity,
    base: u32,
    skill: SkillKind,
) -> u32 {
    if rule.economy.is_none() {
        return base.max(1);
    }
    let ticks = scaled_ticks(base, craft::level_of(world, entity, skill));
    // M4: a ground-stone axe chops faster than a stone axe (`axe_permille`).
    let factor = (skill == SkillKind::Chop)
        .then(|| world.get::<&Tool>(entity).ok().map(|t| t.item))
        .flatten()
        .and_then(|item| axe_permille(item.kind, quality_bucket(item.quality)));
    match factor {
        Some(factor) => ((u64::from(ticks) * u64::from(factor) + 500) / 1000).max(1) as u32,
        None => ticks.max(1),
    }
}

/// At the axe stack: take one axe as the worker's `Tool` (if the stack is
/// still there), release the stack and walk on to the tree.
fn pick_up_axe(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    job: &mut Job,
    axe: (u32, u32),
    pos: (u32, u32),
) {
    // M4: the stack holds a stone or a ground-stone axe.
    let kind = res
        .colony
        .ground_items
        .get(&axe)
        .map_or(ItemKind::role("legacy_axe"), |s| s.kind);
    let (taken, spec) = take_items(res, axe, kind, 1);
    res.colony.reservations.remove(&ReserveKey::Stack(axe));
    if taken == 1 && world.get::<&Tool>(entity).is_err() {
        let _ = world.insert_one(
            entity,
            Tool {
                item: spec,
                uses: 0,
            },
        );
    } else if taken == 1 {
        put_items(res, pos, spec, 1);
    }
    job.fetch = None;
    let tree = job.kind.source();
    job.best_distance = chebyshev(pos, tree);
    job.progress_tick = res.current_tick;
    if let Ok(mut seek) = world.get::<&mut SeekTarget>(entity) {
        *seek = SeekTarget::new(tree);
    }
}

/// A chop done with an axe: count it, wear the axe; a worn-out axe becomes
/// one stone at the worker's tile (ledger: an axe consumed, a stone created),
/// or under M4's tool wear (`EconomyRule::tools_break`) is used up (ledger:
/// consumed; chronicle `ToolBroke`).
fn use_axe(
    world: &mut World,
    res: &mut SimResources,
    rule: &ColonyRule,
    id: AgentId,
    entity: Entity,
    pos: (u32, u32),
) {
    let Ok(mut tool) = world.get::<&mut Tool>(entity).map(|t| *t) else {
        return;
    };
    let Some(chop) = tool.item.kind.definition().chop.as_ref() else {
        return;
    };
    tool.uses += 1;
    res.colony.tallies.chops_with_axe += 1;
    let limit = rule.economy.map_or(0, |e| match chop.wear_rule.as_deref() {
        Some("ground_axe_uses") => e.ground_axe_uses,
        Some("axe_uses") => e.axe_uses,
        _ => chop.wear_default,
    });
    if limit > 0 && tool.uses >= limit && rule.economy.is_some_and(|e| e.tools_break) {
        let _ = world.remove_one::<Tool>(entity);
        res.colony.item_ledger.consume(tool.item.kind, 1);
        res.colony.tallies.tools_broken += 1;
        res.chronicle.push(
            sim_core::chronicle::ChronicleEntry::new(
                res.current_tick,
                sim_core::chronicle::ChronicleKind::ToolBroke,
            )
            .subject(id)
            .cause(tool.item.kind.code())
            .value(i64::from(tool.uses)),
        );
    } else if limit > 0 && tool.uses >= limit {
        let _ = world.remove_one::<Tool>(entity);
        res.colony.item_ledger.consume(tool.item.kind, 1);
        if let Some((kind, count)) = chop.break_return {
            res.colony.item_ledger.create(kind, u64::from(count));
            let material = chop
                .preserve_return_material
                .then_some(tool.item.material)
                .flatten();
            put_items(res, pos, ItemSpec::raw(kind, material), count);
        }
        res.colony.tallies.axes_worn_out += 1;
    } else if let Ok(mut held) = world.get::<&mut Tool>(entity) {
        *held = tool;
    }
}

fn put_job(world: &mut World, entity: Entity, job: Job) {
    if let Ok(mut held) = world.get::<&mut Job>(entity) {
        *held = job;
    }
}

/// [`walk`] for the job layer's other modules (M5 hunting and dressing).
pub(super) fn walk_job(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    job: &mut Job,
    pos: (u32, u32),
    goal: (u32, u32),
) {
    walk(world, res, id, entity, job, pos, goal);
}

/// Track walking progress; cancel a job stuck for `stuck_ticks`. On a stored
/// path (M3) progress is fewer steps left, and a path (re)planned after the
/// last progress restarts the count from its length.
fn walk(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    job: &mut Job,
    pos: (u32, u32),
    goal: (u32, u32),
) {
    let tick = res.current_tick;
    let path = res.nav.paths.get(&id).filter(|p| p.goal == goal);
    let distance = path.map_or(chebyshev(pos, goal), |p| p.remaining());
    // A (re)planned path restarts the count; else progress is a shorter way left.
    if path.is_some_and(|p| p.planned > job.progress_tick) || distance < job.best_distance {
        job.best_distance = distance;
        job.progress_tick = tick;
    } else if tick.saturating_sub(job.progress_tick)
        >= res.rules.colony.map_or(u64::MAX, |r| r.stuck_ticks)
    {
        cancel(world, res, entity, CancelReason::Stuck);
        return;
    }
    put_job(world, entity, *job);
}

/// A felled tree: gone with its mark, its logs on its tile (7.3).
fn fell(res: &mut SimResources, tree: (u32, u32)) -> bool {
    let Some(material) = res.content_state.tree_material(tree) else {
        return false;
    };
    let flora = sim_core::colony::content::installed_or_bundled().flora();
    if flora.species_by_material(material).is_none()
        || res.colony.materials.stats(material).is_none()
    {
        return false;
    }
    let Some(felled) = res.colony.trees.remove(&tree) else {
        return false;
    };
    res.content_state.forget_tree_material(tree);
    res.colony.designations.remove(&tree);
    res.colony.touch(true);
    res.colony.tallies.trees_felled += 1;
    let logs = u16::from(felled.logs);
    res.colony
        .item_ledger
        .create(ItemKind::role("tree_output"), u64::from(logs));
    put_items(
        res,
        tree,
        ItemSpec::raw(ItemKind::role("tree_output"), Some(material)),
        logs,
    );
    true
}

/// Work ticks of one deadwood batch (M2 goal 3.8).
pub const GATHER_BATCH_TICKS: u32 = 30;
/// Firewood per deadwood batch (goal 3.8: 2 per tile batch).
pub const GATHER_BATCH: u8 = 2;

/// One deadwood batch: up to 2 wood units of the spot become firewood items in
/// the gatherer's hands (ledger: created). The job goes on with another batch
/// while the spot holds wood and the hands have room, else it walks the load
/// to `to` (`true` = the job continues; `false` = it ended with nothing carried).
fn gather_batch(
    world: &mut World,
    res: &mut SimResources,
    rule: &ColonyRule,
    entity: Entity,
    job: &mut Job,
    pos: (u32, u32),
) -> bool {
    let JobKind::Gather { spot, to } = job.kind else {
        return false;
    };
    let units = res.wood_tiles.get(&spot).copied().unwrap_or(0);
    let take = units.min(GATHER_BATCH);
    if take > 0 {
        let left = units - take;
        if left == 0 {
            res.wood_tiles.remove(&spot);
        } else {
            res.wood_tiles.insert(spot, left);
        }
    }
    let spec = ItemSpec::raw(ItemKind::role("deadwood_output"), Some(OAK));
    let held = world.get::<&Carry>(entity).map_or(0, |c| c.count);
    let total = held + u16::from(take);
    if take > 0 {
        res.colony
            .item_ledger
            .create(ItemKind::role("deadwood_output"), u64::from(take));
        let _ = world.insert_one(entity, Carry::of(spec, total));
    }
    if total == 0 {
        return false;
    }
    let more = res.wood_tiles.get(&spot).is_some_and(|u| *u > 0);
    if more && total < rule.carry(ItemKind::role("deadwood_output")) {
        job.stage = JobStage::Working {
            ticks_left: GATHER_BATCH_TICKS,
        };
        return true;
    }
    res.colony.reservations.remove(&ReserveKey::Wood(spot));
    job.stage = JobStage::ToDest;
    job.amount = total;
    job.best_distance = chebyshev(pos, to);
    job.progress_tick = res.current_tick;
    set_state(world, entity, SEEK_JOB);
    let _ = world.insert_one(entity, SeekTarget::new(to));
    true
}

/// One quarry batch (M2 goal 3.2): up to `quarry_batch` items of the node's
/// yield and material land on its tile (M3: at the quarrier's tile when the
/// node's tile cannot hold items, e.g. a boulder on undug hill); an exhausted
/// node is removed with its mark.
fn quarry_batch(res: &mut SimResources, rule: &ColonyRule, node: (u32, u32), at: (u32, u32)) {
    let batch = rule.economy.map_or(4, |e| e.quarry_batch).max(1);
    let Some(found) = res.colony.nodes.get_mut(&node) else {
        return;
    };
    let n = found.left.min(batch);
    found.left -= n;
    let (spec, left) = (
        ItemSpec::raw(found.kind.yields(), Some(found.material)),
        found.left,
    );
    if left == 0 {
        res.colony.nodes.remove(&node);
        res.colony.designations.remove(&node);
        res.colony.tallies.nodes_exhausted += 1;
    }
    res.colony.tallies.quarry_batches += 1;
    res.colony.touch(false);
    if n > 0 {
        res.colony.item_ledger.create(spec.kind, u64::from(n));
        let drop = if super::items::can_hold_items(res, node) {
            node
        } else {
            at
        };
        put_items(res, drop, spec, u16::from(n));
    }
}

/// Work ticks of one clay batch at skill level 5 (M4 goal 3.5: 60).
pub const CLAY_TICKS: u32 = 60;
/// Clay one batch yields (M4 goal 3.5: 3).
pub const CLAY_BATCH: u8 = 3;

/// One clay batch (M4 goal 3.5): up to [`CLAY_BATCH`] clay of the bank lands
/// on the digger's tile (a bank on the river's edge may not hold items); a
/// bank dug out (after 6 batches) is removed with its mark.
fn clay_batch(res: &mut SimResources, node: (u32, u32), at: (u32, u32)) {
    let Some(found) = res.colony.nodes.get_mut(&node) else {
        return;
    };
    let n = found.left.min(CLAY_BATCH);
    found.left -= n;
    let (spec, left) = (
        ItemSpec::raw(ItemKind::role("clay_output"), Some(found.material)),
        found.left,
    );
    if left == 0 {
        res.colony.nodes.remove(&node);
        res.colony.designations.remove(&node);
        res.colony.tallies.nodes_exhausted += 1;
    }
    res.colony.tallies.clay_batches += 1;
    res.colony.touch(false);
    if n > 0 {
        res.colony
            .item_ledger
            .create(ItemKind::role("clay_output"), u64::from(n));
        let drop = if super::items::can_hold_items(res, node) {
            node
        } else {
            at
        };
        put_items(res, drop, spec, u16::from(n));
    }
}

/// The load reached its destination: drop it in the cell or deliver it to the site.
fn arrive(
    world: &mut World,
    res: &mut SimResources,
    _id: AgentId,
    entity: Entity,
    job: &Job,
    pos: (u32, u32),
) {
    let carry = world.get::<&Carry>(entity).ok().map(|c| *c);
    if let Some(carry) = carry.filter(|carry| carry.kind.definition().food.is_some()) {
        super::food_transfers::arrive(world, res, entity, job.kind, pos, carry);
        finish(world, res, entity);
        return;
    }
    let _ = world.remove_one::<Carry>(entity);
    if let Some(carry) = carry {
        match job.kind {
            JobKind::Supply { workshop, .. } => match res.colony.workshops.get_mut(&workshop) {
                Some(shop) => {
                    let spec = carry.spec();
                    match shop.shelf.iter_mut().find(|s| s.spec().merges_with(&spec)) {
                        Some(stack) => stack.absorb(spec, carry.count),
                        None => shop.shelf.push(ItemStack::of(spec, carry.count)),
                    }
                    res.colony.touch(false);
                }
                None => {
                    put_items(res, pos, carry.spec(), carry.count);
                }
            },
            JobKind::Deliver { site, .. } => {
                let mut rest = carry.count;
                if let Some((site_entity, _)) = site_needs(world, site) {
                    if let Ok(mut needs) = world.get::<&mut MaterialNeeds>(site_entity) {
                        rest -= needs.deliver_spec(carry.spec(), rest);
                    }
                    res.colony.touch(false);
                }
                if rest > 0 {
                    put_items(res, pos, carry.spec(), rest);
                }
            }
            // M5.5 (ADR 0032): handed to the caravan — it leaves the colony's ledger.
            JobKind::Trade { .. } => {
                res.colony
                    .item_ledger
                    .trade_out(carry.kind, u64::from(carry.count));
                res.colony.neighbours.version += 1;
            }
            _ => {
                put_items(res, pos, carry.spec(), carry.count);
            }
        }
    }
    finish(world, res, entity);
}

impl RuntimeSystem for JobExecuteSystem {
    fn name(&self) -> &str {
        "JobExecuteSystem"
    }

    fn priority(&self) -> u32 {
        JOB_EXECUTE_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.colony else {
            return;
        };
        let mut rows: Vec<(AgentId, Entity, Job)> = world
            .query::<(&Agent, &Job)>()
            .iter()
            .map(|(entity, (agent, job))| (agent.id, entity, *job))
            .collect();
        rows.sort_by_key(|row| row.0);
        let pot_sources = super::pot_supply::SourceIndex::default();
        for (id, entity, job) in rows {
            step(world, resources, &rule, id, entity, job, &pot_sources);
        }
        #[cfg(debug_assertions)]
        if let Err(problem) = super::invariants::check(world, resources) {
            panic!(
                "colony invariant broken at tick {}: {problem}",
                resources.current_tick
            );
        }
    }
}

#[cfg(test)]
mod tree_identity_tests {
    use super::*;
    #[test]
    fn invalid_tree_identity_cannot_destroy_tree_or_credit_logs() {
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.colony
            .trees
            .insert((2, 2), sim_core::colony::Tree { logs: 4 });
        assert!(!fell(&mut res, (2, 2)));
        assert_eq!(res.colony.trees[&(2, 2)].logs, 4);
        assert!(res.colony.ground_items.is_empty());
        res.content_state.tree_material_default =
            Some(sim_core::material::MaterialId::from_str_hash("unknown"));
        assert!(!fell(&mut res, (2, 2)));
        assert!(res.colony.trees.contains_key(&(2, 2)));
        res.content_state.tree_material_default = Some(OAK);
        assert!(!fell(&mut res, (2, 2)), "missing saved stats");
        assert!(res.colony.trees.contains_key(&(2, 2)));
    }
}

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
use sim_core::colony::skills::{scaled_ticks, AXE_PERMILLE};
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

/// The state a worker in `stage` of `kind` must be in.
fn expected_state(kind: JobKind, stage: JobStage) -> AgentState {
    match (kind, stage) {
        (JobKind::Build { .. }, JobStage::Working { .. }) => BUILD,
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
fn target_exists(world: &World, res: &SimResources, job: &Job) -> bool {
    let colony = &res.colony;
    let stack = |tile: (u32, u32)| {
        colony
            .ground_items
            .get(&tile)
            .is_some_and(|stack| stack.kind == job.item && stack.count > 0)
    };
    match (job.kind, job.stage) {
        (JobKind::Chop { tree }, _) => {
            colony.trees.contains_key(&tree) && colony.designations.contains_key(&tree)
        }
        (JobKind::Haul { from, to }, JobStage::ToSource) => {
            stack(from) && colony.zone_of_tile.contains_key(&to)
        }
        (JobKind::Haul { to, .. }, _) => colony.zone_of_tile.contains_key(&to),
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
        (JobKind::Supply { from, workshop, .. }, JobStage::ToSource) => {
            stack(from) && colony.workshops.contains_key(&workshop)
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
) {
    let tick = res.current_tick;
    let state = state_of(world, entity);
    if state != expected_state(job.kind, job.stage) {
        let built =
            matches!(job.kind, JobKind::Build { site } if site_needs(world, site).is_none());
        if built && matches!(job.stage, JobStage::Working { .. }) {
            finish_in_place(world, res, entity);
        } else {
            release_job(world, res, entity, CancelReason::Interrupted);
        }
        return;
    }
    if !target_exists(world, res, &job) {
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
                | JobKind::Quarry { node: near }
                | JobKind::Gather { spot: near, .. } => chebyshev(pos, near) <= 1,
                _ => res.nav.arrived(pos, source),
            };
            if !reached {
                walk(world, res, id, entity, &mut job, pos, source);
                return;
            }
            match job.kind {
                JobKind::Chop { .. } => {
                    let ticks = work_ticks(world, rule, entity, rule.chop_ticks, SkillKind::Chop);
                    job.amount = ticks.min(u32::from(u16::MAX)) as u16;
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Quarry { .. } => {
                    let base = rule.economy.map_or(120, |e| e.quarry_ticks);
                    let ticks = work_ticks(world, rule, entity, base, SkillKind::Quarry);
                    job.amount = ticks.min(u32::from(u16::MAX)) as u16;
                    job.stage = JobStage::Working { ticks_left: ticks };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Build { .. } => {
                    job.stage = JobStage::Working { ticks_left: 0 };
                    set_state(world, entity, BUILD);
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
                JobKind::Haul { from, .. }
                | JobKind::Deliver { from, .. }
                | JobKind::Supply { from, .. } => {
                    let (taken, spec) = take_items(res, from, job.item, job.amount);
                    res.colony.reservations.remove(&ReserveKey::Stack(from));
                    if taken == 0 {
                        cancel(world, res, entity, CancelReason::TargetGone);
                        return;
                    }
                    job.amount = taken;
                    job.stage = JobStage::ToDest;
                    let dest = job.kind.dest().unwrap_or(pos);
                    job.best_distance = chebyshev(pos, dest);
                    job.progress_tick = tick;
                    let _ = world.insert(entity, (Carry::of(spec, taken), SeekTarget::new(dest)));
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
            if matches!(job.kind, JobKind::Build { .. }) {
                // A builder: ConstructionSystem does the work; it trains Build (M2).
                if rule.economy.is_some() {
                    craft::train(world, entity, SkillKind::Build, 1);
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
                JobKind::Chop { tree } => {
                    fell(res, tree);
                    if rule.economy.is_some() {
                        craft::train(world, entity, SkillKind::Chop, u32::from(job.amount));
                        use_axe(world, res, rule, entity, pos);
                    }
                }
                JobKind::Quarry { node } => {
                    quarry_batch(res, rule, node, pos);
                    craft::train(world, entity, SkillKind::Quarry, u32::from(job.amount));
                }
                JobKind::Craft { .. } => craft::finish_craft(world, res, id, entity, &job),
                JobKind::Gather { .. } => {
                    if gather_batch(world, res, rule, entity, &mut job, pos) {
                        put_job(world, entity, job);
                        return;
                    }
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
    let axe = (skill == SkillKind::Chop)
        .then(|| world.get::<&Tool>(entity).ok().map(|t| t.item))
        .flatten()
        .filter(|item| item.kind == ItemKind::StoneAxe);
    match axe {
        Some(item) => {
            let factor = AXE_PERMILLE[usize::from(quality_bucket(item.quality).min(10))];
            ((u64::from(ticks) * u64::from(factor) + 500) / 1000).max(1) as u32
        }
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
    let (taken, spec) = take_items(res, axe, ItemKind::StoneAxe, 1);
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
/// one stone at the worker's tile (ledger: an axe consumed, a stone created).
fn use_axe(
    world: &mut World,
    res: &mut SimResources,
    rule: &ColonyRule,
    entity: Entity,
    pos: (u32, u32),
) {
    let Ok(mut tool) = world.get::<&mut Tool>(entity).map(|t| *t) else {
        return;
    };
    if tool.item.kind != ItemKind::StoneAxe {
        return;
    }
    tool.uses += 1;
    res.colony.tallies.chops_with_axe += 1;
    let limit = rule.economy.map_or(0, |e| e.axe_uses);
    if limit > 0 && tool.uses >= limit {
        let _ = world.remove_one::<Tool>(entity);
        res.colony.item_ledger.consume(ItemKind::StoneAxe, 1);
        res.colony.item_ledger.create(ItemKind::Stone, 1);
        put_items(
            res,
            pos,
            ItemSpec::raw(ItemKind::Stone, tool.item.material),
            1,
        );
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
fn fell(res: &mut SimResources, tree: (u32, u32)) {
    let Some(felled) = res.colony.trees.remove(&tree) else {
        return;
    };
    res.colony.designations.remove(&tree);
    res.colony.touch(true);
    res.colony.tallies.trees_felled += 1;
    let logs = u16::from(felled.logs);
    res.colony
        .item_ledger
        .create(ItemKind::Log, u64::from(logs));
    put_items(res, tree, ItemSpec::raw(ItemKind::Log, Some(OAK)), logs);
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
    let spec = ItemSpec::raw(ItemKind::Firewood, Some(OAK));
    let held = world.get::<&Carry>(entity).map_or(0, |c| c.count);
    let total = held + u16::from(take);
    if take > 0 {
        res.colony
            .item_ledger
            .create(ItemKind::Firewood, u64::from(take));
        let _ = world.insert_one(entity, Carry::of(spec, total));
    }
    if total == 0 {
        return false;
    }
    let more = res.wood_tiles.get(&spot).is_some_and(|u| *u > 0);
    if more && total < rule.carry(ItemKind::Firewood) {
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
        for (id, entity, job) in rows {
            step(world, resources, &rule, id, entity, job);
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

//! JobAssignSystem: gives available colonists a job — Build (deliver, then build) > Chop > Haul.
//!
//! Priority: 124 (`JOB_ASSIGN_PRIORITY`). Interval: 1 tick. Gate: `WorldRules.colony`.
//! Reads: `Agent`, `Position`, `AgentState`, `Job`, `Child`, `PorterTrip`; `SimResources.colony` (trees, designations, ground items, zones, reservations, footprints), `combat_pairs`, `rules.colony`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`)
//! Writes: `Job`, `SeekTarget`, `AgentState` (`Seeking { Job }`); `SimResources.colony.reservations`, `colony.tallies`
//! Events: none
//!
//! An agent is available when it is an adult or elder (no `Child`), not on a
//! porter trip, holds no `Job`, is not in a combat pair, and is `Idle` or at
//! leisure (`Seeking`/`Consuming { Activity }`): jobs replace leisure and idle
//! time, never survival work (ADR 0013). Agents are served in ascending
//! `AgentId`; each takes the first work type with a target within
//! `job_scan_radius` (Chebyshev), nearest first, ties by `(y, x)` (goal 7.2).
//! Reservations are written at once, so a later agent this tick sees them.
//! A worker leaving leisure keeps its `Activity` component for one tick:
//! the routine's own cleanup at 125 records it as interrupted.
use hecs::{Entity, World};
use sim_core::colony::{ItemKind, Job, JobKind, JobStage, ReserveKey};
use sim_core::components::{
    Agent, AgentId, AgentState, Child, PorterTrip, Position, SeekTarget, TargetKind,
};
use sim_core::rules::ColonyRule;
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeMap;

use super::cancel::reserve;
use super::items::{chebyshev, colony_sites, room_on, SiteView};

/// Priority of [`JobAssignSystem`].
pub const JOB_ASSIGN_PRIORITY: u32 = 124;

/// See the module header.
#[derive(Debug, Default)]
pub struct JobAssignSystem;

impl JobAssignSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

/// Logs on their way to each site (sum of `Deliver` jobs' amounts), by site centre.
pub fn in_transit(world: &World) -> BTreeMap<(u32, u32), u16> {
    let mut out = BTreeMap::new();
    for (_, job) in world.query::<&Job>().iter() {
        if let JobKind::Deliver { site, .. } = job.kind {
            *out.entry(site).or_insert(0) += job.amount;
        }
    }
    out
}

fn nearest<T: Copy>(
    from: (u32, u32),
    radius: u32,
    items: impl Iterator<Item = ((u32, u32), T)>,
) -> Option<((u32, u32), T)> {
    items
        .filter(|(tile, _)| chebyshev(from, *tile) <= radius)
        .min_by_key(|(tile, _)| (chebyshev(from, *tile), tile.1, tile.0))
}

/// The first job (in work-type order) for an agent at `pos`, or `None`.
fn pick(
    res: &SimResources,
    rule: &ColonyRule,
    pos: (u32, u32),
    sites: &BTreeMap<(u32, u32), SiteView>,
    transit: &BTreeMap<(u32, u32), u16>,
) -> Option<(JobKind, u16)> {
    let colony = &res.colony;
    let radius = rule.job_scan_radius;
    let free_stack =
        |tile: &(u32, u32)| !colony.reservations.contains_key(&ReserveKey::Stack(*tile));
    let logs = || {
        colony
            .ground_items
            .iter()
            .filter(|(tile, stack)| {
                stack.kind == ItemKind::Log && stack.count > 0 && free_stack(tile)
            })
            .map(|(tile, stack)| (*tile, stack.count))
    };
    // Build, first half: deliver logs to the nearest site still short of them.
    let short = sites.iter().filter_map(|(centre, view)| {
        let coming = view.needs.delivered + transit.get(centre).copied().unwrap_or(0);
        (coming < view.needs.needed).then_some((*centre, view.needs.needed - coming))
    });
    if let Some((site, missing)) = nearest(pos, radius, short) {
        let stocked = nearest(
            pos,
            radius,
            logs().filter(|(tile, _)| colony.zone_of_tile.contains_key(tile)),
        );
        let source = stocked.or_else(|| nearest(pos, radius, logs()));
        if let Some((from, count)) = source {
            let amount = rule.carry_logs.min(count).min(missing);
            return Some((JobKind::Deliver { from, site }, amount));
        }
    }
    // Build, second half: build a site whose logs are all there.
    let ready = sites.iter().filter_map(|(centre, view)| {
        (view.needs.delivered >= view.needs.needed
            && view.progress < view.required
            && !colony.reservations.contains_key(&ReserveKey::Site(*centre)))
        .then_some((*centre, ()))
    });
    if let Some((site, ())) = nearest(pos, radius, ready) {
        return Some((JobKind::Build { site }, 0));
    }
    // Chop: the nearest marked tree no one has taken.
    let trees = colony.designations.keys().filter_map(|tile| {
        (colony.trees.contains_key(tile)
            && !colony.reservations.contains_key(&ReserveKey::Tree(*tile)))
        .then_some((*tile, ()))
    });
    if let Some((tree, ())) = nearest(pos, radius, trees) {
        return Some((JobKind::Chop { tree }, 0));
    }
    // Haul: the nearest loose stack, to the stockpile cell nearest that stack.
    let loose = logs().filter(|(tile, _)| !colony.zone_of_tile.contains_key(tile));
    if let Some((from, count)) = nearest(pos, radius, loose) {
        let cells = colony.zone_of_tile.keys().filter_map(|cell| {
            let room = room_on(res, *cell, ItemKind::Log, rule.stack_max_logs);
            (room > 0 && !colony.reservations.contains_key(&ReserveKey::Cell(*cell)))
                .then_some((*cell, room))
        });
        if let Some((to, room)) = nearest(from, u32::MAX, cells) {
            let amount = rule.carry_logs.min(count).min(room);
            return Some((JobKind::Haul { from, to }, amount));
        }
    }
    None
}

impl RuntimeSystem for JobAssignSystem {
    fn name(&self) -> &str {
        "JobAssignSystem"
    }

    fn priority(&self) -> u32 {
        JOB_ASSIGN_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.colony else {
            return;
        };
        let tick = resources.current_tick;
        let in_combat = |id: AgentId| {
            resources
                .combat_pairs
                .iter()
                .any(|&(a, b)| a == id || b == id)
        };
        let mut available: Vec<(AgentId, Entity, (u32, u32))> = world
            .query::<(&Agent, &Position, &AgentState)>()
            .without::<&Job>()
            .without::<&Child>()
            .without::<&PorterTrip>()
            .iter()
            .filter(|(_, (agent, _, state))| {
                matches!(
                    state,
                    AgentState::Idle
                        | AgentState::Seeking {
                            target: TargetKind::Activity
                        }
                        | AgentState::Consuming {
                            target: TargetKind::Activity
                        }
                ) && !in_combat(agent.id)
            })
            .map(|(entity, (agent, pos, _))| (agent.id, entity, (pos.x, pos.y)))
            .collect();
        if available.is_empty() {
            return;
        }
        available.sort_by_key(|row| row.0);
        let sites = colony_sites(world);
        let mut transit = in_transit(world);
        let mut taken: Vec<(Entity, Job)> = Vec::new();
        for (id, entity, pos) in available {
            let Some((kind, amount)) = pick(resources, &rule, pos, &sites, &transit) else {
                continue;
            };
            let job = Job {
                kind,
                stage: JobStage::ToSource,
                taken_at: tick,
                amount,
                best_distance: chebyshev(pos, kind.source()),
                progress_tick: tick,
            };
            for key in job.keys() {
                reserve(resources, key, id);
            }
            if let JobKind::Deliver { site, .. } = kind {
                *transit.entry(site).or_insert(0) += amount;
            }
            resources.colony.tallies.jobs_taken += 1;
            taken.push((entity, job));
        }
        for (entity, job) in taken {
            let _ = world.insert(entity, (job, SeekTarget::new(job.kind.source())));
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = AgentState::Seeking {
                    target: TargetKind::Job,
                };
            }
        }
    }
}

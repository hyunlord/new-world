//! JobAssignSystem: gives available colonists a job — Build (deliver, then build) > Chop > Haul.
//!
//! Priority: 124 (`JOB_ASSIGN_PRIORITY`). Interval: 1 tick. Gate: `WorldRules.colony`.
//! Reads: `Agent`, `Position`, `AgentState`, `Job`, `Child`, `PorterTrip`; `SimResources.colony` (trees, designations, ground items, zones, reservations, footprints), `combat_pairs`, `rules.colony`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`)
//! Writes: `Job`, `SeekTarget`, `AgentState` (`Seeking { Job }`); `SimResources.colony.reservations`, `colony.tallies`, `spatial.colony` (rebuilt when the overlay changed)
//! Events: none
//!
//! An agent is available when it is an adult or elder (no `Child`), not on a
//! porter trip, holds no `Job`, is not in a combat pair, and is `Idle` or at
//! leisure (`Seeking`/`Consuming { Activity }`): jobs replace leisure and idle
//! time, never survival work (ADR 0013). Agents are served in ascending
//! `AgentId`; each takes the first work type with a target within
//! `job_scan_radius` (Chebyshev), nearest first, ties by `(y, x)` (goal 7.2),
//! found through the spatial index (`SimResources.spatial`, M1.5 goal 6.2).
//! Reservations are written at once, so a later agent this tick sees them.
//! A worker leaving leisure keeps its `Activity` component for one tick:
//! the routine's own cleanup at 125 records it as interrupted.
use hecs::{Entity, World};
use sim_core::colony::{ItemKind, Job, JobKind, JobStage, ReserveKey};
use sim_core::components::{
    Agent, AgentId, AgentState, Child, PorterTrip, Position, SeekTarget, TargetKind,
};
use sim_core::rules::ColonyRule;
use sim_core::spatial::ChunkIndex;
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

/// Searches that found no candidate anywhere on the map this tick. Within one
/// `JobAssignSystem` tick the candidates only shrink (each assignment adds
/// reservations and logs in transit), so an empty search stays empty for
/// every later agent of the tick and is skipped (M1.5 DECISIONS D15).
#[derive(Debug, Default, Clone, Copy)]
struct Exhausted {
    short_site: bool,
    stocked_logs: bool,
    any_logs: bool,
    ready_site: bool,
    marked_tree: bool,
    loose_logs: bool,
    open_cell: bool,
}

/// `index.nearest_where(pos, radius, pred)`, unless `done` says nothing
/// passes `pred` anywhere; sets `done` when a map-wide search finds nothing.
fn search<T: Copy + Ord>(
    index: &ChunkIndex<T>,
    pos: (u32, u32),
    radius: u32,
    done: &mut bool,
    mut pred: impl FnMut((u32, u32), &T) -> bool,
) -> Option<((u32, u32), T)> {
    if *done {
        return None;
    }
    let found = index.nearest_where(pos, radius, &mut pred);
    if found.is_none()
        && (radius == u32::MAX || index.nearest_where(pos, u32::MAX, &mut pred).is_none())
    {
        *done = true;
    }
    found
}

/// The first job (in work-type order) for an agent at `pos`, or `None`.
///
/// M1.5 (goal 6.2): every candidate search goes through the spatial index —
/// `ChunkIndex::nearest_where` returns the same tile the M1 linear
/// `min_by_key((Chebyshev distance, y, x))` over the filtered candidates
/// returned (ties `(y, x)`), so assignments are unchanged (the M1 fixtures
/// are byte-identical before and after, DECISIONS D11).
fn pick(
    res: &SimResources,
    rule: &ColonyRule,
    pos: (u32, u32),
    sites: &BTreeMap<(u32, u32), SiteView>,
    site_index: &ChunkIndex<()>,
    transit: &BTreeMap<(u32, u32), u16>,
    ex: &mut Exhausted,
) -> Option<(JobKind, u16)> {
    let colony = &res.colony;
    let index = &res.spatial.colony;
    let radius = rule.job_scan_radius;
    let free_stack =
        |tile: &(u32, u32)| !colony.reservations.contains_key(&ReserveKey::Stack(*tile));
    // A loose or stocked log stack someone may take (`zoned` = in a stockpile zone).
    let log_stack = |tile: (u32, u32), zoned: Option<bool>| {
        colony.ground_items.get(&tile).is_some_and(|stack| {
            stack.kind == ItemKind::Log
                && stack.count > 0
                && free_stack(&tile)
                && zoned.is_none_or(|z| colony.zone_of_tile.contains_key(&tile) == z)
        })
    };
    let count = |tile: (u32, u32)| colony.ground_items.get(&tile).map_or(0, |s| s.count);
    // Build, first half: deliver logs to the nearest site still short of them.
    let missing = |centre: &(u32, u32)| {
        sites.get(centre).and_then(|view| {
            let coming = view.needs.delivered + transit.get(centre).copied().unwrap_or(0);
            (coming < view.needs.needed).then_some(view.needs.needed - coming)
        })
    };
    let short = search(site_index, pos, radius, &mut ex.short_site, |c, _| {
        missing(&c).is_some()
    });
    if let Some((site, ())) = short {
        let missing = missing(&site).unwrap_or(0);
        let stocked = search(&index.ground, pos, radius, &mut ex.stocked_logs, |t, _| {
            log_stack(t, Some(true))
        });
        let source = stocked.or_else(|| {
            search(&index.ground, pos, radius, &mut ex.any_logs, |t, _| {
                log_stack(t, None)
            })
        });
        if let Some((from, ())) = source {
            let amount = rule.carry_logs.min(count(from)).min(missing);
            return Some((JobKind::Deliver { from, site }, amount));
        }
    }
    // Build, second half: build a site whose logs are all there.
    let ready = |centre: (u32, u32)| {
        sites.get(&centre).is_some_and(|view| {
            view.needs.delivered >= view.needs.needed
                && view.progress < view.required
                && !colony.reservations.contains_key(&ReserveKey::Site(centre))
        })
    };
    if let Some((site, ())) = search(site_index, pos, radius, &mut ex.ready_site, |c, _| ready(c)) {
        return Some((JobKind::Build { site }, 0));
    }
    // Chop: the nearest marked tree no one has taken.
    let choppable = |tile: (u32, u32)| {
        colony.trees.contains_key(&tile)
            && !colony.reservations.contains_key(&ReserveKey::Tree(tile))
    };
    if let Some((tree, ())) = search(&index.marked, pos, radius, &mut ex.marked_tree, |t, _| {
        choppable(t)
    }) {
        return Some((JobKind::Chop { tree }, 0));
    }
    // Haul: the nearest loose stack, to the stockpile cell nearest that stack.
    if ex.open_cell {
        return None;
    }
    let loose = search(&index.ground, pos, radius, &mut ex.loose_logs, |t, _| {
        log_stack(t, Some(false))
    });
    if let Some((from, ())) = loose {
        let room = |cell: (u32, u32)| room_on(res, cell, ItemKind::Log, rule.stack_max_logs);
        let open = |cell: (u32, u32)| {
            room(cell) > 0 && !colony.reservations.contains_key(&ReserveKey::Cell(cell))
        };
        if let Some((to, ())) = search(&index.cells, from, u32::MAX, &mut ex.open_cell, |c, _| {
            open(c)
        }) {
            let amount = rule.carry_logs.min(count(from)).min(room(to));
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
        resources.spatial.colony.refresh(&resources.colony);
        let sites = colony_sites(world);
        let mut site_index = ChunkIndex::new();
        for centre in sites.keys() {
            site_index.insert(*centre, ());
        }
        let mut transit = in_transit(world);
        let mut taken: Vec<(Entity, Job)> = Vec::new();
        let mut exhausted = Exhausted::default();
        for (id, entity, pos) in available {
            let Some((kind, amount)) = pick(
                resources,
                &rule,
                pos,
                &sites,
                &site_index,
                &transit,
                &mut exhausted,
            ) else {
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

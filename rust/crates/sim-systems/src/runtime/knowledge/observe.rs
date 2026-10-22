//! ObservationSystem: a person working near a practitioner at gated work picks a little of it up by watching.
//!
//! Priority: 161 (`OBSERVATION_PRIORITY`). Interval: 30 ticks (fixed; `KnowledgeRule::observation_interval` must match — a test checks). Gate: `ColonyRule::knowledge`.
//! Reads: `Agent`, `Position`, `AgentState`, `Job`, `Knowledge`; `SimResources.colony` (workshops and bills, sites), `rules`
//! Writes: `Knowledge` (`observed`, `Source::Observed` entries); `SimResources.colony.tallies` (observed)
//! Events: none
//!
//! M4 goal 3.3 (droppable 7.4; ADR 0025). The practitioners at work are the
//! people whose job is in its `Working` stage and is gated (a craft by its
//! recipe, a build by its site's blueprint, a quarry or a dig by its
//! designation) and who practise its gate. Every other person at a job of any
//! kind (walking or working) within `observation_radius` (Chebyshev) of one
//! gains `observation_gain` ‰ of each gate node they do not practise, never
//! past the threshold − 50 (watching alone never makes a practitioner).
//! Watchers in `AgentId` order; each gains at most once per check per node.
use hecs::World;
use sim_core::colony::knowledge::{defs, gates, ids_of};
use sim_core::colony::{Designation, Job, JobKind, JobStage};
use sim_core::components::{Agent, Position};
use sim_engine::{RuntimeSystem, SimResources};

use super::{knowledge_of, practice_of, rule, store};

/// Priority of [`ObservationSystem`].
pub const OBSERVATION_PRIORITY: u32 = 161;

/// See the module header.
#[derive(Debug, Default)]
pub struct ObservationSystem;

/// The gate mask of `job` (0 for ungated work).
pub fn job_gate(world: &World, res: &SimResources, job: &Job) -> u32 {
    let g = gates();
    match job.kind {
        JobKind::Craft { workshop, bill, .. } => res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|s| s.bill(bill))
            .map_or(0, |b| g.recipe(b.recipe)),
        JobKind::Build { site } => crate::runtime::jobs::items::colony_sites(world)
            .get(&site)
            .map_or(0, |v| g.build(v.site.kind)),
        JobKind::Quarry { .. } => g.designation(Designation::Quarry),
        JobKind::Dig { .. } => g.designation(Designation::Dig),
        JobKind::DigClay { .. } => g.designation(Designation::ClayPit),
        _ => 0,
    }
}

impl RuntimeSystem for ObservationSystem {
    fn name(&self) -> &str {
        "ObservationSystem"
    }

    fn priority(&self) -> u32 {
        OBSERVATION_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        30
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = rule(resources) else {
            return;
        };
        let tick = resources.current_tick;
        // Every worker: (id, entity, tile, job working, gate).
        let mut workers: Vec<(u64, hecs::Entity, (u32, u32), bool)> = world
            .query::<(&Agent, &Position, &Job)>()
            .iter()
            .map(|(e, (a, p, j))| {
                (
                    a.id,
                    e,
                    (p.x, p.y),
                    matches!(j.stage, JobStage::Working { .. }),
                )
            })
            .collect();
        if workers.len() < 2 {
            return;
        }
        workers.sort_by_key(|w| w.0);
        // Practitioners at gated work: (tile, gate).
        let mut shown: Vec<((u32, u32), u32)> = Vec::new();
        for (_, entity, tile, working) in &workers {
            if !working {
                continue;
            }
            let job = world.get::<&Job>(*entity).map(|j| *j).ok();
            let gate = job.map_or(0, |j| job_gate(world, resources, &j));
            if gate != 0 && practice_of(world, *entity) & gate == gate {
                shown.push((*tile, gate));
            }
        }
        if shown.is_empty() {
            return;
        }
        let d = defs();
        let mut gained = 0u64;
        for (_, entity, tile, _) in &workers {
            let mut mask = 0u32;
            for (at, gate) in &shown {
                if at != tile
                    && crate::runtime::jobs::items::chebyshev(*at, *tile) <= rule.observation_radius
                {
                    mask |= gate;
                }
            }
            let mut k = knowledge_of(world, *entity);
            mask &= !k.practice();
            if mask == 0 {
                continue;
            }
            let mut changed = false;
            for id in ids_of(mask) {
                let cap = d.threshold(id).saturating_sub(50);
                if k.observe(id, rule.observation_gain, cap, tick) {
                    changed = true;
                    gained += 1;
                }
            }
            if changed {
                store(world, *entity, k);
            }
        }
        resources.colony.tallies.observed += gained;
    }
}

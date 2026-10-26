//! The decision seam (goal 4.3, ADR 0013): the only job code the decision
//! system calls. `decision/system.rs` gains about 25 lines that call these
//! three functions, plus the compile-forced `TargetKind::Job` arms.
//!
//! A job holder (an agent with a `Job` walking or working for it —
//! `Seeking { Job }` / `Consuming { Job }` — or a builder in
//! `Consuming { ConstructionSite }`) runs the decision's Idle cascade as a
//! temporary `Idle`, with leisure (the routine arm) skipped for it. After the
//! loop, `settle_after_decision` compares the choice:
//! - the decision chose nothing (idle wandering, the meal wait) — or chose to
//!   build the colony site the worker stands on — so the worker keeps its job
//!   and its previous state is restored;
//! - anything else (food, water, sleep, foraging, firewood, a V7 site,
//!   social, fire, flight) — the job is released (`CancelReason::Needs`) and
//!   the decision's choice stands; the job's `SeekTarget` is dropped so the
//!   post-pass re-resolves the new goal. The release is counted (measurement
//!   only) in `colony.release_counts` by need, job kind, stage and need value.
//!
//! M1.5 (goal 8.2, review MAJOR-1): under `ColonyRule::yield_margin_permille`
//! and `finish_leg_tiles` (the colony's `ColonyRule::M1_5`) only a worker that
//! may yield runs the cascade at all. It may yield with an urgent need — hunger
//! or thirst at or past its seek threshold plus the margin, fatigue at or past
//! its own threshold plus the margin — unless it carries a load within reach of
//! its destination; with a critical hunger or thirst (`critical_preemption`, 75)
//! it may yield wherever it is. Every other worker skips the decision this tick
//! (`skips_decision`): it keeps its job and its state, and no arm runs, so no
//! causal event, memory, trip tally or claim is left behind by a choice that
//! would be undone. M1's `ColonyRule::M1` (`None`) keeps M1: every holder runs it.
//! M2: a craft at work yields only to a critical need (DECISIONS D13).

use hecs::{Entity, World};
use sim_core::colony::{CancelReason, Job, JobReleaseKey, JobStage};
use sim_core::components::{AgentState, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_engine::SimResources;
use std::collections::BTreeSet;

use super::cancel::release_job;

/// The measurement key of a release to `choice` (see `JobReleaseKey`).
fn release_key(world: &World, entity: Entity, choice: AgentState) -> JobReleaseKey {
    #[cfg(feature = "perf_arms")]
    let _perf_arm_0 = crate::perf_arms::Scope::start("seam.release_key");
    let job = world.get::<&Job>(entity).ok().map(|job| *job);
    // `JobKind::code` − 1: 0 chop, 1 haul, 2 deliver, 3 build (and no job), 4 quarry, …
    let kind = job.map_or(3, |j| j.kind.code().saturating_sub(1));
    let stage = match job.map(|j| j.stage) {
        Some(JobStage::ToSource) | None => 0,
        Some(JobStage::Working { .. }) => 1,
        Some(JobStage::ToDest) => 2,
    };
    let bin = |v: f64| (v.clamp(0.0, 100.0) / 10.0) as u8;
    let (need, value) = match choice.target() {
        Some(TargetKind::Food) => (
            0,
            world
                .get::<&Hunger>(entity)
                .ok()
                .map(|h| f64::from(h.value)),
        ),
        Some(TargetKind::Water) => (1, world.get::<&Thirst>(entity).ok().map(|t| t.value)),
        Some(TargetKind::Sleep) => (2, world.get::<&Sleep>(entity).ok().map(|s| s.fatigue)),
        Some(TargetKind::Agent(_)) => (3, None),
        Some(TargetKind::GatherFood) => (4, None),
        Some(TargetKind::GatherWood) => (5, None),
        _ => (6, None),
    };
    (need, kind, stage, value.map_or(255, bin))
}

/// The job holders of a tick (sorted sets: membership only): every entity with
/// a `Job`, and those of them that may not yield this tick (`inert`).
#[derive(Debug, Default, Clone)]
pub struct JobHolders {
    all: BTreeSet<Entity>,
    inert: BTreeSet<Entity>,
}

impl JobHolders {
    /// Whether `entity` holds a `Job` (membership; M1's set semantics).
    pub fn contains(&self, entity: &Entity) -> bool {
        self.all.contains(entity)
    }
}

/// A need value in tenths of a point (the 0–100 scales), so a permille margin
/// of the scale is a whole number of tenths.
fn tenths(value: f64) -> i64 {
    (value * 10.0) as i64
}

/// Whether the worker `entity` may yield its job this tick (see the module header).
fn may_yield(world: &World, res: &SimResources, entity: Entity) -> bool {
    #[cfg(feature = "perf_arms")]
    let _perf_arm_1 = crate::perf_arms::Scope::start("seam.may_yield");
    let Some(rule) = res.rules.colony else {
        #[cfg(feature = "perf_arms")]
        let _perf_arm_2 = crate::perf_arms::Scope::start("seam.no_colony");
        return true;
    };
    // M2 (DECISIONS D13): a craft at work finishes before yielding to an
    // urgent need (its inputs were consumed when it started); only a critical
    // hunger or thirst takes the crafter away, whatever the margin rule.
    let crafting = rule.economy.is_some()
        && world.get::<&Job>(entity).is_ok_and(|j| {
            matches!(j.kind, sim_core::colony::JobKind::Craft { .. })
                && matches!(j.stage, JobStage::Working { .. })
        });
    let Some(margin) = rule.yield_margin_permille.or(crafting.then_some(0)) else {
        #[cfg(feature = "perf_arms")]
        let _perf_arm_3 = crate::perf_arms::Scope::start("seam.no_margin");
        return true;
    };
    use crate::runtime::agent_actions::needs as n;
    let margin = i64::from(margin);
    let hunger = world
        .get::<&Hunger>(entity)
        .map_or(0, |h| tenths(f64::from(h.value)));
    let thirst = world.get::<&Thirst>(entity).map_or(0, |t| tenths(t.value));
    let critical = tenths(
        res.rules
            .agent
            .critical_preemption
            .unwrap_or(n::CRITICAL_NEED),
    );
    if arm_eval!(
        "eval.seam.critical",
        hunger >= critical || thirst >= critical
    ) {
        #[cfg(feature = "perf_arms")]
        let _perf_arm_4 = crate::perf_arms::Scope::start("seam.critical");
        return true;
    }
    if arm_eval!("eval.seam.crafting", crafting) {
        #[cfg(feature = "perf_arms")]
        let _perf_arm_5 = crate::perf_arms::Scope::start("seam.craft_locked");
        return false;
    }
    let job = world.get::<&Job>(entity).ok().map(|job| *job);
    if let (Some(limit), Some(job), Ok(pos)) =
        (rule.finish_leg_tiles, job, world.get::<&Position>(entity))
    {
        let near = job
            .kind
            .dest()
            .is_some_and(|to| super::items::chebyshev((pos.x, pos.y), to) <= limit);
        if arm_eval!(
            "eval.seam.finish_leg",
            job.stage == JobStage::ToDest && near
        ) {
            #[cfg(feature = "perf_arms")]
            let _perf_arm_6 = crate::perf_arms::Scope::start("seam.finish_leg_locked");
            return false;
        }
    }
    let day_tick = (res.ticks_per_day != 0).then(|| {
        res.current_tick
            .saturating_add(res.rules.clock_offset_ticks)
            % res.ticks_per_day
    });
    let temperament = world
        .get::<&sim_core::components::Temperament>(entity)
        .ok()
        .map(|t| *t);
    let fatigue_threshold = match (res.rules.agent.temperament, temperament) {
        (Some(lean), Some(t)) => {
            n::person_night_rest(res.rules.agent.night_rest, Some(lean), Some(&t), day_tick)
                .map_or(n::FATIGUE_THRESHOLD, |r| r.fatigue_threshold)
        }
        _ => n::fatigue_threshold_at(res),
    };
    let fatigue = world.get::<&Sleep>(entity).map_or(0, |s| tenths(s.fatigue));
    arm_outcome!(
        "count.seam.urgent",
        "count.seam.nonurgent",
        hunger >= tenths(f64::from(n::HUNGER_THRESHOLD)) + margin
            || thirst >= tenths(n::THIRST_THRESHOLD) + margin
            || fatigue >= tenths(fatigue_threshold) + margin
    )
}

/// Every entity that holds a `Job`, and which of them may not yield this tick.
pub fn holders(world: &World, res: &SimResources) -> JobHolders {
    #[cfg(feature = "perf_arms")]
    let _perf_arm_7 = crate::perf_arms::Scope::start("seam.holders");
    let all: BTreeSet<Entity> = world
        .query::<&Job>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    let mut inert: BTreeSet<Entity> = all
        .iter()
        .copied()
        .filter(|&entity| !may_yield(world, res, entity))
        .collect();
    // M5 (ADR 0028): a drafted person takes only direct orders; the decision skips them.
    #[cfg(feature = "perf_arms")]
    let _perf_arm_8 = crate::perf_arms::Scope::start("seam.drafted_snapshot");
    #[cfg(feature = "perf_arms")]
    crate::perf_arms::count(
        "count.people.drafted",
        u64::try_from(
            world
                .query::<&sim_core::colony::people::Drafted>()
                .iter()
                .count(),
        )
        .unwrap_or(u64::MAX),
    );
    inert.extend(
        world
            .query::<&sim_core::colony::people::Drafted>()
            .iter()
            .map(|(entity, _)| entity),
    );
    #[cfg(feature = "perf_arms")]
    drop(_perf_arm_8);
    // M5.5 (ADR 0028 amendment): a fleeing person runs; the decision skips them as it skips the drafted.
    #[cfg(feature = "perf_arms")]
    let _perf_arm_9 = crate::perf_arms::Scope::start("seam.fleeing_snapshot");
    #[cfg(feature = "perf_arms")]
    crate::perf_arms::count(
        "count.people.fleeing",
        u64::try_from(
            world
                .query::<&sim_core::colony::people::Fleeing>()
                .iter()
                .count(),
        )
        .unwrap_or(u64::MAX),
    );
    inert.extend(
        world
            .query::<&sim_core::colony::people::Fleeing>()
            .iter()
            .map(|(entity, _)| entity),
    );
    #[cfg(feature = "perf_arms")]
    drop(_perf_arm_9);
    JobHolders { all, inert }
}

/// Whether the decision should run `entity` (in `state`) as a temporary Idle.
pub fn hold_for_decision(entity: Entity, state: AgentState, holders: &JobHolders) -> bool {
    #[cfg(feature = "perf_arms")]
    let _perf_arm_10 = crate::perf_arms::Scope::start("seam.hold_for_decision");
    match state {
        AgentState::Seeking {
            target: TargetKind::Job,
        }
        | AgentState::Consuming {
            target: TargetKind::Job,
        } => true,
        AgentState::Consuming {
            target: TargetKind::ConstructionSite,
        } => holders.all.contains(&entity),
        _ => false,
    }
}

/// Whether a held worker skips the decision this tick: it may not yield, so it
/// keeps its job and state and no arm runs for it (M1.5, review MAJOR-1); M5:
/// a drafted person (walking or holding as `Seeking`/`Consuming { Job }`) too.
pub fn skips_decision(entity: Entity, holders: &JobHolders) -> bool {
    #[cfg(feature = "perf_arms")]
    let _perf_arm_11 = crate::perf_arms::Scope::start("seam.skips_decision");
    holders.inert.contains(&entity)
}

/// Settle each held worker after the decision loop (see the module header).
/// `held` is in decision order (AgentId); structural changes follow it.
pub fn settle_after_decision(
    world: &mut World,
    res: &mut SimResources,
    held: &[(Entity, AgentState)],
    clear_seek: &mut Vec<Entity>,
) {
    #[cfg(feature = "perf_arms")]
    let _perf_arm_12 = crate::perf_arms::Scope::start("seam.settle");
    for &(entity, previous) in held {
        #[cfg(feature = "perf_arms")]
        let _perf_arm_13 = crate::perf_arms::Scope::start("seam.settle_person");
        if arm_eval!("eval.seam.stale", world.get::<&Job>(entity).is_err()) {
            #[cfg(feature = "perf_arms")]
            let _perf_arm_14 = crate::perf_arms::Scope::start("seam.stale_job");
            #[cfg(feature = "mind_shadow")]
            if let Ok(agent) = world.get::<&sim_core::components::Agent>(entity) {
                crate::mind_observer::settle(res.current_tick, agent.id, "stale_job", false);
            }
            // A stale job state without a job: the decision's choice stands.
            continue;
        }
        let now = world
            .get::<&AgentState>(entity)
            .map(|state| *state)
            .unwrap_or_default();
        let own_site = matches!(
            now,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            } | AgentState::Consuming {
                target: TargetKind::ConstructionSite
            }
        ) && world
            .get::<&Position>(entity)
            .is_ok_and(|pos| res.colony.occupied.contains_key(&(pos.x, pos.y)));
        if arm_eval!("eval.seam.restore", now == AgentState::Idle || own_site) {
            #[cfg(feature = "perf_arms")]
            let _perf_arm_15 = crate::perf_arms::Scope::start("seam.restore");
            #[cfg(feature = "mind_shadow")]
            if let Ok(agent) = world.get::<&sim_core::components::Agent>(entity) {
                crate::mind_observer::settle(res.current_tick, agent.id, "restore", true);
            }
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = previous;
            }
        } else {
            #[cfg(feature = "perf_arms")]
            let _perf_arm_16 = crate::perf_arms::Scope::start("seam.release");
            #[cfg(feature = "mind_shadow")]
            if let Ok(agent) = world.get::<&sim_core::components::Agent>(entity) {
                crate::mind_observer::settle(res.current_tick, agent.id, "release", false);
            }
            clear_seek.push(entity);
            let key = release_key(world, entity, now);
            *res.colony.release_counts.entry(key).or_insert(0) += 1;
            release_job(world, res, entity, CancelReason::Needs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::JobStage;
    use sim_core::rules::ColonyRule;

    fn worker(
        stage: JobStage,
        at: (u32, u32),
        hunger: f32,
        thirst: f64,
        fatigue: f64,
    ) -> (World, Entity) {
        let mut world = World::new();
        let job = Job {
            kind: sim_core::colony::JobKind::Haul {
                from: (0, 0),
                to: (30, 0),
            },
            stage,
            taken_at: 0,
            amount: 5,
            item: sim_core::colony::ItemKind::Log,
            material: None,
            fetch: None,
            best_distance: 0,
            progress_tick: 0,
        };
        let mut sleep = Sleep::new(0.0, 0.03);
        sleep.fatigue = fatigue;
        let e = world.spawn((
            job,
            Position::new(at.0, at.1),
            Hunger::new(hunger, 0.05),
            Thirst::new(thirst, 0.08),
            sleep,
        ));
        (world, e)
    }

    fn res(rule: ColonyRule) -> SimResources {
        let mut res = SimResources::new(64, 64, sim_core::MaterialRegistry::new());
        res.rules.colony = Some(rule);
        // 12:00 of the clock: outside the night-rest window, fatigue threshold 50.
        res.rules.agent.night_rest = sim_core::rules::WorldRules::baseline(0).agent.night_rest;
        res.rules.agent.critical_preemption = Some(75.0);
        res.current_tick = 720;
        res
    }

    fn inert(world: &World, res: &SimResources, e: Entity) -> bool {
        let h = holders(world, res);
        assert!(hold_for_decision(
            e,
            AgentState::Seeking {
                target: TargetKind::Job
            },
            &h
        ));
        skips_decision(e, &h)
    }

    #[test]
    fn only_an_urgent_or_critical_need_lets_a_worker_run_the_decision() {
        let m15 = res(ColonyRule::M1_5);
        // Thresholds 50, margin 150 ‰ of the scale = 15 points.
        for (h, t, f, runs) in [
            (10.0, 64.9, 10.0, false),
            (10.0, 65.0, 10.0, true),
            (64.9, 10.0, 10.0, false),
            (65.0, 10.0, 10.0, true),
            (10.0, 10.0, 64.9, false),
            (10.0, 10.0, 65.0, true),
        ] {
            let (w, e) = worker(JobStage::ToSource, (0, 0), h, t, f);
            assert_eq!(
                !inert(&w, &m15, e),
                runs,
                "hunger {h} thirst {t} fatigue {f}"
            );
        }
        // A load 12 tiles from its destination is delivered first even at an
        // urgent thirst, 13 tiles away it is not; a critical thirst (75) always runs.
        let (w, e) = worker(JobStage::ToDest, (18, 0), 10.0, 70.0, 10.0);
        assert!(inert(&w, &m15, e));
        let (w, e) = worker(JobStage::ToDest, (17, 0), 10.0, 70.0, 10.0);
        assert!(!inert(&w, &m15, e));
        let (w, e) = worker(JobStage::ToDest, (18, 0), 10.0, 75.0, 10.0);
        assert!(!inert(&w, &m15, e));
        // M1: every holder runs the decision.
        let (w, e) = worker(JobStage::ToDest, (18, 0), 10.0, 10.0, 10.0);
        assert!(!inert(&w, &res(ColonyRule::M1), e));
    }
}

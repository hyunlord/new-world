//! Apprenticeship (M4 goal 3.3, ADR 0025): `OrderTeach` orders and the paired
//! Teach / Learn jobs.
//!
//! An order names a teacher, a learner and a knowledge node. It is accepted
//! when the teacher practises the node, the learner is an adult or a child of
//! at least `learner_min_years` and is below the node's threshold. Each day up
//! to `sessions_per_day` sessions are held: [`assign`] (called by
//! `JobAssignSystem` before its rounds) sends partners only when a session
//! can happen: both have a Craft priority above 0 and both can walk to the
//! meeting tile — the work tile of the nearest workshop of a kind the node
//! gates, else the hearth. Either partner who is free takes its job and walks there; the
//! learner waits (`Working`); the teacher, once both have arrived, works
//! `session_ticks`, and at the end the learner gains
//! `session_gain × fidelity` ‰ (`Source::Apprenticed`) and the teacher uses the
//! node (and instruction). Fidelity is `fidelity` with instruction, else
//! `fidelity_untrained` (DECISIONS D8), × `master_permille` ‰ when the
//! teacher's proficiency is at least `master_at`. The order completes when the
//! learner practises the node (chronicle `KnowledgeLearned`, cause 4
//! apprenticed); it is cancelled when either dies or the teacher no longer
//! practises the node. A lesson job that waited more than [`WAIT_TICKS`] for
//! its partner without a session under way ends (DECISIONS D18).
use hecs::{Entity, World};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::knowledge::{
    bit, defs, gates, Gate, Knowledge, KnowledgeId, Source, TeachOrder,
};
use sim_core::colony::{
    BlueprintKind, CancelReason, CommandResult, Job, JobKind, JobStage, RejectReason,
    WorkPriorities, WorkType,
};
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, Life, PorterTrip, Position, TargetKind,
};
use sim_core::rules::KnowledgeRule;
use sim_engine::SimResources;
use std::collections::BTreeSet;

use super::cancel::{cancel, finish};
use super::items::chebyshev;

/// Ticks a teacher at the meeting tile waits for the learner before giving up.
pub const WAIT_TICKS: u64 = 240;

/// The entity of person `id` (a linear scan; apprenticeship paths are rare).
pub fn entity_of(world: &World, id: AgentId) -> Option<Entity> {
    world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == id)
        .map(|(e, _)| e)
}

fn knowledge(world: &World, e: Entity) -> Knowledge {
    crate::runtime::knowledge::knowledge_of(world, e)
}

/// Where a lesson of `node` is held: the work tile of the workshop nearest the
/// hearth of a kind the node gates (by recipe or workshop), else the hearth.
pub fn meeting_tile(res: &SimResources, node: KnowledgeId) -> Option<(u32, u32)> {
    let hearth = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied();
    let kinds: Vec<BlueprintKind> = defs()
        .get(node)
        .map(|n| {
            n.gates
                .iter()
                .filter_map(|g| match g {
                    Gate::Workshop(k) => Some(*k),
                    Gate::Recipe(r) => r.recipe().map(|r| r.workshop),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let from = hearth.unwrap_or((0, 0));
    res.colony
        .workshops
        .values()
        .filter(|s| kinds.contains(&s.kind))
        .map(|s| s.work_tile())
        .min_by_key(|t| (chebyshev(from, *t), t.1, t.0))
        .or(hearth)
}

/// Apply `OrderTeach` (see the module header).
pub fn order(
    world: &World,
    res: &mut SimResources,
    teacher: AgentId,
    learner: AgentId,
    node: KnowledgeId,
) -> CommandResult {
    let Some(rule) = crate::runtime::knowledge::rule(res) else {
        return CommandResult::rejected(RejectReason::NotYet);
    };
    if defs().get(node).is_none() {
        return CommandResult::rejected(RejectReason::Unknown);
    }
    let (Some(t), Some(l)) = (entity_of(world, teacher), entity_of(world, learner)) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    if teacher == learner {
        return CommandResult::rejected(RejectReason::Blocked);
    }
    if !knowledge(world, t).practises(node) {
        return CommandResult::rejected(RejectReason::NoPractitioner);
    }
    if knowledge(world, l).practises(node) {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    let year = res.rules.calendar.years_to_ticks(1).max(1);
    let years = world.get::<&Life>(l).map_or(0, |life| {
        u64::try_from(life.age_ticks(res.current_tick)).unwrap_or(0) / year
    });
    if world.get::<&Child>(l).is_ok() && years < rule.learner_min_years {
        return CommandResult::rejected(RejectReason::TooYoung);
    }
    if res
        .colony
        .teach_orders
        .iter()
        .any(|o| o.learner == learner && o.knowledge == node)
    {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    let id = res.colony.next_teach_id.max(1);
    res.colony.next_teach_id = id + 1;
    res.colony.teach_orders.push(TeachOrder {
        id,
        teacher,
        learner,
        knowledge: node,
        sessions: 0,
        day: 0,
        sessions_today: 0,
        placed: res.current_tick,
    });
    res.colony.touch(false);
    CommandResult::ok(id)
}

/// Remove order `id` and end its jobs (counted cancelled unless `done`).
fn close(world: &mut World, res: &mut SimResources, id: u32, done: bool) {
    let Some(i) = res.colony.teach_orders.iter().position(|o| o.id == id) else {
        return;
    };
    res.colony.teach_orders.remove(i);
    if done {
        res.colony.tallies.teach_done += 1;
    } else {
        res.colony.tallies.teach_cancelled += 1;
    }
    let mut holders: Vec<(AgentId, Entity)> = world
        .query::<(&Agent, &Job)>()
        .iter()
        .filter(|(_, (_, j))| {
            matches!(j.kind, JobKind::Teach { order, .. } | JobKind::Learn { order, .. } if order == id)
        })
        .map(|(e, (a, _))| (a.id, e))
        .collect();
    holders.sort_by_key(|h| h.0);
    for (_, e) in holders {
        if done {
            finish(world, res, e);
        } else {
            cancel(world, res, e, CancelReason::Command);
        }
    }
    res.colony.touch(false);
}

/// Apply `CancelTeach`.
pub fn cancel_order(world: &mut World, res: &mut SimResources, id: u32) -> CommandResult {
    if !res.colony.teach_orders.iter().any(|o| o.id == id) {
        return CommandResult::rejected(RejectReason::Unknown);
    }
    close(world, res, id, false);
    CommandResult::ok(1)
}

/// Cancel the orders whose teacher or learner is gone or whose teacher no
/// longer practises the node; complete those whose learner already practises
/// it (e.g. learned at the fire). In id order.
pub fn tidy(world: &mut World, res: &mut SimResources) {
    let checks: Vec<(u32, bool, bool)> = res
        .colony
        .teach_orders
        .iter()
        .map(|o| {
            let t = entity_of(world, o.teacher);
            let l = entity_of(world, o.learner);
            let broken = t.is_none()
                || l.is_none()
                || t.is_some_and(|t| !knowledge(world, t).practises(o.knowledge));
            let learned = l.is_some_and(|l| knowledge(world, l).practises(o.knowledge));
            (o.id, broken, learned)
        })
        .collect();
    for (id, broken, learned) in checks {
        if learned {
            close(world, res, id, true);
        } else if broken {
            close(world, res, id, false);
        }
    }
}

fn free(world: &World, e: Entity) -> bool {
    world.get::<&Job>(e).is_err()
        && world.get::<&PorterTrip>(e).is_err()
        && world.get::<&AgentState>(e).is_ok_and(|s| {
            matches!(
                *s,
                AgentState::Idle
                    | AgentState::Seeking {
                        target: TargetKind::Activity
                    }
                    | AgentState::Consuming {
                        target: TargetKind::Activity
                    }
            )
        })
}

fn crafts(world: &World, e: Entity) -> bool {
    let p = world
        .get::<&WorkPriorities>(e)
        .map(|p| *p)
        .unwrap_or(WorkPriorities::defaults(world.get::<&Elder>(e).is_ok()));
    p.level(WorkType::Craft) > 0
}

/// Give today's sessions their jobs (see the module header). `assigned` holds
/// the people already given a job this tick; both partners are added to it.
pub fn assign(
    world: &World,
    res: &mut SimResources,
    rule: &KnowledgeRule,
    assigned: &mut BTreeSet<AgentId>,
    taken: &mut Vec<(Entity, Job, (u32, u32))>,
) {
    let tick = res.current_tick;
    let day = tick / res.ticks_per_day.max(1);
    let orders: Vec<TeachOrder> = res.colony.teach_orders.clone();
    let lesson = |e: Entity, id: u32| {
        world.get::<&Job>(e).is_ok_and(|j| {
            matches!(j.kind, JobKind::Teach { order, .. } | JobKind::Learn { order, .. } if order == id)
        })
    };
    for o in orders {
        let today = if o.day == day { o.sessions_today } else { 0 };
        if today >= rule.sessions_per_day {
            continue;
        }
        let (Some(t), Some(l)) = (entity_of(world, o.teacher), entity_of(world, o.learner)) else {
            continue;
        };
        let Some(at) = meeting_tile(res, o.knowledge) else {
            continue;
        };
        let pos = |e: Entity| world.get::<&Position>(e).map(|p| (p.x, p.y)).ok();
        let reach =
            |p: (u32, u32)| !res.nav.is_built() || res.nav.reaches(res.nav.region_of(p), at);
        // A session needs both: each with Craft priority and able to walk to
        // the meeting tile. Otherwise nobody is sent (one partner alone would
        // wait, time out and be sent again forever; code review F1); the
        // order stays open, listed in the K dialog.
        let can_meet = |e: Entity| crafts(world, e) && pos(e).is_some_and(&reach);
        if !can_meet(t) || !can_meet(l) {
            continue;
        }
        // Either partner who is free goes to the meeting tile and waits there (up
        // to `WAIT_TICKS`) for the other (DECISIONS D18) — not one about to leave
        // for food or water, in a fight, or kept off this tile by the
        // unreachable cooldown (the main assignment's guards; code review F3).
        for (e, id, kind) in [
            (t, o.teacher, JobKind::Teach { order: o.id, at }),
            (l, o.learner, JobKind::Learn { order: o.id, at }),
        ] {
            if assigned.contains(&id)
                || lesson(e, o.id)
                || !free(world, e)
                || super::assign::needy(world, res, e)
                || super::assign::in_combat(res, id)
                || res
                    .colony
                    .unreachable
                    .get(&(id, at))
                    .is_some_and(|until| *until > tick)
            {
                continue;
            }
            let Some(p) = pos(e) else {
                continue;
            };
            taken.push((
                e,
                Job {
                    kind,
                    stage: JobStage::ToSource,
                    taken_at: tick,
                    amount: 0,
                    item: sim_core::colony::ItemKind::Log,
                    material: None,
                    fetch: None,
                    best_distance: chebyshev(p, at),
                    progress_tick: tick,
                },
                at,
            ));
            assigned.insert(id);
            res.colony.tallies.jobs_taken += 1;
        }
    }
}

/// The fidelity ‰ of `teacher` teaching `node`.
pub fn fidelity(rule: &KnowledgeRule, teacher: &Knowledge, node: KnowledgeId) -> u32 {
    let base = if teacher.practice() & gates().apprenticeship() != 0 {
        rule.fidelity
    } else {
        rule.fidelity_untrained
    };
    if teacher.proficiency(node) >= rule.master_at {
        base * rule.master_permille / 1000
    } else {
        base
    }
}

/// The partner's job for order `id` is of the other kind and in `Working`.
fn partner_waiting(world: &World, partner: Entity, id: u32) -> bool {
    world.get::<&Job>(partner).is_ok_and(|j| {
        matches!(j.kind, JobKind::Learn { order, .. } if order == id)
            && matches!(j.stage, JobStage::Working { .. })
    })
}

/// One session ended: the learner gains, the teacher uses, the order counts.
fn session_done(
    world: &mut World,
    res: &mut SimResources,
    rule: &KnowledgeRule,
    o: TeachOrder,
    t: Entity,
    l: Entity,
) {
    let tick = res.current_tick;
    let tk = knowledge(world, t);
    let gain = (u32::from(rule.session_gain) * fidelity(rule, &tk, o.knowledge) + 500) / 1000;
    let mut lk = knowledge(world, l);
    let before = lk.proficiency(o.knowledge);
    lk.set(
        o.knowledge,
        before
            .saturating_add(u16::try_from(gain).unwrap_or(u16::MAX))
            .min(1000),
        Source::Apprenticed,
        tick,
    );
    let learned = lk.practises(o.knowledge);
    crate::runtime::knowledge::store(world, l, lk);
    let used = bit(o.knowledge) | (tk.practice() & gates().apprenticeship());
    crate::runtime::knowledge::used(world, res, t, used);
    let day = tick / res.ticks_per_day.max(1);
    if let Some(order) = res.colony.teach_orders.iter_mut().find(|x| x.id == o.id) {
        if order.day != day {
            order.day = day;
            order.sessions_today = 0;
        }
        order.sessions += 1;
        order.sessions_today = order.sessions_today.saturating_add(1);
    }
    res.colony.tallies.teach_sessions += 1;
    if learned {
        res.chronicle.push(
            ChronicleEntry::new(tick, ChronicleKind::KnowledgeLearned)
                .subject(o.learner)
                .other(Some(o.teacher))
                .cause(Source::Apprenticed.code())
                .value(i64::from(o.knowledge)),
        );
        close(world, res, o.id, true);
    } else {
        finish(world, res, t);
        finish(world, res, l);
    }
}

/// What a Teach or Learn job does this tick, after the job layer's common
/// checks; returns the job to store, or `None` when it ended.
pub fn step(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    mut job: Job,
    pos: (u32, u32),
) -> Option<Job> {
    let rule = crate::runtime::knowledge::rule(res)?;
    let tick = res.current_tick;
    let (id, at, teaching) = match job.kind {
        JobKind::Teach { order, at } => (order, at, true),
        JobKind::Learn { order, at } => (order, at, false),
        _ => return Some(job),
    };
    let Some(o) = res.colony.teach_orders.iter().find(|o| o.id == id).copied() else {
        cancel(world, res, entity, CancelReason::TargetGone);
        return None;
    };
    let Some(partner) = entity_of(world, if teaching { o.learner } else { o.teacher }) else {
        cancel(world, res, entity, CancelReason::TargetGone);
        return None;
    };
    // The partner may come later (DECISIONS D18); a lesson job waits at the
    // meeting tile at most `WAIT_TICKS` unless a session is under way. The wait
    // counts from its arrival (`progress_tick`: the walk's last step, renewed
    // when a session starts or stops), not from taking the job (code review F10).
    let in_session = |e: Entity| {
        world.get::<&Job>(e).is_ok_and(|j| {
            matches!(j.kind, JobKind::Teach { order, .. } if order == id)
                && matches!(j.stage, JobStage::Working { .. })
        })
    };
    let session = if teaching {
        matches!(job.stage, JobStage::Working { .. })
    } else {
        in_session(partner)
    };
    if !session && tick.saturating_sub(job.progress_tick) > WAIT_TICKS {
        cancel(world, res, entity, CancelReason::TargetGone);
        return None;
    }
    let here = chebyshev(pos, at) <= 1;
    match job.stage {
        JobStage::ToSource if !here => Some(job),
        JobStage::ToSource if !teaching => {
            job.stage = JobStage::Working {
                ticks_left: u32::MAX,
            };
            set_state(world, entity, WORKING);
            Some(job)
        }
        JobStage::ToSource => {
            if partner_waiting(world, partner, id) {
                job.stage = JobStage::Working {
                    ticks_left: rule.session_ticks.max(1),
                };
                job.progress_tick = tick;
                set_state(world, entity, WORKING);
            }
            Some(job)
        }
        JobStage::Working { .. } if !teaching => Some(job),
        // The learner left (a need, a cancel): the session stops; wait again.
        JobStage::Working { .. } if !partner_waiting(world, partner, id) => {
            job.stage = JobStage::ToSource;
            job.progress_tick = tick;
            set_state(world, entity, SEEKING);
            Some(job)
        }
        JobStage::Working { ticks_left } if ticks_left > 1 => {
            job.stage = JobStage::Working {
                ticks_left: ticks_left - 1,
            };
            Some(job)
        }
        JobStage::Working { .. } => {
            session_done(world, res, &rule, o, entity, partner);
            None
        }
        JobStage::ToDest => Some(job),
    }
}

const WORKING: AgentState = AgentState::Consuming {
    target: TargetKind::Job,
};
const SEEKING: AgentState = AgentState::Seeking {
    target: TargetKind::Job,
};

fn set_state(world: &mut World, entity: Entity, next: AgentState) {
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = next;
    }
}

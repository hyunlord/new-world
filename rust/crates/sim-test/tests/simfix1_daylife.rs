//! SIMFIX1 harnesses (SIMAUDIT1 N2 tickets SIMFIX1, SIMFIX2, WORK1, PURPOSE1 starving search): the
//! mechanisms on a real steward-run colony, tick by tick. The seed 0–7 × 60-day acceptance bars are
//! measured by the SIMAUDIT1 probe (`docs/v8/simfix1/SIMFIX1_DECISIONS.md`).
use sim_core::colony::building_template::TemplateCatalog;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::steward::StewardScope;
use sim_core::colony::{Command, Job, JobKind};
use sim_core::components::{Agent, AgentId, AgentState, Position, TargetKind};
use sim_core::rules::DaylifeRule;
use sim_systems::runtime::jobs::assign::{admission_estimate, admission_horizon};
use std::collections::BTreeMap;

const BUILDINGS: &str = include_str!("../../sim-core/data/colony/buildings.ron");
const HUT_ROOF: &str = "(beds: 4, storage: None, workshop_tags: [], warmth: Roofed)";

/// ADR 0033 data-only: the hut counts as a roof only because its data says `warmth: Roofed`.
#[test]
fn a_hut_is_a_roof_by_its_data_only() {
    let catalog = installed_or_bundled();
    let bundled = TemplateCatalog::parse(BUILDINGS, None, catalog).expect("bundled templates");
    assert!(sim_systems::runtime::housing::template_roofed(
        &bundled, "hut"
    ));
    assert_eq!(
        BUILDINGS.matches(HUT_ROOF).count(),
        1,
        "the hut row is unique"
    );
    let open = BUILDINGS.replace(HUT_ROOF, &HUT_ROOF.replace("Roofed", "Open"));
    let open = TemplateCatalog::parse(&open, None, catalog).expect("open-hut variant");
    assert!(
        !sim_systems::runtime::housing::template_roofed(&open, "hut"),
        "a hut variant with warmth: Open is no roof, so its sleepers get slept_outside"
    );
    assert!(!sim_systems::runtime::housing::template_roofed(
        &bundled,
        "no_such_template"
    ));
}

fn minute(engine: &sim_engine::SimEngine, tick: u64) -> u64 {
    let r = &engine.resources;
    tick.saturating_add(r.rules.clock_offset_ticks) % r.ticks_per_day.max(1)
}

/// One steward-run colony for four days, every tick: no job is admitted past the bedtime horizon,
/// no hunt outlives `hunt.max_ticks`, an untaken chase ends within `social.stall_ticks`, talks last
/// `social.talk_ticks`, no person alternates two mind nodes every tick, and the steward never repeats
/// a rejected hunt zone.
#[test]
fn harness_simfix1_daylife_invariants() {
    let rule = *DaylifeRule::get().expect("daylife.ron");
    let mut engine = sim_test::colony_engine(0);
    assert!(
        sim_systems::commands::submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            }
        )
        .accepted
    );
    let days = 4;
    let ticks = engine.resources.ticks_per_day * days;
    let mut positions: BTreeMap<AgentId, (u32, u32)> = BTreeMap::new();
    let mut previous_jobs: BTreeMap<AgentId, JobKind> = BTreeMap::new();
    let mut hunts: BTreeMap<AgentId, u64> = BTreeMap::new();
    let mut chases: BTreeMap<AgentId, (AgentId, u64)> = BTreeMap::new();
    let mut talking: BTreeMap<AgentId, u64> = BTreeMap::new();
    let mut nodes: BTreeMap<AgentId, (String, String, u32)> = BTreeMap::new();
    let (mut admitted, mut hunts_seen, mut hunts_ended, mut talks_done, mut chases_ended) =
        (0u32, 0u32, 0u32, 0u32, 0u32);
    let mut talk_lengths = Vec::new();
    let mut late = Vec::new();
    for _ in 0..ticks {
        // The tick the systems run as (`resources.current_tick` still holds the previous one here).
        let tick = engine.current_tick();
        engine.tick();
        let world = &engine.world;
        let res = &engine.resources;
        let dictionary = res.mind.history.as_ref().map(|h| h.dictionary());
        let mut seen_hunt = BTreeMap::new();
        for (entity, (agent, pos, state, job)) in world
            .query::<(&Agent, &Position, &AgentState, Option<&Job>)>()
            .iter()
        {
            let id = agent.id;
            // The running mind node after this tick.
            let node = res
                .mind
                .running
                .get(&id)
                .and_then(|rc| dictionary.and_then(|d| d.node_label(rc.node.index())))
                .map(|l| l.name.to_string())
                .unwrap_or_default();
            // Admission: a job the job layer took this tick fits the horizon (the estimate from the
            // position before the tick, a few ticks of movement allowed for). Not admissions: a
            // harvest the mind's `harvest_food` nodes start for a hungry person, and the haul that
            // carries off the yield of the agent's harvest from the tick before (D-SIMFIX1-14).
            let mind_harvest = node.contains("harvest_food") || node.starts_with("stock_");
            let handoff = matches!(
                (job.map(|j| j.kind), previous_jobs.get(&id)),
                (Some(JobKind::Haul { .. }), Some(JobKind::Harvest { .. }))
            );
            if let (Some(job), Some(before)) = (job, positions.get(&id)) {
                if job.taken_at == tick
                    && !matches!(job.kind, JobKind::Craft { .. })
                    && !(mind_harvest && matches!(job.kind, JobKind::Harvest { .. }))
                    && !handoff
                {
                    let target = if matches!(job.kind, JobKind::Hunt { .. }) {
                        *before
                    } else {
                        job.kind.source()
                    };
                    if let (Some(estimate), Some((horizon, night_end))) = (
                        admission_estimate(res, *before, job.fetch, target, job.kind),
                        admission_horizon(world, res, entity),
                    ) {
                        let m = minute(&engine, tick);
                        let slack = 3 * res.rules.agent.move_interval_ticks;
                        admitted += 1;
                        if m >= night_end && m + estimate > horizon + slack {
                            late.push((id, tick, job.kind, m, estimate, horizon));
                        }
                    }
                }
                if let JobKind::Hunt { .. } = job.kind {
                    seen_hunt.insert(id, job.taken_at);
                }
            }
            // Chases and talks.
            match *state {
                AgentState::Seeking {
                    target: TargetKind::Agent(peer),
                } => {
                    let chase = chases.entry(id).or_insert((peer, tick));
                    if chase.0 != peer {
                        *chase = (peer, tick);
                    }
                    let since = chase.1;
                    let reciprocated = world
                        .query::<(&Agent, &AgentState)>()
                        .iter()
                        .any(|(_, (other, s))| {
                            other.id == peer
                                && matches!(*s, AgentState::Seeking { target: TargetKind::Agent(x) } | AgentState::Consuming { target: TargetKind::Agent(x) } if x == id)
                        });
                    if reciprocated {
                        chases.insert(id, (peer, tick));
                    } else {
                        assert!(
                            tick - since <= rule.social.stall_ticks + 8,
                            "agent {id} chased {peer} untaken for {} ticks at {tick}",
                            tick - since
                        );
                    }
                }
                AgentState::Consuming {
                    target: TargetKind::Agent(_),
                } => {
                    if chases.remove(&id).is_some() {
                        chases_ended += 1;
                    }
                    talking.entry(id).or_insert(tick);
                }
                _ => {
                    if chases.remove(&id).is_some() {
                        chases_ended += 1;
                    }
                    if let Some(start) = talking.remove(&id) {
                        talks_done += 1;
                        talk_lengths.push(tick - start);
                    }
                }
            }
            positions.insert(id, (pos.x, pos.y));
            match job {
                Some(job) => previous_jobs.insert(id, job.kind),
                None => previous_jobs.remove(&id),
            };
            // One-tick alternation between two mind nodes.
            let entry = nodes.entry(id).or_insert((String::new(), String::new(), 0));
            if node != entry.0 {
                entry.2 = if node == entry.1 { entry.2 + 1 } else { 1 };
                entry.1 = std::mem::replace(&mut entry.0, node);
            } else {
                entry.2 = 0;
            }
            assert!(
                entry.2 < 10,
                "agent {id} alternates {} / {} every tick at {tick}",
                entry.0,
                entry.1
            );
        }
        // Hunts end in time.
        for (id, taken) in &hunts {
            if !seen_hunt.contains_key(id) {
                hunts_ended += 1;
                assert!(
                    tick - taken <= rule.hunt.max_ticks + 1,
                    "hunt of {id} taken at {taken} ran to {tick}"
                );
            }
        }
        for (id, taken) in &seen_hunt {
            if hunts.get(id) != Some(taken) {
                hunts_seen += 1;
            }
            assert!(tick - taken <= rule.hunt.max_ticks + 1);
        }
        hunts = seen_hunt;
    }
    assert!(late.is_empty(), "jobs admitted past the horizon: {late:?}");
    assert!(admitted > 100, "the harness saw {admitted} admissions");
    assert!(
        hunts_seen > 0 && hunts_ended > 0,
        "hunts {hunts_seen}/{hunts_ended}"
    );
    assert!(
        talks_done > 0 && chases_ended > 0,
        "talks {talks_done}, chases {chases_ended}"
    );
    talk_lengths.sort_unstable();
    assert!(
        talk_lengths[talk_lengths.len() / 2] >= u64::from(rule.social.talk_ticks) - 1,
        "median talk {} ticks",
        talk_lengths[talk_lengths.len() / 2]
    );
    let rejected_zones = engine
        .resources
        .colony
        .command_log
        .iter()
        .filter(|r| {
            !r.result.accepted
                && matches!(&r.command, Command::Steward { command, .. } if matches!(**command, Command::AddHuntZone { .. }))
        })
        .count();
    assert_eq!(rejected_zones, 0, "rejected AddHuntZone re-issues");
    assert!(
        sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources).is_ok()
    );
}

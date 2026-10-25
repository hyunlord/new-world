//! D40 whole-wave admission regressions through public commands and VisitorSystem.
//! Birth/death boundaries use controlled live-Agent perturbations, not reproduction timing.
pub(super) use sim_core::colony::knowledge::Knowledge;
pub(super) use sim_core::colony::neighbours::{VisitState, Wave, WaveAnswer};
pub(super) use sim_core::colony::people::Apparel;
pub(super) use sim_core::colony::{Command, RejectReason};
pub(super) use sim_core::components::{Agent, Child};
pub(super) use sim_engine::{RuntimeSystem, SimEngine};
pub(super) use sim_systems::commands::submit;
pub(super) use sim_systems::runtime::jobs::invariants;
pub(super) use sim_systems::runtime::neighbours::{self, VisitorSystem};
pub(super) use sim_systems::save;

pub(super) const ARRIVAL: u64 = 100;

pub(super) fn people(e: &SimEngine) -> usize {
    e.world.query::<&Agent>().iter().count()
}

pub(super) fn fixture(population: usize, children: usize, cap: u32) -> SimEngine {
    let mut e = sim_test::colony_engine(0);
    let rule = e
        .resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .neighbours
        .as_mut()
        .expect("neighbours");
    rule.population_cap = cap;
    let rule = *rule;
    e.resources.colony.neighbours.list = neighbours::generate(0, &rule);
    assert!(e.resources.colony.neighbours.waves.is_empty());
    while people(&e) < population {
        e.spawn_agent(5, 5);
    }
    assert_eq!(people(&e), population);
    let mut entities: Vec<_> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, a)| (a.id, entity))
        .collect();
    entities.sort_by_key(|row| row.0);
    assert!(
        e.resources.settlements.is_empty(),
        "fresh fixture has no formed settlement"
    );
    let settlement_id = e.resources.issue_settlement_id();
    let mut settlement = sim_core::components::Settlement::new_with_id(settlement_id, 0);
    settlement.formation_tile =
        sim_systems::worldgen::colony::hearth(&e.resources).expect("actual colony hearth");
    for &(id, _) in &entities {
        assert!(settlement.add_member_agent(id));
    }
    settlement.population_stats.current = u32::try_from(population).expect("fixture population");
    e.resources.settlements.insert(settlement_id, settlement);
    assert_eq!(e.resources.settlements.len(), 1);
    assert_eq!(
        e.resources.settlements[&settlement_id].member_agents.len(),
        population
    );

    for (_, entity) in entities.into_iter().take(children) {
        e.world
            .insert_one(entity, Child { born_tick: 0 })
            .expect("child marker");
    }
    assert_eq!(e.world.query::<(&Agent, &Child)>().iter().count(), children);
    sim_systems::runtime::nav::ensure(&mut e.resources);
    assert!(
        neighbours::meeting_tile(&e.resources).is_some(),
        "real hearth arrival"
    );
    invariants::check(&e.world, &e.resources).expect("prepared world conserved");
    e
}

pub(super) fn wave(e: &mut SimEngine, id: u32, size: u8, arriving: bool) {
    let rule = neighbours::rule(&e.resources).expect("neighbour rule");
    let neighbour = e.resources.colony.neighbours.list[0].clone();
    let mut migrants = neighbours::migrants(
        e.resources.world_seed,
        &rule,
        &neighbour,
        id,
        size,
        e.resources.rules.society.aging,
    );
    for person in &mut migrants {
        person.agent = e.resources.issue_agent_id();
        person.wrap = true;
        assert!(person.agent > 0);
        assert!(
            !person.knowledge.is_empty(),
            "grants make partial admission observable"
        );
    }
    e.resources.colony.neighbours.waves.push(Wave {
        id,
        neighbour: neighbour.id,
        people: migrants,
        pos: neighbours::meeting_tile(&e.resources).expect("hearth meeting tile"),
        path: Vec::new(),
        state: if arriving {
            VisitState::Arriving
        } else {
            VisitState::Waiting { since: 0 }
        },
        day: 0,
        answer: None,
        joined: Vec::new(),
    });
}

pub(super) fn get(e: &SimEngine, id: u32) -> &Wave {
    e.resources
        .colony
        .neighbours
        .waves
        .iter()
        .find(|w| w.id == id)
        .expect("wave exists")
}

pub(super) fn arrive(e: &mut SimEngine, tick: u64) {
    e.resources.current_tick = tick;
    VisitorSystem.tick(&mut e.world, &mut e.resources);
    invariants::check(&e.world, &e.resources).expect("visitor conservation");
}

/// Full saved domain snapshot, excluding only the two documented command bookkeeping fields.
pub(super) fn domain(e: &mut SimEngine) -> Vec<u8> {
    let log = std::mem::take(&mut e.resources.colony.command_log);
    let rejected = std::mem::replace(&mut e.resources.colony.tallies.commands_rejected, 0);
    let bytes = save::save(e).expect("domain snapshot");
    e.resources.colony.command_log = log;
    e.resources.colony.tallies.commands_rejected = rejected;
    bytes
}

pub(super) fn blocked(e: &mut SimEngine, id: u32) {
    let before = domain(e);
    let log = e.resources.colony.command_log.len();
    let rejected = e.resources.colony.tallies.commands_rejected;
    let result = submit(e, Command::AcceptMigrants { wave: id });
    assert!(!result.accepted, "whole wave must be refused");
    assert_eq!(
        (result.count, result.reason),
        (0, Some(RejectReason::Blocked))
    );
    assert_eq!(e.resources.colony.command_log.len(), log + 1);
    assert_eq!(e.resources.colony.tallies.commands_rejected, rejected + 1);
    assert_eq!(
        domain(e),
        before,
        "no domain mutation beyond declared rejection bookkeeping"
    );
    invariants::check(&e.world, &e.resources).expect("rejection conservation");
}

pub(super) fn accepted_early(e: &mut SimEngine, id: u32) {
    let before = people(e);
    let result = submit(e, Command::AcceptMigrants { wave: id });
    assert!(result.accepted);
    assert_eq!(result.count as usize, get(e, id).people.len());
    assert_eq!(people(e), before, "early answer never spawns");
    assert_eq!(
        (get(e, id).state, get(e, id).answer),
        (VisitState::Arriving, Some(WaveAnswer::Accepted))
    );
    assert!(get(e, id).joined.is_empty());
}

pub(super) fn joined(e: &SimEngine, id: u32, population: usize) {
    let w = get(e, id);
    assert_eq!(
        (w.state, w.answer),
        (VisitState::Gone, Some(WaveAnswer::Accepted))
    );
    assert_eq!(
        w.joined,
        w.people.iter().map(|p| p.agent).collect::<Vec<_>>()
    );
    assert!(!w.joined.is_empty());
    assert_eq!(people(e), population);
    for spec in &w.people {
        let matches: Vec<_> = e
            .world
            .query::<&Agent>()
            .iter()
            .filter(|(_, a)| a.id == spec.agent)
            .map(|(entity, _)| entity)
            .collect();
        assert_eq!(matches.len(), 1, "reserved ID exists exactly once");
        let entity = matches[0];
        let knowledge = e
            .world
            .get::<&Knowledge>(entity)
            .expect("migrant knowledge");
        for &(node, p) in &spec.knowledge {
            assert_eq!(knowledge.get(node).expect("granted node").proficiency, p);
        }
        assert!(e.world.get::<&Apparel>(entity).is_ok(), "actual wrap grant");
        assert!(
            e.resources
                .settlements
                .values()
                .any(|s| s.member_agents.contains(&spec.agent)),
            "membership granted"
        );
    }
    invariants::check(&e.world, &e.resources).expect("joined conservation");
}

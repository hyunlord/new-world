//! External runtime ownership must survive a change of mind node.
use super::*;
use sim_core::colony::{ItemKind, Job, JobKind, JobStage};
use sim_core::components::{Agent, Hunger, Position, TargetKind};
use sim_core::MaterialRegistry;

const SOURCE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"eat",band:"critical",behaviour:"eat",target:Some("food"),when:[(fact:"hunger",op:">",value:Some(80000))],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"job",band:"work",behaviour:"do_job",target:Some("job"),when:[(fact:"has_job",op:"has")],score:(constant:10,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"JOB"),
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn job() -> Job {
    Job {
        kind: JobKind::Chop { tree: (3, 3) },
        stage: JobStage::ToSource,
        taken_at: 0,
        amount: 0,
        item: ItemKind::Log,
        material: None,
        fetch: None,
        best_distance: 2,
        progress_tick: 0,
    }
}
fn fixture(hunger: f32) -> (ColonyMindSystem, World, SimResources, hecs::Entity) {
    let mut resources = SimResources::new(5, 5, MaterialRegistry::new());
    resources.rules.colony = sim_core::rules::WorldRules::colony(1).colony;
    resources.food_tiles.insert((2, 1), 5);
    let mut world = World::new();
    let person = world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(hunger, 0.0),
    ));
    (
        ColonyMindSystem::new(MindProgram::load(SOURCE).expect("policy")),
        world,
        resources,
        person,
    )
}

#[test]
fn newly_assigned_job_keeps_owner_state_and_target_after_prior_idle_cleanup_and_postpass() {
    let (mut system, mut world, mut resources, person) = fixture(20.0);
    system
        .try_tick(&mut world, &mut resources)
        .expect("idle tick");
    let owned = AgentState::Seeking {
        target: TargetKind::Job,
    };
    world
        .insert(person, (job(), owned, SeekTarget { tile: (3, 3) }))
        .expect("assign job");
    resources.current_tick += 1;
    system
        .try_tick(&mut world, &mut resources)
        .expect("job entry");
    assert_eq!(*world.get::<&AgentState>(person).expect("state"), owned);
    assert_eq!(
        world.get::<&SeekTarget>(person).expect("target").tile,
        (3, 3)
    );
    assert!(world.get::<&Job>(person).is_ok());
    assert!(resources.colony.release_counts.is_empty());
}

#[test]
fn startup_job_yielding_to_critical_food_is_released_without_prior_job_choice() {
    let (mut system, mut world, mut resources, person) = fixture(90.0);
    world
        .insert(
            person,
            (
                job(),
                AgentState::Seeking {
                    target: TargetKind::Job,
                },
                SeekTarget { tile: (3, 3) },
            ),
        )
        .expect("assign job");
    system
        .try_tick(&mut world, &mut resources)
        .expect("critical preemption");
    assert!(world.get::<&Job>(person).is_err());
    assert_eq!(
        *world.get::<&AgentState>(person).expect("state"),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );
    assert_eq!(
        world.get::<&SeekTarget>(person).expect("food target").tile,
        (2, 1)
    );
    assert_eq!(resources.colony.release_counts.values().sum::<u64>(), 1);
}

#[test]
fn legacy_consuming_food_without_mind_history_consumes_instead_of_restarting_seek() {
    let (mut system, mut world, mut resources, person) = fixture(90.0);
    resources.rules.agent.strict_consume = true;
    resources.food_tiles.insert((1, 1), 2);
    world
        .insert_one(
            person,
            AgentState::Consuming {
                target: TargetKind::Food,
            },
        )
        .expect("legacy consumption");
    assert!(resources.mind.running.is_empty());
    system
        .try_tick(&mut world, &mut resources)
        .expect("adopt consumption");
    assert_eq!(resources.tallies.meals_eaten, 1);
    assert!(world.get::<&Hunger>(person).expect("hunger").value < 90.0);
    assert_eq!(
        *world.get::<&AgentState>(person).expect("state"),
        AgentState::Idle
    );
    assert!(
        resources
            .mind
            .running
            .get(&7)
            .expect("recorded choice")
            .finished
    );
}

#[test]
fn legacy_activity_without_mind_history_is_continued_with_original_identity() {
    use sim_core::components::{Activity, ActivityKind, Settlement};
    let source = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
    (id:"existing",band:"leisure",behaviour:"routine_sit_by_fire",when:[(fact:"hunger",op:">",value:Some(0))],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EXISTING"),
    (id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let (_, mut world, mut resources, person) = fixture(20.0);
    resources.rules.agent.routine = sim_core::rules::WorldRules::colony(1).agent.routine;
    assert!(resources.rules.agent.routine.is_some());
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (3, 3);
    settlement.member_agents.insert(7);
    resources.settlements.insert(1, settlement);
    let activity = Activity {
        kind: ActivityKind::SitByFire,
        settlement: 1,
        place: (3, 3),
        chosen_tick: 0,
        arrived_tick: None,
        until_tick: None,
    };
    world
        .insert(
            person,
            (
                activity,
                AgentState::Seeking {
                    target: TargetKind::Activity,
                },
                SeekTarget { tile: (3, 3) },
            ),
        )
        .expect("legacy activity");
    let mut system = ColonyMindSystem::new(MindProgram::load(source).expect("policy"));
    system
        .try_tick(&mut world, &mut resources)
        .expect("adopt activity");
    assert_eq!(
        *world.get::<&Activity>(person).expect("same activity"),
        activity
    );
    assert_eq!(
        world
            .get::<&SeekTarget>(person)
            .expect("same destination")
            .tile,
        (3, 3)
    );
    assert!(!resources.mind.running.get(&7).expect("running").finished);
}

#[test]
fn invitation_to_later_active_person_reselects_social_in_the_same_tick() {
    use sim_core::components::{Activity, ActivityKind, Social};
    const SOCIAL: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
    (id:"social",band:"need",behaviour:"socialise",target:Some("social_peer"),when:[(fact:"idle",op:"has"),(fact:"social_candidate",op:"has")],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"SOCIAL"),
    (id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let mut resources = SimResources::new(5, 5, MaterialRegistry::new());
    resources.rules = sim_core::rules::WorldRules::colony(1);
    assert!(resources.rules.agent.routine.is_some());
    let mut world = World::new();
    let inviter = world.spawn((
        Agent { id: 1 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        Social::new(90.0, 0.0),
    ));
    let invited = world.spawn((
        Agent { id: 2 },
        Position::new(1, 1),
        AgentState::Seeking {
            target: TargetKind::Activity,
        },
        Hunger::new(0.0, 0.0),
        Social::new(90.0, 0.0),
        Activity {
            kind: ActivityKind::Play,
            settlement: 1,
            place: (2, 1),
            chosen_tick: 0,
            arrived_tick: None,
            until_tick: None,
        },
        SeekTarget { tile: (2, 1) },
    ));
    let mut system = ColonyMindSystem::new(MindProgram::load(SOCIAL).expect("social policy"));
    system
        .try_tick(&mut world, &mut resources)
        .expect("invitation tick");
    assert_eq!(
        *world.get::<&AgentState>(inviter).expect("inviter state"),
        AgentState::Seeking {
            target: TargetKind::Agent(2)
        }
    );
    assert_eq!(
        *world.get::<&AgentState>(invited).expect("invited state"),
        AgentState::Seeking {
            target: TargetKind::Agent(1)
        }
    );
    assert!(world.get::<&Activity>(invited).is_err());
    assert_eq!(
        world.get::<&SeekTarget>(invited).expect("peer target").tile,
        (1, 1)
    );
    assert!(!resources.mind.running.get(&2).expect("choice").finished);
}

const SOCIAL_REENTRY: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"continue_social",band:"need",behaviour:"socialise",target:Some("social_current_peer"),when:[(fact:"target_social",op:"has")],score:(constant:200,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"SOCIAL"),
(id:"enter_social",band:"need",behaviour:"socialise",target:Some("social_peer"),when:[(fact:"idle",op:"has"),(fact:"social_candidate",op:"has")],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"SOCIAL"),
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn social_fixture() -> (
    ColonyMindSystem,
    World,
    SimResources,
    hecs::Entity,
    hecs::Entity,
) {
    use sim_core::components::Social;
    let mut world = World::new();
    let first = world.spawn((
        Agent { id: 1 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        Social::new(90.0, 0.0),
    ));
    let second = world.spawn((
        Agent { id: 2 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(0.0, 0.0),
        Social::new(90.0, 0.0),
    ));
    let mut resources = SimResources::new(5, 5, MaterialRegistry::new());
    resources.rules.colony = sim_core::rules::WorldRules::colony(1).colony;
    resources.rules.agent.social_seek = Some(sim_core::rules::SocialSeekRule {
        max_chase_ticks: 20,
        retry_cooldown_ticks: 5,
    });
    (
        ColonyMindSystem::new(MindProgram::load(SOCIAL_REENTRY).expect("social policy")),
        world,
        resources,
        first,
        second,
    )
}

#[test]
fn external_social_completion_allows_same_behaviour_to_start_a_new_chase() {
    use sim_core::components::Social;
    let (mut system, mut world, mut resources, first, second) = social_fixture();
    system
        .try_tick(&mut world, &mut resources)
        .expect("start social");
    resources.current_tick += 1;
    system
        .try_tick(&mut world, &mut resources)
        .expect("mutual handshake");
    assert_eq!(
        *world.get::<&AgentState>(first).expect("first"),
        AgentState::Consuming {
            target: TargetKind::Agent(2)
        }
    );
    let mut interaction = crate::runtime::social::SocialInteractionSystem::new();
    // The actual handshake tick seeds progress at zero; later ticks advance it.
    interaction.tick(&mut world, &mut resources);
    for _ in 0..crate::runtime::agent_actions::needs::REQUIRED_INTERACTION_PROGRESS {
        resources.current_tick += 1;
        interaction.tick(&mut world, &mut resources);
    }
    assert_eq!(resources.tallies.social_completed, 1);
    assert!(!resources.mind.running[&1].finished);
    for person in [first, second] {
        assert_eq!(
            *world
                .get::<&AgentState>(person)
                .expect("external completion"),
            AgentState::Idle
        );
        world.get::<&mut Social>(person).expect("social").loneliness = 90.0;
    }
    resources.current_tick += 1;
    system
        .try_tick(&mut world, &mut resources)
        .expect("new social entry");
    assert_eq!(
        *world.get::<&AgentState>(first).expect("new chase"),
        AgentState::Seeking {
            target: TargetKind::Agent(2)
        }
    );
    assert!(!resources.mind.running[&1].finished);
    assert_eq!(
        resources.social_chase_since.get(&1),
        Some(&resources.current_tick)
    );
}

#[test]
fn missing_social_peer_still_finishes_through_existing_cooldown_path() {
    let (mut system, mut world, mut resources, first, second) = social_fixture();
    system
        .try_tick(&mut world, &mut resources)
        .expect("start social");
    world.despawn(second).expect("remove partner");
    resources.current_tick += 1;
    system
        .try_tick(&mut world, &mut resources)
        .expect("lost partner completion");
    assert_eq!(
        *world.get::<&AgentState>(first).expect("state"),
        AgentState::Idle
    );
    assert_eq!(
        resources.social_retry_after.get(&1),
        Some(&(resources.current_tick + 5))
    );
    assert!(resources.mind.running[&1].finished);
}

#[test]
fn source_less_provenance_is_rejected_without_installing_or_erasing_state() {
    use sim_core::colony::mind_job_origins::{MindDraftOrigin, MindJobOrigin, MindOrderKey};
    use sim_core::colony::mind_records::MindFactSource;
    let source = MindFactSource::Command(0);
    for case in 0..4 {
        let (mut system, mut world, mut resources, _) = fixture(20.0);
        match case {
            0 => {
                resources
                    .mind
                    .order_origins
                    .insert(MindOrderKey::Dress, source);
            }
            1 => {
                resources.mind.job_origins.insert(
                    7,
                    MindJobOrigin {
                        kind: job().kind,
                        taken_at: 0,
                        source,
                    },
                );
            }
            2 => {
                resources.mind.draft_origins.insert(
                    7,
                    MindDraftOrigin {
                        since: 0,
                        order: sim_core::colony::people::DraftOrder::Hold,
                        source,
                    },
                );
            }
            3 => {
                resources.causal_log.push(
                    0,
                    sim_core::causal::CausalEvent::MindFactOrigin {
                        id: 1,
                        source,
                        tick: 0,
                    },
                );
            }
            _ => unreachable!(),
        }
        let error = system
            .try_tick(&mut world, &mut resources)
            .expect_err("source-less provenance must fail");
        assert!(error.0.contains("nonempty mind lacks authored source"));
        assert!(resources.mind.policy_source.is_none());
        assert!(resources.mind.history.is_none());
        let preserved = match case {
            0 => !resources.mind.order_origins.is_empty(),
            1 => !resources.mind.job_origins.is_empty(),
            2 => !resources.mind.draft_origins.is_empty(),
            3 => resources.causal_log.lookup(1).is_some(),
            _ => unreachable!(),
        };
        assert!(preserved);
    }
}

#[test]
fn idle_person_with_retained_job_can_choose_food_without_releasing_the_job() {
    let (mut system, mut world, mut resources, person) = fixture(90.0);
    let retained = job();
    world.insert_one(person, retained).expect("retained job");
    let holders = crate::runtime::jobs::holders(&world, &resources);
    assert!(!crate::runtime::jobs::hold_for_decision(
        person,
        AgentState::Idle,
        &holders
    ));
    system
        .try_tick(&mut world, &mut resources)
        .expect("real mind food selection");
    assert_eq!(
        *world.get::<&AgentState>(person).expect("state"),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );
    assert_eq!(
        world.get::<&Job>(person).ok().map(|job| *job),
        Some(retained)
    );
    assert!(resources.colony.release_counts.is_empty());
}

#[test]
fn externally_idled_prior_job_choice_keeps_retained_job_when_switching_to_food() {
    let (mut system, mut world, mut resources, person) = fixture(20.0);
    let retained = job();
    world
        .insert(
            person,
            (
                retained,
                AgentState::Seeking {
                    target: TargetKind::Job,
                },
            ),
        )
        .expect("owned job");
    system
        .try_tick(&mut world, &mut resources)
        .expect("record prior job choice");
    world
        .insert_one(person, AgentState::Idle)
        .expect("external idle transition");
    world.get::<&mut Hunger>(person).expect("hunger").value = 90.0;
    resources.current_tick += 1;
    system
        .try_tick(&mut world, &mut resources)
        .expect("switch to food");
    assert_eq!(
        *world.get::<&AgentState>(person).expect("state"),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );
    assert_eq!(
        world.get::<&Job>(person).ok().map(|job| *job),
        Some(retained)
    );
    assert!(resources.colony.release_counts.is_empty());
}

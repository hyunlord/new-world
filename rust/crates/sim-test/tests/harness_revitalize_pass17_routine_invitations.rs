mod pass17_routine_support;
use pass17_routine_support::*;
use sim_core::causal::CausalEvent;
use sim_core::components::{
    Activity, ActivityKind, ActivityRest, Agent, AgentState, Hunger, Memory, MemoryArm,
    MemoryEntry, RoutineObservationBatch, RoutineOutcome, Social, TargetKind,
};
use sim_engine::RuntimeSystem;
use sim_systems::runtime::social::SocialInteractionSystem;

fn memory_for(arm: MemoryArm, valence: f64) -> Memory {
    let mut memory = Memory::new();
    memory.insert(MemoryEntry::new(100, 600, valence, 1.0, arm));
    memory.insert(MemoryEntry::new(101, 600, valence, 1.0, arm));
    memory
}

#[test]
fn mutual_activity_peers_start_once_and_complete_three_ticks_later() {
    for activity_holders in [false, true] {
        // Given two lonely activity holders at the same hearth.
        let mut engine = fixture();
        if !activity_holders {
            engine.resources.rules.agent.routine = None;
        }
        let peers = [resident(&mut engine, 11), resident(&mut engine, 12)];
        for entity in peers {
            if activity_holders {
                activity(&mut engine, entity, ActivityKind::HearthCircle, true);
            }
            engine
                .world
                .insert_one(entity, Social::new(60.0, 0.0))
                .expect("social need");
        }
        // When the real decision and social systems run through the mutual handshake.
        for tick in 600..=604 {
            engine.resources.current_tick = tick;
            decide(&mut engine);
            SocialInteractionSystem::new().tick(&mut engine.world, &mut engine.resources);
        }
        // Then one start completes three ticks later with its causal parent intact.
        let tile = 24 * engine.resources.tile_grid.width + 24;
        let log = engine
            .resources
            .causal_log
            .get(tile)
            .expect("social events");
        let starts: Vec<_> = log
            .as_slice()
            .iter()
            .filter_map(|event| match event {
                CausalEvent::SocialInteractionStarted { id, tick, .. } => Some((*id, *tick)),
                _ => None,
            })
            .collect();
        let completions: Vec<_> = log
            .as_slice()
            .iter()
            .filter_map(|event| match event {
                CausalEvent::SocialInteractionCompleted { parent, tick, .. } => {
                    Some((*parent, *tick))
                }
                _ => None,
            })
            .collect();
        assert_eq!(starts.len(), 1);
        assert_eq!(starts[0].1, 601);
        assert_eq!(completions, vec![(Some(starts[0].0), starts[0].1 + 3)]);
        assert_eq!(
            engine
                .resources
                .interaction_progress
                .values()
                .copied()
                .collect::<Vec<_>>(),
            vec![3]
        );
        for entity in peers {
            assert_eq!(
                *engine.world.get::<&AgentState>(entity).expect("peer state"),
                AgentState::Idle
            );
            assert!(engine.world.get::<&Activity>(entity).is_err());
        }
        engine.resources.current_tick = 605;
        SocialInteractionSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert!(engine.resources.interaction_progress.is_empty());
        assert_eq!(engine.resources.tallies.social_completed, 1);
    }
}

#[test]
fn invitation_replays_earlier_peers_real_combat_response_without_interrupting_uninvited_peers() {
    // Given an earlier activity holder whose latent Idle response is combat.
    let mut engine = fixture();
    let peer = resident(&mut engine, 1);
    let actor = resident(&mut engine, 2);
    activity(&mut engine, peer, ActivityKind::Craft, true);
    engine
        .world
        .insert_one(peer, memory_for(MemoryArm::Combat, -1.0))
        .expect("combat memory");
    decide(&mut engine);
    assert!(
        engine.world.get::<&Activity>(peer).is_ok(),
        "no new unconditional combat exit"
    );
    engine.resources.current_tick += 1;
    engine
        .world
        .insert(
            actor,
            (
                AgentState::Idle,
                Hunger::new(60.0, 0.0),
                memory_for(MemoryArm::Social, 1.0),
            ),
        )
        .expect("inviter");
    engine.resources.set_food_tile(40, 40, 5);
    let actor_id = engine.world.get::<&Agent>(actor).expect("actor").id;
    // When a later actor invites it, the same Idle cascade replays only for that peer.
    decide(&mut engine);
    // Then its actual combat response and causal side effects are retained.
    assert_eq!(
        *engine.world.get::<&AgentState>(peer).expect("peer state"),
        AgentState::Consuming {
            target: TargetKind::Agent(actor_id)
        }
    );
    assert!(engine.world.get::<&Activity>(peer).is_err());
    assert_eq!(engine.resources.combat_pairs.len(), 1);
}

#[test]
fn same_tick_start_and_invitation_keep_both_observations() {
    // Given an Idle peer ordered before a memory-biased inviter.
    let mut engine = fixture();
    let peer = resident(&mut engine, 3);
    let actor = resident(&mut engine, 4);
    engine
        .world
        .insert(
            actor,
            (Hunger::new(60.0, 0.0), memory_for(MemoryArm::Social, 1.0)),
        )
        .expect("inviter");
    engine.resources.set_food_tile(40, 40, 5);
    // When the peer starts an activity and the later actor selects it in the same tick.
    decide(&mut engine);
    // Then the zero-minute start and end both survive for the probe.
    assert!(engine.world.get::<&Activity>(peer).is_err());
    let batch = engine
        .world
        .get::<&RoutineObservationBatch>(peer)
        .expect("ordered events");
    assert_eq!(batch.tick, 600);
    assert_eq!(batch.events.len(), 2);
    assert!(matches!(
        batch.events[0].outcome,
        RoutineOutcome::Started { .. }
    ));
    assert_eq!(
        batch.events[1].outcome,
        RoutineOutcome::Ended { completed: false }
    );
    assert_eq!(batch.events[0].kind, batch.events[1].kind);
    assert_eq!(
        engine
            .world
            .get::<&ActivityRest>(peer)
            .expect("guard")
            .until_tick,
        601
    );
}

#[test]
fn bias_selected_nonlonely_activity_peer_exits_in_both_id_orders() {
    // Given a memory-biased actor and an activity peer who is not lonely.
    for peer_first in [false, true] {
        let mut engine = fixture();
        let first = resident(&mut engine, 7);
        let second = resident(&mut engine, 8);
        let (peer, actor) = if peer_first {
            (first, second)
        } else {
            (second, first)
        };
        activity(&mut engine, peer, ActivityKind::Craft, true);
        let mut memory = Memory::new();
        memory.insert(MemoryEntry::new(100, 600, 1.0, 1.0, MemoryArm::Social));
        memory.insert(MemoryEntry::new(101, 600, 1.0, 1.0, MemoryArm::Social));
        engine
            .world
            .insert(actor, (Hunger::new(60.0, 0.0), memory))
            .expect("memory actor");
        engine.resources.set_food_tile(40, 40, 5);
        let peer_id = engine.world.get::<&Agent>(peer).expect("peer").id;
        // When decision and the actual downstream social system run once.
        decide(&mut engine);
        SocialInteractionSystem::new().tick(&mut engine.world, &mut engine.resources);
        // Then selection ends the peer's activity, without fabricating a mutual interaction.
        assert_eq!(
            *engine.world.get::<&AgentState>(actor).expect("actor state"),
            AgentState::Seeking {
                target: TargetKind::Agent(peer_id)
            }
        );
        assert!(
            engine.world.get::<&Activity>(peer).is_err(),
            "peer_first={peer_first}"
        );
        assert_eq!(
            *engine.world.get::<&AgentState>(peer).expect("peer state"),
            AgentState::Idle
        );
        assert_eq!(
            engine
                .world
                .get::<&ActivityRest>(peer)
                .expect("guard rest")
                .until_tick,
            601
        );
        assert!(engine.resources.interaction_progress.is_empty());
    }
}

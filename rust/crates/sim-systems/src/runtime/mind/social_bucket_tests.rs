//! Compare bounded buckets with both full ID vectors and the complete old snapshot.
use super::*;
use sim_core::components::{
    Agent, BuildingBlueprint, Child, ConstructionSite, Hunger, Memory, MemoryArm, MemoryEntry,
    Position, Sleep, Social, Thirst,
};
#[path = "social_bucket_reference.rs"]
mod reference;

#[test]
fn first_two_ids_preserve_every_subset_and_focal_position() {
    for mask in 0_u32..256 {
        let original: Vec<u64> = (0..8).filter(|id| mask & (1 << id) != 0).collect();
        let mut bucket = SocialPeers::default();
        for id in &original {
            bucket.push(*id);
        }
        for focal in 0..=9 {
            assert_eq!(
                bucket.other(focal),
                original.iter().copied().find(|id| *id != focal)
            );
        }
    }
    let mut bucket = SocialPeers::default();
    bucket.push(0);
    bucket.push(u64::MAX);
    assert_eq!(bucket.other(0), Some(u64::MAX));
    assert_eq!(bucket.other(u64::MAX), Some(0));
}
fn compare(a: &TickContext, b: &TickContext) {
    for row in a.rows() {
        for name in [
            "social_candidate",
            "social_memory_candidate",
            "social_handshake",
        ] {
            assert_eq!(a.derived(row.agent, name), b.derived(row.agent, name));
        }
        for name in ["social_peer", "social_memory_peer"] {
            assert_eq!(a.target(row.agent, name), b.target(row.agent, name));
        }
    }
}
#[test]
fn complete_snapshot_matches_old_maps_and_remains_frozen_after_mutation() {
    for count in [0, 1, 2, 3, 8, 32] {
        for mode in 0..8 {
            let mut world = hecs::World::new();
            let mut entities = Vec::new();
            for id in (0..count).rev() {
                let mut memory = Memory::new();
                if id % 3 == 0 {
                    memory.entries.push(MemoryEntry {
                        event_id: id + 100,
                        arm: MemoryArm::Social,
                        encoded_tick: 0,
                        valence: 1.0,
                        salience: 1.0,
                        reinforcement_count: 0,
                    });
                }
                let state = if mode & 4 != 0 && id < 2 {
                    AgentState::Seeking {
                        target: TargetKind::Agent(1 - id),
                    }
                } else {
                    AgentState::Idle
                };
                let entity = world.spawn((
                    Agent { id },
                    Position::new(if mode & 1 == 0 { 1 } else { (id % 3) as u32 }, 1),
                    state,
                    Hunger::new(if id % 5 == 0 { 90.0 } else { 10.0 }, 0.0),
                    Thirst::new(if id % 7 == 0 { 90.0 } else { 10.0 }, 0.0),
                    Sleep::new(if id % 4 == 0 { 90.0 } else { 10.0 }, 0.0),
                    Social::new(if id % 3 == 0 { 10.0 } else { 90.0 }, 0.0),
                    memory,
                ));
                if mode & 2 != 0 && id % 5 == 1 {
                    world.insert_one(entity, Child { born_tick: 0 }).unwrap();
                }
                if mode == 7 && id % 4 == 3 {
                    world.remove_one::<Social>(entity).unwrap();
                    world.remove_one::<Hunger>(entity).unwrap();
                    world.remove_one::<Thirst>(entity).unwrap();
                    world.remove_one::<Sleep>(entity).unwrap();
                }
                entities.push((entity, state, *world.get::<&Position>(entity).unwrap()));
            }
            if mode & 1 != 0 {
                world.spawn((ConstructionSite::new(
                    BuildingBlueprint::new(1, 1, 1, 10),
                    Position::new(1, 1),
                ),));
                let mut complete =
                    ConstructionSite::new(BuildingBlueprint::new(2, 1, 1, 10), Position::new(2, 1));
                complete.progress = 10;
                world.spawn((complete,));
            }
            let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
            resources.current_tick = if mode & 2 == 0 { 720 } else { 1300 };
            resources.ticks_per_day = if mode == 7 { 0 } else { 1440 };
            resources.rules.agent.social_seek = Some(sim_core::rules::SocialSeekRule {
                max_chase_ticks: 240,
                retry_cooldown_ticks: 480,
            });
            resources.rules.agent.night_rest = Some(sim_core::rules::NightRestRule {
                start_minute: 1260,
                end_minute: 360,
                fatigue_threshold: 0.0,
                recovery_per_tick: 0.12,
                wake_preempt: 75.0,
            });
            for id in 0..count {
                if id % 6 == 0 {
                    resources
                        .social_retry_after
                        .insert(id, resources.current_tick + 1);
                }
            }
            for light in [false, true] {
                let capture = || {
                    if light {
                        TickContext::capture_continuation(&world, &resources)
                    } else {
                        TickContext::capture(&world, &resources)
                    }
                };
                let mut actual = capture().unwrap();
                let mut old = capture().unwrap();
                prepare_social_snapshot(&mut actual, &world, &resources).unwrap();
                reference::prepare_social_snapshot(&mut old, &world, &resources).unwrap();
                compare(&actual, &old);
                for (entity, _, _) in &entities {
                    *world.get::<&mut AgentState>(*entity).unwrap() = AgentState::Idle;
                    *world.get::<&mut Position>(*entity).unwrap() = Position::new(7, 7);
                }
                for row in actual.rows().to_vec() {
                    actual.refresh_person(&world, row.agent).unwrap();
                    old.refresh_person(&world, row.agent).unwrap();
                }
                compare(&actual, &old);
                for (entity, state, position) in &entities {
                    *world.get::<&mut AgentState>(*entity).unwrap() = *state;
                    *world.get::<&mut Position>(*entity).unwrap() = *position;
                }
            }
        }
    }
}

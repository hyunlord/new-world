//! Merged in V8-0 (8.4) from `harness_revitalize_pass17_routine.rs`,
//! `harness_revitalize_pass17_routine_edges.rs`,
//! `harness_revitalize_pass17_routine_invitations.rs`; each source is one
//! module below, unchanged.

mod routine_support;

// From `harness_revitalize_pass17_routine.rs`.
mod schedule {
    use super::routine_support::*;
    use sim_core::components::{
        Activity, ActivityKind, ActivityRest, AgentState, Child, Hunger, Inventory, Position,
        ResourceKind, RoutineObservation, RoutineOutcome, SeekTarget, Social, TargetKind,
    };
    use sim_core::rules::WorldRules;
    use sim_engine::RuntimeSystem;
    use sim_systems::runtime::agent::AgentMovementSystem;

    #[test]
    fn routine_arrival_starts_duration_and_preserves_walking_target() {
        // Given a rested child, whose only eligible kind is Play.
        let mut engine = fixture();
        let entity = resident(&mut engine, 7);
        engine
            .world
            .insert_one(entity, Child { born_tick: 0 })
            .expect("child");
        // When the decision starts the activity and the movement system reaches its place.
        decide(&mut engine);
        let chosen = *engine
            .world
            .get::<&Activity>(entity)
            .expect("chosen activity");
        assert_eq!(chosen.kind, ActivityKind::Play);
        assert_eq!(chosen.arrived_tick, None);
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("walk target")
                .tile,
            chosen.place
        );
        for _ in 0..4 {
            engine.resources.current_tick += 1;
            AgentMovementSystem::new().tick(&mut engine.world, &mut engine.resources);
            decide(&mut engine);
        }
        // Then arrival, rather than the choice tick, anchors the full duration.
        let reached = *engine
            .world
            .get::<&Activity>(entity)
            .expect("arrived activity");
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Consuming {
                target: TargetKind::Activity
            }
        );
        assert!(reached.arrived_tick.expect("arrival") > chosen.chosen_tick);
        assert_eq!(
            reached.until_tick,
            reached.arrived_tick.map(|tick| tick + 90)
        );
    }

    #[test]
    fn need_interrupts_activity_and_selects_need_in_the_same_tick() {
        // Given a hungry activity holder and an available food source.
        let mut engine = fixture();
        let entity = resident(&mut engine, 9);
        activity(&mut engine, entity, ActivityKind::Craft, true);
        engine
            .world
            .insert_one(entity, Hunger::new(60.0, 0.0))
            .expect("hunger");
        engine.resources.set_food_tile(30, 30, 5);
        // When one decision tick runs.
        decide(&mut engine);
        // Then the unchanged Idle cascade immediately responds, without routine rest.
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::Food
            }
        );
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert!(engine.world.get::<&ActivityRest>(entity).is_err());
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("food target")
                .tile,
            (30, 30)
        );
    }

    #[test]
    fn natural_completion_rests_but_timeout_is_not_completed() {
        // Given one completed activity and one walker that has timed out.
        let mut engine = fixture();
        let complete = resident(&mut engine, 10);
        let timeout = resident(&mut engine, 11);
        activity(&mut engine, complete, ActivityKind::Craft, true);
        activity(&mut engine, timeout, ActivityKind::Explore, false);
        engine
            .world
            .get::<&mut Activity>(complete)
            .expect("activity")
            .until_tick = Some(600);
        engine
            .world
            .get::<&mut Activity>(timeout)
            .expect("activity")
            .chosen_tick = 450;
        // When one decision tick runs.
        decide(&mut engine);
        // Then both rest, and only the arrived normal ending is a completion.
        for (entity, completed) in [(complete, true), (timeout, false)] {
            assert!(engine.world.get::<&Activity>(entity).is_err());
            let rest = engine
                .world
                .get::<&ActivityRest>(entity)
                .expect("rest")
                .until_tick;
            assert!((620..=660).contains(&rest));
            assert_eq!(
                engine
                    .world
                    .get::<&RoutineObservation>(entity)
                    .expect("event")
                    .outcome,
                RoutineOutcome::Ended { completed }
            );
        }
    }

    #[test]
    fn place_occupancy_never_exceeds_two_and_stale_holders_are_removed_first() {
        // Given more children than the first place can hold, plus a stale holder.
        let mut engine = fixture();
        let stale = resident(&mut engine, 1);
        activity(&mut engine, stale, ActivityKind::Play, true);
        engine
            .world
            .insert_one(stale, AgentState::Idle)
            .expect("external interruption");
        for key in 2..40 {
            let entity = resident(&mut engine, key);
            engine
                .world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child");
        }
        // When everyone chooses in the same tick.
        decide(&mut engine);
        // Then reservation counts stay bounded and the stale completion is not overwritten.
        let mut counts = std::collections::BTreeMap::new();
        for (_, activity) in engine.world.query::<&Activity>().iter() {
            *counts.entry(activity.place).or_insert(0) += 1;
        }
        assert!(counts.values().all(|count| *count <= 2));
        assert!(engine.world.get::<&Activity>(stale).is_err());
        assert_eq!(
            engine
                .world
                .get::<&ActivityRest>(stale)
                .expect("one tick rest")
                .until_tick,
            601
        );
    }

    #[test]
    fn social_peers_interrupt_together() {
        // Given two lonely activity holders on the same tile.
        let mut engine = fixture();
        let a = resident(&mut engine, 5);
        let b = resident(&mut engine, 6);
        for entity in [a, b] {
            activity(&mut engine, entity, ActivityKind::HearthCircle, true);
            engine
                .world
                .insert_one(entity, Social::new(60.0, 0.0))
                .expect("social");
        }
        // When the normal social cascade sees activity holders as idle peers.
        decide(&mut engine);
        // Then both select the peer immediately and clear their activities.
        for entity in [a, b] {
            assert!(matches!(
                *engine.world.get::<&AgentState>(entity).expect("state"),
                AgentState::Seeking {
                    target: TargetKind::Agent(_)
                }
            ));
            assert!(engine.world.get::<&Activity>(entity).is_err());
            assert!(engine.world.get::<&SeekTarget>(entity).is_ok());
        }
    }

    #[test]
    fn full_haul_blocks_routine_entry_until_deposit() {
        // Given a loaded resident.
        let mut engine = fixture();
        let loaded = resident(&mut engine, 8);
        // R1: the haul-block check reads the firewood rule's gather window and
        // haul threshold, unchanged from v20 to the baseline.
        engine.resources.rules.society.firewood = WorldRules::baseline(7).society.firewood;
        engine
            .world
            .get::<&mut Inventory>(loaded)
            .expect("inventory")
            .add(ResourceKind::Wood, 10);
        // When the decision runs before the deposit system.
        decide(&mut engine);
        // Then the full haul prevents a new activity.
        assert!(engine.world.get::<&Activity>(loaded).is_err());
    }

    #[test]
    fn play_moves_without_leaving_the_place_neighborhood() {
        // Given an arrived child and a fixed place.
        let mut engine = fixture();
        let entity = resident(&mut engine, 12);
        engine
            .world
            .insert_one(entity, Child { born_tick: 0 })
            .expect("child");
        activity(&mut engine, entity, ActivityKind::Play, true);
        let mut positions = std::collections::BTreeSet::new();
        // When 40 movement ticks run.
        for tick in 600..640 {
            engine.resources.current_tick = tick;
            AgentMovementSystem::new().tick(&mut engine.world, &mut engine.resources);
            let position = engine.world.get::<&Position>(entity).expect("position");
            positions.insert((position.x, position.y));
            assert!(position.x.abs_diff(24).max(position.y.abs_diff(23)) <= 1);
        }
        // Then the child visibly moves while preserving the activity state.
        assert!(positions.len() > 1);
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Consuming {
                target: TargetKind::Activity
            }
        );
    }
}

// From `harness_revitalize_pass17_routine_edges.rs`.
mod edges {
    use super::routine_support::*;
    use sim_core::components::{
        shelter_blueprint, Activity, ActivityKind, ActivityRest, AgentState, Child,
        ConstructionSite, Hunger, Position, RoutineObservation, RoutineOutcome, SeekTarget,
        TargetKind,
    };
    use sim_core::material::TerrainType;
    use sim_core::rng::{keyed_below, STREAM_ROUTINE};
    use sim_core::rules::WorldRules;
    use sim_engine::RuntimeSystem;
    use sim_systems::runtime::settlement::ShelterPlanningSystem;

    #[test]
    fn no_place_rest_prevents_per_tick_retries() {
        // Given a child's entire neighborhood is unwalkable.
        let mut engine = fixture();
        engine
            .resources
            .tile_grid
            .terrain_type
            .fill(TerrainType::River);
        let entity = resident(&mut engine, 1);
        engine
            .world
            .insert_one(entity, Child { born_tick: 0 })
            .expect("child");
        // When two consecutive decision ticks run.
        decide(&mut engine);
        engine.resources.current_tick += 1;
        decide(&mut engine);
        // Then only the original no-place event exists and the pause lasts 30 minutes.
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert_eq!(
            engine
                .world
                .get::<&ActivityRest>(entity)
                .expect("rest")
                .until_tick,
            630
        );
        let observed = *engine
            .world
            .get::<&RoutineObservation>(entity)
            .expect("event");
        assert_eq!(observed.tick, 600);
        assert_eq!(observed.outcome, RoutineOutcome::NoPlace { redrawn: true });
    }

    #[test]
    fn missing_stockpile_redraws_once_into_a_kind_with_a_place() {
        // Given a Craft draw at a camp without a stockpile.
        let mut engine = fixture();
        engine
            .resources
            .rules
            .agent
            .routine
            .as_mut()
            .expect("rule")
            .adult_weights = [0, 1000, 0, 1, 0];
        let key = (0..1000)
            .find(|key| keyed_below(7, STREAM_ROUTINE, *key, 600, 0, 0, 1001) < 1000)
            .expect("craft key");
        let entity = resident(&mut engine, key);
        // When the one permitted redraw runs.
        decide(&mut engine);
        // Then the remaining weighted kind is selected and the redraw is observable.
        assert_eq!(
            engine
                .world
                .get::<&Activity>(entity)
                .expect("activity")
                .kind,
            ActivityKind::HearthCircle
        );
        assert_eq!(
            engine
                .world
                .get::<&RoutineObservation>(entity)
                .expect("event")
                .outcome,
            RoutineOutcome::Started { redrawn: true }
        );
    }

    #[test]
    fn unavailable_foraging_does_not_interrupt_until_claims_allow_a_real_tile() {
        // Given a below-target store but every forage bush already has its full claims.
        let mut engine = fixture();
        // R1: the live forage and store-target rules (forage is exact since v11;
        // `max_foragers_per_bush` is 2 and read from the rule below).
        let rules = WorldRules::baseline(7);
        engine.resources.rules.agent.forage = rules.agent.forage;
        engine.resources.rules.society.stock_target = rules.society.stock_target;
        let entity = resident(&mut engine, 2);
        engine
            .world
            .insert_one(entity, Hunger::new(0.0, 0.05))
            .expect("hunger");
        activity(&mut engine, entity, ActivityKind::TendFire, true);
        engine.resources.set_food_tile(34, 24, 8);
        let cap = engine
            .resources
            .rules
            .agent
            .forage
            .expect("forage")
            .max_foragers_per_bush;
        let blockers: Vec<_> = (0..cap)
            .map(|_| {
                let blocker = engine.spawn_agent(0, 0);
                engine
                    .world
                    .insert(
                        blocker,
                        (
                            AgentState::Seeking {
                                target: TargetKind::GatherFood,
                            },
                            SeekTarget { tile: (34, 24) },
                        ),
                    )
                    .expect("claim");
                blocker
            })
            .collect();
        // When the first decision has no available tile, then claims are released for the next tick.
        decide(&mut engine);
        assert!(engine.world.get::<&Activity>(entity).is_ok());
        for blocker in blockers {
            engine
                .world
                .insert_one(
                    blocker,
                    AgentState::Consuming {
                        target: TargetKind::Sleep,
                    },
                )
                .expect("release claim");
        }
        engine.resources.current_tick += 1;
        decide(&mut engine);
        // Then the real work predicate interrupts and immediately starts the forage trip.
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::GatherFood
            }
        );
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("forage target")
                .tile,
            (34, 24)
        );
    }

    #[test]
    fn builders_are_recruited_from_activities_and_respond_in_the_same_tick() {
        // Given an active worker and an owned unfinished shelter.
        let mut engine = fixture();
        let entity = resident(&mut engine, 3);
        activity(&mut engine, entity, ActivityKind::Repair, true);
        // R1 exact: the live shelter-planning rule (unchanged from v20).
        engine.resources.rules.society.shelter_planning =
            WorldRules::baseline(7).society.shelter_planning;
        engine.world.spawn((ConstructionSite::new(
            shelter_blueprint(240),
            Position::new(26, 24),
        ),));
        engine.resources.site_owner.insert((26, 24), 1);
        // When the builder pool and decision cascade run.
        decide(&mut engine);
        // Then the worker walks to the construction site without an intermediate idle tick.
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            }
        );
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("site target")
                .tile,
            (26, 24)
        );
    }

    #[test]
    fn membership_change_aborts_and_day_end_completes_only_arrived_activities() {
        // Given three activity holders with distinct exit conditions.
        let mut engine = fixture();
        let changed = resident(&mut engine, 4);
        let arrived = resident(&mut engine, 5);
        let walking = resident(&mut engine, 6);
        for (entity, at_place) in [(changed, true), (arrived, true), (walking, false)] {
            activity(&mut engine, entity, ActivityKind::Explore, at_place);
            engine
                .world
                .get::<&mut Activity>(entity)
                .expect("activity")
                .until_tick = Some(1500);
        }
        engine
            .world
            .get::<&mut Activity>(changed)
            .expect("activity")
            .settlement = 2;
        // Under the live routine's `end_stagger`, keys 4–6 (key / 60 = 0) with
        // P 500 end their day at 1065 + 0 + 0 = 17:45, so 18:00 is past every end.
        let rule = engine.resources.rules.agent.routine.expect("live routine");
        assert!(rule.end_stagger.is_some());
        assert!((4..=6).all(|key| rule.end_minute(key, None) <= 1080));
        engine.resources.current_tick = 1080;
        // When the day closes.
        decide(&mut engine);
        // Then all leave and rest, but only arrival followed by a normal end is completed.
        for (entity, completed) in [(changed, false), (arrived, true), (walking, false)] {
            assert!(engine.world.get::<&Activity>(entity).is_err());
            assert!(engine.world.get::<&ActivityRest>(entity).is_ok());
            assert_eq!(
                engine
                    .world
                    .get::<&RoutineObservation>(entity)
                    .expect("event")
                    .outcome,
                RoutineOutcome::Ended { completed }
            );
        }
    }

    #[test]
    fn planned_shelter_is_built_by_a_worker_recruited_from_an_activity() {
        // Given one resident on an activity in a settlement that owns no housing,
        // under the live shelter-planning rule (one pending site per settlement,
        // headroom 2, rings 2–5, every 60 ticks).
        let mut engine = fixture();
        let rule = WorldRules::baseline(7)
            .society
            .shelter_planning
            .expect("live planning");
        engine.resources.rules.society.shelter_planning = Some(rule);
        let entity = resident(&mut engine, 3);
        activity(&mut engine, entity, ActivityKind::Repair, true);
        // When the planner runs on a planning tick (600 is a multiple of 60) and
        // again an interval later.
        let mut planner = ShelterPlanningSystem::default();
        planner.tick(&mut engine.world, &mut engine.resources);
        engine.resources.current_tick += rule.plan_interval_ticks;
        planner.tick(&mut engine.world, &mut engine.resources);
        // Then exactly one site exists (1 member + headroom 2 >= capacity 0 plans
        // one; a pending site blocks a second), owned by the settlement, on the
        // innermost ring around the hearth.
        let sites: Vec<_> = engine
            .world
            .query::<&ConstructionSite>()
            .iter()
            .map(|(_, site)| (site.position.x, site.position.y))
            .collect();
        assert_eq!(sites.len(), 1, "sites {sites:?}");
        let tile = sites[0];
        assert_eq!(engine.resources.site_owner.get(&tile), Some(&1));
        assert_eq!(tile.0.abs_diff(24).max(tile.1.abs_diff(24)), rule.ring_min);
        assert_eq!(planner.skipped_no_tile, 0);
        // And the decision cascade recruits the active worker straight to it.
        decide(&mut engine);
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            }
        );
        assert_eq!(
            engine.world.get::<&SeekTarget>(entity).expect("site").tile,
            tile
        );
    }
}

// From `harness_revitalize_pass17_routine_invitations.rs`.
mod invitations {
    use super::routine_support::*;
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
    fn invitation_replays_earlier_peers_real_combat_response_without_interrupting_uninvited_peers()
    {
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
}

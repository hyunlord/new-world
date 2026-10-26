//! Read-only selector beside the real legacy loop; never runs lifecycle effects.
//! Running IDs are observer-only; factual eligibility invalidates unfinished choices.
//! No finish/restart transition equivalence is claimed without pure completion hooks.
mod compare;
use super::{
    context::{Domain, PersonUpdate, RunningMetadata, TickContext},
    facts, MindProgram,
};
use crate::runtime::agent_actions::routine::RoutineTick;
pub use compare::{Counts, Example, Prediction, Summary};
use sim_core::{
    colony::mind::{choose, EvalKey, EvalScratch, PersonFacts, RunningChoice},
    components::{AgentId, AgentState, TargetKind},
};
use sim_engine::SimResources;
use std::{cell::RefCell, collections::BTreeMap};

struct Session {
    program: MindProgram,
    running: BTreeMap<AgentId, RunningChoice>,
    predictions: BTreeMap<AgentId, Prediction>,
    scratch: EvalScratch,
    summary: Summary,
}
thread_local! { static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) }; }
/// Start once per world/thread with the exact authored policy; no effects run.
pub fn begin(source: &str) -> Result<(), String> {
    let program = MindProgram::load(source).map_err(|e| e.to_string())?;
    SESSION.with(|slot| {
        if slot.borrow().is_some() {
            return Err("mind shadow session already active".into());
        }
        *slot.borrow_mut() = Some(Session {
            program,
            running: BTreeMap::new(),
            predictions: BTreeMap::new(),
            scratch: EvalScratch::default(),
            summary: Summary::default(),
        });
        crate::mind_observer::begin();
        Ok(())
    })
}
/// Whether this thread has an explicitly enabled shadow session.
pub fn active() -> bool {
    SESSION.with(|s| s.borrow().is_some())
}
/// Finish after all complete ticks; errors remain in the returned report.
pub fn finish() -> Result<Summary, String> {
    crate::mind_observer::end();
    SESSION.with(|s| {
        s.borrow_mut()
            .take()
            .map(|s| s.summary)
            .ok_or("no mind shadow session".into())
    })
}
/// One columnar table/context per real decision tick, outside simulation state.
pub(in crate::runtime) struct ShadowTick {
    context: TickContext,
    table: PersonFacts,
    source_version: u64,
    claims_version: u64,
}
/// Called after owner routine preparation and before borrowing the legacy query.
pub(in crate::runtime) fn prepare(
    world: &hecs::World,
    resources: &SimResources,
    routine: Option<&RoutineTick>,
) -> Option<ShadowTick> {
    SESSION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let session = slot.as_mut()?;
        let result = (|| {
            let mut context =
                facts::prepare_context(world, resources).map_err(|e| e.to_string())?;
            let agents: Vec<_> = context.rows().iter().map(|p| p.agent).collect();
            for agent in agents {
                observe_running(&mut context, agent, &session.program)?;
                facts::refresh_routine(&mut context, routine, agent).map_err(|e| e.to_string())?;
            }
            let table = session
                .program
                .providers
                .build(&session.program.policy, &context, resources)
                .map_err(|e| e.to_string())?;
            Ok::<_, String>(ShadowTick {
                context,
                table,
                source_version: 0,
                claims_version: u64::MAX,
            })
        })();
        match result {
            Ok(tick) => Some(tick),
            Err(error) => {
                session
                    .summary
                    .error(format!("prepare tick {}: {error}", resources.current_tick));
                None
            }
        }
    })
}
/// Evaluate immediately before this person's old effects, using its live query row.
pub(in crate::runtime) fn evaluate(
    tick: &mut ShadowTick,
    resources: &SimResources,
    routine: Option<&RoutineTick>,
    agent: AgentId,
    update: PersonUpdate,
    claims: &BTreeMap<(u32, u32), u32>,
    versions: (u64, u64),
) {
    let (claims_version, source_version) = versions;
    SESSION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(session) = slot.as_mut() else {
            return;
        };
        session.summary.prediction_evaluations += 1;
        let result = (|| {
            // The legacy loop consumes pending invitations immediately after this
            // observation point, before choosing an action. Preview only its row
            // normalization; the real transaction still owns consumption/effects.
            let mut update = update;
            if routine.is_some_and(|routine| routine.invitation_pending(agent)) {
                update.state = AgentState::Idle;
            }
            tick.context
                .patch_person(agent, update)
                .map_err(|e| e.to_string())?;
            if tick.claims_version != claims_version {
                tick.context
                    .replace_claims(claims.clone())
                    .map_err(|e| e.to_string())?;
                tick.claims_version = claims_version;
            }
            if tick.source_version != source_version {
                tick.context
                    .invalidate(Domain::FoodSources)
                    .map_err(|e| e.to_string())?;
                tick.source_version = source_version;
            }
            observe_running(&mut tick.context, agent, &session.program)?;
            facts::refresh_routine(&mut tick.context, routine, agent).map_err(|e| e.to_string())?;
            session
                .program
                .providers
                .refresh_row(&mut tick.table, &tick.context, resources, agent)
                .map_err(|e| e.to_string())?;
            let key = EvalKey {
                world_seed: resources.world_seed,
                agent,
                tick: resources.current_tick,
                replicate: resources.noise_replicate_for(resources.current_tick),
            };
            let selected = choose(
                &session.program.policy,
                tick.table.row(agent).map_err(|e| e.to_string())?,
                session.running.get(&agent).copied(),
                key,
                &mut session.scratch,
            )
            .map_err(|e| e.to_string())?;
            let node = session
                .program
                .policy
                .node(selected.selected.node)
                .ok_or("selected node absent")?;
            let behaviour = session
                .program
                .policy
                .behaviour(node.behaviour)
                .ok_or("selected behaviour absent")?;
            let band = session
                .program
                .policy
                .band_name(node.band)
                .ok_or("selected band absent")?;
            let prediction = Prediction {
                node: node.name.clone(),
                behaviour: behaviour.name.clone(),
                band: band.into(),
                score: selected.selected.score,
                routine_entry_draw: node.tags.iter().any(|tag| tag == "routine_entry_draw"),
                facts: session
                    .scratch
                    .witnesses()
                    .iter()
                    .map(|w| {
                        (
                            session.program.policy.facts()[w.fact.index()].name.clone(),
                            w.value,
                        )
                    })
                    .collect(),
            };
            session.running.insert(
                agent,
                RunningChoice {
                    revision: session.program.policy.revision(),
                    node: node.id,
                    finished: false,
                },
            );
            Ok::<_, String>(prediction)
        })();
        match result {
            Ok(prediction) => {
                session.predictions.insert(agent, prediction);
            }
            Err(error) => {
                session.predictions.remove(&agent);
                session.summary.error(format!(
                    "evaluate tick {} agent {agent}: {error}",
                    resources.current_tick
                ));
            }
        }
    });
}
/// Compare unique finalized old rows after replay, settlement and the full tick.
pub(in crate::runtime) fn complete_tick() {
    SESSION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(session) = slot.as_mut() else {
            return;
        };
        for row in crate::mind_observer::drain() {
            let prediction = session.predictions.remove(&row.agent);
            if row.final_choice.exit.is_some() {
                if let Some(running) = session.running.get_mut(&row.agent) {
                    let matches_dispatch = prediction.as_ref().is_some_and(|p| {
                        p.behaviour == compare::observed_behaviour(row.final_choice)
                    });
                    if matches_dispatch {
                        running.finished = true;
                    }
                }
            }
            session.summary.compare(row, prediction);
        }
        session.summary.unmatched_predictions += session.predictions.len() as u64;
        session.predictions.clear();
    });
}

// Descriptive old ownership is independent from the predicted policy incumbent.
fn observe_running(
    context: &mut TickContext,
    agent: AgentId,
    program: &MindProgram,
) -> Result<(), String> {
    let prior = crate::mind_observer::previous_choice(agent);
    let running = match prior {
        Some(choice) if choice.verdict == crate::mind_observer::Verdict::Uncovered => {
            return Err(format!(
                "unknown prior old ownership for agent {agent}: {}",
                choice.arm
            ));
        }
        Some(choice) if choice.behaviour != "no_choice" => Some(RunningMetadata {
            behaviour: compare::observed_behaviour(choice).into(),
            target: None,
            finished: choice.exit.is_some(),
        }),
        _ => {
            if matches!(
                context.row(agent).map_err(|e| e.to_string())?.state,
                AgentState::Seeking {
                    target: TargetKind::GatherFood
                } | AgentState::Consuming {
                    target: TargetKind::GatherFood
                }
            ) {
                return Err(format!(
                    "unknown historical gather origin for agent {agent}"
                ));
            }
            None
        }
    };
    context
        .set_observed_running(agent, running, &program.policy)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod invitation_tests {
    use super::*;
    use crate::runtime::agent_actions::routine::PlacesCache;
    use sim_core::components::{Activity, ActivityKind, Agent, Hunger, Position};

    #[test]
    fn pending_invitation_is_previewed_as_idle_without_consuming_it() {
        const POLICY: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
        (id:"invited_idle",band:"need",behaviour:"rest",when:[(fact:"idle",op:"has")],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"REST"),
        (id:"fallback",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
        let mut world = hecs::World::new();
        let state = AgentState::Seeking {
            target: TargetKind::Activity,
        };
        let entity = world.spawn((
            Agent { id: 2 },
            Position::new(1, 1),
            state,
            Hunger::new(0.0, 0.0),
            Activity {
                kind: ActivityKind::Play,
                settlement: 1,
                place: (2, 1),
                chosen_tick: 0,
                arrived_tick: None,
                until_tick: None,
            },
        ));
        let mut resources = SimResources::new(5, 5, sim_core::MaterialRegistry::new());
        resources.rules = sim_core::rules::WorldRules::colony(1);
        let mut routine = RoutineTick::new(
            &mut world,
            &resources,
            resources.rules.agent.routine.expect("routine"),
            PlacesCache::default(),
        );
        routine.prepare_preview_places(&resources);
        begin(POLICY).expect("shadow session");
        let mut tick = prepare(&world, &resources, Some(&routine)).expect("snapshot");
        routine.invite(2, &world);
        assert!(routine.invitation_pending(2));
        evaluate(
            &mut tick,
            &resources,
            Some(&routine),
            2,
            PersonUpdate {
                position: Position::new(1, 1),
                state,
                hunger: Some(0.0),
                thirst: None,
                fatigue: None,
                social: None,
                inventory_total: None,
                inventory_food: None,
            },
            &BTreeMap::new(),
            (0, 0),
        );
        let predicted = SESSION.with(|slot| {
            slot.borrow()
                .as_ref()
                .expect("session")
                .predictions
                .get(&2)
                .map(|p| p.node.clone())
        });
        let summary = finish().expect("finish session");
        assert_eq!(summary.error_count, 0);
        assert!(
            routine.invitation_pending(2),
            "preview must not consume the real invitation"
        );
        assert_eq!(*world.get::<&AgentState>(entity).expect("state"), state);
        assert_eq!(predicted.as_deref(), Some("invited_idle"));
    }
}

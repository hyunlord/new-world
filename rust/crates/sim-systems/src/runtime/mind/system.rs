//! Real tick coordinator: factual rows, pure selection, then registered effects.

use super::{
    behaviours::{
        self, ActionTarget, EffectContext, EffectDomain, EffectState, EffectStatus, FinishReason,
    },
    context::{Domain, TickContext},
    facts, record, MindProgram,
};
use crate::runtime::agent_actions::{
    needs,
    routine::{PlacesCache, RoutineTick},
};
use hecs::World;
use sim_core::colony::mind::{choose_with_reader, EvalKey, EvalScratch, FinishMode, RunningChoice};
use sim_core::colony::mind_records::MindHistory;
use sim_core::components::{AgentId, AgentState, SeekTarget};
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeSet;
use std::fmt;

/// A broken factual, policy, persistence or effect invariant aborts the real tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindTickError(pub String);
impl fmt::Display for MindTickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for MindTickError {}
fn failure(error: impl fmt::Display) -> MindTickError {
    MindTickError(error.to_string())
}

/// Unregistered until the explicit switch; all persistent choices live in resources.
pub struct ColonyMindSystem {
    harvest_food: super::harvest_food::PlantCache,
    program: MindProgram,
    places: PlacesCache,
    scratch: EvalScratch,
    fact_values: Vec<Option<i64>>,
    initialized: bool,
    #[cfg(test)]
    reference_full_refresh: bool,
    #[cfg(test)]
    refresh_visits: [usize; 3],
    #[cfg(test)]
    snapshot_refresh_visits: [usize; 2],
    diagnostics: Option<super::scheduling::Diagnostics>,
    continuation: super::scheduling::ContinuationGate,
}
impl ColonyMindSystem {
    /// Construct from a policy already resolved against the actual registries.
    pub fn new(program: MindProgram) -> Self {
        let diagnostics = (std::env::var_os("WORLDSIM_MIND_CONTINUATION_DIAGNOSTICS").as_deref()
            == Some(std::ffi::OsStr::new("1")))
        .then(|| super::scheduling::Diagnostics::new(&program.policy));
        let continuation = super::scheduling::ContinuationGate::new(&program.policy);
        Self {
            harvest_food: super::harvest_food::PlantCache::default(),
            continuation,
            diagnostics,
            program,
            places: PlacesCache::default(),
            scratch: EvalScratch::default(),
            fact_values: Vec::new(),
            initialized: false,
            #[cfg(test)]
            reference_full_refresh: false,
            #[cfg(test)]
            refresh_visits: [0; 3],
            #[cfg(test)]
            snapshot_refresh_visits: [0; 2],
        }
    }

    /// Install or validate persistent policy state before commands are admitted.
    pub(crate) fn initialize(&mut self, resources: &mut SimResources) -> Result<(), MindTickError> {
        if self.initialized {
            return resources
                .mind
                .history
                .as_mut()
                .ok_or_else(|| failure("initialized mind lacks history"))?
                .check_initialized_policy(&self.program.policy)
                .map_err(failure);
        }
        let state = &mut resources.mind;
        match state.policy_source.as_deref() {
            Some(source) if source != self.program.source() => {
                return Err(failure("runtime and saved mind sources differ"))
            }
            Some(_) => {
                state
                    .history
                    .as_mut()
                    .ok_or_else(|| failure("saved mind lacks history"))?
                    .check_initialized_policy(&self.program.policy)
                    .map_err(failure)?;
            }
            None => {
                if !state.running.is_empty()
                    || state.history.is_some()
                    || !state.fact_origins.is_empty()
                    || !state.fact_source_events.is_empty()
                    || !state.order_origins.is_empty()
                    || !state.job_origins.is_empty()
                    || !state.draft_origins.is_empty()
                    || resources.causal_log.iter().any(|(_, log)| {
                        log.iter().any(|event| {
                            matches!(
                                event,
                                sim_core::causal::CausalEvent::MindChoice { .. }
                                    | sim_core::causal::CausalEvent::MindFactOrigin { .. }
                            )
                        })
                    })
                {
                    return Err(failure("nonempty mind lacks authored source"));
                }
                state.history = Some(MindHistory::new(&self.program.policy).map_err(failure)?);
                state.policy_source = Some(self.program.source().to_owned());
            }
        }
        self.initialized = true;
        Ok(())
    }

    /// Execute one real transaction. Errors propagate; no alternate selector is used.
    pub fn try_tick(
        &mut self,
        world: &mut World,
        resources: &mut SimResources,
    ) -> Result<(), MindTickError> {
        self.initialize(resources)?;
        if resources.tile_grid.width == 0 || resources.tile_grid.height == 0 {
            return Ok(());
        }
        let mut routine =
            resources.rules.agent.routine.map(|rule| {
                RoutineTick::new(world, resources, rule, std::mem::take(&mut self.places))
            });
        if let Some(transaction) = routine.as_mut() {
            transaction.prepare_preview_places(resources);
        }
        let mut effects = EffectState::prepare(world, resources);
        let mut context = facts::prepare_continuation_context_with_claims(
            world,
            resources,
            effects.claims.clone(),
            Some(&effects.forage_plans),
        )
        .map_err(failure)?;
        *context.harvest_food.borrow_mut() = Some(self.harvest_food.snapshot(resources));
        let mut pending: Vec<_> = context.rows().iter().map(|row| row.agent).collect();
        let mut completed = BTreeSet::new();
        let mut replay_visit = false;
        loop {
            for agent in pending {
                // A completed consumption/activity keeps the historical one-tick Idle boundary.
                if completed.contains(&agent) {
                    continue;
                }
                *context.plant_query_observation.borrow_mut() = None;
                self.person(
                    world,
                    resources,
                    &mut context,
                    &mut effects,
                    &mut routine,
                    agent,
                    &mut completed,
                    replay_visit,
                )?;
                let observed = context.plant_query_observation.borrow_mut().take();
                let ate_now = observed == Some((agent, true))
                    && resources
                        .food
                        .meals
                        .get(&agent)
                        .and_then(|history| history.last_meals.last())
                        .is_some_and(|meal| meal.tick == resources.current_tick);
                if observed == Some((agent, true)) && !ate_now {
                    let tick = resources.current_tick;
                    crate::runtime::food::evidence::record_failure(
                        resources,
                        agent,
                        sim_core::colony::food_evidence::AcquisitionReceipt {
                            tick,
                            target: None,
                            job_started: None,
                            reason:
                                sim_core::colony::food_evidence::AcquisitionReason::NoRipeInRegion,
                        },
                    );
                }
            }
            let Some(transaction) = routine.as_mut() else {
                break;
            };
            let replay = transaction.replay(world);
            if replay.is_empty() {
                break;
            }
            context.invalidate(Domain::Invitations).map_err(failure)?;
            context
                .invalidate(Domain::RoutineOccupancy)
                .map_err(failure)?;
            replay_visit = true;
            pending = replay
                .into_iter()
                .map(|(agent, entity)| {
                    effects.seek_clear.push(entity);
                    agent
                })
                .collect();
            pending.sort_unstable();
            pending.dedup();
        }
        behaviours::finish_tick(world, resources, &mut effects, routine.as_ref());
        for (entity, (state, seek)) in &effects.external_states {
            restore_external(world, *entity, *state, *seek)?;
        }
        if let Some(transaction) = routine {
            self.places = transaction.finish(world);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn person(
        &mut self,
        world: &mut World,
        resources: &mut SimResources,
        context: &mut TickContext,
        effects: &mut EffectState,
        routine: &mut Option<RoutineTick>,
        agent: AgentId,
        completed: &mut BTreeSet<AgentId>,
        replay_visit: bool,
    ) -> Result<(), MindTickError> {
        // Match the legacy per-person invitation boundary: the reservation has
        // already ended, but its deferred Activity/FSM cleanup is not yet applied.
        let invited = routine
            .as_mut()
            .is_some_and(|transaction| transaction.take_invitation(agent));
        context.begin_visit(agent).map_err(failure)?;
        if invited {
            let entity = context.row(agent).map_err(failure)?.entity;
            *world.get::<&mut AgentState>(entity).map_err(failure)? = AgentState::Idle;
            effects.seek_clear.push(entity);
        }
        let primary_refresh = !replay_visit && !invited;
        #[cfg(test)]
        {
            self.refresh_visits[if replay_visit {
                2
            } else if invited {
                1
            } else {
                0
            }] += 1;
        }
        #[cfg(test)]
        let primary_refresh = primary_refresh && !self.reference_full_refresh;
        if primary_refresh {
            let acknowledged = context
                .acknowledge_primary_capture(agent)
                .map_err(failure)?;
            #[cfg(test)]
            {
                self.snapshot_refresh_visits[usize::from(!acknowledged)] += 1;
            }
            if acknowledged {
                Ok(())
            } else {
                context.refresh_primary_continuation_exclusive(world, agent)
            }
        } else {
            context.refresh_continuation_exclusive(world, agent)
        }
        .map_err(failure)?;
        let prior = resources.mind.running.get(&agent).copied();
        context
            .set_program_running(agent, prior, &self.program)
            .map_err(failure)?;
        facts::refresh_routine_lifecycle(context, routine.as_ref(), agent).map_err(failure)?;
        let proposed = self.diagnostics.as_mut().and_then(|diagnostics| {
            diagnostics.observe(
                &self.program,
                world,
                resources,
                context,
                agent,
                prior,
                replay_visit || invited,
            )
        });
        let person = context.row(agent).map_err(failure)?;
        let seek = world
            .get::<&SeekTarget>(person.entity)
            .ok()
            .map(|value| *value);
        let latest = resources
            .mind
            .history
            .as_ref()
            .and_then(|history| history.latest(agent));
        let prior_cause = latest.map(|choice| choice.id());
        let history_matches = prior.is_some_and(|running| {
            latest.is_some_and(|choice| choice.matches_running(agent, running).unwrap_or(false))
        });
        let evidence = super::scheduling::ContinuationEvidence {
            state: person.state,
            activity: person.activity,
            seek,
            history_matches,
            replay_or_invitation: replay_visit || invited,
        };
        // Unsupported or extra guard-read errors fall back to the unchanged selector.
        let retained = if resources.rules.colony.is_some() {
            match self.continuation.evaluate(
                &self.program,
                context,
                resources,
                agent,
                prior,
                evidence,
            ) {
                Ok(sim_core::colony::mind::GuardOutcome::ContinueOwned { node }) => Some(node),
                Ok(sim_core::colony::mind::GuardOutcome::FullEvaluation { .. }) | Err(_) => None,
            }
        } else {
            None
        };
        let selected = if retained.is_none() {
            context
                .upgrade_continuation_exclusive(world, agent)
                .map_err(failure)?;
            facts::refresh_routine_lifecycle(context, routine.as_ref(), agent).map_err(failure)?;
            let mut row = self
                .program
                .providers
                .lazy_row(
                    context,
                    resources,
                    routine.as_ref(),
                    agent,
                    &mut self.fact_values,
                )
                .map_err(failure)?;
            let selected = choose_with_reader(
                &self.program.policy,
                &mut row,
                prior,
                EvalKey {
                    world_seed: resources.world_seed,
                    agent,
                    tick: resources.current_tick,
                    replicate: resources.noise_replicate_for(resources.current_tick),
                },
                &mut self.scratch,
            )
            .map_err(failure)?;
            Some(selected)
        } else {
            None
        };
        if let (Some(diagnostics), Some(selected)) = (self.diagnostics.as_mut(), selected.as_ref())
        {
            diagnostics.compare(&self.program.policy, proposed, selected.selected.node);
        }
        let node_id = retained
            .or_else(|| selected.as_ref().map(|value| value.selected.node))
            .ok_or_else(|| failure("missing continuation or selection"))?;
        let node = self
            .program
            .policy
            .node(node_id)
            .ok_or_else(|| failure("selected node missing"))?;
        let effect = self.program.effect(node.id).map_err(failure)?;
        let target = if retained.is_some()
            && !(resources.rules.colony.is_some()
                && matches!(
                    effect.lifecycle,
                    behaviours::LifecycleBinding::Resource(sim_core::components::TargetKind::Food)
                )) {
            super::scheduling::owned_target(
                effect.lifecycle,
                context.row(agent).map_err(failure)?,
                context,
                seek,
            )
            .ok_or_else(|| failure("retained lifecycle target missing"))?
        } else {
            let candidate = match node.target.as_deref() {
                Some(name) => {
                    if name.starts_with("routine_") && context.target(agent, name).is_err() {
                        facts::prepare_routine_fact(
                            context,
                            routine.as_ref(),
                            agent,
                            &format!("{name}_available"),
                        )
                        .map_err(failure)?;
                    }
                    facts::resolve_target(context, resources, agent, name).map_err(failure)?
                }
                None => None,
            };
            let target = effect
                .action_target(candidate)
                .map_err(|error| failure(format!("target: {error:?}")))?;
            target
        };
        let person = context.row(agent).map_err(failure)?;
        let scope = context.target_scope(agent).map_err(failure)?;
        let external = effect.externally_owned && node.finish.is_none();
        let owner_snapshot = external.then(|| {
            (
                person.state,
                world
                    .get::<&SeekTarget>(person.entity)
                    .ok()
                    .map(|target| *target),
            )
        });
        let day_tick = (resources.ticks_per_day != 0).then(|| {
            resources
                .current_tick
                .saturating_add(resources.rules.clock_offset_ticks)
                % resources.ticks_per_day
        });
        let night_rest = needs::person_night_rest(
            resources.rules.agent.night_rest,
            resources.rules.agent.temperament,
            person.temperament.as_ref(),
            day_tick,
        );
        let availability = needs::NeedAvailability {
            food: scope.food_available,
            ..context.availability
        };
        let handshake = context
            .derived(agent, "social_handshake")
            .map_err(failure)?
            == 1;
        if node.finish.is_some()
            && prior
                .filter(|choice| !choice.finished)
                .and_then(|choice| self.program.policy.node(choice.node))
                .is_some_and(|old| old.behaviour != node.behaviour)
        {
            return Err(failure("finish action does not match running lifecycle"));
        }
        let cause = if let Some(selected) = selected.as_ref() {
            let origins =
                if record::is_unfinished_continuation(&self.program.policy, prior, selected) {
                    Vec::new()
                } else {
                    let mut origins = record::automatic_origins(
                        resources,
                        &self.program.policy,
                        agent,
                        person.fleeing,
                        self.scratch.witnesses(),
                    );
                    origins.extend(record::memory_origins(
                        &self.program.policy,
                        selected,
                        &person.memory_causes,
                        self.scratch.witnesses(),
                    ));
                    let drafted = world
                        .get::<&sim_core::colony::people::Drafted>(person.entity)
                        .ok()
                        .map(|value| *value);
                    origins.extend(record::draft_origins(
                        resources,
                        &self.program.policy,
                        agent,
                        drafted,
                        self.scratch.witnesses(),
                    ));
                    origins.extend(record::assigned_job_origins(
                        resources,
                        &self.program.policy,
                        agent,
                        person.job,
                        self.scratch.witnesses(),
                    ));
                    origins
                };
            let cause = record::record_selection(
                resources,
                &self.program.policy,
                record::RecordPerson {
                    agent,
                    position: (person.position.x, person.position.y),
                    settlement: scope.settlement,
                },
                selected,
                self.scratch.witnesses(),
                &origins,
            )
            .map_err(failure)?;
            cause
        } else {
            None
        };
        let cause = if retained.is_some() {
            prior_cause
        } else {
            cause.or_else(|| {
                resources
                    .mind
                    .history
                    .as_ref()
                    .and_then(|history| history.latest(agent))
                    .map(|choice| choice.id())
            })
        };
        let previous = prior
            .filter(|choice| !choice.finished)
            .map(|choice| self.program.effect(choice.node))
            .transpose()
            .map_err(failure)?;
        let adopting = prior.is_none() && effect.adopts(person.state, person.activity, target);
        let continuing = adopting
            || prior
                .filter(|choice| !choice.finished)
                .and_then(|choice| self.program.policy.node(choice.node))
                .is_some_and(|old| {
                    old.behaviour == node.behaviour
                        && effect.adopts(person.state, person.activity, target)
                });
        let mut call = EffectContext {
            world,
            resources,
            effects,
            routine: routine.as_mut(),
            entity: person.entity,
            agent,
            position: person.position,
            target,
            availability,
            night_rest,
            interrupted: false,
            authored_completion: node.finish.is_some(),
            social_handshake: handshake,
            cause,
            lineage_key: person.lineage,
            member: scope.settlement,
            node_random_key: node.random_key,
            finish_reason: FinishReason::Interrupted,
        };
        if prior.is_none() && !adopting && node.finish.is_none() {
            if person.activity.is_some() {
                if let Some(transaction) = call.routine.as_deref_mut() {
                    transaction.interrupt(agent, call.entity);
                    call.effects.dirty.insert(EffectDomain::RoutineOccupancy);
                }
            }
            call.resources.social_chase_since.remove(&agent);
            call.clear();
        }
        if !continuing && node.finish.is_none() {
            if let Some(previous) = previous {
                call.target = ActionTarget::None;
                call.interrupted = true;
                (previous.finish)(&mut call)
                    .map_err(|error| failure(format!("finish: {error:?}")))?;
                call.target = target;
                call.interrupted = false;
            }
        }
        if let Some((state, seek)) = owner_snapshot {
            restore_external(call.world, call.entity, state, seek)?;
            call.effects
                .seek_clear
                .retain(|entity| *entity != call.entity);
            call.effects
                .seek_idle
                .retain(|entity| *entity != call.entity);
            call.effects
                .seek_set
                .retain(|(entity, _)| *entity != call.entity);
            call.effects
                .job_releases
                .retain(|entity| *entity != call.entity);
            call.effects
                .external_states
                .insert(call.entity, (state, seek));
        } else {
            call.effects.external_states.remove(&call.entity);
            if call.effects.previous_jobs.contains_key(&call.entity) {
                call.effects.job_releases.push(call.entity);
            }
        }
        let status = match node.finish {
            Some(mode) => {
                call.finish_reason = match mode {
                    FinishMode::Completed => FinishReason::Completed,
                    FinishMode::Interrupted => FinishReason::Interrupted,
                };
                call.interrupted = mode == FinishMode::Interrupted;
                (effect.finish)(&mut call)
            }
            None if continuing => (effect.continue_running)(&mut call),
            None => (effect.start)(&mut call),
        }
        .map_err(|error| failure(format!("effect {}: {error:?}", effect.name)))?;
        for eater in std::mem::take(&mut call.effects.completed_meals) {
            let choice_id = call
                .resources
                .mind
                .history
                .as_ref()
                .and_then(|history| history.latest(eater))
                .map(|choice| choice.id())
                .ok_or_else(|| failure("completed meal lacks initiating mind choice"))?;
            record::remember_completed_meal(call.resources, &self.program.policy, eater, choice_id)
                .map_err(failure)?;
        }
        if node.finish.is_some() && status != EffectStatus::Finished {
            return Err(failure("explicit finish action did not finish lifecycle"));
        }
        let finished = status == EffectStatus::Finished;
        call.resources.mind.running.insert(
            agent,
            RunningChoice {
                revision: self.program.policy.revision(),
                node: node.id,
                finished,
            },
        );
        if finished {
            completed.insert(agent);
        }
        invalidate(context, call.effects, call.resources)?;
        Ok(())
    }
}

fn restore_external(
    world: &mut World,
    entity: hecs::Entity,
    state: AgentState,
    seek: Option<SeekTarget>,
) -> Result<(), MindTickError> {
    *world.get::<&mut AgentState>(entity).map_err(failure)? = state;
    match seek {
        Some(target) => world.insert_one(entity, target).map_err(failure)?,
        None => {
            let _ = world.remove_one::<SeekTarget>(entity);
        }
    }
    Ok(())
}

fn invalidate(
    context: &mut TickContext,
    effects: &mut EffectState,
    resources: &SimResources,
) -> Result<(), MindTickError> {
    let dirty = std::mem::take(&mut effects.dirty);
    let mut food_invalidated = false;
    for domain in dirty.iter().copied() {
        match domain {
            EffectDomain::Needs | EffectDomain::Targets | EffectDomain::Relations => {
                context.invalidate(Domain::Person)
            }
            EffectDomain::FoodSources | EffectDomain::FoodTile(_) => {
                if food_invalidated {
                    continue;
                }
                food_invalidated = true;
                if dirty.contains(&EffectDomain::FoodSources) {
                    context.invalidate(Domain::FoodSources)
                } else {
                    context.invalidate_food_tiles(
                        resources,
                        dirty.iter().filter_map(|domain| match domain {
                            EffectDomain::FoodTile(tile) => Some(*tile),
                            _ => None,
                        }),
                    )
                }
            }
            EffectDomain::Claims => context.replace_claims(effects.claims.clone()),
            EffectDomain::RoutineOccupancy => context.invalidate(Domain::RoutineOccupancy),
            EffectDomain::Invitations => context.invalidate(Domain::Invitations),
            EffectDomain::Jobs => context.invalidate(Domain::Jobs),
        }
        .map_err(failure)?;
    }
    Ok(())
}

impl RuntimeSystem for ColonyMindSystem {
    fn name(&self) -> &str {
        "ColonyMindSystem"
    }
    fn priority(&self) -> u32 {
        125
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        if let Err(error) = self.try_tick(world, resources) {
            panic!("ColonyMindSystem invariant: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{Agent, AgentState, Hunger, Position};
    use sim_core::MaterialRegistry;

    fn source(rest_score: i64) -> String {
        format!(
            r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
        (id:"wander",band:"idle",behaviour:"wander",when:[],score:(constant:1,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"WANDER"),
        (id:"rest",band:"idle",behaviour:"rest",when:[],score:(constant:{rest_score},terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"REST")])"#
        )
    }

    #[test]
    fn authored_scores_change_real_selected_node_and_unchanged_choice_is_not_logged_again() {
        for (score, expected) in [(0, "wander"), (2, "rest")] {
            let program = MindProgram::load(&source(score)).expect("registered policy");
            let expected_node = program.policy.node_id(expected).expect("node");
            let mut system = ColonyMindSystem::new(program);
            let mut world = World::new();
            world.spawn((
                Agent { id: 7 },
                Position::new(1, 1),
                AgentState::Idle,
                Hunger::new(20.0, 0.0),
            ));
            let mut resources = SimResources::new(4, 4, MaterialRegistry::new());
            resources.rules.colony = sim_core::rules::WorldRules::colony(1).colony;
            system
                .try_tick(&mut world, &mut resources)
                .expect("first real tick");
            assert_eq!(
                resources.mind.running.get(&7).expect("running").node,
                expected_node
            );
            resources.current_tick += 1;
            system
                .try_tick(&mut world, &mut resources)
                .expect("continuation");
            assert_eq!(
                resources
                    .mind
                    .history
                    .as_ref()
                    .expect("history")
                    .records(7)
                    .expect("person")
                    .len(),
                1
            );
        }
    }

    #[test]
    fn saved_source_mismatch_errors_before_effects() {
        let mut system = ColonyMindSystem::new(MindProgram::load(&source(0)).expect("policy"));
        let mut resources = SimResources::new(4, 4, MaterialRegistry::new());
        resources.mind.policy_source = Some(source(2));
        let error = system
            .try_tick(&mut World::new(), &mut resources)
            .expect_err("source mismatch");
        assert!(error.0.contains("sources differ"));
        assert!(resources.causal_log.is_empty());
    }
}

#[cfg(test)]
#[path = "system_owner_tests.rs"]
mod owner_tests;

#[cfg(test)]
mod food_invalidation_tests;

#[cfg(test)]
#[path = "primary_refresh_system_tests.rs"]
mod primary_refresh_tests;

#[cfg(test)]
#[path = "primary_snapshot_system_tests.rs"]
mod primary_snapshot_tests;

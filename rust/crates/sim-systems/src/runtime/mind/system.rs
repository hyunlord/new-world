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
use sim_core::colony::mind::{
    choose, EvalKey, EvalScratch, FinishMode, PersonFacts, RunningChoice,
};
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
    program: MindProgram,
    places: PlacesCache,
    scratch: EvalScratch,
    initialized: bool,
}
impl ColonyMindSystem {
    /// Construct from a policy already resolved against the actual registries.
    pub fn new(program: MindProgram) -> Self {
        Self {
            program,
            places: PlacesCache::default(),
            scratch: EvalScratch::default(),
            initialized: false,
        }
    }

    /// Install or validate persistent policy state before commands are admitted.
    pub(crate) fn initialize(&mut self, resources: &mut SimResources) -> Result<(), MindTickError> {
        if self.initialized {
            return Ok(());
        }
        let state = &mut resources.mind;
        match state.policy_source.as_deref() {
            Some(source) if source != self.program.source() => {
                return Err(failure("runtime and saved mind sources differ"))
            }
            Some(_) => {
                state
                    .history
                    .as_ref()
                    .ok_or_else(|| failure("saved mind lacks history"))?
                    .check_policy(&self.program.policy)
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
        let mut context = facts::prepare_context(world, resources).map_err(failure)?;
        context
            .replace_claims(effects.claims.clone())
            .map_err(failure)?;
        let mut pending: Vec<_> = context.rows().iter().map(|row| row.agent).collect();
        let mut table = PersonFacts::new(
            &self.program.policy,
            pending.clone(),
            resources.current_tick,
        )
        .map_err(failure)?;
        let mut completed = BTreeSet::new();
        loop {
            for agent in pending {
                // A completed consumption/activity keeps the historical one-tick Idle boundary.
                if completed.contains(&agent) {
                    continue;
                }
                self.person(
                    world,
                    resources,
                    &mut context,
                    &mut table,
                    &mut effects,
                    &mut routine,
                    agent,
                    &mut completed,
                )?;
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
        table: &mut PersonFacts,
        effects: &mut EffectState,
        routine: &mut Option<RoutineTick>,
        agent: AgentId,
        completed: &mut BTreeSet<AgentId>,
    ) -> Result<(), MindTickError> {
        // Match the legacy per-person invitation boundary: the reservation has
        // already ended, but its deferred Activity/FSM cleanup is not yet applied.
        if routine
            .as_mut()
            .is_some_and(|transaction| transaction.take_invitation(agent))
        {
            let entity = context.row(agent).map_err(failure)?.entity;
            *world.get::<&mut AgentState>(entity).map_err(failure)? = AgentState::Idle;
            effects.seek_clear.push(entity);
        }
        context.refresh_person(world, agent).map_err(failure)?;
        let prior = resources.mind.running.get(&agent).copied();
        context
            .set_running(agent, prior, &self.program.policy)
            .map_err(failure)?;
        facts::refresh_routine(context, routine.as_ref(), agent).map_err(failure)?;
        self.program
            .providers
            .refresh_row(table, context, resources, agent)
            .map_err(failure)?;
        let selected = choose(
            &self.program.policy,
            table.row(agent).map_err(failure)?,
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
        let node = self
            .program
            .policy
            .node(selected.selected.node)
            .ok_or_else(|| failure("selected node missing"))?;
        let effect = self.program.effect(node.id).map_err(failure)?;
        let candidate = match node.target.as_deref() {
            Some(name) => {
                facts::resolve_target(context, resources, agent, name).map_err(failure)?
            }
            None => None,
        };
        let target = effect
            .action_target(candidate)
            .map_err(|error| failure(format!("target: {error:?}")))?;
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
        let mut origins = record::automatic_origins(
            resources,
            &self.program.policy,
            agent,
            person.fleeing,
            self.scratch.witnesses(),
        );
        origins.extend(record::memory_origins(
            &self.program.policy,
            &selected,
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
        let cause = record::record_selection(
            resources,
            &self.program.policy,
            record::RecordPerson {
                agent,
                position: (person.position.x, person.position.y),
                settlement: scope.settlement,
            },
            &selected,
            self.scratch.witnesses(),
            &origins,
        )
        .map_err(failure)?;
        let cause = cause.or_else(|| {
            resources
                .mind
                .history
                .as_ref()
                .and_then(|history| history.latest(agent))
                .map(|choice| choice.id())
        });
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
        invalidate(context, call.effects)?;
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

fn invalidate(context: &mut TickContext, effects: &mut EffectState) -> Result<(), MindTickError> {
    for domain in std::mem::take(&mut effects.dirty) {
        match domain {
            EffectDomain::Needs | EffectDomain::Targets | EffectDomain::Relations => {
                context.invalidate(Domain::Person)
            }
            EffectDomain::FoodSources => context.invalidate(Domain::FoodSources),
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

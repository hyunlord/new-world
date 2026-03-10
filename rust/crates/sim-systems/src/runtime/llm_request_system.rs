use std::collections::HashMap;

use hecs::{Entity, World};
use sim_core::components::{
    Behavior, Emotion, Identity, LlmCapable, LlmContent, LlmPending, LlmRequestType, LlmResult,
    NarrativeCache, Needs, Personality, Stress, Values,
};
use sim_engine::{
    generate_fallback_content, EventStore, LlmPromptVariant, LlmRequest, SimEventType,
    SimResources, SimSystem,
};

/// Runtime system that submits non-blocking LLM requests when narrative-worthy state changes occur.
#[derive(Debug, Clone)]
pub struct LlmRequestRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

#[derive(Clone, Debug)]
struct RecentEventInfo {
    event_type: SimEventType,
    cause: String,
    target_name: Option<String>,
}

#[derive(Clone, Debug)]
struct RequestPlan {
    request_type: LlmRequestType,
    variant: LlmPromptVariant,
    recent_event_type: Option<String>,
    recent_event_cause: Option<String>,
    recent_target_name: Option<String>,
}

#[derive(Clone, Debug)]
struct PendingSubmission {
    entity: Entity,
    request: LlmRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LlmSubmissionStrategy {
    Queue,
    Fallback,
}

fn cache_has_any_text(cache: &NarrativeCache) -> bool {
    cache.personality_desc.is_some()
        || cache.last_event_narrative.is_some()
        || cache.last_inner_monologue.is_some()
}

impl LlmRequestRuntimeSystem {
    /// Creates a request system with explicit scheduling metadata.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for LlmRequestRuntimeSystem {
    fn name(&self) -> &'static str {
        "llm_request_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let name_lookup = build_name_lookup(world);
        let llm_available = resources.is_llm_available();
        let submission_strategy = llm_submission_strategy(llm_available);
        let mut submissions: Vec<PendingSubmission> = Vec::new();
        let mut total_capable: usize = 0;
        let mut qualifying_count: usize = 0;

        {
            let mut query = world.query::<(
                &Identity,
                &Personality,
                &Emotion,
                &Behavior,
                &Needs,
                &Stress,
                &Values,
                &LlmCapable,
                Option<&NarrativeCache>,
                Option<&LlmPending>,
                Option<&LlmResult>,
            )>();

            for (
                entity,
                (
                    identity,
                    personality,
                    emotion,
                    behavior,
                    needs,
                    stress,
                    values,
                    capable,
                    cache,
                    pending,
                    result,
                ),
            ) in &mut query
            {
                total_capable += 1;
                if pending.is_some() {
                    continue;
                }
                let should_enforce_cooldown =
                    result.is_some() || cache.map(cache_has_any_text).unwrap_or(false);
                if should_enforce_cooldown
                    && tick.saturating_sub(capable.last_request_tick)
                        < u64::from(capable.cooldown_ticks)
                {
                    continue;
                }

                let lookback_ticks = u64::from(capable.cooldown_ticks.max(60));
                let recent_event = latest_event_for_actor(
                    &resources.event_store,
                    entity.id(),
                    tick.saturating_sub(lookback_ticks),
                    &name_lookup,
                );
                let Some(plan) = plan_request(
                    cache,
                    recent_event.as_ref(),
                    tick,
                    matches!(submission_strategy, LlmSubmissionStrategy::Fallback),
                ) else {
                    continue;
                };

                submissions.push(PendingSubmission {
                    entity,
                    request: LlmRequest {
                        request_id: 0,
                        entity_id: entity.to_bits().get(),
                        request_type: plan.request_type,
                        variant: plan.variant,
                        entity_name: identity.name.clone(),
                        role: capable.role,
                        growth_stage: identity.growth_stage,
                        sex: identity.sex,
                        occupation: behavior.occupation.clone(),
                        action_id: behavior.current_action as u32,
                        action_label: behavior.current_action.to_string(),
                        personality_axes: personality.axes,
                        emotions: emotion.primary,
                        needs: needs.values,
                        values: values.values,
                        stress_level: stress.level,
                        stress_state: stress_state_code(stress),
                        recent_event_type: plan.recent_event_type,
                        recent_event_cause: plan.recent_event_cause,
                        recent_target_name: plan.recent_target_name,
                    },
                });
                qualifying_count += 1;
                if matches!(submission_strategy, LlmSubmissionStrategy::Queue) {
                    break;
                }
            }
        }

        if total_capable > 0 && (qualifying_count > 0 || tick.is_multiple_of(120)) {
            resources.llm_runtime.push_debug_log(format!(
                "[LLM-DEBUG] llm_request_system tick={} found {} LlmCapable entities, {} qualify for request, strategy={:?}",
                tick,
                total_capable,
                qualifying_count,
                submission_strategy
            ));
        }

        for submission in submissions {
            if matches!(submission_strategy, LlmSubmissionStrategy::Queue) {
                match resources.submit_llm_request(submission.request.clone()) {
                    Ok(request_id) => {
                        let pending = LlmPending {
                            request_id,
                            request_type: submission.request.request_type,
                            submitted_tick: tick,
                            timeout_ticks: sim_core::config::LLM_TIMEOUT_TICKS,
                        };
                        let _ = world.insert_one(submission.entity, pending);
                        if let Ok(mut capable) = world.get::<&mut LlmCapable>(submission.entity) {
                            capable.last_request_tick = tick;
                        }
                    }
                    Err(sim_engine::LlmRuntimeError::Unavailable) => {
                        apply_fallback_result(world, submission.entity, &submission.request, tick);
                    }
                    Err(sim_engine::LlmRuntimeError::QueueFull) => {}
                    Err(_) => {}
                }
            } else {
                apply_fallback_result(world, submission.entity, &submission.request, tick);
            }
        }
    }
}

fn llm_submission_strategy(llm_available: bool) -> LlmSubmissionStrategy {
    if llm_available {
        LlmSubmissionStrategy::Queue
    } else {
        LlmSubmissionStrategy::Fallback
    }
}

fn build_name_lookup(world: &World) -> HashMap<u32, String> {
    let mut lookup: HashMap<u32, String> = HashMap::new();
    let mut query = world.query::<&Identity>();
    for (entity, identity) in &mut query {
        lookup.insert(entity.id(), identity.name.clone());
    }
    lookup
}

fn latest_event_for_actor(
    store: &EventStore,
    actor: u32,
    since_tick: u64,
    names: &HashMap<u32, String>,
) -> Option<RecentEventInfo> {
    store
        .by_actor(actor, since_tick)
        .into_iter()
        .rev()
        .find(|event| is_qualifying_event(&event.event_type))
        .map(|event| RecentEventInfo {
            event_type: event.event_type.clone(),
            cause: event.cause.clone(),
            target_name: event.target.and_then(|target| names.get(&target).cloned()),
        })
}

fn is_qualifying_event(event_type: &SimEventType) -> bool {
    matches!(
        event_type,
        SimEventType::ActionChanged
            | SimEventType::NeedCritical
            | SimEventType::NeedSatisfied
            | SimEventType::StressEscalated
            | SimEventType::MentalBreakStart
            | SimEventType::MentalBreakEnd
            | SimEventType::RelationshipFormed
            | SimEventType::RelationshipBroken
            | SimEventType::SocialConflict
            | SimEventType::SocialCooperation
            | SimEventType::TaskCompleted
            | SimEventType::Birth
            | SimEventType::Death
            | SimEventType::AgeTransition
            | SimEventType::FirstOccurrence
    )
}

fn plan_request(
    cache: Option<&NarrativeCache>,
    recent_event: Option<&RecentEventInfo>,
    tick: u64,
    allow_prefill_without_event: bool,
) -> Option<RequestPlan> {
    if recent_event.is_none() && !allow_prefill_without_event {
        return None;
    }

    if cache_field_stale(cache, tick, |value| value.personality_desc.is_some()) {
        return Some(RequestPlan {
            request_type: LlmRequestType::Layer4Narrative,
            variant: LlmPromptVariant::Personality,
            recent_event_type: None,
            recent_event_cause: None,
            recent_target_name: None,
        });
    }

    if let Some(event) = recent_event {
        if is_judgment_event(&event.event_type) {
            return Some(RequestPlan {
                request_type: LlmRequestType::Layer3Judgment,
                variant: LlmPromptVariant::Judgment,
                recent_event_type: Some(label_for_event_type(&event.event_type).to_string()),
                recent_event_cause: if event.cause.is_empty() {
                    None
                } else {
                    Some(event.cause.clone())
                },
                recent_target_name: event.target_name.clone(),
            });
        }

        if cache_field_stale(cache, tick, |value| value.last_event_narrative.is_some()) {
            return Some(RequestPlan {
                request_type: LlmRequestType::Layer4Narrative,
                variant: LlmPromptVariant::Narrative,
                recent_event_type: Some(label_for_event_type(&event.event_type).to_string()),
                recent_event_cause: if event.cause.is_empty() {
                    None
                } else {
                    Some(event.cause.clone())
                },
                recent_target_name: event.target_name.clone(),
            });
        }
    }

    if cache_field_stale(cache, tick, |value| value.last_inner_monologue.is_some()) {
        return Some(RequestPlan {
            request_type: LlmRequestType::Layer4Narrative,
            variant: LlmPromptVariant::Narrative,
            recent_event_type: None,
            recent_event_cause: None,
            recent_target_name: None,
        });
    }

    None
}

fn cache_field_stale(
    cache: Option<&NarrativeCache>,
    tick: u64,
    has_value: impl Fn(&NarrativeCache) -> bool,
) -> bool {
    match cache {
        Some(existing) => {
            !has_value(existing)
                || tick.saturating_sub(existing.cache_tick) >= u64::from(existing.cache_ttl_ticks)
        }
        None => true,
    }
}

fn is_judgment_event(event_type: &SimEventType) -> bool {
    matches!(
        event_type,
        SimEventType::ActionChanged
            | SimEventType::NeedCritical
            | SimEventType::NeedSatisfied
            | SimEventType::StressEscalated
            | SimEventType::MentalBreakStart
    )
}

fn label_for_event_type(event_type: &SimEventType) -> &'static str {
    match event_type {
        SimEventType::NeedCritical => "need_critical",
        SimEventType::NeedSatisfied => "need_satisfied",
        SimEventType::EmotionShift => "emotion_shift",
        SimEventType::MoodChanged => "mood_changed",
        SimEventType::StressEscalated => "stress_escalated",
        SimEventType::MentalBreakStart => "mental_break_start",
        SimEventType::MentalBreakEnd => "mental_break_end",
        SimEventType::RelationshipFormed => "relationship_formed",
        SimEventType::RelationshipBroken => "relationship_broken",
        SimEventType::SocialConflict => "social_conflict",
        SimEventType::SocialCooperation => "social_cooperation",
        SimEventType::ActionChanged => "action_changed",
        SimEventType::TaskCompleted => "task_completed",
        SimEventType::Birth => "birth",
        SimEventType::Death => "death",
        SimEventType::AgeTransition => "age_transition",
        SimEventType::FirstOccurrence => "first_occurrence",
        SimEventType::Custom(_) => "custom",
    }
}

fn stress_state_code(stress: &Stress) -> u8 {
    match stress.state {
        sim_core::enums::StressState::Calm => 0,
        sim_core::enums::StressState::Alert => 1,
        sim_core::enums::StressState::Resistance => 2,
        sim_core::enums::StressState::Exhaustion => 3,
        sim_core::enums::StressState::Collapse => 4,
    }
}

fn apply_fallback_result(world: &mut World, entity: Entity, request: &LlmRequest, tick: u64) {
    let fallback_content =
        generate_fallback_content(request.request_type, request.entity_name.as_str());
    let result = LlmResult {
        request_id: request.request_id,
        content: fallback_content.clone(),
        generation_ms: 0,
        model_id: "fallback".to_string(),
    };
    upsert_result(world, entity, result);
    update_cache(
        world,
        entity,
        &request.variant,
        &request.recent_event_type,
        &fallback_content,
        tick,
    );
    if let Ok(mut capable) = world.get::<&mut LlmCapable>(entity) {
        capable.last_request_tick = tick;
    }
}

fn upsert_result(world: &mut World, entity: Entity, result: LlmResult) {
    if let Ok(mut existing) = world.get::<&mut LlmResult>(entity) {
        *existing = result;
        return;
    }
    let _ = world.insert_one(entity, result);
}

fn update_cache(
    world: &mut World,
    entity: Entity,
    variant: &LlmPromptVariant,
    recent_event_type: &Option<String>,
    content: &LlmContent,
    tick: u64,
) {
    let narrative_text = match content {
        LlmContent::Narrative(text) => Some(text.clone()),
        LlmContent::Judgment(_) => None,
    };
    let Some(text) = narrative_text else {
        return;
    };

    if let Ok(mut cache) = world.get::<&mut NarrativeCache>(entity) {
        cache.cache_tick = tick;
        match variant {
            LlmPromptVariant::Personality => cache.personality_desc = Some(text),
            LlmPromptVariant::Narrative => {
                if recent_event_type.is_some() {
                    cache.last_event_narrative = Some(text);
                } else {
                    cache.last_inner_monologue = Some(text);
                }
            }
            LlmPromptVariant::Judgment => {}
        }
        return;
    }

    let mut cache = NarrativeCache {
        cache_tick: tick,
        ..NarrativeCache::default()
    };
    match variant {
        LlmPromptVariant::Personality => cache.personality_desc = Some(text),
        LlmPromptVariant::Narrative => {
            if recent_event_type.is_some() {
                cache.last_event_narrative = Some(text);
            } else {
                cache.last_inner_monologue = Some(text);
            }
        }
        LlmPromptVariant::Judgment => {}
    }
    let _ = world.insert_one(entity, cache);
}

#[cfg(test)]
mod tests {
    use super::{llm_submission_strategy, LlmRequestRuntimeSystem, LlmSubmissionStrategy};
    use hecs::World;
    use sim_core::components::{
        Behavior, Emotion, Identity, LlmCapable, LlmContent, LlmResult, NarrativeCache, Needs,
        Personality, Stress, Values,
    };
    use sim_core::config::GameConfig;
    use sim_core::{GameCalendar, WorldMap};
    use sim_engine::{SimResources, SimSystem};

    #[test]
    fn request_system_generates_immediate_fallback_when_llm_is_unavailable() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 7);
        let mut resources = SimResources::new(calendar, map, 7);
        let mut world = World::new();
        let entity = world.spawn((
            Identity {
                name: "Kaya".to_string(),
                ..Identity::default()
            },
            Personality::default(),
            Emotion::default(),
            Behavior::default(),
            Needs::default(),
            Stress::default(),
            Values::default(),
            LlmCapable::default(),
        ));

        let mut system = LlmRequestRuntimeSystem::new(800, 1);
        system.run(&mut world, &mut resources, 600);

        let result = world
            .get::<&LlmResult>(entity)
            .expect("request system should attach fallback result");
        assert!(matches!(result.content, LlmContent::Narrative(_)));
        let capable = world
            .get::<&LlmCapable>(entity)
            .expect("capable component should remain present");
        assert_eq!(capable.last_request_tick, 600);
        assert!(world
            .get::<&sim_core::components::LlmPending>(entity)
            .is_err());
    }

    #[test]
    fn request_system_allows_initial_request_before_cooldown_has_elapsed() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 7);
        let mut resources = SimResources::new(calendar, map, 7);
        let mut world = World::new();
        let entity = world.spawn((
            Identity {
                name: "Aria".to_string(),
                ..Identity::default()
            },
            Personality::default(),
            Emotion::default(),
            Behavior::default(),
            Needs::default(),
            Stress::default(),
            Values::default(),
            LlmCapable::default(),
            NarrativeCache::default(),
        ));

        let mut system = LlmRequestRuntimeSystem::new(800, 1);
        system.run(&mut world, &mut resources, 0);

        let result = world
            .get::<&LlmResult>(entity)
            .expect("initial request should not be blocked by cooldown");
        assert!(matches!(result.content, LlmContent::Narrative(_)));
    }

    #[test]
    fn auto_request_plan_requires_a_recent_event() {
        let plan = super::plan_request(None, None, 0, false);
        assert!(
            plan.is_none(),
            "background request planning should not prefill entities with no recent event"
        );
    }

    #[test]
    fn available_runtime_uses_queue_submission_strategy() {
        assert_eq!(llm_submission_strategy(true), LlmSubmissionStrategy::Queue);
        assert_eq!(
            llm_submission_strategy(false),
            LlmSubmissionStrategy::Fallback
        );
    }
}

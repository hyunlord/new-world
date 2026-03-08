use hecs::{Entity, World};
use sim_core::components::{LlmContent, LlmPending, LlmResult, NarrativeCache};
use sim_engine::{
    GameEvent, LlmEvent, LlmPromptVariant, LlmRequestMeta, LlmResponse, SimResources, SimSystem,
};

/// Runtime system that drains completed LLM responses and attaches results to entities.
#[derive(Debug, Clone)]
pub struct LlmResponseRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl LlmResponseRuntimeSystem {
    /// Creates a response system with explicit scheduling metadata.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for LlmResponseRuntimeSystem {
    fn name(&self) -> &'static str {
        "llm_response_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        drain_and_apply_llm_responses(world, resources, tick);
    }
}

/// Drains all available LLM responses, applies them to ECS state, and emits
/// the corresponding runtime events.
pub fn drain_and_apply_llm_responses(world: &mut World, resources: &mut SimResources, tick: u64) {
    let responses = resources.drain_llm_responses();
    for response in responses {
        resources.llm_runtime.push_debug_log(format!(
            "[LLM-DEBUG] llm_response_system received response: id={}, entity_id={}, success={}, generation_ms={}",
            response.request_id,
            response.entity_id,
            response.success,
            response.generation_ms
        ));
        let Some(meta) = resources.take_llm_request_meta(response.request_id) else {
            resources.llm_runtime.push_debug_log(format!(
                "[LLM-DEBUG] llm_response_system dropped response without meta: id={}",
                response.request_id
            ));
            continue;
        };
        apply_response_to_entity(world, tick, &response, &meta);
        resources.llm_runtime.push_debug_log(format!(
            "[LLM-DEBUG] llm_response_system applied response: id={}, variant={:?}",
            response.request_id,
            meta.variant
        ));
        resources
            .event_bus
            .emit(GameEvent::Llm(LlmEvent::ResponseReceived {
                entity_id: response.entity_id,
                generation_ms: response.generation_ms,
            }));
    }
}

fn apply_response_to_entity(
    world: &mut World,
    tick: u64,
    response: &LlmResponse,
    meta: &LlmRequestMeta,
) {
    let Some(entity) = Entity::from_bits(response.entity_id) else {
        return;
    };
    if !world.contains(entity) {
        return;
    }
    let _ = world.remove_one::<LlmPending>(entity);
    let result = LlmResult {
        request_id: response.request_id,
        content: response.content.clone(),
        generation_ms: response.generation_ms,
        model_id: response.model_id.clone(),
    };
    upsert_result(world, entity, result);
    update_cache(world, entity, meta, &response.content, tick);
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
    meta: &LlmRequestMeta,
    content: &LlmContent,
    tick: u64,
) {
    let narrative = match content {
        LlmContent::Narrative(text) => Some(text.clone()),
        LlmContent::Judgment(_) => None,
    };
    let Some(text) = narrative else {
        return;
    };

    if let Ok(mut cache) = world.get::<&mut NarrativeCache>(entity) {
        cache.cache_tick = tick;
        match meta.variant {
            LlmPromptVariant::Personality => cache.personality_desc = Some(text),
            LlmPromptVariant::Narrative => {
                if meta.recent_event_type.is_some() {
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
    match meta.variant {
        LlmPromptVariant::Personality => cache.personality_desc = Some(text),
        LlmPromptVariant::Narrative => {
            if meta.recent_event_type.is_some() {
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
    use super::apply_response_to_entity;
    use hecs::World;
    use sim_core::components::{
        LlmContent, LlmPending, LlmRequestType, NarrativeCache,
    };
    use sim_engine::{LlmPromptVariant, LlmRequestMeta, LlmResponse};

    #[test]
    fn response_system_applies_result_and_updates_personality_cache() {
        let mut world = World::new();
        let entity = world.spawn((NarrativeCache::default(), LlmPending::default()));
        let response = LlmResponse {
            request_id: 7,
            entity_id: entity.to_bits().get(),
            content: LlmContent::Narrative("카야는 신중한 계획가처럼 움직였다.".to_string()),
            generation_ms: 123,
            success: true,
            model_id: "test-model".to_string(),
        };
        let meta = LlmRequestMeta {
            request_type: LlmRequestType::Layer4Narrative,
            variant: LlmPromptVariant::Personality,
            entity_name: "Kaya".to_string(),
            recent_event_type: None,
        };

        apply_response_to_entity(&mut world, 500, &response, &meta);

        let result = world
            .get::<&sim_core::components::LlmResult>(entity)
            .expect("result should be attached");
        assert_eq!(result.request_id, 7);
        let cache = world
            .get::<&NarrativeCache>(entity)
            .expect("cache should exist");
        assert_eq!(
            cache.personality_desc.as_deref(),
            Some("카야는 신중한 계획가처럼 움직였다.")
        );
        assert!(world.get::<&LlmPending>(entity).is_err());
    }
}

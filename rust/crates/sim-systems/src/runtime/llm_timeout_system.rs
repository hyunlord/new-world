use hecs::{Entity, World};
use sim_core::components::{LlmContent, LlmPending, LlmResult, NarrativeCache};
use sim_engine::{
    generate_fallback_content, GameEvent, LlmEvent, LlmPromptVariant, LlmRequestMeta,
    SimResources, SimSystem,
};

/// Runtime system that resolves expired in-flight LLM requests with deterministic fallback text.
#[derive(Debug, Clone)]
pub struct LlmTimeoutRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

#[derive(Clone, Debug)]
struct ExpiredRequest {
    entity: Entity,
    request_id: u64,
    request_type: sim_core::components::LlmRequestType,
    entity_name: String,
}

impl LlmTimeoutRuntimeSystem {
    /// Creates a timeout system with explicit scheduling metadata.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for LlmTimeoutRuntimeSystem {
    fn name(&self) -> &'static str {
        "llm_timeout_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let mut expired: Vec<ExpiredRequest> = Vec::new();
        {
            let mut query = world.query::<(&LlmPending, Option<&sim_core::components::Identity>)>();
            for (entity, (pending, identity)) in &mut query {
                if tick.saturating_sub(pending.submitted_tick) <= u64::from(pending.timeout_ticks) {
                    continue;
                }
                expired.push(ExpiredRequest {
                    entity,
                    request_id: pending.request_id,
                    request_type: pending.request_type,
                    entity_name: identity
                        .map(|value| value.name.clone())
                        .unwrap_or_else(|| "누군가".to_string()),
                });
            }
        }

        for request in expired {
            let meta = resources.take_llm_request_meta(request.request_id).unwrap_or(LlmRequestMeta {
                request_type: request.request_type,
                variant: LlmPromptVariant::Narrative,
                entity_name: request.entity_name.clone(),
                recent_event_type: None,
            });
            apply_timeout_result(
                world,
                tick,
                request.entity,
                request.request_id,
                &meta,
                request.entity_name.as_str(),
            );
            resources
                .event_bus
                .emit(GameEvent::Llm(LlmEvent::RequestTimedOut {
                    entity_id: request.entity.to_bits().get(),
                }));
        }
    }
}

fn apply_timeout_result(
    world: &mut World,
    tick: u64,
    entity: Entity,
    request_id: u64,
    meta: &LlmRequestMeta,
    entity_name: &str,
) {
    let _ = world.remove_one::<LlmPending>(entity);
    let content = generate_fallback_content(meta.request_type, entity_name);
    let result = LlmResult {
        request_id,
        content: content.clone(),
        generation_ms: 0,
        model_id: "fallback".to_string(),
    };
    upsert_result(world, entity, result);
    update_cache(world, entity, meta, &content, tick);
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
    use super::apply_timeout_result;
    use hecs::World;
    use sim_core::components::{LlmPending, LlmRequestType, NarrativeCache};
    use sim_engine::LlmPromptVariant;
    use sim_engine::LlmRequestMeta;

    #[test]
    fn timeout_system_replaces_pending_with_fallback_result() {
        let mut world = World::new();
        let entity = world.spawn((NarrativeCache::default(), LlmPending::default()));
        let meta = LlmRequestMeta {
            request_type: LlmRequestType::Layer4Narrative,
            variant: LlmPromptVariant::Narrative,
            entity_name: "Kaya".to_string(),
            recent_event_type: None,
        };

        apply_timeout_result(&mut world, 900, entity, 33, &meta, "Kaya");

        let result = world
            .get::<&sim_core::components::LlmResult>(entity)
            .expect("timeout should attach fallback result");
        assert_eq!(result.request_id, 33);
        let cache = world
            .get::<&NarrativeCache>(entity)
            .expect("cache should remain present");
        assert_eq!(
            cache.last_inner_monologue.as_deref(),
            Some("Kaya은(는) 주변을 살폈다.")
        );
        assert!(world.get::<&LlmPending>(entity).is_err());
    }
}

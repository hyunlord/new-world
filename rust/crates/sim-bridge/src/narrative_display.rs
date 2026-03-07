use godot::builtin::{Array, GString, Variant, VarDictionary};
use hecs::{Entity, World};
use sim_core::components::{LlmPending, LlmResult, NarrativeCache};
use sim_engine::SimResources;

use crate::locale_bindings::format_active_fluent_message;

/// Pre-computed display state for the narrative UI panel.
///
/// GDScript reads this payload and maps fields directly to node properties
/// without branching on simulation state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NarrativeDisplayData {
    /// Personality description text, or empty when unavailable.
    pub personality_text: String,
    /// Event narrative text, or empty when unavailable.
    pub event_text: String,
    /// Inner-monologue text, or empty when unavailable.
    pub inner_text: String,
    /// Whether the personality section should be visible.
    pub show_personality: bool,
    /// Whether the event section should be visible.
    pub show_event: bool,
    /// Whether the inner-thoughts section should be visible.
    pub show_inner: bool,
    /// Whether the personality shimmer placeholder should be visible.
    pub show_personality_shimmer: bool,
    /// Whether the event shimmer placeholder should be visible.
    pub show_event_shimmer: bool,
    /// Whether the inner shimmer placeholder should be visible.
    pub show_inner_shimmer: bool,
    /// Whether the disabled overlay should be shown.
    pub show_disabled_overlay: bool,
    /// AI icon visual state.
    pub ai_icon_state: u8,
    /// Tooltip explaining AI-generated text provenance.
    pub ai_label_tooltip: String,
    /// Localized panel title.
    pub panel_title: String,
    /// Localized section labels in personality / event / inner order.
    pub section_labels: [String; 3],
    /// Localized disabled-state message.
    pub disabled_message: String,
    /// Whether any visible text came from a non-fallback LLM result.
    pub ai_generated: bool,
    /// Raw runtime entity id this payload belongs to.
    pub entity_id: u64,
}

/// Builds the fully-determined narrative UI display state for one entity.
pub(crate) fn build_narrative_display(
    world: &World,
    resources: &SimResources,
    entity: Entity,
    entity_id: u64,
) -> NarrativeDisplayData {
    let mut data = NarrativeDisplayData {
        entity_id,
        panel_title: localized_text("LLM_NARRATIVE_TITLE"),
        section_labels: [
            localized_text("LLM_NARRATIVE_PERSONALITY"),
            localized_text("LLM_NARRATIVE_EVENT"),
            localized_text("LLM_NARRATIVE_INNER"),
        ],
        ..NarrativeDisplayData::default()
    };

    let llm_enabled = resources.llm_runtime.config().enabled_default;
    let has_cache = world.get::<&NarrativeCache>(entity).is_ok();
    let is_loading = world.get::<&LlmPending>(entity).is_ok();
    let server_running = resources.llm_runtime.is_running();
    log::info!(
        "[LLM-DEBUG] build_narrative_display entity={} llm_enabled={} server_running={} is_loading={} has_cache={}",
        entity_id,
        llm_enabled,
        server_running,
        is_loading,
        has_cache
    );
    if !llm_enabled {
        data.show_disabled_overlay = true;
        data.disabled_message = localized_text("LLM_NARRATIVE_DISABLED");
        data.ai_icon_state = 3;
        return data;
    }

    data.ai_icon_state = if resources.is_llm_available() { 1 } else { 4 };
    if is_loading {
        data.ai_icon_state = 2;
    }

    if let Ok(cache) = world.get::<&NarrativeCache>(entity) {
        if let Some(text) = cache.personality_desc.as_ref() {
            data.personality_text = text.clone();
            data.show_personality = true;
        }
        if let Some(text) = cache.last_event_narrative.as_ref() {
            data.event_text = text.clone();
            data.show_event = true;
        }
        if let Some(text) = cache.last_inner_monologue.as_ref() {
            data.inner_text = text.clone();
            data.show_inner = true;
        }
    }

    if is_loading {
        data.show_personality_shimmer = !data.show_personality;
        data.show_event_shimmer = !data.show_event;
        data.show_inner_shimmer = !data.show_inner;
    }

    let latest_result_is_ai = world
        .get::<&LlmResult>(entity)
        .ok()
        .map(|result| result.model_id != "fallback")
        .unwrap_or(false);
    data.ai_generated =
        latest_result_is_ai && (data.show_personality || data.show_event || data.show_inner);
    if data.ai_generated {
        data.ai_label_tooltip = localized_text("LLM_AI_LABEL_FULL");
    }

    data
}

/// Converts a Rust-side narrative display payload into a Godot dictionary.
pub(crate) fn narrative_display_to_dict(data: &NarrativeDisplayData) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("personality_text", GString::from(data.personality_text.as_str()));
    dict.set("event_text", GString::from(data.event_text.as_str()));
    dict.set("inner_text", GString::from(data.inner_text.as_str()));
    dict.set("show_personality", data.show_personality);
    dict.set("show_event", data.show_event);
    dict.set("show_inner", data.show_inner);
    dict.set("show_personality_shimmer", data.show_personality_shimmer);
    dict.set("show_event_shimmer", data.show_event_shimmer);
    dict.set("show_inner_shimmer", data.show_inner_shimmer);
    dict.set("show_disabled_overlay", data.show_disabled_overlay);
    dict.set("ai_icon_state", i64::from(data.ai_icon_state));
    dict.set("ai_label_tooltip", GString::from(data.ai_label_tooltip.as_str()));
    dict.set("panel_title", GString::from(data.panel_title.as_str()));
    let mut labels: Array<Variant> = Array::new();
    for label in &data.section_labels {
        labels.push(&Variant::from(GString::from(label.as_str())));
    }
    dict.set("section_labels", labels);
    dict.set("disabled_message", GString::from(data.disabled_message.as_str()));
    dict.set("ai_generated", data.ai_generated);
    dict.set("entity_id", data.entity_id as i64);
    dict
}

fn localized_text(key: &str) -> String {
    format_active_fluent_message(key).unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests {
    use super::build_narrative_display;
    use crate::locale_bindings::{clear_fluent_source, locale_test_lock, store_fluent_source};
    use hecs::World;
    use sim_core::calendar::GameCalendar;
    use sim_core::components::{LlmContent, LlmPending, LlmRequestType, LlmResult, NarrativeCache};
    use sim_core::config::GameConfig;
    use sim_core::world::WorldMap;
    use sim_engine::llm_server::{LlmConfig, LlmRuntime};
    use sim_engine::SimResources;

    fn install_test_locale() {
        let source = concat!(
            "LLM_NARRATIVE_TITLE = 서사\n",
            "LLM_NARRATIVE_PERSONALITY = 성격\n",
            "LLM_NARRATIVE_EVENT = 사건\n",
            "LLM_NARRATIVE_INNER = 내면\n",
            "LLM_NARRATIVE_DISABLED = AI 서사가 비활성화되어 있습니다.\n",
            "LLM_AI_LABEL_FULL = 본 텍스트는 인공지능에 의해 생성되었습니다.\n",
        );
        assert!(store_fluent_source("ko", source));
    }

    fn test_resources() -> SimResources {
        let calendar = GameCalendar::new(&GameConfig::default());
        let map = WorldMap::new(4, 4, 1);
        SimResources::new(calendar, map, 1)
    }

    #[test]
    fn narrative_display_builder_marks_disabled_overlay_when_llm_is_off() {
        let _guard = locale_test_lock().lock().expect("locale test lock");
        install_test_locale();
        let mut world = World::new();
        let mut resources = test_resources();
        let mut config = LlmConfig::load_default().expect("default llm config should load");
        config.enabled_default = false;
        resources.llm_runtime = LlmRuntime::new(config);
        let entity = world.spawn(());
        let display = build_narrative_display(&world, &resources, entity, 7);
        assert!(display.show_disabled_overlay);
        assert_eq!(display.ai_icon_state, 3);
        clear_fluent_source("ko");
    }

    #[test]
    fn narrative_display_builder_uses_cached_text_and_loading_shimmer() {
        let _guard = locale_test_lock().lock().expect("locale test lock");
        install_test_locale();
        let mut world = World::new();
        let entity = world.spawn((
            NarrativeCache {
                personality_desc: Some("차분한 관찰자다.".to_string()),
                last_event_narrative: None,
                last_inner_monologue: Some("지금은 잠시 숨을 고른다.".to_string()),
                cache_tick: 10,
                cache_ttl_ticks: 3600,
            },
            LlmPending {
                request_id: 11,
                request_type: LlmRequestType::Layer4Narrative,
                submitted_tick: 10,
                timeout_ticks: 600,
            },
            LlmResult {
                request_id: 10,
                content: LlmContent::Narrative("이전 결과".to_string()),
                generation_ms: 1200,
                model_id: "qwen".to_string(),
            },
        ));
        let mut resources = test_resources();
        let mut config = LlmConfig::load_default().expect("default llm config should load");
        config.enabled_default = true;
        resources.llm_runtime = LlmRuntime::new(config);
        resources.llm_runtime.stop();
        let display = build_narrative_display(&world, &resources, entity, 42);
        assert!(display.show_personality);
        assert!(display.show_inner);
        assert!(display.show_event_shimmer);
        assert_eq!(display.ai_icon_state, 2);
        assert!(display.ai_generated);
        assert_eq!(display.panel_title, "서사");
        assert_eq!(display.section_labels[0], "성격");
        clear_fluent_source("ko");
    }
}

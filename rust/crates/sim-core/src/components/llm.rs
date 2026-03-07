use serde::{Deserialize, Serialize};

/// Marker component declaring that an entity can participate in LLM generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmCapable {
    /// Narrative role used to frame prompt tone and responsibility.
    pub role: LlmRole,
    /// Minimum delay between submitted requests for one entity.
    pub cooldown_ticks: u32,
    /// Tick when the last request was submitted.
    pub last_request_tick: u64,
}

impl Default for LlmCapable {
    fn default() -> Self {
        Self {
            role: LlmRole::Agent,
            cooldown_ticks: crate::config::LLM_COOLDOWN_TICKS,
            last_request_tick: 0,
        }
    }
}

/// Component attached while an LLM request is currently in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmPending {
    /// Monotonic request identifier allocated by the LLM runtime.
    pub request_id: u64,
    /// High-level request layer currently being processed.
    pub request_type: LlmRequestType,
    /// Tick when the request was submitted.
    pub submitted_tick: u64,
    /// Maximum time the request may remain pending before fallback.
    pub timeout_ticks: u32,
}

impl Default for LlmPending {
    fn default() -> Self {
        Self {
            request_id: 0,
            request_type: LlmRequestType::Layer4Narrative,
            submitted_tick: 0,
            timeout_ticks: crate::config::LLM_TIMEOUT_TICKS,
        }
    }
}

/// Result component storing the latest completed LLM output for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmResult {
    /// Request identifier this result belongs to.
    pub request_id: u64,
    /// Structured or narrative LLM output.
    pub content: LlmContent,
    /// End-to-end generation time in milliseconds.
    pub generation_ms: u32,
    /// Model identifier recorded for diagnostics.
    pub model_id: String,
}

impl Default for LlmResult {
    fn default() -> Self {
        Self {
            request_id: 0,
            content: LlmContent::Narrative(String::new()),
            generation_ms: 0,
            model_id: String::new(),
        }
    }
}

/// Short-lived cache for previously generated narrative strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NarrativeCache {
    /// Personality description text generated for this entity.
    pub personality_desc: Option<String>,
    /// Narrative bound to the latest salient event.
    pub last_event_narrative: Option<String>,
    /// Free-form inner monologue or thought-stream text.
    pub last_inner_monologue: Option<String>,
    /// Tick when the cache was last refreshed.
    pub cache_tick: u64,
    /// Cache time-to-live before the request system asks again.
    pub cache_ttl_ticks: u32,
}

impl Default for NarrativeCache {
    fn default() -> Self {
        Self {
            personality_desc: None,
            last_event_narrative: None,
            last_inner_monologue: None,
            cache_tick: 0,
            cache_ttl_ticks: crate::config::LLM_CACHE_TTL_TICKS,
        }
    }
}

/// Narrative role categories used by the local LLM overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmRole {
    Agent,
    Leader,
    Shaman,
    Oracle,
}

/// Request layers supported by the Phase 1 LLM runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmRequestType {
    /// GBNF-constrained JSON output for closed-set judgment.
    Layer3Judgment,
    /// Free-form Korean narrative text.
    Layer4Narrative,
}

/// Content payload stored in [`LlmResult`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LlmContent {
    /// Layer 3 structured judgment result.
    Judgment(JudgmentData),
    /// Layer 4 free-form narrative result.
    Narrative(String),
}

/// Structured Layer 3 output describing a chosen action from a closed set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JudgmentData {
    /// Action selected by the model from the provided closed set.
    pub action_id: u32,
    /// Confidence score in the range `0.0..=1.0`.
    pub confidence: f64,
    /// Short reasoning hint meant for UI display or debug logging.
    pub reasoning_hint: String,
}

impl Default for JudgmentData {
    fn default() -> Self {
        Self {
            action_id: 0,
            confidence: 0.5,
            reasoning_hint: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        JudgmentData, LlmCapable, LlmContent, LlmPending, LlmRequestType, LlmResult, LlmRole,
        NarrativeCache,
    };

    #[test]
    fn llm_capable_defaults_match_config() {
        let component = LlmCapable::default();
        assert_eq!(component.role, LlmRole::Agent);
        assert_eq!(component.cooldown_ticks, crate::config::LLM_COOLDOWN_TICKS);
        assert_eq!(component.last_request_tick, 0);
    }

    #[test]
    fn llm_pending_defaults_match_config() {
        let pending = LlmPending::default();
        assert_eq!(pending.request_type, LlmRequestType::Layer4Narrative);
        assert_eq!(pending.timeout_ticks, crate::config::LLM_TIMEOUT_TICKS);
    }

    #[test]
    fn narrative_cache_defaults_match_config() {
        let cache = NarrativeCache::default();
        assert_eq!(cache.cache_tick, 0);
        assert_eq!(cache.cache_ttl_ticks, crate::config::LLM_CACHE_TTL_TICKS);
        assert!(cache.personality_desc.is_none());
    }

    #[test]
    fn llm_result_can_store_judgment_and_narrative() {
        let judgment = JudgmentData {
            action_id: 4,
            confidence: 0.9,
            reasoning_hint: "focused".to_string(),
        };
        let result = LlmResult {
            request_id: 9,
            content: LlmContent::Judgment(judgment.clone()),
            generation_ms: 123,
            model_id: "test-model".to_string(),
        };
        assert_eq!(result.request_id, 9);
        assert_eq!(result.generation_ms, 123);
        assert!(matches!(result.content, LlmContent::Judgment(ref data) if data == &judgment));

        let narrative = LlmResult {
            request_id: 10,
            content: LlmContent::Narrative("서사를 본다.".to_string()),
            generation_ms: 456,
            model_id: "test-model".to_string(),
        };
        assert!(matches!(narrative.content, LlmContent::Narrative(_)));
    }
}

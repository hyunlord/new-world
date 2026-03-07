use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crossbeam_channel::{Receiver, Sender};

use sim_core::components::{JudgmentData, LlmContent, LlmRequestType, LlmRole};

use crate::llm_prompt::{LlmPromptContext, LlmPromptError, LlmPromptTemplates, RenderedPrompt};
use crate::llm_server::LlmConfig;

/// Internal prompt-template variant for Phase 1 LLM generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmPromptVariant {
    /// Closed-set judgment request.
    Judgment,
    /// Free-form event narrative request.
    Narrative,
    /// Personality-description request.
    Personality,
}

/// Full request payload submitted to the LLM worker thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Monotonic request identifier allocated by the runtime.
    pub request_id: u64,
    /// ECS entity bits used to route the response back to the correct entity.
    pub entity_id: u64,
    /// High-level request layer.
    pub request_type: LlmRequestType,
    /// Template variant rendered by the worker.
    pub variant: LlmPromptVariant,
    /// Entity display name.
    pub entity_name: String,
    /// Narrative role label.
    pub role: LlmRole,
    /// Closed-set action identifier.
    pub action_id: u32,
    /// Human-readable action label.
    pub action_label: String,
    /// HEXACO axes.
    pub personality_axes: [f64; 6],
    /// Plutchik primary emotions.
    pub emotions: [f64; 8],
    /// 13-need values.
    pub needs: [f64; 13],
    /// Current stress level.
    pub stress_level: f64,
    /// Current stress-state bucket.
    pub stress_state: u8,
    /// Optional recent event label.
    pub recent_event_type: Option<String>,
    /// Optional recent event cause.
    pub recent_event_cause: Option<String>,
    /// Optional recent target/partner name.
    pub recent_target_name: Option<String>,
}

impl LlmRequest {
    /// Returns lightweight metadata retained while the request is in flight.
    pub fn meta(&self) -> LlmRequestMeta {
        LlmRequestMeta {
            request_type: self.request_type,
            variant: self.variant,
            entity_name: self.entity_name.clone(),
            recent_event_type: self.recent_event_type.clone(),
        }
    }

    fn prompt_context(&self) -> LlmPromptContext {
        LlmPromptContext {
            entity_name: self.entity_name.clone(),
            role: format!("{:?}", self.role).to_lowercase(),
            action_id: self.action_id,
            action_label: self.action_label.clone(),
            personality_axes: self.personality_axes,
            emotions: self.emotions,
            needs: self.needs,
            stress_level: self.stress_level,
            stress_state: self.stress_state,
            recent_event_type: self.recent_event_type.clone(),
            recent_event_cause: self.recent_event_cause.clone(),
            recent_target_name: self.recent_target_name.clone(),
        }
    }
}

/// Metadata kept by the runtime while a request remains pending.
#[derive(Debug, Clone)]
pub struct LlmRequestMeta {
    /// High-level request type.
    pub request_type: LlmRequestType,
    /// Prompt variant selected when the request was submitted.
    pub variant: LlmPromptVariant,
    /// Entity name used to generate fallback text.
    pub entity_name: String,
    /// Most recent event type bundled into the prompt, if any.
    pub recent_event_type: Option<String>,
}

/// Response payload produced by the worker thread.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Request identifier.
    pub request_id: u64,
    /// ECS entity bits.
    pub entity_id: u64,
    /// Final content after parse/validation.
    pub content: LlmContent,
    /// End-to-end generation time in milliseconds.
    pub generation_ms: u32,
    /// Whether the worker produced non-fallback content.
    pub success: bool,
    /// Model identifier used for the response.
    pub model_id: String,
}

/// Errors emitted while building prompts or talking to llama-server.
#[derive(Debug, Error)]
pub enum LlmWorkerError {
    #[error("prompt rendering failed: {0}")]
    Prompt(#[from] LlmPromptError),
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("malformed LLM response: {0}")]
    MalformedResponse(String),
    #[error("judgment parse failed")]
    JudgmentParseFailed,
}

/// Worker thread loop that receives requests and produces responses.
pub fn llm_worker_loop(
    rx: Receiver<LlmRequest>,
    tx: Sender<LlmResponse>,
    config: LlmConfig,
) {
    let timeout = Duration::from_millis(config.http_timeout_ms);
    let client = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let templates = LlmPromptTemplates::load(&config.prompt_dir).ok();

    while let Ok(request) = rx.recv() {
        log::info!(
            "[LLM-DEBUG] llm_worker received request: id={}, type={:?}, variant={:?}, entity_id={}",
            request.request_id,
            request.request_type,
            request.variant,
            request.entity_id
        );
        let start = Instant::now();
        let response = match templates.as_ref() {
            Some(loaded_templates) => process_request(&client, loaded_templates, &config, &request),
            None => Err(LlmWorkerError::Prompt(LlmPromptError::MissingTemplate(
                "system.jinja".to_string(),
            ))),
        };
        let elapsed = start.elapsed().as_millis() as u32;
        let llm_response = match response {
            Ok(content) => LlmResponse {
                request_id: request.request_id,
                entity_id: request.entity_id,
                content,
                generation_ms: elapsed,
                success: true,
                model_id: config.model_id.clone(),
            },
            Err(error) => {
                log::warn!(
                    "[llm_worker] request {} failed: {}",
                    request.request_id,
                    error
                );
                LlmResponse {
                    request_id: request.request_id,
                    entity_id: request.entity_id,
                    content: generate_fallback_content(
                        request.request_type,
                        request.entity_name.as_str(),
                    ),
                    generation_ms: elapsed,
                    success: false,
                    model_id: "fallback".to_string(),
                }
            }
        };
        if tx.send(llm_response).is_err() {
            break;
        }
    }
}

/// Processes a single request by rendering prompts, calling llama-server, and validating output.
pub fn process_request(
    client: &ureq::Agent,
    templates: &LlmPromptTemplates,
    config: &LlmConfig,
    request: &LlmRequest,
) -> Result<LlmContent, LlmWorkerError> {
    let prompt = match request.variant {
        LlmPromptVariant::Judgment => templates.render_layer3_judgment(&request.prompt_context())?,
        LlmPromptVariant::Narrative => templates.render_layer4_narrative(&request.prompt_context())?,
        LlmPromptVariant::Personality => {
            templates.render_layer4_personality(&request.prompt_context())?
        }
    };

    let body = request_body(config, request, &prompt);
    let endpoint = format!("{}/v1/chat/completions", config.base_url());
    let response = client
        .post(endpoint.as_str())
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|error| LlmWorkerError::Http(error.to_string()))?;
    let response_value: Value = response
        .into_json()
        .map_err(|error| LlmWorkerError::Http(error.to_string()))?;
    let content = response_value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| LlmWorkerError::MalformedResponse("missing message content".to_string()))?;

    match request.request_type {
        LlmRequestType::Layer3Judgment => parse_judgment_content(content).map(LlmContent::Judgment),
        LlmRequestType::Layer4Narrative => {
            if looks_like_garbage(content) {
                return Err(LlmWorkerError::MalformedResponse(
                    "narrative content looked like garbage".to_string(),
                ));
            }
            Ok(LlmContent::Narrative(content.to_string()))
        }
    }
}

/// Generates deterministic fallback content when the worker cannot produce a valid response.
pub fn generate_fallback_content(request_type: LlmRequestType, entity_name: &str) -> LlmContent {
    match request_type {
        LlmRequestType::Layer3Judgment => LlmContent::Judgment(JudgmentData {
            action_id: 0,
            confidence: 0.5,
            reasoning_hint: "[시스템 판단]".to_string(),
        }),
        LlmRequestType::Layer4Narrative => {
            LlmContent::Narrative(format!("{entity_name}은(는) 주변을 살폈다."))
        }
    }
}

fn request_body(config: &LlmConfig, request: &LlmRequest, prompt: &RenderedPrompt) -> Value {
    let max_tokens = match request.request_type {
        LlmRequestType::Layer3Judgment => config.max_tokens_l3,
        LlmRequestType::Layer4Narrative => config.max_tokens_l4,
    };
    let temperature = match request.request_type {
        LlmRequestType::Layer3Judgment => config.temperature_l3,
        LlmRequestType::Layer4Narrative => config.temperature_l4,
    };

    let mut body = serde_json::json!({
        "model": config.model_id,
        "messages": [
            { "role": "system", "content": prompt.system },
            { "role": "user", "content": prompt.user }
        ],
        "max_tokens": max_tokens,
        "temperature": temperature,
        "stream": false,
    });
    if matches!(request.request_type, LlmRequestType::Layer3Judgment) {
        body["grammar"] = serde_json::json!(config.layer3_grammar);
    }
    body
}

fn parse_judgment_content(content: &str) -> Result<JudgmentData, LlmWorkerError> {
    parse_judgment_json(content).or_else(|_| {
        extract_json_candidate(content)
            .and_then(|candidate| parse_judgment_json(candidate.as_str()).ok())
            .ok_or(LlmWorkerError::JudgmentParseFailed)
    })
}

fn parse_judgment_json(content: &str) -> Result<JudgmentData, LlmWorkerError> {
    serde_json::from_str::<JudgmentData>(content)
        .map_err(|_| LlmWorkerError::JudgmentParseFailed)
}

fn extract_json_candidate(content: &str) -> Option<String> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(content[start..=end].to_string())
}

fn looks_like_garbage(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.chars().count() < 10 {
        return true;
    }
    let repeated = repeated_char_ratio(trimmed);
    repeated >= 0.9
}

fn repeated_char_ratio(content: &str) -> f64 {
    let total = content.chars().count();
    if total == 0 {
        return 1.0;
    }
    let repeated = content
        .chars()
        .fold(std::collections::HashMap::<char, usize>::new(), |mut acc, ch| {
            *acc.entry(ch).or_insert(0) += 1;
            acc
        })
        .values()
        .copied()
        .max()
        .unwrap_or(0);
    repeated as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::{
        extract_json_candidate, generate_fallback_content, looks_like_garbage,
        parse_judgment_content, repeated_char_ratio,
    };
    use sim_core::components::{LlmContent, LlmRequestType};

    #[test]
    fn judgment_parser_accepts_embedded_json() {
        let parsed = parse_judgment_content(
            "prefix {\"action_id\":3,\"confidence\":0.8,\"reasoning_hint\":\"집중\"} suffix",
        )
        .expect("embedded JSON should parse");
        assert_eq!(parsed.action_id, 3);
    }

    #[test]
    fn extract_json_candidate_finds_braced_block() {
        let candidate = extract_json_candidate("abc {\"x\":1} def").expect("candidate expected");
        assert_eq!(candidate, "{\"x\":1}");
    }

    #[test]
    fn garbage_detector_flags_repeated_text() {
        assert!(looks_like_garbage("aaaaaaaaaaaaaaaaaaaa"));
        assert!(repeated_char_ratio("bbbbbbbbbb") > 0.9);
        assert!(!looks_like_garbage("이 사람은 오늘도 조용히 주변을 살폈다."));
    }

    #[test]
    fn fallback_content_matches_request_type() {
        let narrative = generate_fallback_content(LlmRequestType::Layer4Narrative, "카야");
        let judgment = generate_fallback_content(LlmRequestType::Layer3Judgment, "카야");
        assert!(matches!(narrative, LlmContent::Narrative(_)));
        assert!(matches!(judgment, LlmContent::Judgment(_)));
    }
}

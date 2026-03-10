use std::collections::VecDeque;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crossbeam_channel::{bounded, Receiver, RecvTimeoutError, Sender};

use sim_core::components::{JudgmentData, LlmContent, LlmRequestType, LlmRole};
use sim_core::enums::{GrowthStage, Sex};

use crate::llm_prompt::{
    build_inner_prompt_from_context, build_judgment_prompt_from_context,
    build_notification_prompt_from_context, build_personality_prompt_from_context, ActionOption,
    LlmPromptContext, LlmPromptError, LlmPromptTemplates, PromptPayload, SpeechRegister,
};
use crate::llm_server::LlmConfig;
use crate::llm_validator::{load_forbidden_word_list, validate_korean_output};

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
#[derive(Debug, Clone)]
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
    /// Growth stage used for age-sensitive Korean narration.
    pub growth_stage: GrowthStage,
    /// Biological sex used for role-aware wording.
    pub sex: Sex,
    /// Occupation or duty label.
    pub occupation: String,
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
    /// Personal values that shape narration tone.
    pub values: [f64; 33],
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
            role_kind: self.role,
            growth_stage: self.growth_stage,
            sex: self.sex,
            occupation: self.occupation.clone(),
            action_id: self.action_id,
            action_label: self.action_label.clone(),
            personality_axes: self.personality_axes,
            emotions: self.emotions,
            needs: self.needs,
            values: self.values,
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
    debug_log: Arc<Mutex<VecDeque<String>>>,
) {
    let templates = LlmPromptTemplates::load(&config.prompt_dir).ok();

    while let Ok(request) = rx.recv() {
        push_debug_log(
            &debug_log,
            format!(
                "[LLM-DEBUG] llm_worker received request: id={}, type={:?}, variant={:?}, entity_id={}",
                request.request_id,
                request.request_type,
                request.variant,
                request.entity_id
            ),
        );
        let start = Instant::now();
        let response = match templates.as_ref() {
            Some(loaded_templates) => {
                process_request(loaded_templates, &config, &request, &debug_log)
            }
            None => Err(LlmWorkerError::Prompt(LlmPromptError::MissingTemplate(
                "system.jinja".to_string(),
            ))),
        };
        let elapsed = start.elapsed().as_millis() as u32;
        push_debug_log(
            &debug_log,
            format!(
                "[LLM-DEBUG] llm_worker request finished: id={}, success={}, elapsed_ms={}",
                request.request_id,
                response.is_ok(),
                elapsed
            ),
        );
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
        push_debug_log(
            &debug_log,
            format!(
                "[LLM-DEBUG] llm_worker sending response: id={}, success={}, model={}",
                llm_response.request_id, llm_response.success, llm_response.model_id
            ),
        );
        if tx.send(llm_response).is_err() {
            break;
        }
    }
}

fn push_debug_log(debug_log: &Arc<Mutex<VecDeque<String>>>, message: String) {
    let Ok(mut log) = debug_log.lock() else {
        return;
    };
    log.push_back(message);
    while log.len() > sim_core::config::LLM_DEBUG_LOG_CAPACITY {
        let _ = log.pop_front();
    }
}

/// Processes a single request by rendering prompts, calling llama-server, and validating output.
pub fn process_request(
    templates: &LlmPromptTemplates,
    config: &LlmConfig,
    request: &LlmRequest,
    debug_log: &Arc<Mutex<VecDeque<String>>>,
) -> Result<LlmContent, LlmWorkerError> {
    let prompt_context = request.prompt_context();
    let prompt = build_prompt_payload(templates, request, &prompt_context)?;
    push_debug_log(
        debug_log,
        format!(
            "[LLM-DEBUG] llm_worker prepared prompt: id={}, variant={:?}, system_chars={}, user_chars={}",
            request.request_id,
            request.variant,
            prompt.system_prompt.chars().count(),
            prompt.user_prompt.chars().count()
        ),
    );

    let content = process_request_http(config, request, &prompt, debug_log)?;
    match request.request_type {
        LlmRequestType::Layer3Judgment => {
            parse_judgment_content(content.as_str()).map(LlmContent::Judgment)
        }
        LlmRequestType::Layer4Narrative => {
            let expected_register = expected_register_for_request(request);
            let validation = validate_korean_output(
                content.as_str(),
                load_forbidden_word_list(),
                expected_register,
            );
            if validation.pass {
                return Ok(LlmContent::Narrative(validation.cleaned_text));
            }
            push_debug_log(
                debug_log,
                format!(
                    "[LLM-DEBUG] llm_worker validation failed: id={}, violations={}, register_match={}",
                    request.request_id,
                    validation.violations.len(),
                    validation.register_match
                ),
            );
            let repair_prompt = build_layer4_repair_prompt(&prompt);
            let repaired_content =
                process_request_http(config, request, &repair_prompt, debug_log)?;
            let repaired_validation = validate_korean_output(
                repaired_content.as_str(),
                load_forbidden_word_list(),
                expected_register,
            );
            if !repaired_validation.pass {
                push_debug_log(
                    debug_log,
                    format!(
                        "[LLM-DEBUG] llm_worker repair validation failed: id={}, violations={}, register_match={}",
                        request.request_id,
                        repaired_validation.violations.len(),
                        repaired_validation.register_match
                    ),
                );
                return Err(LlmWorkerError::MalformedResponse(
                    "narrative content failed Korean validation".to_string(),
                ));
            }
            Ok(LlmContent::Narrative(repaired_validation.cleaned_text))
        }
    }
}

fn build_prompt_payload(
    templates: &LlmPromptTemplates,
    request: &LlmRequest,
    context: &LlmPromptContext,
) -> Result<PromptPayload, LlmWorkerError> {
    let payload = match request.variant {
        LlmPromptVariant::Judgment => build_judgment_prompt_from_context(
            context,
            &[ActionOption {
                id: request.action_id,
                label: request.action_label.clone(),
            }],
            templates,
        )?,
        LlmPromptVariant::Narrative => {
            if request.recent_event_type.is_some() {
                build_notification_prompt_from_context(context, templates)?
            } else {
                build_inner_prompt_from_context(context, templates)?
            }
        }
        LlmPromptVariant::Personality => build_personality_prompt_from_context(context, templates)?,
    };
    Ok(payload)
}

fn process_request_http(
    config: &LlmConfig,
    request: &LlmRequest,
    prompt: &PromptPayload,
    debug_log: &Arc<Mutex<VecDeque<String>>>,
) -> Result<String, LlmWorkerError> {
    match process_request_http_with_curl(config, request, prompt, debug_log) {
        Ok(content) => return Ok(content),
        Err(error) => {
            push_debug_log(
                debug_log,
                format!(
                    "[LLM-DEBUG] llm_worker curl fallback to ureq: id={}, reason={}",
                    request.request_id, error
                ),
            );
        }
    }

    process_request_http_with_ureq(config, request, prompt, debug_log)
}

fn process_request_http_with_curl(
    config: &LlmConfig,
    request: &LlmRequest,
    prompt: &PromptPayload,
    debug_log: &Arc<Mutex<VecDeque<String>>>,
) -> Result<String, LlmWorkerError> {
    let endpoint = format!("{}/v1/chat/completions", config.base_url());
    let body = request_body(config, request, prompt);
    let body_json =
        serde_json::to_string(&body).map_err(|error| LlmWorkerError::Http(error.to_string()))?;
    let timeout_secs = config.http_timeout_ms.div_ceil(1000).max(1).to_string();
    push_debug_log(
        debug_log,
        format!(
            "[LLM-DEBUG] llm_worker HTTP start: id={}, transport=curl, endpoint={}, max_tokens={}, temperature={:.2}",
            request.request_id,
            endpoint,
            match request.request_type {
                LlmRequestType::Layer3Judgment => config.max_tokens_l3,
                LlmRequestType::Layer4Narrative => config.max_tokens_l4,
            },
            match request.request_type {
                LlmRequestType::Layer3Judgment => config.temperature_l3,
                LlmRequestType::Layer4Narrative => config.temperature_l4,
            }
        ),
    );

    let curl_binary = if std::path::Path::new("/usr/bin/curl").is_file() {
        "/usr/bin/curl"
    } else {
        "curl"
    };

    let output = Command::new(curl_binary)
        .args([
            "-sS",
            "--noproxy",
            "*",
            "--max-time",
            timeout_secs.as_str(),
            "--connect-timeout",
            timeout_secs.as_str(),
            "-H",
            "Content-Type: application/json",
            "-X",
            "POST",
            endpoint.as_str(),
            "--data",
            body_json.as_str(),
        ])
        .output()
        .map_err(|error| LlmWorkerError::Http(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(LlmWorkerError::Http(format!(
            "curl exited with status {}: {}",
            output.status, stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| LlmWorkerError::Http(error.to_string()))?;
    let response_value: Value = serde_json::from_str(stdout.as_str())
        .map_err(|error| LlmWorkerError::MalformedResponse(error.to_string()))?;
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
    push_debug_log(
        debug_log,
        format!(
            "[LLM-DEBUG] llm_worker HTTP content received: id={}, transport=curl, chars={}",
            request.request_id,
            content.chars().count()
        ),
    );
    Ok(content.to_string())
}

fn process_request_http_with_ureq(
    config: &LlmConfig,
    request: &LlmRequest,
    prompt: &PromptPayload,
    debug_log: &Arc<Mutex<VecDeque<String>>>,
) -> Result<String, LlmWorkerError> {
    let request_id = request.request_id;
    let endpoint = format!("{}/v1/chat/completions", config.base_url());
    let config_for_thread = config.clone();
    let request_for_thread = request.clone();
    let prompt_for_thread = prompt.clone();
    let endpoint_for_thread = endpoint.clone();
    let debug_log_for_thread = Arc::clone(debug_log);
    let (response_tx, response_rx) = bounded::<Result<String, LlmWorkerError>>(1);

    thread::spawn(move || {
        let timeout = Duration::from_millis(config_for_thread.http_timeout_ms);
        let client = ureq::AgentBuilder::new()
            .try_proxy_from_env(false)
            .timeout(timeout)
            .timeout_connect(timeout)
            .build();
        let body = request_body(&config_for_thread, &request_for_thread, &prompt_for_thread);
        push_debug_log(
            &debug_log_for_thread,
            format!(
                "[LLM-DEBUG] llm_worker HTTP start: id={}, transport=ureq, endpoint={}, max_tokens={}, temperature={:.2}",
                request_for_thread.request_id,
                endpoint_for_thread,
                match request_for_thread.request_type {
                    LlmRequestType::Layer3Judgment => config_for_thread.max_tokens_l3,
                    LlmRequestType::Layer4Narrative => config_for_thread.max_tokens_l4,
                },
                match request_for_thread.request_type {
                    LlmRequestType::Layer3Judgment => config_for_thread.temperature_l3,
                    LlmRequestType::Layer4Narrative => config_for_thread.temperature_l4,
                }
            ),
        );

        let result = (|| -> Result<String, LlmWorkerError> {
            let response = client
                .post(endpoint_for_thread.as_str())
                .set("Content-Type", "application/json")
                .send_json(body)
                .map_err(|error| LlmWorkerError::Http(error.to_string()))?;
            push_debug_log(
                &debug_log_for_thread,
                format!(
                    "[LLM-DEBUG] llm_worker HTTP response headers: id={}, transport=ureq, status={}",
                    request_for_thread.request_id,
                    response.status()
                ),
            );
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
                .ok_or_else(|| {
                    LlmWorkerError::MalformedResponse("missing message content".to_string())
                })?;
            push_debug_log(
                &debug_log_for_thread,
                format!(
                    "[LLM-DEBUG] llm_worker HTTP content received: id={}, transport=ureq, chars={}",
                    request_for_thread.request_id,
                    content.chars().count()
                ),
            );
            Ok(content.to_string())
        })();

        let _ = response_tx.send(result);
    });

    match response_rx.recv_timeout(Duration::from_millis(config.http_timeout_ms)) {
        Ok(Ok(content)) => Ok(content),
        Ok(Err(error)) => Err(error),
        Err(RecvTimeoutError::Timeout) => {
            push_debug_log(
                debug_log,
                format!(
                    "[LLM-DEBUG] llm_worker HTTP timeout: id={}, transport=ureq, timeout_ms={}",
                    request_id, config.http_timeout_ms
                ),
            );
            Err(LlmWorkerError::Http(format!(
                "request timed out after {}ms",
                config.http_timeout_ms
            )))
        }
        Err(RecvTimeoutError::Disconnected) => Err(LlmWorkerError::Http(format!(
            "request worker disconnected for id={request_id}"
        ))),
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

fn request_body(config: &LlmConfig, request: &LlmRequest, prompt: &PromptPayload) -> Value {
    let mut body = serde_json::json!({
        "model": config.model_id,
        "messages": [
            { "role": "system", "content": prompt.system_prompt },
            { "role": "user", "content": prompt.user_prompt }
        ],
        "max_tokens": prompt.max_tokens,
        "temperature": prompt.temperature,
        "stream": false,
    });
    if let Some(grammar) = &prompt.grammar {
        body["grammar"] = serde_json::json!(grammar);
    } else if matches!(request.request_type, LlmRequestType::Layer3Judgment) {
        body["grammar"] = serde_json::json!(config.layer3_grammar);
    }
    body
}

fn expected_register_for_request(request: &LlmRequest) -> SpeechRegister {
    match request.role {
        LlmRole::Leader | LlmRole::Shaman | LlmRole::Oracle => SpeechRegister::Hao,
        LlmRole::Agent => {
            if request.personality_axes[1] > 0.7
                || matches!(
                    request.growth_stage,
                    GrowthStage::Infant
                        | GrowthStage::Toddler
                        | GrowthStage::Child
                        | GrowthStage::Teen
                )
            {
                SpeechRegister::Hae
            } else {
                SpeechRegister::Haera
            }
        }
    }
}

fn build_layer4_repair_prompt(prompt: &PromptPayload) -> PromptPayload {
    let mut repaired = prompt.clone();
    repaired
        .user_prompt
        .push_str("\n덧붙임: 머릿말과 항목 이름 없이 짧은 본문만 1-2문장으로 다시 적어라.");
    repaired.max_tokens = repaired
        .max_tokens
        .min(sim_core::config::LLM_MAX_TOKENS_L4_REPAIR);
    repaired.temperature = sim_core::config::LLM_TEMPERATURE_L4_REPAIR;
    repaired
}

fn parse_judgment_content(content: &str) -> Result<JudgmentData, LlmWorkerError> {
    parse_judgment_json(content).or_else(|_| {
        extract_json_candidate(content)
            .and_then(|candidate| parse_judgment_json(candidate.as_str()).ok())
            .ok_or(LlmWorkerError::JudgmentParseFailed)
    })
}

fn parse_judgment_json(content: &str) -> Result<JudgmentData, LlmWorkerError> {
    serde_json::from_str::<JudgmentData>(content).map_err(|_| LlmWorkerError::JudgmentParseFailed)
}

fn extract_json_candidate(content: &str) -> Option<String> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(content[start..=end].to_string())
}

#[cfg(test)]
fn looks_like_garbage(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.chars().count() < 10 {
        return true;
    }
    let repeated = repeated_char_ratio(trimmed);
    repeated >= 0.9
}

#[cfg(test)]
fn repeated_char_ratio(content: &str) -> f64 {
    let total = content.chars().count();
    if total == 0 {
        return 1.0;
    }
    let repeated = content
        .chars()
        .fold(
            std::collections::HashMap::<char, usize>::new(),
            |mut acc, ch| {
                *acc.entry(ch).or_insert(0) += 1;
                acc
            },
        )
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
        assert!(!looks_like_garbage(
            "이 사람은 오늘도 조용히 주변을 살폈다."
        ));
    }

    #[test]
    fn fallback_content_matches_request_type() {
        let narrative = generate_fallback_content(LlmRequestType::Layer4Narrative, "카야");
        let judgment = generate_fallback_content(LlmRequestType::Layer3Judgment, "카야");
        assert!(matches!(narrative, LlmContent::Narrative(_)));
        assert!(matches!(judgment, LlmContent::Judgment(_)));
    }
}

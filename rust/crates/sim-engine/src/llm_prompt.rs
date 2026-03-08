use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use minijinja::Environment;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use sim_core::components::{Behavior, Emotion, Identity, LlmRole, Personality, Values};
use sim_core::config;
use sim_core::enums::{EmotionType, GrowthStage, Sex};

/// Request context serialized into prompt templates and worker requests.
#[derive(Debug, Clone)]
pub struct LlmPromptContext {
    /// Entity display name.
    pub entity_name: String,
    /// Narrative role label.
    pub role: String,
    /// Narrative role enum.
    pub role_kind: LlmRole,
    /// Growth stage used for age-aware prompt wording.
    pub growth_stage: GrowthStage,
    /// Biological sex label.
    pub sex: Sex,
    /// Occupation or current duty label.
    pub occupation: String,
    /// Current action identifier from the closed action space.
    pub action_id: u32,
    /// Human-readable action label.
    pub action_label: String,
    /// Current HEXACO axes.
    pub personality_axes: [f64; 6],
    /// Current Plutchik primary emotion values.
    pub emotions: [f64; 8],
    /// Current 13-need values.
    pub needs: [f64; 13],
    /// Current personal values.
    pub values: [f64; 33],
    /// Current stress level.
    pub stress_level: f64,
    /// Current stress-state bucket.
    pub stress_state: u8,
    /// Optional recent event label.
    pub recent_event_type: Option<String>,
    /// Optional recent event cause description.
    pub recent_event_cause: Option<String>,
    /// Optional recent target/partner name.
    pub recent_target_name: Option<String>,
}

/// Fully rendered prompt payload sent to llama-server.
#[derive(Debug, Clone)]
pub struct PromptPayload {
    /// Shared system prompt.
    pub system_prompt: String,
    /// Request-specific user prompt.
    pub user_prompt: String,
    /// Optional grammar constraint.
    pub grammar: Option<String>,
    /// Per-request token cap.
    pub max_tokens: u32,
    /// Per-request sampling temperature.
    pub temperature: f64,
}

/// Top-level closed action option passed to Layer 3 prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOption {
    /// Closed-set numeric action id.
    pub id: u32,
    /// Human-readable label shown to the model.
    pub label: String,
}

/// Human-readable Korean HEXACO descriptor bundle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HexacoDescriptors {
    /// Honesty-humility descriptor.
    pub h_desc: String,
    /// Emotionality descriptor.
    pub e_desc: String,
    /// Extraversion descriptor.
    pub x_desc: String,
    /// Agreeableness descriptor.
    pub a_desc: String,
    /// Conscientiousness descriptor.
    pub c_desc: String,
    /// Openness descriptor.
    pub o_desc: String,
    /// Top active trait summaries.
    pub dominant_traits: Vec<String>,
    /// Top embraced values.
    pub dominant_values: Vec<String>,
}

impl HexacoDescriptors {
    /// Returns a short summary string suitable for the shared system prompt.
    pub fn to_summary_string(&self) -> String {
        self.dominant_traits.join(", ")
    }
}

/// Narrative speech register expected from the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeechRegister {
    /// 해라체: matter-of-fact narration and internal monologue.
    Haera,
    /// 하오체: leader/elder/shaman declaration tone.
    Hao,
    /// 해체: emotionally exposed, intimate tone.
    Hae,
}

impl SpeechRegister {
    /// Returns the register rule inserted into the system prompt.
    pub fn to_instruction_string(self) -> &'static str {
        match self {
            Self::Haera => "해라체로 말하라. (-다, -는다, -냐, -라로 끝맺어라)",
            Self::Hao => "하오체로 말하라. (-오, -소, -구려로 끝맺어라)",
            Self::Hae => "해체로 말하라. (-해, -야, -지로 끝맺어라)",
        }
    }

    /// Returns whether the text ends in a suffix that matches this register.
    pub fn matches_text(self, text: &str) -> bool {
        let trimmed = text
            .trim()
            .trim_end_matches(['.', '!', '?', '\n', '\r', ' ']);
        let sentence = trimmed.rsplit('\n').next().unwrap_or(trimmed).trim();

        let endings: &[&str] = match self {
            Self::Haera => &["다", "는다", "란다", "렸다", "라", "냐"],
            Self::Hao => &["오", "소", "구려", "이오"],
            Self::Hae => &["해", "야", "지", "네"],
        };

        endings.iter().any(|ending| sentence.ends_with(ending))
    }
}

/// Loaded prompt templates for the local LLM runtime.
pub struct LlmPromptTemplates {
    env: Environment<'static>,
}

/// Errors emitted while loading or rendering prompt templates.
#[derive(Debug, Error)]
pub enum LlmPromptError {
    #[error("prompt template IO failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("prompt template not found: {0}")]
    MissingTemplate(String),
    #[error("prompt render failed: {0}")]
    Render(String),
}

#[derive(Debug, Clone, Deserialize)]
struct AxisDescriptorSet {
    high: String,
    mid: String,
    low: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
struct HexacoDescriptorTable {
    H: AxisDescriptorSet,
    E: AxisDescriptorSet,
    X: AxisDescriptorSet,
    A: AxisDescriptorSet,
    C: AxisDescriptorSet,
    O: AxisDescriptorSet,
}

const NEED_LABELS: [&str; 13] = [
    "배고픔",
    "목마름",
    "잠",
    "따스함",
    "안전함",
    "한패 됨",
    "살닿는 가까움",
    "인정",
    "제 뜻대로 함",
    "솜씨",
    "제 삶을 이룸",
    "삶의 뜻",
    "제 몸 너머의 뜻",
];

const VALUE_LABELS: [&str; 33] = [
    "법도", "한패 의리", "집안", "벗됨", "힘", "참말", "꾀", "말재주", "곧음", "예절",
    "옛법", "꾸밈결", "서로 도움", "홀로섬", "굳셈", "제 마음 들여다봄", "절제", "고요함",
    "화목", "흥겨움", "손솜씨", "싸움 기운", "재주", "부지런함", "내어줌", "겨룸",
    "버팀", "쉼", "바꾸기", "사랑", "앎", "온 누리", "평온",
];

static HEXACO_DESCRIPTOR_TABLE: OnceLock<HexacoDescriptorTable> = OnceLock::new();

impl LlmPromptTemplates {
    /// Loads all prompt templates from the given directory.
    pub fn load(prompt_dir: &Path) -> Result<Self, LlmPromptError> {
        let mut env = Environment::new();
        for template_name in [
            "system.jinja",
            "system_korean.jinja",
            "layer3_judgment.jinja",
            "layer4_narrative.jinja",
            "layer4_personality.jinja",
            "use_case_b_personality.jinja",
            "use_case_h_notification.jinja",
        ] {
            let template_path = prompt_dir.join(template_name);
            if !template_path.is_file() {
                continue;
            }
            let source = fs::read_to_string(&template_path)?;
            let leaked_source: &'static str = Box::leak(source.into_boxed_str());
            env.add_template(template_name, leaked_source)
                .map_err(|error| LlmPromptError::Render(error.to_string()))?;
        }
        Ok(Self { env })
    }

    /// Loads templates from the default project prompt directory.
    pub fn load_default() -> Result<Self, LlmPromptError> {
        Self::load(&default_prompt_dir())
    }

    fn render<S: Serialize>(
        &self,
        template_name: &str,
        context: S,
    ) -> Result<String, LlmPromptError> {
        let template = self
            .env
            .get_template(template_name)
            .map_err(|_| LlmPromptError::MissingTemplate(template_name.to_string()))?;
        template
            .render(context)
            .map_err(|error| LlmPromptError::Render(error.to_string()))
    }
}

/// Builds a full Layer 4 personality prompt for the given entity state.
pub fn build_personality_prompt(
    identity: &Identity,
    role: LlmRole,
    personality: &Personality,
    emotion: &Emotion,
    values: &Values,
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let context = context_from_components(identity, role, personality, emotion, None, values, None);
    build_personality_prompt_from_context(&context, templates)
}

/// Builds a full Layer 4 notification prompt for the given entity state.
#[allow(clippy::too_many_arguments)]
pub fn build_notification_prompt(
    identity: &Identity,
    role: LlmRole,
    personality: &Personality,
    emotion: &Emotion,
    behavior: &Behavior,
    values: &Values,
    event_context: &str,
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let context = context_from_components(
        identity,
        role,
        personality,
        emotion,
        Some(behavior),
        values,
        Some(event_context.to_string()),
    );
    build_notification_prompt_from_context(&context, templates)
}

/// Builds a Layer 3 closed-set judgment prompt for the given entity state.
#[allow(clippy::too_many_arguments)]
pub fn build_judgment_prompt(
    identity: &Identity,
    role: LlmRole,
    personality: &Personality,
    emotion: &Emotion,
    behavior: &Behavior,
    values: &Values,
    action_options: &[ActionOption],
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let context = context_from_components(identity, role, personality, emotion, Some(behavior), values, None);
    build_judgment_prompt_from_context(&context, action_options, templates)
}

/// Maps HEXACO scores to Korean descriptors and trait summaries.
pub fn hexaco_to_korean_descriptors(
    personality: &Personality,
    values: Option<&Values>,
) -> HexacoDescriptors {
    let table = load_hexaco_descriptor_table();
    let levels = personality.axes.map(score_to_level);

    let mut ranked_axes: Vec<(usize, f64)> = personality
        .axes
        .iter()
        .enumerate()
        .map(|(idx, value)| (idx, (value - 0.5).abs()))
        .collect();
    ranked_axes.sort_by(|left, right| right.1.total_cmp(&left.1));

    let dominant_traits = ranked_axes
        .into_iter()
        .take(5)
        .map(|(axis, _)| dominant_trait_for_axis(axis, personality.axes[axis]))
        .collect();

    HexacoDescriptors {
        h_desc: select_axis_descriptor(&table.H, levels[0]).to_string(),
        e_desc: select_axis_descriptor(&table.E, levels[1]).to_string(),
        x_desc: select_axis_descriptor(&table.X, levels[2]).to_string(),
        a_desc: select_axis_descriptor(&table.A, levels[3]).to_string(),
        c_desc: select_axis_descriptor(&table.C, levels[4]).to_string(),
        o_desc: select_axis_descriptor(&table.O, levels[5]).to_string(),
        dominant_traits,
        dominant_values: values
            .map(values_to_korean_descriptors)
            .unwrap_or_default(),
    }
}

/// Maps current Plutchik intensities to a short Korean emotional context.
pub fn emotion_to_korean_context(emotion: &Emotion) -> String {
    let mut ranked: Vec<(usize, f64)> = emotion
        .primary
        .iter()
        .copied()
        .enumerate()
        .collect();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));

    let (dominant_idx, dominant_value) = ranked.first().copied().unwrap_or((0, 0.0));
    let dominant_desc = emotion_phrase(dominant_idx);
    let intensity_prefix = if dominant_value > 0.8 {
        "극도로 "
    } else if dominant_value > 0.5 {
        ""
    } else {
        "살짝 "
    };

    let secondary = ranked
        .iter()
        .copied()
        .skip(1)
        .find(|(_, value)| *value > 0.35);
    if let Some((secondary_idx, _)) = secondary {
        format!(
            "{}{} 그러면서도 {}",
            intensity_prefix,
            dominant_desc,
            emotion_secondary_phrase(secondary_idx)
        )
    } else {
        format!("{intensity_prefix}{dominant_desc}")
    }
}

/// Selects the narrative speech register for the current role and personality.
pub fn select_register(
    personality: &Personality,
    identity: &Identity,
    role: LlmRole,
) -> SpeechRegister {
    match role {
        LlmRole::Leader | LlmRole::Shaman | LlmRole::Oracle => SpeechRegister::Hao,
        LlmRole::Agent => {
            if personality.axes[1] > 0.7 || identity.growth_stage.is_child_age() {
                SpeechRegister::Hae
            } else {
                SpeechRegister::Haera
            }
        }
    }
}

/// Builds the Layer 4 personality prompt from pre-serialized request context.
pub fn build_personality_prompt_from_context(
    context: &LlmPromptContext,
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let descriptors = context_descriptors(context);
    let register = select_register_from_context(context);
    let emotion_context = emotion_to_korean_context_from_values(&context.emotions);

    let system_prompt = templates.render(
        pick_system_template(templates),
        minijinja::context! {
            register_instruction => register.to_instruction_string(),
            hexaco_summary => descriptors.to_summary_string(),
            emotion_context => &emotion_context,
            length_instruction => "3-5문장으로 적어라.",
        },
    )?;
    let user_prompt = templates.render(
        "use_case_b_personality.jinja",
        minijinja::context! {
            name => &context.entity_name,
            age => growth_stage_to_korean(context.growth_stage),
            role => role_to_korean(context.role_kind),
            h_desc => &descriptors.h_desc,
            e_desc => &descriptors.e_desc,
            x_desc => &descriptors.x_desc,
            a_desc => &descriptors.a_desc,
            c_desc => &descriptors.c_desc,
            o_desc => &descriptors.o_desc,
            dominant_traits => &descriptors.dominant_traits,
            dominant_values => &descriptors.dominant_values,
            emotion_context => &emotion_context,
        },
    )?;

    Ok(PromptPayload {
        system_prompt,
        user_prompt,
        grammar: load_layer4_bounded_grammar(),
        max_tokens: config::LLM_MAX_TOKENS_L4_PERSONALITY,
        temperature: config::LLM_TEMPERATURE_L4_PERSONALITY,
    })
}

/// Builds the Layer 4 notification prompt from pre-serialized request context.
pub fn build_notification_prompt_from_context(
    context: &LlmPromptContext,
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let descriptors = context_descriptors(context);
    let register = select_register_from_context(context);
    let emotion_context = emotion_to_korean_context_from_values(&context.emotions);
    let event_description = build_event_description(context);

    let system_prompt = templates.render(
        pick_system_template(templates),
        minijinja::context! {
            register_instruction => register.to_instruction_string(),
            hexaco_summary => descriptors.to_summary_string(),
            emotion_context => &emotion_context,
            length_instruction => "1-2문장으로 적어라.",
        },
    )?;
    let user_prompt = templates.render(
        "use_case_h_notification.jinja",
        minijinja::context! {
            name => &context.entity_name,
            age => growth_stage_to_korean(context.growth_stage),
            role => role_to_korean(context.role_kind),
            action_label => action_label_to_korean(context.action_label.as_str()),
            hexaco_summary => descriptors.to_summary_string(),
            emotion_context => &emotion_context,
            dominant_values => &descriptors.dominant_values,
            event_description => event_description,
        },
    )?;

    Ok(PromptPayload {
        system_prompt,
        user_prompt,
        grammar: load_layer4_bounded_grammar(),
        max_tokens: config::LLM_MAX_TOKENS_L4_NOTIFICATION,
        temperature: config::LLM_TEMPERATURE_L4_NOTIFICATION,
    })
}

/// Builds the Layer 4 inner-monologue prompt from pre-serialized request context.
pub fn build_inner_prompt_from_context(
    context: &LlmPromptContext,
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let descriptors = context_descriptors(context);
    let register = select_register_from_context(context);
    let emotion_context = emotion_to_korean_context_from_values(&context.emotions);
    let need_context = needs_to_korean_context(&context.needs);

    let system_prompt = templates.render(
        pick_system_template(templates),
        minijinja::context! {
            register_instruction => register.to_instruction_string(),
            hexaco_summary => descriptors.to_summary_string(),
            emotion_context => &emotion_context,
            length_instruction => "2-4문장으로 적어라.",
        },
    )?;
    let user_prompt = templates.render(
        "layer4_narrative.jinja",
        minijinja::context! {
            name => &context.entity_name,
            age => growth_stage_to_korean(context.growth_stage),
            role => role_to_korean(context.role_kind),
            action_label => action_label_to_korean(context.action_label.as_str()),
            hexaco_summary => descriptors.to_summary_string(),
            emotion_context => &emotion_context,
            need_context => &need_context,
        },
    )?;

    Ok(PromptPayload {
        system_prompt,
        user_prompt,
        grammar: load_layer4_bounded_grammar(),
        max_tokens: config::LLM_MAX_TOKENS_L4_INNER,
        temperature: config::LLM_TEMPERATURE_L4_INNER,
    })
}

/// Builds the Layer 3 judgment prompt from pre-serialized request context.
pub fn build_judgment_prompt_from_context(
    context: &LlmPromptContext,
    action_options: &[ActionOption],
    templates: &LlmPromptTemplates,
) -> Result<PromptPayload, LlmPromptError> {
    let descriptors = context_descriptors(context);
    let register = select_register_from_context(context);
    let emotion_context = emotion_to_korean_context_from_values(&context.emotions);
    let need_context = needs_to_korean_context(&context.needs);

    let system_prompt = templates.render(
        pick_system_template(templates),
        minijinja::context! {
            register_instruction => register.to_instruction_string(),
            hexaco_summary => descriptors.to_summary_string(),
            emotion_context => &emotion_context,
            length_instruction => "JSON 하나만 적어라.",
        },
    )?;
    let user_prompt = templates.render(
        "layer3_judgment.jinja",
        minijinja::context! {
            name => &context.entity_name,
            age => growth_stage_to_korean(context.growth_stage),
            role => role_to_korean(context.role_kind),
            action_label => action_label_to_korean(context.action_label.as_str()),
            h_desc => &descriptors.h_desc,
            e_desc => &descriptors.e_desc,
            x_desc => &descriptors.x_desc,
            a_desc => &descriptors.a_desc,
            c_desc => &descriptors.c_desc,
            o_desc => &descriptors.o_desc,
            emotion_context => &emotion_context,
            need_context => &need_context,
            action_options => action_options,
        },
    )?;

    Ok(PromptPayload {
        system_prompt,
        user_prompt,
        grammar: Some(load_layer3_grammar()),
        max_tokens: config::LLM_MAX_TOKENS_L3,
        temperature: config::LLM_TEMPERATURE_L3,
    })
}

/// Returns the default project prompt directory.
pub fn default_prompt_dir() -> PathBuf {
    project_root().join(config::LLM_PROMPT_DIR)
}

fn context_from_components(
    identity: &Identity,
    role: LlmRole,
    personality: &Personality,
    emotion: &Emotion,
    behavior: Option<&Behavior>,
    values: &Values,
    event_description: Option<String>,
) -> LlmPromptContext {
    LlmPromptContext {
        entity_name: identity.name.clone(),
        role: role_to_korean(role).to_string(),
        role_kind: role,
        growth_stage: identity.growth_stage,
        sex: identity.sex,
        occupation: behavior
            .map(|value| value.occupation.clone())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "없음".to_string()),
        action_id: behavior.map(|value| value.current_action as u32).unwrap_or_default(),
        action_label: behavior
            .map(|value| value.current_action.to_string())
            .unwrap_or_else(|| "Idle".to_string()),
        personality_axes: personality.axes,
        emotions: emotion.primary,
        needs: [0.5; 13],
        values: values.values,
        stress_level: 0.0,
        stress_state: 0,
        recent_event_type: event_description,
        recent_event_cause: None,
        recent_target_name: None,
    }
}

fn context_descriptors(context: &LlmPromptContext) -> HexacoDescriptors {
    let personality = Personality {
        axes: context.personality_axes,
        facets: [0.5; 24],
    };
    let values = Values {
        values: context.values,
    };
    hexaco_to_korean_descriptors(&personality, Some(&values))
}

fn select_register_from_context(context: &LlmPromptContext) -> SpeechRegister {
    let personality = Personality {
        axes: context.personality_axes,
        facets: [0.5; 24],
    };
    let identity = Identity {
        name: context.entity_name.clone(),
        growth_stage: context.growth_stage,
        sex: context.sex,
        ..Identity::default()
    };
    select_register(&personality, &identity, context.role_kind)
}

fn emotion_to_korean_context_from_values(emotions: &[f64; 8]) -> String {
    let emotion = Emotion {
        primary: *emotions,
        baseline: [0.0; 8],
    };
    emotion_to_korean_context(&emotion)
}

fn build_event_description(context: &LlmPromptContext) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(event_type) = &context.recent_event_type {
        parts.push(event_type_to_korean(event_type).to_string());
    }
    if let Some(cause) = &context.recent_event_cause {
        parts.push(event_cause_to_korean(cause));
    }
    if let Some(target) = &context.recent_target_name {
        parts.push(format!("{target}와 얽혔다"));
    }
    if parts.is_empty() {
        parts.push(format!(
            "{}이(가) {}에 매달려 있다",
            context.entity_name,
            action_label_to_korean(context.action_label.as_str())
        ));
    }
    parts.join(". ")
}

fn pick_system_template(templates: &LlmPromptTemplates) -> &'static str {
    if templates.env.get_template("system_korean.jinja").is_ok() {
        "system_korean.jinja"
    } else {
        "system.jinja"
    }
}

fn load_hexaco_descriptor_table() -> &'static HexacoDescriptorTable {
    HEXACO_DESCRIPTOR_TABLE.get_or_init(|| {
        let path = project_root().join(config::LLM_HEXACO_DESCRIPTOR_PATH);
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<HexacoDescriptorTable>(&raw).ok())
            .unwrap_or_else(default_hexaco_descriptor_table)
    })
}

fn load_layer3_grammar() -> String {
    fs::read_to_string(project_root().join(config::LLM_LAYER3_GRAMMAR_PATH))
        .unwrap_or_else(|_| String::new())
}

fn load_layer4_bounded_grammar() -> Option<String> {
    let grammar = fs::read_to_string(project_root().join(config::LLM_LAYER4_BOUNDED_GRAMMAR_PATH))
        .unwrap_or_else(|_| String::new());
    if grammar.trim().is_empty() {
        None
    } else {
        Some(grammar)
    }
}

fn project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

fn default_hexaco_descriptor_table() -> HexacoDescriptorTable {
    HexacoDescriptorTable {
        H: AxisDescriptorSet {
            high: "높은 정직-겸손: 정직하고 욕심이 적으며, 남을 속이지 않는다".to_string(),
            mid: "가운데 정직-겸손: 때로는 솔직하고, 때로는 제 이득을 챙긴다".to_string(),
            low: "낮은 정직-겸손: 교활하고 탐욕스러우며, 제 것을 남에게 내주지 않는다".to_string(),
        },
        E: AxisDescriptorSet {
            high: "높은 정서성: 겁이 많고 불안해하며, 남에게 기대려 한다".to_string(),
            mid: "가운데 정서성: 보통의 두려움을 느끼며, 위험 앞에서 망설인다".to_string(),
            low: "낮은 정서성: 두려움이 없고 대담하며, 위험을 즐긴다".to_string(),
        },
        X: AxisDescriptorSet {
            high: "높은 외향성: 사람들 사이에서 활기차고, 무리를 이끌려 한다".to_string(),
            mid: "가운데 외향성: 때로는 나서고, 때로는 물러선다".to_string(),
            low: "낮은 외향성: 혼자 있기를 좋아하며, 말이 적다".to_string(),
        },
        A: AxisDescriptorSet {
            high: "높은 원만함: 남의 잘못을 쉽게 용서하고, 부드럽게 대한다".to_string(),
            mid: "가운데 원만함: 보통의 참을성을 가지고, 판에 따라 맞선다".to_string(),
            low: "낮은 원만함: 원한을 오래 품고, 화를 잘 내며, 앙갚음을 벼른다".to_string(),
        },
        C: AxisDescriptorSet {
            high: "높은 성실성: 꼼꼼하고 부지런하며, 맡은 일을 끝까지 해낸다".to_string(),
            mid: "가운데 성실성: 보통의 성실함으로, 해야 할 일은 한다".to_string(),
            low: "낮은 성실성: 되는대로 살며, 앞뒤 헤아림 없이 움직인다".to_string(),
        },
        O: AxisDescriptorSet {
            high: "높은 개방성: 새것에 끌리며, 보지 못한 것을 기웃거린다".to_string(),
            mid: "가운데 개방성: 익숙한 것을 편히 여기되, 새것도 마다하지 않는다".to_string(),
            low: "낮은 개방성: 달라짐을 싫어하고, 옛 방식을 굳게 붙든다".to_string(),
        },
    }
}

fn select_axis_descriptor<'a>(axis: &'a AxisDescriptorSet, level: &'static str) -> &'a str {
    match level {
        "high" => axis.high.as_str(),
        "mid" => axis.mid.as_str(),
        _ => axis.low.as_str(),
    }
}

fn score_to_level(score: f64) -> &'static str {
    if score >= 0.7 {
        "high"
    } else if score >= 0.3 {
        "mid"
    } else {
        "low"
    }
}

fn dominant_trait_for_axis(axis: usize, score: f64) -> String {
    let level = score_to_level(score);
    match (axis, level) {
        (0, "high") => "남을 속이기보다 곧게 말하려 든다".to_string(),
        (0, "low") => "제 몫을 놓치지 않으려 눈치를 날카롭게 본다".to_string(),
        (1, "high") => "겁과 근심이 쉽게 마음을 흔든다".to_string(),
        (1, "low") => "겁보다 대담함이 앞서 나간다".to_string(),
        (2, "high") => "사람들 틈으로 먼저 걸어 들어가려 한다".to_string(),
        (2, "low") => "무리 한켠에 조용히 머무르려 한다".to_string(),
        (3, "high") => "쉽게 누그러지고 남의 잘못을 오래 물지 않는다".to_string(),
        (3, "low") => "서운함을 오래 품고 맞서려 든다".to_string(),
        (4, "high") => "맡은 일을 끝맺을 때까지 손을 놓지 않는다".to_string(),
        (4, "low") => "그때그때 마음 가는 쪽으로 움직인다".to_string(),
        (5, "high") => "새것을 보면 다가가 만져 보고 싶어 한다".to_string(),
        (5, "low") => "낯선 것보다 익숙한 길을 붙든다".to_string(),
        (0, _) => "곧음과 제 이익 사이를 오락가락한다".to_string(),
        (1, _) => "두려움과 담대함이 비슷하게 맞선다".to_string(),
        (2, _) => "나설 때와 물러설 때가 반반쯤 된다".to_string(),
        (3, _) => "참을 때와 맞설 때를 저울질한다".to_string(),
        (4, _) => "해야 할 일은 하되 느슨함도 남아 있다".to_string(),
        _ => "옛것과 새것을 함께 살핀다".to_string(),
    }
}

fn values_to_korean_descriptors(values: &Values) -> Vec<String> {
    let mut ranked: Vec<(usize, f64)> = values
        .values
        .iter()
        .copied()
        .enumerate()
        .collect();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    ranked
        .into_iter()
        .take(4)
        .map(|(idx, _)| VALUE_LABELS[idx].to_string())
        .collect()
}

fn needs_to_korean_context(needs: &[f64; 13]) -> String {
    let mut ranked: Vec<(usize, f64)> = needs.iter().copied().enumerate().collect();
    ranked.sort_by(|left, right| left.1.total_cmp(&right.1));
    let labels: Vec<&str> = ranked
        .into_iter()
        .take(3)
        .map(|(idx, _)| NEED_LABELS[idx])
        .collect();
    format!("지금 가장 모자란 것은 {} 쪽이다.", labels.join(", "))
}

fn role_to_korean(role: LlmRole) -> &'static str {
    match role {
        LlmRole::Agent => "사람",
        LlmRole::Leader => "우두머리",
        LlmRole::Shaman => "무당",
        LlmRole::Oracle => "점치는 이",
    }
}

fn growth_stage_to_korean(stage: GrowthStage) -> &'static str {
    match stage {
        GrowthStage::Infant => "젖먹이",
        GrowthStage::Toddler => "걸음마 아이",
        GrowthStage::Child => "아이",
        GrowthStage::Teen => "청소년",
        GrowthStage::Adult => "어른",
        GrowthStage::Elder => "늙은이",
    }
}

fn action_label_to_korean(action_label: &str) -> String {
    match action_label {
        "Idle" => "가만히 머묾".to_string(),
        "Rest" => "쉬기".to_string(),
        "Sleep" => "잠자기".to_string(),
        "Forage" => "먹을거리 찾기".to_string(),
        "Hunt" => "짐승 쫓기".to_string(),
        "Fish" => "물고기 잡기".to_string(),
        "Build" => "집짓기".to_string(),
        "Drink" => "물 마시기".to_string(),
        "Explore" => "낯선 곳 살피기".to_string(),
        "Wander" => "둘러다니기".to_string(),
        "Socialize" => "어울리기".to_string(),
        "GatherWood" => "나무 모으기".to_string(),
        "GatherStone" => "돌 모으기".to_string(),
        _ => action_label.to_string(),
    }
}

fn event_type_to_korean(event_type: &str) -> &'static str {
    match event_type {
        "need_critical" => "무언가 몹시 모자라다",
        "need_satisfied" => "모자라던 바람을 잠깐 채웠다",
        "emotion_shift" => "마음결이 달라졌다",
        "mood_changed" => "기분이 바뀌었다",
        "stress_escalated" => "속이 더 다급해졌다",
        "mental_break_start" => "마음이 무너질 듯 흔들렸다",
        "mental_break_end" => "무너진 마음이 조금 가라앉았다",
        "relationship_formed" => "새로운 가까운 사이가 맺어졌다",
        "relationship_broken" => "가까운 사이가 금이 갔다",
        "social_conflict" => "서로 맞부딪쳤다",
        "social_cooperation" => "힘을 모아 함께 움직였다",
        "action_changed" => "하던 일을 바꾸었다",
        "task_completed" => "하던 일을 끝냈다",
        "birth" => "새 숨결이 태어났다",
        "death" => "숨이 다하였다",
        "age_transition" => "삶의 새 갈피로 접어들었다",
        "first_occurrence" => "처음 보는 일이 일어났다",
        _ => "무언가 달라졌다",
    }
}

fn event_cause_to_korean(cause: &str) -> String {
    let translated = match cause {
        "hurtful_words" => Some("모진 말이 오갔다"),
        "broken_promise" => Some("하겠다던 일을 저버렸다"),
        "resource_shortage" => Some("먹을거리와 쓸거리가 달렸다"),
        "resource_conflict" => Some("나눌 몫을 두고 다퉜다"),
        "physical_threat" => Some("몸을 다치게 할 듯이 으르렁댔다"),
        "betrayal" => Some("믿던 손길이 등을 돌렸다"),
        "argument" => Some("말다툼이 길어졌다"),
        "insult" => Some("모욕 섞인 말을 뱉었다"),
        "provocation" => Some("일부러 성을 돋웠다"),
        "public_argument" => Some("여럿 앞에서 언성을 높였다"),
        "betrayal_discovered" => Some("숨긴 배신이 드러났다"),
        _ => None,
    };
    if let Some(value) = translated {
        return value.to_string();
    }
    if cause
        .chars()
        .any(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        "무슨 까닭이 겹쳤다".to_string()
    } else {
        cause.to_string()
    }
}

fn emotion_phrase(index: usize) -> &'static str {
    match index {
        x if x == EmotionType::Joy as usize => "기쁨에 들떠 있다",
        x if x == EmotionType::Trust as usize => "믿음이 살아 있어 마음이 놓인다",
        x if x == EmotionType::Fear as usize => "두려움이 속을 움켜쥐고 있다",
        x if x == EmotionType::Surprise as usize => "뜻밖의 판에 눈이 커져 있다",
        x if x == EmotionType::Sadness as usize => "서글픔이 길게 드리워 있다",
        x if x == EmotionType::Disgust as usize => "역한 마음에 몸이 물러난다",
        x if x == EmotionType::Anger as usize => "성난 기운이 치밀어 오른다",
        _ => "무언가를 기다리며 마음이 앞서 있다",
    }
}

fn emotion_secondary_phrase(index: usize) -> &'static str {
    match index {
        x if x == EmotionType::Joy as usize => "기쁨이 새어 나온다",
        x if x == EmotionType::Trust as usize => "한편으로는 누군가를 믿고 싶다",
        x if x == EmotionType::Fear as usize => "마음 한켠에는 겁이 남아 있다",
        x if x == EmotionType::Surprise as usize => "놀람이 아직 가시지 않았다",
        x if x == EmotionType::Sadness as usize => "서운함도 함께 맴돈다",
        x if x == EmotionType::Disgust as usize => "거부감이 속에 남아 있다",
        x if x == EmotionType::Anger as usize => "짜증과 화가 끓는다",
        _ => "앞일을 자꾸 헤아리게 된다",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_inner_prompt_from_context, build_judgment_prompt_from_context,
        build_notification_prompt_from_context, build_personality_prompt_from_context,
        default_prompt_dir, emotion_to_korean_context, hexaco_to_korean_descriptors,
        ActionOption, LlmPromptContext, LlmPromptTemplates, SpeechRegister,
    };
    use sim_core::components::{Emotion, LlmRole, Personality, Values};
    use sim_core::enums::{GrowthStage, Sex};

    fn sample_context() -> LlmPromptContext {
        let mut values = [0.0; 33];
        values[12] = 0.8;
        values[30] = 0.7;
        values[32] = 0.6;
        LlmPromptContext {
            entity_name: "카야".to_string(),
            role: "사람".to_string(),
            role_kind: LlmRole::Agent,
            growth_stage: GrowthStage::Adult,
            sex: Sex::Female,
            occupation: "채집꾼".to_string(),
            action_id: 3,
            action_label: "Forage".to_string(),
            personality_axes: [0.82, 0.15, 0.73, 0.48, 0.66, 0.92],
            emotions: [0.1, 0.2, 0.82, 0.1, 0.0, 0.0, 0.15, 0.32],
            needs: [0.8, 0.7, 0.2, 0.6, 0.7, 0.5, 0.4, 0.3, 0.6, 0.7, 0.5, 0.2, 0.4],
            values,
            stress_level: 0.3,
            stress_state: 1,
            recent_event_type: Some("social_conflict".to_string()),
            recent_event_cause: Some("먹거리 나눔을 두고 다투었다".to_string()),
            recent_target_name: Some("마루".to_string()),
        }
    }

    #[test]
    fn default_prompt_dir_points_into_project_data_tree() {
        let dir = default_prompt_dir();
        assert!(dir.ends_with("data/llm/prompts"));
    }

    #[test]
    fn hexaco_descriptor_high_uses_loaded_text() {
        let personality = Personality {
            axes: [0.85, 0.5, 0.5, 0.5, 0.5, 0.5],
            ..Personality::default()
        };
        let descriptors = hexaco_to_korean_descriptors(&personality, Some(&Values::default()));
        assert!(descriptors.h_desc.contains("정직"));
    }

    #[test]
    fn emotion_context_fear_is_korean() {
        let mut emotion = Emotion::default();
        emotion.primary[2] = 0.9;
        let context = emotion_to_korean_context(&emotion);
        assert!(context.contains("두려움"));
        assert!(context.contains("극도로"));
    }

    #[test]
    fn prompt_templates_render_real_use_cases() {
        let templates = LlmPromptTemplates::load_default().expect("prompt templates should load");
        let context = sample_context();
        let personality = build_personality_prompt_from_context(&context, &templates)
            .expect("personality prompt should render");
        let notification = build_notification_prompt_from_context(&context, &templates)
            .expect("notification prompt should render");
        let inner = build_inner_prompt_from_context(&context, &templates)
            .expect("inner prompt should render");
        let judgment = build_judgment_prompt_from_context(
            &context,
            &[ActionOption {
                id: 3,
                label: "먹을거리 찾기".to_string(),
            }],
            &templates,
        )
        .expect("judgment prompt should render");

        assert!(personality.system_prompt.contains("월드심"));
        assert!(personality.user_prompt.contains("카야"));
        assert!(notification.user_prompt.contains("서로 맞부딪쳤다"));
        assert!(notification.user_prompt.contains("먹거리 나눔을 두고 다투었다"));
        assert!(inner.user_prompt.contains("속마음"));
        assert!(judgment.grammar.is_some());
    }

    #[test]
    fn speech_register_detects_matching_suffix() {
        assert!(SpeechRegister::Haera.matches_text("주변을 살폈다."));
        assert!(SpeechRegister::Hao.matches_text("이 일은 내가 맡겠소."));
        assert!(SpeechRegister::Hae.matches_text("나 먼저 가볼게야."));
    }

    #[test]
    fn notification_prompt_humanizes_internal_event_causes() {
        let templates = LlmPromptTemplates::load_default().expect("prompt templates should load");
        let mut context = sample_context();
        context.recent_event_cause = Some("hurtful_words".to_string());
        let notification = build_notification_prompt_from_context(&context, &templates)
            .expect("notification prompt should render");

        assert!(notification.user_prompt.contains("모진 말이 오갔다"));
        assert!(!notification.user_prompt.contains("hurtful_words"));
    }
}

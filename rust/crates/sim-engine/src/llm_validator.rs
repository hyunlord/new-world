use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;

use sim_core::config;

use crate::llm_prompt::SpeechRegister;

/// Loaded forbidden-word table for post-generation Korean validation.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ForbiddenWordList {
    /// All forbidden Sino-Korean words and their 순우리말 replacements.
    pub forbidden: Vec<ForbiddenWord>,
}

/// One forbidden-word replacement rule.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ForbiddenWord {
    /// Forbidden token to search for.
    pub word: String,
    /// Replacement token inserted into cleaned output.
    pub replacement: String,
}

/// Validation result returned after scanning and cleaning generated Korean text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// Text after forbidden-word replacement.
    pub cleaned_text: String,
    /// Original raw model output.
    pub original_text: String,
    /// All detected forbidden-word violations.
    pub violations: Vec<Violation>,
    /// Whether the text endings match the requested speech register.
    pub register_match: bool,
    /// Whether the output passes all validation checks.
    pub pass: bool,
}

/// One forbidden-word violation detected in generated output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Forbidden word found in the original text.
    pub word: String,
    /// Replacement applied to the cleaned text.
    pub replacement: String,
    /// Byte offset of the first occurrence in the original text.
    pub position: usize,
}

static FORBIDDEN_WORD_LIST: OnceLock<ForbiddenWordList> = OnceLock::new();

/// Loads the default forbidden-word replacement table from the project data directory.
pub fn load_forbidden_word_list() -> &'static ForbiddenWordList {
    FORBIDDEN_WORD_LIST.get_or_init(|| {
        let path = project_root().join(config::LLM_FORBIDDEN_SINOKOREAN_PATH);
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<ForbiddenWordList>(&raw).ok())
            .unwrap_or_default()
    })
}

/// Validates generated text against WorldSim Korean narrative rules.
pub fn validate_korean_output(
    text: &str,
    forbidden_words: &ForbiddenWordList,
    expected_register: SpeechRegister,
) -> ValidationResult {
    let original_text = text.trim().to_string();
    let (cleaned_text, violations) =
        replace_forbidden_words(original_text.as_str(), forbidden_words);
    let register_match = expected_register.matches_text(cleaned_text.as_str());
    let too_short = cleaned_text.chars().count() < config::LLM_VALIDATION_MIN_CHARS;
    let too_repetitive =
        repeated_char_ratio(cleaned_text.as_str()) > config::LLM_VALIDATION_MAX_REPEATED_CHAR_RATIO;
    let has_meta = contains_meta_utterance(cleaned_text.as_str());
    let has_prompt_echo = contains_prompt_echo(cleaned_text.as_str());
    let pass = !cleaned_text.is_empty()
        && !too_short
        && !too_repetitive
        && !has_meta
        && !has_prompt_echo
        && register_match
        && violations.len() <= config::LLM_VALIDATION_MAX_VIOLATIONS;

    ValidationResult {
        cleaned_text,
        original_text,
        violations,
        register_match,
        pass,
    }
}

fn replace_forbidden_words(
    text: &str,
    forbidden_words: &ForbiddenWordList,
) -> (String, Vec<Violation>) {
    let mut cleaned = text.to_string();
    let mut violations: Vec<Violation> = Vec::new();

    for forbidden in &forbidden_words.forbidden {
        let mut search_start: usize = 0;
        while let Some(found_at) = cleaned[search_start..].find(forbidden.word.as_str()) {
            let absolute = search_start + found_at;
            violations.push(Violation {
                word: forbidden.word.clone(),
                replacement: forbidden.replacement.clone(),
                position: absolute,
            });
            let range_end = absolute + forbidden.word.len();
            cleaned.replace_range(absolute..range_end, forbidden.replacement.as_str());
            search_start = absolute + forbidden.replacement.len();
        }
    }

    (cleaned, violations)
}

fn contains_meta_utterance(text: &str) -> bool {
    let lowered = text.to_lowercase();
    [
        "나는 ai",
        "인공지능",
        "ai 모델",
        "language model",
        "prompt",
        "프롬프트",
        "system prompt",
        "assistant",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn contains_prompt_echo(text: &str) -> bool {
    [
        "[인물]",
        "[인물 정보]",
        "[지시]",
        "[성격 요약]",
        "[값지게 여기는 것]",
        "[일어난 일]",
        "[채워지지 않은 바람]",
        "이름:",
        "나이:",
        "역할:",
        "지금 하는 일:",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn repeated_char_ratio(text: &str) -> f64 {
    let total = text.chars().count();
    if total == 0 {
        return 1.0;
    }
    let mut counts = std::collections::HashMap::<char, usize>::new();
    for ch in text.chars() {
        *counts.entry(ch).or_insert(0) += 1;
    }
    let repeated = counts.values().copied().max().unwrap_or(0);
    repeated as f64 / total as f64
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

#[cfg(test)]
mod tests {
    use super::{
        load_forbidden_word_list, validate_korean_output, ForbiddenWord, ForbiddenWordList,
    };
    use crate::llm_prompt::SpeechRegister;

    #[test]
    fn forbidden_word_replacement_uses_korean_substitute() {
        let forbidden = ForbiddenWordList {
            forbidden: vec![ForbiddenWord {
                word: "식량".to_string(),
                replacement: "먹거리".to_string(),
            }],
        };
        let result = validate_korean_output("식량이 모자라다.", &forbidden, SpeechRegister::Haera);
        assert!(result.cleaned_text.contains("먹거리"));
        assert!(!result.cleaned_text.contains("식량"));
    }

    #[test]
    fn register_detection_accepts_matching_haera_text() {
        let result = validate_korean_output(
            "주변을 오래 살피며 숨을 골랐다.",
            &ForbiddenWordList::default(),
            SpeechRegister::Haera,
        );
        assert!(result.register_match);
        assert!(result.pass);
    }

    #[test]
    fn meta_utterance_is_rejected() {
        let result = validate_korean_output(
            "나는 AI 모델이다.",
            &ForbiddenWordList::default(),
            SpeechRegister::Haera,
        );
        assert!(!result.pass);
    }

    #[test]
    fn prompt_echo_is_rejected() {
        let result = validate_korean_output(
            "[인물]\n이름: 카야\n역할: 사람",
            &ForbiddenWordList::default(),
            SpeechRegister::Haera,
        );
        assert!(!result.pass);
    }

    #[test]
    fn default_forbidden_list_loads_project_data() {
        let list = load_forbidden_word_list();
        assert!(!list.forbidden.is_empty());
    }
}

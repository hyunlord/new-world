use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use godot::builtin::{GString, Variant, VariantType};
use godot::prelude::VarDictionary;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use unic_langid::LanguageIdentifier;

static FLUENT_SOURCES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static ACTIVE_LOCALE: OnceLock<Mutex<String>> = OnceLock::new();

fn fluent_sources() -> &'static Mutex<HashMap<String, String>> {
    FLUENT_SOURCES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn active_locale() -> &'static Mutex<String> {
    ACTIVE_LOCALE.get_or_init(|| Mutex::new("en".to_string()))
}

#[cfg(test)]
pub(crate) fn locale_test_lock() -> &'static Mutex<()> {
    static TEST_LOCALE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCALE_LOCK.get_or_init(|| Mutex::new(()))
}

fn lookup_plain_message(source: &str, key: &str) -> Option<String> {
    let needle = key.trim();
    if needle.is_empty() {
        return None;
    }
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };
        if raw_key.trim() == needle {
            return Some(raw_value.trim().to_string());
        }
    }
    None
}

fn locale_key(locale: &str) -> String {
    locale.trim().to_lowercase()
}

fn parse_language_identifier(locale: &str) -> LanguageIdentifier {
    locale
        .parse::<LanguageIdentifier>()
        .ok()
        .or_else(|| "en-US".parse::<LanguageIdentifier>().ok())
        .expect("fallback locale should parse")
}

pub(crate) fn store_fluent_source(locale: &str, source: &str) -> bool {
    let key = locale_key(locale);
    if key.is_empty() || source.trim().is_empty() {
        return false;
    }
    let Ok(mut sources) = fluent_sources().lock() else {
        return false;
    };
    sources.insert(key, source.to_string());
    if let Ok(mut active) = active_locale().lock() {
        *active = locale.to_string();
    }
    true
}

pub(crate) fn clear_fluent_source(locale: &str) {
    let key = locale_key(locale);
    if key.is_empty() {
        return;
    }
    let Ok(mut sources) = fluent_sources().lock() else {
        return;
    };
    sources.remove(&key);
}

pub(crate) fn format_active_fluent_message(key: &str) -> Option<String> {
    let locale = active_locale().lock().ok().map(|value| value.clone())?;
    let source = lookup_fluent_source(locale.as_str())?;
    format_fluent_from_source_args(&source, locale.as_str(), key, None)
}

fn lookup_fluent_source(locale: &str) -> Option<String> {
    let key = locale_key(locale);
    let Ok(sources) = fluent_sources().lock() else {
        return None;
    };
    if let Some(source) = sources.get(&key) {
        return Some(source.clone());
    }
    if key.contains('-') {
        let base = key.split('-').next().unwrap_or_default();
        if let Some(source) = sources.get(base) {
            return Some(source.clone());
        }
    }
    sources.get("en").cloned()
}

fn variant_to_fluent_value(value: &Variant) -> FluentValue<'static> {
    match value.get_type() {
        VariantType::INT => FluentValue::Number(FluentNumber::from(value.to::<i64>() as f64)),
        VariantType::FLOAT => FluentValue::Number(FluentNumber::from(value.to::<f64>())),
        VariantType::BOOL => {
            let value_text = if value.to::<bool>() { "true" } else { "false" };
            FluentValue::String(value_text.to_string().into())
        }
        _ => FluentValue::String(value.to::<GString>().to_string().into()),
    }
}

fn build_fluent_args(params: &VarDictionary) -> Option<FluentArgs<'static>> {
    let mut args = FluentArgs::new();
    let mut has_arg = false;
    for (key_var, value_var) in params.iter_shared() {
        let key = key_var.to::<GString>().to_string();
        if key.is_empty() {
            continue;
        }
        args.set(key, variant_to_fluent_value(&value_var));
        has_arg = true;
    }
    if has_arg {
        Some(args)
    } else {
        None
    }
}

pub(crate) fn format_fluent_from_source(
    source: &str,
    locale: &str,
    key: &str,
    params: &VarDictionary,
) -> Option<String> {
    let args = build_fluent_args(params);
    format_fluent_from_source_args(source, locale, key, args)
}

pub(crate) fn format_fluent_from_source_args(
    source: &str,
    locale: &str,
    key: &str,
    args: Option<FluentArgs<'static>>,
) -> Option<String> {
    let fallback = if args.is_none() {
        lookup_plain_message(source, key)
    } else {
        None
    };
    let resource = match FluentResource::try_new(source.to_string()) {
        Ok(resource) => resource,
        Err(_) => return fallback,
    };
    let language_id = parse_language_identifier(locale);
    let mut bundle = FluentBundle::new(vec![language_id]);
    bundle.set_use_isolating(false);
    if bundle.add_resource(resource).is_err() {
        return fallback;
    }
    let message = match bundle.get_message(key) {
        Some(message) => message,
        None => return fallback,
    };
    let pattern = match message.value() {
        Some(pattern) => pattern,
        None => return fallback,
    };
    let mut errors = Vec::new();
    let resolved = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
    Some(resolved.into_owned())
}

pub(crate) fn format_fluent_message(
    locale: &str,
    key: &str,
    params: &VarDictionary,
) -> Option<String> {
    let source = lookup_fluent_source(locale)?;
    format_fluent_from_source(&source, locale, key, params)
}

#[cfg(test)]
mod tests {
    use super::{
        clear_fluent_source, format_active_fluent_message, locale_test_lock, store_fluent_source,
    };

    #[test]
    fn active_locale_message_uses_last_loaded_source() {
        let _guard = locale_test_lock().lock().expect("locale test lock");
        assert!(store_fluent_source("ko", "llm-title = 서사"));
        let resolved = format_active_fluent_message("llm-title");
        assert_eq!(resolved.as_deref(), Some("서사"));
        clear_fluent_source("ko");
    }

    #[test]
    fn active_locale_message_falls_back_to_plain_uppercase_keys() {
        let _guard = locale_test_lock().lock().expect("locale test lock");
        assert!(store_fluent_source("ko", "LLM_NARRATIVE_TITLE = 서사"));
        let resolved = format_active_fluent_message("LLM_NARRATIVE_TITLE");
        assert_eq!(resolved.as_deref(), Some("서사"));
        clear_fluent_source("ko");
    }
}

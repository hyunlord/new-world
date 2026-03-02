use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use godot::builtin::{GString, Variant, VariantType};
use godot::prelude::VarDictionary;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use unic_langid::LanguageIdentifier;

static FLUENT_SOURCES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn fluent_sources() -> &'static Mutex<HashMap<String, String>> {
    FLUENT_SOURCES.get_or_init(|| Mutex::new(HashMap::new()))
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
    let resource = FluentResource::try_new(source.to_string()).ok()?;
    let language_id = parse_language_identifier(locale);
    let mut bundle = FluentBundle::new(vec![language_id]);
    bundle.set_use_isolating(false);
    bundle.add_resource(resource).ok()?;
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;
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

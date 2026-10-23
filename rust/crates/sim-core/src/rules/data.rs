//! Strict loading of the colony's rule data files (`data/colony/*.ron`; owner directive 2026-10-24, M5.5 DECISIONS
//! D28–D29): a file whose `schema_version` is not the one the code reads, or whose `rule` names a field the rule does
//! not have (a typo would otherwise be ignored and the field's default used), is refused — the caller then leaves
//! the feature off, never panics.
use serde::Serialize;

/// Whether every field name under the file's `rule` is a field of `rule` (the parsed value): the names in the text
/// must be a subset of the names `rule` serializes to. `false` on any parse problem.
pub fn rule_fields_known<T: Serialize>(text: &str, rule: &T) -> bool {
    let Ok(ron::Value::Map(file)) = ron::from_str::<ron::Value>(text) else {
        return false;
    };
    let rule_key = ron::Value::String("rule".into());
    let written: Vec<ron::Value> = match file.iter().find(|(k, _)| **k == rule_key) {
        Some((_, ron::Value::Map(m))) => m.keys().cloned().collect(),
        _ => return false,
    };
    let Some(ron::Value::Map(known)) = ron::to_string(rule)
        .ok()
        .and_then(|s| ron::from_str::<ron::Value>(&s).ok())
    else {
        return false;
    };
    written.iter().all(|k| known.keys().any(|j| j == k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Rule {
        a: u32,
        b: (u8, u8),
    }

    #[test]
    fn a_misspelt_field_is_caught() {
        let rule = Rule { a: 1, b: (2, 3) };
        assert!(rule_fields_known(
            "(schema_version: 1, rule: (a: 1, b: (2, 3)))",
            &rule
        ));
        assert!(
            rule_fields_known("(schema_version: 1, rule: (a: 1))", &rule),
            "missing fields keep defaults"
        );
        assert!(!rule_fields_known(
            "(schema_version: 1, rule: (a: 1, bb: (2, 3)))",
            &rule
        ));
        assert!(!rule_fields_known("(schema_version: 1)", &rule));
    }
}

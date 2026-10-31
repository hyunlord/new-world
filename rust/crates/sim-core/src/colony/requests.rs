//! Villager requests (M-READ §2.3): data-driven mapping from blocked mind
//! nodes (why-not records) to actionable player requests. Parsed from
//! `data/colony/requests.ron`. Requests are presentation state: not saved,
//! not hashed, no effect on the simulation.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Embedded request rules.
const REQUESTS_RON: &str = include_str!("../../data/colony/requests.ron");

// ── Raw RON types ──────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestsFile {
    schema_version: u32,
    rules: Vec<RawRequestRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequestRule {
    id: String,
    match_tags: Vec<String>,
    match_facts: Vec<String>,
    text_key: String,
    action_key: String,
    priority: u32,
    #[serde(default)]
    steward_tags: Vec<String>,
}

// ── Public resolved types ──────────────────────────────────────────────

/// One mapping rule from a why-not record to a request kind.
#[derive(Debug, Clone)]
pub struct RequestRule {
    /// Unique rule id.
    pub id: String,
    /// Mind node tags that trigger this rule.
    pub match_tags: Vec<String>,
    /// Fact names whose failure triggers this rule.
    pub match_facts: Vec<String>,
    /// Locale key for the request text.
    pub text_key: String,
    /// Locale key for the suggested action.
    pub action_key: String,
    /// Display priority (lower = more urgent).
    pub priority: u32,
    /// Steward condition tags this request matches (for "집사가 하고 있어요").
    pub steward_tags: Vec<String>,
}

/// All request rules in priority order.
#[derive(Debug, Clone)]
pub struct RequestsData {
    /// Rules in authored order.
    pub rules: Vec<RequestRule>,
    /// Tag → rule indices.
    pub tag_index: BTreeMap<String, Vec<usize>>,
    /// Fact name → rule indices.
    pub fact_index: BTreeMap<String, Vec<usize>>,
}

/// Parse the request rules data file.
pub fn parse_requests(text: &str) -> Option<RequestsData> {
    let file: RequestsFile = ron::from_str(text).ok()?;
    if file.schema_version != 1 {
        return None;
    }
    let mut seen_ids = std::collections::BTreeSet::new();
    let mut rules = Vec::with_capacity(file.rules.len());
    let mut tag_index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut fact_index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, raw) in file.rules.into_iter().enumerate() {
        if !seen_ids.insert(raw.id.clone()) {
            return None; // duplicate rule id
        }
        for tag in &raw.match_tags {
            tag_index.entry(tag.clone()).or_default().push(i);
        }
        for fact in &raw.match_facts {
            fact_index.entry(fact.clone()).or_default().push(i);
        }
        rules.push(RequestRule {
            id: raw.id,
            match_tags: raw.match_tags,
            match_facts: raw.match_facts,
            text_key: raw.text_key,
            action_key: raw.action_key,
            priority: raw.priority,
            steward_tags: raw.steward_tags,
        });
    }
    Some(RequestsData {
        rules,
        tag_index,
        fact_index,
    })
}

/// Load the bundled request rules; `None` when the data is malformed.
pub fn requests_data() -> Option<&'static RequestsData> {
    use std::sync::OnceLock;
    static DATA: OnceLock<Option<RequestsData>> = OnceLock::new();
    DATA.get_or_init(|| parse_requests(REQUESTS_RON)).as_ref()
}

/// Threshold: a request is active when blocked for >= this many of the last
/// 4 hours (from the brief: "≥ 2 of the last 4 hours").
pub const ACTIVE_THRESHOLD: u32 = 2;

/// Number of recent hours tracked per request.
pub const WINDOW_HOURS: usize = 4;

/// Maximum requests shown.
pub const MAX_SHOWN: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_requests_parse() {
        let data =
            parse_requests(REQUESTS_RON).expect("requests.ron must parse with schema version 1");
        assert!(!data.rules.is_empty(), "must have at least one rule");
        // All rules have unique ids.
        let mut ids = std::collections::BTreeSet::new();
        for rule in &data.rules {
            assert!(ids.insert(&rule.id), "duplicate rule id: {}", rule.id);
            // Each rule matches at least one tag or fact.
            assert!(
                !rule.match_tags.is_empty() || !rule.match_facts.is_empty(),
                "rule {} matches nothing",
                rule.id
            );
        }
    }

    #[test]
    fn tag_index_populated() {
        let data = parse_requests(REQUESTS_RON).unwrap();
        // The "food" tag should map to the need_food rule.
        assert!(
            data.tag_index.contains_key("food"),
            "food tag must be indexed"
        );
    }
}

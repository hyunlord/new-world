//! Read-only person mind records; dictionary decoding never recompiles policy or mutates a world.

use godot::prelude::*;
use sim_core::components::Agent;
use sim_engine::SimEngine;

/// One authored candidate with its localized name key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindCandidateView {
    /// Stable authored node name.
    pub node: String,
    /// Locale key for the authored node.
    pub name_key: String,
    /// Registered effect lifecycle name.
    pub behaviour: String,
    /// Exact integer score at selection.
    pub score: i64,
}

/// An exact deciding value and its actual causal reference, if present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindFactView {
    /// Registered provider name.
    pub name: String,
    /// Locale key derived from that provider name.
    pub name_key: String,
    /// Recorded value in provider-defined units.
    pub value: i64,
    /// Actual causal event, if the source has one.
    pub cause: Option<u64>,
    /// All authored conditions on this selected input: operator and optional threshold.
    pub predicates: Vec<(String, Option<i64>)>,
    /// Every signed permille weight and its exact truncated integer contribution.
    pub score_terms: Vec<(i64, i64)>,
    /// Checked sum of this input's score contributions.
    pub score_contribution: i64,
    /// Data-derived presentation category, without interpreting the provider name.
    pub reason_kind: &'static str,
}

/// Latest choice for a living selected person, independent of tile-log eviction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMindView {
    /// Stable person identity, distinct from the query's entity bits.
    pub agent_id: u64,
    /// Retained choice's causal event identity.
    pub event_id: u64,
    /// Primary selected-input cause, if recorded.
    pub parent: Option<u64>,
    /// Simulation tick when the choice was recorded.
    pub tick: u64,
    /// Actual selected candidate, including a hysteresis-retained incumbent.
    pub selected: MindCandidateView,
    /// Structural priority band name.
    pub band: String,
    /// Raw top three candidates in score and authored tie order.
    pub top3: Vec<MindCandidateView>,
    /// Selected node's factual inputs, excluding rejected-only witnesses.
    pub facts: Vec<MindFactView>,
    /// Number of retained choices, bounded by sixteen.
    pub history_count: usize,
}

/// Stale entity handles and a baseline without mind history return no row.
/// Corrupt dictionary bindings remain an explicit error, never an invented reason.
pub fn collect_agent_mind(
    engine: &SimEngine,
    entity_bits: u64,
) -> Result<Option<AgentMindView>, String> {
    let Some(entity) = hecs::Entity::from_bits(entity_bits) else {
        return Ok(None);
    };
    let Ok(agent) = engine.world.get::<&Agent>(entity) else {
        return Ok(None);
    };
    let Some(history) = engine.resources.mind.history.as_ref() else {
        return Ok(None);
    };
    let Some(choice) = history.latest(agent.id) else {
        return Ok(None);
    };
    let view = choice
        .decode_display(history.dictionary())
        .map_err(|error| error.to_string())?;
    Ok(Some(AgentMindView {
        agent_id: view.agent,
        event_id: view.id,
        parent: view.parent,
        tick: view.tick,
        selected: MindCandidateView {
            node: view.selected.labels.name.to_owned(),
            name_key: view.selected.labels.name_key.to_owned(),
            behaviour: view.selected.labels.behaviour.to_owned(),
            score: view.selected.score,
        },
        band: view.band.to_owned(),
        top3: view
            .top3
            .iter()
            .map(|candidate| MindCandidateView {
                node: candidate.labels.name.to_owned(),
                name_key: candidate.labels.name_key.to_owned(),
                behaviour: candidate.labels.behaviour.to_owned(),
                score: candidate.score,
            })
            .collect(),
        facts: reason_facts(&view)?,
        history_count: history.records(agent.id).map_or(0, |records| records.len()),
    }))
}

fn reason_facts(
    view: &sim_core::colony::mind_records::MindChoiceDisplay<'_>,
) -> Result<Vec<MindFactView>, String> {
    let labels = view.selected.labels;
    let authored: Vec<_> = labels
        .predicates
        .iter()
        .map(|p| p.0)
        .chain(labels.score_terms.iter().map(|term| term.0))
        .collect();
    let mut facts = Vec::with_capacity(labels.inputs.len());
    for &slot in labels.inputs {
        let fact = view
            .facts
            .iter()
            .find(|fact| fact.fact == slot)
            .ok_or_else(|| "selected reason input is missing".to_owned())?;
        let predicates: Vec<_> = labels
            .predicates
            .iter()
            .filter(|p| p.0 == slot)
            .map(|p| (p.1.clone(), p.2))
            .collect();
        let mut score_terms = Vec::new();
        let mut score_contribution = 0_i64;
        for &(_, weight) in labels.score_terms.iter().filter(|term| term.0 == slot) {
            // Same i128 product, truncation toward zero and checked i64 conversion as selector.
            let contribution = i64::try_from(i128::from(fact.value) * i128::from(weight) / 1000)
                .map_err(|_| "reason score contribution overflow".to_owned())?;
            score_contribution = score_contribution
                .checked_add(contribution)
                .ok_or_else(|| "reason score sum overflow".to_owned())?;
            score_terms.push((weight, contribution));
        }
        let positive = predicates
            .iter()
            .find(|p| matches!(p.0.as_str(), "has" | ">" | ">="));
        let condition = positive.or_else(|| predicates.first());
        let reason_kind = if score_contribution > 0 {
            "score"
        } else {
            match condition.map(|p| p.0.as_str()) {
                Some("has") => "present",
                Some("not") => "absent",
                Some(">" | ">=") => "high",
                Some("<" | "<=") => "low",
                Some("==") => "equal",
                None if !score_terms.is_empty() => "score",
                None => return Err("reason input has no authored metadata".to_owned()),
                Some(_) => return Err("unknown reason predicate".to_owned()),
            }
        };
        let rank = if score_contribution > 0 {
            0
        } else if positive.is_some() {
            1
        } else {
            2
        };
        let order = authored
            .iter()
            .position(|&fact| fact == slot)
            .ok_or_else(|| "reason input has no authored position".to_owned())?;
        facts.push((
            rank,
            order,
            MindFactView {
                name: fact.name.to_owned(),
                name_key: format!("MIND_FACT_{}", fact.name.to_ascii_uppercase()),
                value: fact.value,
                cause: fact.cause,
                predicates,
                score_terms,
                score_contribution,
                reason_kind,
            },
        ));
    }
    facts.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| {
                if a.0 == 0 {
                    b.2.score_contribution.cmp(&a.2.score_contribution)
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .then(a.1.cmp(&b.1))
    });
    Ok(facts.into_iter().map(|(_, _, fact)| fact).collect())
}

fn wire_id(id: u64) -> i64 {
    i64::from_ne_bytes(id.to_ne_bytes())
}

fn candidate_to_dict(candidate: &MindCandidateView) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("node", candidate.node.as_str());
    out.set("name_key", candidate.name_key.as_str());
    out.set("behaviour", candidate.behaviour.as_str());
    out.set("score", candidate.score);
    out
}

pub(crate) fn agent_mind_to_dict(result: Result<Option<AgentMindView>, String>) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("found", false);
    out.set("invalid", result.is_err());
    let Ok(Some(view)) = result else { return out };
    out.set("found", true);
    out.set("agent_id", wire_id(view.agent_id));
    out.set("event_id", wire_id(view.event_id));
    out.set("parent", view.parent.map_or(-1, wire_id));
    out.set("tick", wire_id(view.tick));
    out.set("node", view.selected.node.as_str());
    out.set("name_key", view.selected.name_key.as_str());
    out.set("behaviour", view.selected.behaviour.as_str());
    out.set("band", view.band.as_str());
    out.set("score", view.selected.score);
    out.set(
        "history_count",
        i64::try_from(view.history_count).unwrap_or(i64::MAX),
    );
    let mut candidates: Array<VarDictionary> = Array::new();
    for candidate in &view.top3 {
        candidates.push(&candidate_to_dict(candidate));
    }
    out.set("top3", candidates);
    let mut facts: Array<VarDictionary> = Array::new();
    for fact in &view.facts {
        let mut row = VarDictionary::new();
        row.set("name", fact.name.as_str());
        row.set("name_key", fact.name_key.as_str());
        row.set("value", fact.value);
        row.set("cause", fact.cause.map_or(-1, wire_id));
        row.set("reason_kind", fact.reason_kind);
        row.set("score_contribution", fact.score_contribution);
        let mut predicates: Array<VarDictionary> = Array::new();
        for (op, threshold) in &fact.predicates {
            let mut predicate = VarDictionary::new();
            predicate.set("op", op.as_str());
            predicate.set(
                "threshold",
                threshold.map_or(Variant::nil(), |value| value.to_variant()),
            );
            predicates.push(&predicate);
        }
        row.set("predicates", predicates);
        let mut terms: Array<VarDictionary> = Array::new();
        for &(weight, contribution) in &fact.score_terms {
            let mut term = VarDictionary::new();
            term.set("weight_permille", weight);
            term.set("contribution", contribution);
            terms.push(&term);
        }
        row.set("score_terms", terms);
        facts.push(&row);
    }
    out.set("facts", facts);
    out
}

//! Exact saved name/slot bindings supplement the policy's compact revision token.
use super::MindRecordError;
use crate::colony::mind::{CompiledMind, FactKind, FinishMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NodeBinding {
    name: String,
    band: usize,
    behaviour: String,
    target: Option<String>,
    name_key: String,
    tags: Vec<String>,
    inputs: Vec<usize>,
    finish: Option<FinishMode>,
    predicates: Vec<(usize, String, Option<i64>)>,
    score_terms: Vec<(usize, i64)>,
    score_constant: i64,
}
/// Borrowed presentation labels from the immutable retained dictionary.
#[derive(Debug, Clone, Copy)]
pub struct NodeLabel<'a> {
    /// Stable authored identity.
    pub name: &'a str,
    /// Localized display key.
    pub name_key: &'a str,
    /// Exact authored semantic tags, retained for downstream event interpretation.
    pub tags: &'a [String],
    /// Sorted unique slots used by this node predicates and score.
    pub inputs: &'a [usize],
    /// Authored predicate slots, operator spellings and optional comparison thresholds.
    pub predicates: &'a [(usize, String, Option<i64>)],
    /// Authored score input slots and signed permille weights, retaining duplicates.
    pub score_terms: &'a [(usize, i64)],
    /// Exact constant score contribution.
    pub score_constant: i64,
    /// Explicit lifecycle end, if authored.
    pub finish: Option<FinishMode>,
    /// Registered lifecycle name.
    pub behaviour: &'a str,
    /// Numeric priority band.
    pub band: usize,
    /// Authored optional target binding.
    pub target: Option<&'a str>,
}
/// Immutable dictionary stored once per world/history, not once per choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindRecordDictionary {
    revision: u64,
    nodes: Vec<NodeBinding>,
    facts: Vec<(String, bool)>,
}
impl MindRecordDictionary {
    /// Capture exact names, bands, target capabilities and registered behaviour identities.
    pub fn from_policy(policy: &CompiledMind) -> Result<Self, MindRecordError> {
        let mut nodes = Vec::with_capacity(policy.nodes().len());
        for node in policy.nodes() {
            let behaviour = policy
                .behaviour(node.behaviour)
                .ok_or(MindRecordError::UnknownSlot("behaviour"))?;
            nodes.push(NodeBinding {
                name: node.name.clone(),
                band: node.band.index(),
                behaviour: behaviour.name.clone(),
                target: node.target.clone(),
                name_key: node.name_key.clone(),
                tags: node.tags.clone(),
                inputs: node_inputs(node),
                finish: node.finish,
                predicates: node
                    .predicate_metadata()
                    .map(|(fact, op, value)| (fact.index(), op.to_owned(), value))
                    .collect(),
                score_terms: node
                    .score_metadata()
                    .map(|(fact, weight)| (fact.index(), weight))
                    .collect(),
                score_constant: node.score_constant(),
            });
        }
        let facts = policy
            .facts()
            .iter()
            .map(|fact| (fact.name.clone(), matches!(fact.kind, FactKind::Bool)))
            .collect();
        Ok(Self {
            revision: policy.revision().code(),
            nodes,
            facts,
        })
    }
    /// Require exact dictionary equality as well as the compact compatibility token.
    pub fn check(&self, policy: &CompiledMind) -> Result<(), MindRecordError> {
        if self.revision != policy.revision().code()
            || self.nodes.len() != policy.nodes().len()
            || self.facts.len() != policy.facts().len()
        {
            return Err(MindRecordError::PolicyMismatch);
        }
        for (saved, node) in self.nodes.iter().zip(policy.nodes()) {
            let behaviour = policy
                .behaviour(node.behaviour)
                .ok_or(MindRecordError::UnknownSlot("behaviour"))?;
            if saved.name != node.name
                || saved.band != node.band.index()
                || saved.behaviour != behaviour.name
                || saved.target != node.target
                || saved.name_key != node.name_key
                || saved.tags != node.tags
                || saved.inputs.windows(2).any(|pair| pair[0] >= pair[1])
                || saved
                    .inputs
                    .iter()
                    .any(|index| !node.fact_ids().any(|fact| fact.index() == *index))
                || node
                    .fact_ids()
                    .any(|fact| saved.inputs.binary_search(&fact.index()).is_err())
                || saved.finish != node.finish
                || saved.score_constant != node.score_constant()
                || !saved
                    .predicates
                    .iter()
                    .map(|(fact, op, value)| (*fact, op.as_str(), *value))
                    .eq(node
                        .predicate_metadata()
                        .map(|(fact, op, value)| (fact.index(), op, value)))
                || !saved.score_terms.iter().copied().eq(node
                    .score_metadata()
                    .map(|(fact, weight)| (fact.index(), weight)))
            {
                return Err(MindRecordError::PolicyMismatch);
            }
        }
        for (saved, fact) in self.facts.iter().zip(policy.facts()) {
            if saved.0 != fact.name || saved.1 != matches!(fact.kind, FactKind::Bool) {
                return Err(MindRecordError::PolicyMismatch);
            }
        }
        Ok(())
    }
    /// Revision used in each packed payload; not a cryptographic provenance claim.
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    /// Resolve presentation labels without recompiling policy or allocating strings.
    pub fn node_label(&self, index: usize) -> Option<NodeLabel<'_>> {
        let node = self.nodes.get(index)?;
        Some(NodeLabel {
            name: &node.name,
            name_key: &node.name_key,
            tags: &node.tags,
            inputs: &node.inputs,
            predicates: &node.predicates,
            score_terms: &node.score_terms,
            score_constant: node.score_constant,
            finish: node.finish,
            behaviour: &node.behaviour,
            band: node.band,
            target: node.target.as_deref(),
        })
    }
    /// Resolve a fact's stable registered name.
    pub fn fact_name(&self, index: usize) -> Option<&str> {
        self.facts.get(index).map(|fact| fact.0.as_str())
    }
    pub(super) fn fact_is_bool(&self, index: usize) -> Option<bool> {
        self.facts.get(index).map(|fact| fact.1)
    }
    /// Structural schema1 band label, independent of mutable policy data.
    pub fn band_name(index: usize) -> Option<&'static str> {
        ["critical", "duty", "need", "work", "leisure", "idle"]
            .get(index)
            .copied()
    }
}

fn node_inputs(node: &crate::colony::mind::MindNode) -> Vec<usize> {
    let mut inputs: Vec<_> = node.fact_ids().map(|fact| fact.index()).collect();
    inputs.sort_unstable();
    inputs.dedup();
    inputs
}

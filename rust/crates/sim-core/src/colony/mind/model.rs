//! Resolved policy identities and declarative registry metadata; no callbacks.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! slot {
    ($name:ident) => {
        /// Dense slot valid only under its compiled policy revision.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub(super) usize);
        impl $name {
            /// Dense table index; not a standalone persistent identity.
            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}
slot!(NodeId);
slot!(FactId);
slot!(BehaviourId);
slot!(BandId);

/// Structural fact storage kind, not a content name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactKind {
    /// Logical value represented exactly as zero or one.
    Bool,
    /// Signed integer with provider-documented units.
    Signed,
}
/// One registered factual column. Numeric units belong to the provider contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactSpec {
    /// Canonical registered name.
    pub name: String,
    /// Boolean values are exactly zero or one; signed values span i64.
    pub kind: FactKind,
}
/// Registered behaviour's accepted target capability names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviourSpec {
    /// Canonical code registration name.
    pub name: String,
    /// Target capabilities understood by this implementation.
    pub target_bindings: Vec<String>,
}
/// Local immutable registration metadata supplied by systems; contains no ECS callbacks.
#[derive(Debug, Clone, Default)]
pub struct MindRegistrySchema {
    /// Names must be unique; only referenced columns are compiled.
    pub facts: Vec<FactSpec>,
    /// Names must be unique; IDs follow this frozen registration order.
    pub behaviours: Vec<BehaviourSpec>,
}
/// Structural person stage used independently of factual columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeStage {
    /// Adult person.
    Adult,
    /// Child person.
    Child,
    /// Elder person.
    Elder,
}
impl LifeStage {
    pub(super) const fn bit(self) -> u8 {
        match self {
            Self::Adult => 1,
            Self::Child => 2,
            Self::Elder => 4,
        }
    }
}
/// Opaque deterministic revision, valid together with the retained policy dictionary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindRevision(pub(super) u64);
impl MindRevision {
    /// Compatibility token, not a cryptographic build provenance digest.
    pub const fn code(self) -> u64 {
        self.0
    }
}
/// Predicate grammar operations, independent of authored fact names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Op {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Has,
    Not,
}
#[derive(Debug, Clone)]
pub(super) struct Predicate {
    pub fact: FactId,
    pub op: Op,
    pub value: i64,
}
#[derive(Debug, Clone)]
pub(super) struct Term {
    pub fact: FactId,
    pub weight: i64,
}
/// Authored termination of the selected registered lifecycle, without starting another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishMode {
    /// The lifecycle reached its normal completion condition.
    Completed,
    /// A factual crossover or invalidation ends this lifecycle at Idle for this tick.
    Interrupted,
}
/// An authored node resolved to registered code and column slots.
#[derive(Debug, Clone)]
pub struct MindNode {
    /// Policy-local node identity.
    pub id: NodeId,
    /// Stable authored identity for names and persistent dictionaries.
    pub name: String,
    /// Priority band, zero is highest.
    pub band: BandId,
    /// Resolved registered behaviour.
    pub behaviour: BehaviourId,
    /// Optional registered target binding, interpreted only by the executor.
    pub target: Option<String>,
    /// Optional explicit lifecycle end; predicates and priority remain authored policy.
    pub finish: Option<FinishMode>,
    /// Localized presentation key authored in data.
    pub name_key: String,
    /// Data tags for consumers; the selector does not dispatch on them.
    pub tags: Vec<String>,
    /// Nonnegative score margin of this node while running.
    pub hysteresis: i64,
    /// Inactive admission cadence; running eligibility is checked every tick.
    pub every_ticks: u32,
    /// Name-derived deterministic random domain, independent of node order.
    pub random_key: u64,
    pub(super) stages: u8,
    pub(super) predicates: Vec<Predicate>,
    pub(super) terms: Vec<Term>,
    pub(super) constant: i64,
}
impl MindNode {
    /// Authored predicate order and exact comparison metadata, without evaluating facts.
    pub fn predicate_metadata(
        &self,
    ) -> impl Iterator<Item = (FactId, &'static str, Option<i64>)> + '_ {
        self.predicates.iter().map(|p| {
            let op = match p.op {
                Op::Lt => "<",
                Op::Le => "<=",
                Op::Gt => ">",
                Op::Ge => ">=",
                Op::Eq => "==",
                Op::Has => "has",
                Op::Not => "not",
            };
            let threshold = match p.op {
                Op::Has | Op::Not => None,
                Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Eq => Some(p.value),
            };
            (p.fact, op, threshold)
        })
    }
    /// Authored score terms, including duplicate inputs and signed permille weights.
    pub fn score_metadata(&self) -> impl Iterator<Item = (FactId, i64)> + '_ {
        self.terms.iter().map(|term| (term.fact, term.weight))
    }
    /// Constant term retained for exact dictionary compatibility.
    pub const fn score_constant(&self) -> i64 {
        self.constant
    }
    /// All factual inputs used by predicates and score terms; duplicates are preserved.
    pub fn fact_ids(&self) -> impl Iterator<Item = FactId> + '_ {
        self.predicates
            .iter()
            .map(|predicate| predicate.fact)
            .chain(self.terms.iter().map(|term| term.fact))
    }
    /// Whether this node itself reads the given fact.
    pub fn uses_fact(&self, fact: FactId) -> bool {
        self.fact_ids().any(|input| input == fact)
    }
}
/// Compiled immutable policy. It can be created without process-global content state.
#[derive(Debug, Clone)]
pub struct CompiledMind {
    pub(super) revision: MindRevision,
    pub(super) nodes: Vec<MindNode>,
    pub(super) bands: [Vec<NodeId>; 6],
    pub(super) facts: Vec<FactSpec>,
    pub(super) fact_names: BTreeMap<String, FactId>,
    pub(super) behaviours: Vec<BehaviourSpec>,
}
impl CompiledMind {
    /// Policy compatibility token; retain the authored dictionary with saved IDs.
    pub const fn revision(&self) -> MindRevision {
        self.revision
    }
    /// Referenced columns in resolved slot order.
    pub fn facts(&self) -> &[FactSpec] {
        &self.facts
    }
    /// All nodes in original authored tie order.
    pub fn nodes(&self) -> &[MindNode] {
        &self.nodes
    }
    /// Authored nodes referencing this fact in predicates or score terms, without duplicates.
    pub fn fact_node_users(&self, fact: FactId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes
            .iter()
            .filter(move |node| {
                node.predicates.iter().any(|p| p.fact == fact)
                    || node.terms.iter().any(|term| term.fact == fact)
            })
            .map(|node| node.id)
    }
    /// Resolve a registered referenced fact by name.
    pub fn fact_id(&self, name: &str) -> Option<FactId> {
        self.fact_names.get(name).copied()
    }
    /// Resolve an authored node by name.
    pub fn node_id(&self, name: &str) -> Option<NodeId> {
        self.nodes.iter().find(|n| n.name == name).map(|n| n.id)
    }
    /// Resolve a registered behaviour by name.
    pub fn behaviour_id(&self, name: &str) -> Option<BehaviourId> {
        self.behaviours
            .iter()
            .position(|b| b.name == name)
            .map(BehaviourId)
    }
    /// Checked node access for a policy-local ID.
    pub fn node(&self, id: NodeId) -> Option<&MindNode> {
        self.nodes.get(id.0)
    }
    /// Registered behaviour metadata, independent of implementation callbacks.
    pub fn behaviour(&self, id: BehaviourId) -> Option<&BehaviourSpec> {
        self.behaviours.get(id.0)
    }
    /// Validate a decoded node slot against this policy dictionary.
    pub fn node_id_at(&self, index: usize) -> Option<NodeId> {
        self.nodes.get(index).map(|node| node.id)
    }
    /// Validate a decoded fact slot against this policy dictionary.
    pub fn fact_id_at(&self, index: usize) -> Option<FactId> {
        self.facts.get(index).map(|_| FactId(index))
    }
    /// Validate a decoded behaviour slot against this registry dictionary.
    pub fn behaviour_id_at(&self, index: usize) -> Option<BehaviourId> {
        self.behaviours.get(index).map(|_| BehaviourId(index))
    }
    /// Validate a decoded structural band slot.
    pub fn band_id_at(&self, index: usize) -> Option<BandId> {
        self.bands.get(index).map(|_| BandId(index))
    }
    /// Structural band name in schema1 priority order.
    pub fn band_name(&self, id: BandId) -> Option<&'static str> {
        BAND_NAMES.get(id.0).copied()
    }
}
pub(super) const BAND_NAMES: [&str; 6] = ["critical", "duty", "need", "work", "leisure", "idle"];

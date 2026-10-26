//! Read-only bridge decoding from the retained dictionary; no policy compilation.
use super::{codec, MindRecordDictionary, MindRecordError, NodeLabel, PackedMindChoice};
use crate::causal::EventId;
use crate::components::AgentId;

/// One complete candidate with borrowed localized-name metadata.
#[derive(Debug)]
pub struct DisplayCandidate<'a> {
    /// Dictionary slot, meaningful only under the returned revision.
    pub node: usize,
    /// Labels retained with the originating policy.
    pub labels: NodeLabel<'a>,
    /// Exact integer score.
    pub score: i64,
}
/// One exact deciding fact and optional real cause.
#[derive(Debug)]
pub struct DisplayFact<'a> {
    /// Dictionary slot.
    pub fact: usize,
    /// Stable registered fact name.
    pub name: &'a str,
    /// Exact value at selection.
    pub value: i64,
    /// Actual causal event identity, if present.
    pub cause: Option<EventId>,
}
/// Complete decoded logical reason; strings borrow the retained world dictionary.
#[derive(Debug)]
pub struct MindChoiceDisplay<'a> {
    /// Compatibility token bound to this dictionary.
    pub revision: u64,
    /// Actual causal identity.
    pub id: EventId,
    /// Recorded primary parent.
    pub parent: Option<EventId>,
    /// Person whose choice changed.
    pub agent: AgentId,
    /// Decision tick.
    pub tick: u64,
    /// Selected node and its score/labels, including hysteresis-retained candidates.
    pub selected: DisplayCandidate<'a>,
    /// Structural band name.
    pub band: &'static str,
    /// Complete raw top-three ranking.
    pub top3: Vec<DisplayCandidate<'a>>,
    /// Every deciding fact, without truncation.
    pub facts: Vec<DisplayFact<'a>>,
}
impl PackedMindChoice {
    /// Decode for UI reads using the saved dictionary already validated at world load.
    pub fn decode_display<'a>(
        &self,
        dictionary: &'a MindRecordDictionary,
    ) -> Result<MindChoiceDisplay<'a>, MindRecordError> {
        let raw = codec::parse(self.bytes())?;
        if raw.revision != dictionary.revision() {
            return Err(MindRecordError::PolicyMismatch);
        }
        let labels = dictionary
            .node_label(raw.node)
            .ok_or(MindRecordError::UnknownSlot("node"))?;
        if labels.band != raw.band {
            return Err(MindRecordError::Invalid("node band mismatch"));
        }
        let band = MindRecordDictionary::band_name(raw.band)
            .ok_or(MindRecordError::UnknownSlot("band"))?;
        let mut top3 = Vec::with_capacity(raw.top3.len());
        for (node, score) in raw.top3 {
            let labels = dictionary
                .node_label(node)
                .ok_or(MindRecordError::UnknownSlot("candidate"))?;
            if labels.band != raw.band {
                return Err(MindRecordError::Invalid("candidate band mismatch"));
            }
            top3.push(DisplayCandidate {
                node,
                labels,
                score,
            });
        }
        let mut facts = Vec::with_capacity(raw.facts.len());
        for (fact, value, cause) in raw.facts {
            let name = dictionary
                .fact_name(fact)
                .ok_or(MindRecordError::UnknownSlot("fact"))?;
            if dictionary.fact_is_bool(fact) == Some(true) && !matches!(value, 0 | 1) {
                return Err(MindRecordError::Invalid("boolean fact value"));
            }
            facts.push(DisplayFact {
                fact,
                name,
                value,
                cause,
            });
        }
        Ok(MindChoiceDisplay {
            revision: raw.revision,
            id: raw.id,
            parent: raw.parent,
            agent: raw.agent,
            tick: raw.tick,
            selected: DisplayCandidate {
                node: raw.node,
                labels,
                score: raw.score,
            },
            band,
            top3,
            facts,
        })
    }
}

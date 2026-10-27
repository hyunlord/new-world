//! Deterministic person-indexed record ownership; eviction never changes other payloads.
use super::{codec, MindChoiceRecord, MindRecordDictionary, MindRecordError, PackedMindChoice};
use crate::causal::EventId;
use crate::colony::mind::CompiledMind;
use crate::components::AgentId;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Exact maximum retained choice count per person.
pub const MIND_HISTORY_LIMIT: usize = 16;
/// Serialized dictionary and compact per-person histories, in deterministic ID order.
#[derive(Debug, Clone, Serialize)]
pub struct MindHistory {
    dictionary: MindRecordDictionary,
    last_event_id: Option<EventId>,
    people: BTreeMap<AgentId, VecDeque<PackedMindChoice>>,
    // The dictionary is private and immutable; replacement/deserialization must
    // carry its own validation state rather than inheriting a system's trust.
    #[serde(skip)]
    initialized_policy_checked: bool,
}
impl PartialEq for MindHistory {
    fn eq(&self, other: &Self) -> bool {
        self.dictionary == other.dictionary
            && self.last_event_id == other.last_event_id
            && self.people == other.people
    }
}
impl Eq for MindHistory {}

impl MindHistory {
    /// Bind an empty history to one immutable policy dictionary.
    pub fn new(policy: &CompiledMind) -> Result<Self, MindRecordError> {
        Ok(Self {
            dictionary: MindRecordDictionary::from_policy(policy)?,
            last_event_id: None,
            people: BTreeMap::new(),
            initialized_policy_checked: true,
        })
    }
    /// Exact dictionary used by record decoding and save compatibility checks.
    pub const fn dictionary(&self) -> &MindRecordDictionary {
        &self.dictionary
    }
    /// Require exact policy compatibility before using restored records.
    pub fn check_policy(&self, policy: &CompiledMind) -> Result<(), MindRecordError> {
        self.dictionary.check(policy)?;
        for records in self.people.values() {
            for record in records {
                record.decode(policy, &self.dictionary)?;
            }
        }
        Ok(())
    }
    /// Validate a restored/replaced history once before runtime use.
    ///
    /// The immutable dictionary belongs to this object; cloning preserves trust,
    /// while deserialization clears it. Public `check_policy` remains strict.
    pub fn check_initialized_policy(
        &mut self,
        policy: &CompiledMind,
    ) -> Result<(), MindRecordError> {
        if self.dictionary.revision() != policy.revision().code() {
            return Err(MindRecordError::PolicyMismatch);
        }
        if !self.initialized_policy_checked {
            self.check_policy(policy)?;
            self.initialized_policy_checked = true;
        }
        Ok(())
    }
    /// Retain one emitted choice, sharing its allocation with any causal-log clone.
    pub fn push(
        &mut self,
        policy: &CompiledMind,
        record: PackedMindChoice,
    ) -> Result<(), MindRecordError> {
        let decoded = record.decode(policy, &self.dictionary)?;
        self.insert(decoded.agent, record)
    }
    /// Encode once, retain the choice and return a shared causal-log handle.
    pub fn push_record(
        &mut self,
        policy: &CompiledMind,
        record: &MindChoiceRecord,
    ) -> Result<PackedMindChoice, MindRecordError> {
        self.dictionary.check(policy)?;
        let packed = PackedMindChoice::new(policy, record)?;
        self.insert(record.agent, packed.clone())?;
        Ok(packed)
    }
    /// Encode a runtime choice after initialization validated this immutable dictionary.
    ///
    /// Restored histories undergo strict validation before the first insertion.
    /// Later calls retain revision, record-content and event-order checks; the
    /// private immutable dictionary keeps its object-local validation cache.
    pub fn push_record_for_initialized_policy(
        &mut self,
        policy: &CompiledMind,
        record: &MindChoiceRecord,
    ) -> Result<PackedMindChoice, MindRecordError> {
        self.check_initialized_policy(policy)?;
        debug_assert!(self.dictionary.check(policy).is_ok());
        let packed = PackedMindChoice::new(policy, record)?;
        self.insert(record.agent, packed.clone())?;
        Ok(packed)
    }
    fn insert(&mut self, agent: AgentId, record: PackedMindChoice) -> Result<(), MindRecordError> {
        if self.last_event_id.is_some_and(|last| last >= record.id()) {
            return Err(MindRecordError::Invalid("nonmonotonic history event id"));
        }
        let event_id = record.id();
        let records = self.people.entry(agent).or_default();
        if records
            .back()
            .is_some_and(|last| last.id() >= record.id() || last.tick() > record.tick())
        {
            return Err(MindRecordError::Invalid("nonmonotonic person history"));
        }
        if records.len() == MIND_HISTORY_LIMIT {
            records.pop_front();
        }
        records.push_back(record);
        self.last_event_id = Some(event_id);
        Ok(())
    }
    /// The newest choice, or no history for this person.
    pub fn latest(&self, agent: AgentId) -> Option<&PackedMindChoice> {
        self.people.get(&agent)?.back()
    }
    /// Oldest-to-newest retained records for one person.
    pub fn records(&self, agent: AgentId) -> Option<&VecDeque<PackedMindChoice>> {
        self.people.get(&agent)
    }
    /// Canonical recent payload lookup for relinking independently deserialized tile events.
    pub fn find(&self, agent: AgentId, event: EventId) -> Option<&PackedMindChoice> {
        self.people
            .get(&agent)?
            .iter()
            .find(|record| record.id() == event)
    }
    /// Deterministic person order for save/bridge consumers.
    pub fn people(&self) -> impl Iterator<Item = (AgentId, &VecDeque<PackedMindChoice>)> {
        self.people.iter().map(|(agent, records)| (*agent, records))
    }
    /// Release history for a removed person once durable chronicle data has been copied.
    pub fn remove(&mut self, agent: AgentId) {
        self.people.remove(&agent);
    }
}
#[derive(Deserialize)]
struct HistoryWire {
    dictionary: MindRecordDictionary,
    last_event_id: Option<EventId>,
    people: BTreeMap<AgentId, VecDeque<PackedMindChoice>>,
}
impl<'de> Deserialize<'de> for MindHistory {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = HistoryWire::deserialize(deserializer)?;
        let mut ids = BTreeSet::new();
        for (agent, records) in &wire.people {
            if records.len() > MIND_HISTORY_LIMIT {
                return Err(serde::de::Error::custom("mind history exceeds16"));
            }
            let mut previous = None;
            for record in records {
                let raw = codec::parse(record.bytes()).map_err(serde::de::Error::custom)?;
                if raw.agent != *agent || raw.revision != wire.dictionary.revision() {
                    return Err(serde::de::Error::custom(
                        "mind history dictionary/person mismatch",
                    ));
                }
                if !ids.insert(raw.id)
                    || previous.is_some_and(|(id, tick)| id >= raw.id || tick > raw.tick)
                {
                    return Err(serde::de::Error::custom(
                        "mind history event order/identity mismatch",
                    ));
                }
                previous = Some((raw.id, raw.tick));
            }
        }
        if ids
            .last()
            .is_some_and(|last| wire.last_event_id.is_none_or(|saved| saved < *last))
        {
            return Err(serde::de::Error::custom("mind history last event mismatch"));
        }
        Ok(Self {
            last_event_id: wire.last_event_id,
            dictionary: wire.dictionary,
            people: wire.people,
            initialized_policy_checked: false,
        })
    }
}

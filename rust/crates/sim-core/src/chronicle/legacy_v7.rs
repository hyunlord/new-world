//! Frozen format6/7 Chronicle layout; transfer indexes without rebuilding history.

use super::*;

#[derive(serde::Deserialize)]
struct LegacyEntry {
    /// Simulation tick of the event.
    pub tick: u64,
    /// What happened.
    pub kind: ChronicleKind,
    /// The person the entry is about, if any.
    pub subject: Option<AgentId>,
    /// A second person (mother, partner, leader, …).
    pub other: Option<AgentId>,
    /// A third person (father).
    pub third: Option<AgentId>,
    /// The settlement the entry is about, if any.
    pub settlement: Option<SettlementId>,
    /// A second settlement (the parent of a daughter).
    pub settlement2: Option<SettlementId>,
    /// Kind-specific reason code.
    pub cause: u8,
    /// Kind-specific value.
    pub value: i64,
}

/// Chronicle encoded with the historical one-byte mixed reason code.
#[derive(serde::Deserialize)]
pub struct LegacyChronicle {
    entries: Vec<LegacyEntry>,
    by_agent: BTreeMap<AgentId, Vec<u32>>,
    by_settlement: BTreeMap<SettlementId, Vec<u32>>,
    people: BTreeMap<AgentId, PersonRecord>,
    /// M5: the cause of an entry, by entry index (a side table: entries keep
    /// their V7 layout, so the baseline's chronicle files are unchanged).
    #[serde(default)]
    causes: BTreeMap<u32, CauseRef>,
    /// M5.5: need-state samples behind `CauseRef::Sample` causes, in append order (empty in the baseline).
    #[serde(default)]
    samples: Vec<NeedSample>,
}

impl From<LegacyEntry> for ChronicleEntry {
    fn from(old: LegacyEntry) -> Self {
        Self {
            tick: old.tick,
            kind: old.kind,
            subject: old.subject,
            other: old.other,
            third: old.third,
            settlement: old.settlement,
            settlement2: old.settlement2,
            cause: u32::from(old.cause),
            value: old.value,
        }
    }
}

impl From<LegacyChronicle> for Chronicle {
    fn from(old: LegacyChronicle) -> Self {
        Self {
            entries: old.entries.into_iter().map(Into::into).collect(),
            by_agent: old.by_agent,
            by_settlement: old.by_settlement,
            people: old.people,
            causes: old.causes,
            samples: old.samples,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_widens_cause_and_preserves_historical_indexes() {
        let old = LegacyChronicle {
            entries: vec![LegacyEntry {
                tick: 9,
                kind: ChronicleKind::Born,
                subject: Some(3),
                other: None,
                third: None,
                settlement: Some(2),
                settlement2: None,
                cause: u8::MAX,
                value: -7,
            }],
            by_agent: BTreeMap::from([(3, vec![0, 0])]),
            by_settlement: BTreeMap::from([(2, vec![0])]),
            people: BTreeMap::new(),
            causes: BTreeMap::new(),
            samples: Vec::new(),
        };
        let migrated = Chronicle::from(old);
        assert_eq!(migrated.entries[0].cause, 255);
        assert_eq!(migrated.entries[0].value, -7);
        assert_eq!(migrated.by_agent.get(&3), Some(&vec![0, 0]));
        assert_eq!(migrated.by_settlement.get(&2), Some(&vec![0]));
    }
}

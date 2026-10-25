//! Frozen format6/7 Chronicle layout; transfer indexes without rebuilding history.

use super::*;

/// Historical entry with a one-byte mixed cause.
#[derive(serde::Deserialize)]
pub struct LegacyEntry {
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
pub struct LegacyChronicle<E = LegacyEntry> {
    entries: Vec<E>,
    by_agent: BTreeMap<AgentId, Vec<u32>>,
    by_settlement: BTreeMap<SettlementId, Vec<u32>>,
    people: BTreeMap<AgentId, PersonRecord>,
    /// M5: the cause of an entry, by entry index (a side table: entries keep
    /// their V7 layout, so the baseline's chronicle files are unchanged).
    #[serde(default)]
    causes: BTreeMap<u32, LegacyCauseRef>,
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

impl<E: Into<ChronicleEntry>> From<LegacyChronicle<E>> for Chronicle {
    fn from(old: LegacyChronicle<E>) -> Self {
        Self {
            entries: old.entries.into_iter().map(Into::into).collect(),
            by_agent: old.by_agent,
            by_settlement: old.by_settlement,
            people: old.people,
            causes: old
                .causes
                .into_iter()
                .map(|(index, cause)| (index, cause.into()))
                .collect(),
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

/// Frozen mixed-cause layout for formats6/7/8.
#[derive(serde::Deserialize)]
pub enum LegacyCauseRef {
    /// Chronicle entry index.
    Entry(u32),
    /// Historical pressure byte and optional entry.
    Pressure {
        /// Historical pressure code.
        kind: u8,
        /// Chronicle entry index.
        entry: Option<u32>,
    },
    /// Mechanic modifier byte and optional entry.
    Modifier {
        /// Mechanic modifier code.
        code: u8,
        /// Chronicle entry index.
        entry: Option<u32>,
    },
    /// Mechanic modifier byte and sample index.
    Sample {
        /// Mechanic modifier code.
        code: u8,
        /// Need sample index.
        sample: u32,
    },
}
impl From<LegacyCauseRef> for CauseRef {
    fn from(old: LegacyCauseRef) -> Self {
        match old {
            LegacyCauseRef::Entry(index) => Self::Entry(index),
            LegacyCauseRef::Pressure { kind, entry } => Self::Pressure {
                kind: u32::from(kind),
                entry,
            },
            LegacyCauseRef::Modifier { code, entry } => Self::Modifier { code, entry },
            LegacyCauseRef::Sample { code, sample } => Self::Sample { code, sample },
        }
    }
}
/// Format8 already widened the main entry cause, but not the side table.
pub type LegacyChronicleV8 = LegacyChronicle<ChronicleEntry>;

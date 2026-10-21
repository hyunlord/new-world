//! Pass 19: the chronicle — an append-only record of every life and every
//! village, plus a person table that outlives death.
//!
//! The chronicle is **write-only for the simulation**: systems append to it,
//! and nothing that decides behaviour reads it (the only reads on the write
//! side are [`Chronicle::ensure_person`] / [`Chronicle::close_person`] looking
//! up the person table, and [`Chronicle::has_founding_member`], which only
//! shapes which entry is written). It uses no draws, never touches the causal
//! log or event ids, and is recorded in every scene. Probes and the bridge
//! read it.
use std::collections::BTreeMap;

use crate::components::{AgentId, SettlementId};

/// What a [`ChronicleEntry`] records. Discriminants are stable (`as_u8`).
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ChronicleKind {
    /// subject = child, other = mother, third = father, settlement; cause 0
    /// couple / 1 fallback parents.
    Born = 0,
    /// subject; settlement — an original camp (a settlement formed with no
    /// fission behind it).
    FoundingMember = 1,
    /// subject = the woman, other = the man, settlement; value = familiarity
    /// in permille.
    Paired = 2,
    /// subject (the `Child` stage ended).
    CameOfAge = 3,
    /// subject (the `Elder` stage began).
    BecameElder = 4,
    /// subject; settlement = the parent; other = the group leader; cause =
    /// [`DepartCause`]; value = fission id.
    Departed = 5,
    /// subject; settlement = the parent; value = fission id.
    DepartureCancelled = 6,
    /// subject; settlement = the daughter; settlement2 = the parent; value =
    /// fission id.
    Settled = 7,
    /// subject; settlement; cause 0 from no roster / 1 from another roster /
    /// 3 present when a daughter formed.
    Joined = 8,
    /// subject; settlement; cause 0 remnant dissolved / 2 roster change
    /// (exclusivity or proximity).
    LeftSettlement = 9,
    /// subject = the survivor; other = the deceased partner.
    Widowed = 10,
    /// subject; settlement = last roster, if any; cause =
    /// `DeathReason::as_u8`; value = age in ticks (−1 without `Life`); other =
    /// `Kin.partner` as recorded (a partner who died earlier stays set until
    /// the next pairing minute).
    Died = 11,
    /// no subject; settlement; value = founding member count.
    SettlementFormed = 12,
    /// no subject; settlement = parent; value = fission id; other = leader.
    FissionStarted = 13,
    /// no subject; settlement = daughter; settlement2 = parent; value =
    /// fission id.
    DaughterFounded = 14,
    /// no subject; settlement = parent; value = fission id.
    FissionCancelled = 15,
    /// no subject; settlement; cause 0 empty / 1 remnant merge.
    SettlementDissolved = 16,
    /// Pass 20: subject = the porter; settlement = the sender, settlement2 =
    /// the receiver; cause 0 aid food / 1 aid wood / 2 exchange food / 3
    /// exchange wood; value = units carried. One entry per porter.
    PorterDeparted = 17,
    /// Pass 20: subject = the porter; settlement = the receiver, settlement2
    /// = the sender; cause as `PorterDeparted`, value = units delivered; a
    /// porter that turned back: cause 10 + the departure cause, value 0.
    PorterDelivered = 18,
    /// Pass 20: subject = forager a (the one picking up), other = forager b
    /// (its rival); settlement = a's village, settlement2 = b's; cause 0;
    /// value = the relation after the quarrel.
    Quarrel = 19,
    /// Pass 20: no subject; settlement = the lower id, settlement2 = the
    /// higher; cause 0; value = the relation.
    VillagesEstranged = 20,
    /// Pass 20: no subject; settlement = the lower id, settlement2 = the
    /// higher; cause 0; value = the relation.
    VillagesReconciled = 21,
    /// M1: no subject; settlement = the hut's owner; subject, other, third = the
    /// first three builders by AgentId (those working at completion); value =
    /// how many built at completion; cause 0.
    HutBuilt = 22,
}

impl ChronicleKind {
    /// Every kind in discriminant order.
    pub const ALL: [ChronicleKind; 23] = [
        Self::Born,
        Self::FoundingMember,
        Self::Paired,
        Self::CameOfAge,
        Self::BecameElder,
        Self::Departed,
        Self::DepartureCancelled,
        Self::Settled,
        Self::Joined,
        Self::LeftSettlement,
        Self::Widowed,
        Self::Died,
        Self::SettlementFormed,
        Self::FissionStarted,
        Self::DaughterFounded,
        Self::FissionCancelled,
        Self::SettlementDissolved,
        Self::PorterDeparted,
        Self::PorterDelivered,
        Self::Quarrel,
        Self::VillagesEstranged,
        Self::VillagesReconciled,
        Self::HutBuilt,
    ];

    /// Stable numeric discriminant.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Stable snake_case name (probe output).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Born => "born",
            Self::FoundingMember => "founding_member",
            Self::Paired => "paired",
            Self::CameOfAge => "came_of_age",
            Self::BecameElder => "became_elder",
            Self::Departed => "departed",
            Self::DepartureCancelled => "departure_cancelled",
            Self::Settled => "settled",
            Self::Joined => "joined",
            Self::LeftSettlement => "left_settlement",
            Self::Widowed => "widowed",
            Self::Died => "died",
            Self::SettlementFormed => "settlement_formed",
            Self::FissionStarted => "fission_started",
            Self::DaughterFounded => "daughter_founded",
            Self::FissionCancelled => "fission_cancelled",
            Self::SettlementDissolved => "settlement_dissolved",
            Self::PorterDeparted => "porter_departed",
            Self::PorterDelivered => "porter_delivered",
            Self::Quarrel => "quarrel",
            Self::VillagesEstranged => "villages_estranged",
            Self::VillagesReconciled => "villages_reconciled",
            Self::HutBuilt => "hut_built",
        }
    }
}

/// Why a member left with a fission group (the phase of
/// `households::select_departure_explained` that first chose them).
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DepartCause {
    /// Chosen by rank without a family rule (no households).
    Ranked = 0,
    /// The ranked adult whose household the seed phase took.
    Seed = 1,
    /// Another ranked member of a household taken by the seed or the fill.
    Household = 2,
    /// An unranked member of such a household: the kept elder partner
    /// (`game_v23`+).
    ElderPartner = 3,
    /// A single ranked person added by the `group_min` top-up.
    TopUp = 4,
}

impl DepartCause {
    /// Every cause in discriminant order.
    pub const ALL: [DepartCause; 5] = [
        Self::Ranked,
        Self::Seed,
        Self::Household,
        Self::ElderPartner,
        Self::TopUp,
    ];

    /// Stable numeric discriminant.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Stable snake_case name (probe output).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ranked => "ranked",
            Self::Seed => "seed",
            Self::Household => "household",
            Self::ElderPartner => "elder_partner",
            Self::TopUp => "top_up",
        }
    }
}

/// One chronicle record. Field meanings depend on [`ChronicleKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChronicleEntry {
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

impl ChronicleEntry {
    /// An entry of `kind` at `tick` with every optional field empty.
    pub const fn new(tick: u64, kind: ChronicleKind) -> Self {
        Self {
            tick,
            kind,
            subject: None,
            other: None,
            third: None,
            settlement: None,
            settlement2: None,
            cause: 0,
            value: 0,
        }
    }

    /// With `subject`.
    pub const fn subject(mut self, id: AgentId) -> Self {
        self.subject = Some(id);
        self
    }

    /// With `other` (`None` keeps it empty).
    pub const fn other(mut self, id: Option<AgentId>) -> Self {
        self.other = id;
        self
    }

    /// With `third` (`None` keeps it empty).
    pub const fn third(mut self, id: Option<AgentId>) -> Self {
        self.third = id;
        self
    }

    /// With `settlement` (`None` keeps it empty).
    pub const fn settlement(mut self, id: Option<SettlementId>) -> Self {
        self.settlement = id;
        self
    }

    /// With `settlement2` (`None` keeps it empty).
    pub const fn settlement2(mut self, id: Option<SettlementId>) -> Self {
        self.settlement2 = id;
        self
    }

    /// With `cause`.
    pub const fn cause(mut self, cause: u8) -> Self {
        self.cause = cause;
        self
    }

    /// With `value`.
    pub const fn value(mut self, value: i64) -> Self {
        self.value = value;
        self
    }

    /// The distinct people named by the entry, in field order.
    pub fn agents(&self) -> impl Iterator<Item = AgentId> + '_ {
        let subject = self.subject;
        let other = self.other.filter(|id| Some(*id) != subject);
        let third = self
            .third
            .filter(|id| Some(*id) != subject && Some(*id) != other);
        [subject, other, third].into_iter().flatten()
    }

    /// The distinct settlements named by the entry, in field order.
    pub fn settlements(&self) -> impl Iterator<Item = SettlementId> + '_ {
        let first = self.settlement;
        let second = self.settlement2.filter(|id| Some(*id) != first);
        [first, second].into_iter().flatten()
    }
}

/// What is known of a person, kept after death.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersonRecord {
    /// `Sex::female` when the person carried `Sex`.
    pub female: Option<bool>,
    /// `Life::born_tick` (founders before zero) when the person carried `Life`.
    pub born_tick: Option<i64>,
    /// `Kin::mother` when the person carried `Kin`.
    pub mother: Option<AgentId>,
    /// `Kin::father` when the person carried `Kin`.
    pub father: Option<AgentId>,
    /// Temperament as `[ns, ha, rd, p]` when the person carried one.
    pub temperament: Option<[u16; 4]>,
    /// Simulation tick of death.
    pub died_tick: Option<u64>,
    /// `DeathReason::as_u8` of the death.
    pub death_reason: Option<u8>,
}

/// The chronicle store (`SimResources::chronicle`).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Chronicle {
    entries: Vec<ChronicleEntry>,
    by_agent: BTreeMap<AgentId, Vec<u32>>,
    by_settlement: BTreeMap<SettlementId, Vec<u32>>,
    people: BTreeMap<AgentId, PersonRecord>,
}

impl Chronicle {
    /// An empty chronicle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `entry` and index it under every named person and settlement
    /// (once per distinct id).
    pub fn push(&mut self, entry: ChronicleEntry) {
        let index = u32::try_from(self.entries.len()).unwrap_or(u32::MAX);
        for id in entry.agents() {
            self.by_agent.entry(id).or_default().push(index);
        }
        for id in entry.settlements() {
            self.by_settlement.entry(id).or_default().push(index);
        }
        self.entries.push(entry);
    }

    /// Create the record of `id` the first time it is seen: `record` is called
    /// only when the table has no record yet, and `None` (e.g. the person is
    /// not alive) leaves the table unchanged.
    pub fn ensure_person(&mut self, id: AgentId, record: impl FnOnce() -> Option<PersonRecord>) {
        if let std::collections::btree_map::Entry::Vacant(slot) = self.people.entry(id) {
            if let Some(person) = record() {
                slot.insert(person);
            }
        }
    }

    /// Record a death: fill `died_tick` / `death_reason`, creating the record
    /// from `record` (the components read before despawn) if it is missing.
    pub fn close_person(
        &mut self,
        id: AgentId,
        tick: u64,
        reason: u8,
        record: impl FnOnce() -> PersonRecord,
    ) {
        let person = self.people.entry(id).or_insert_with(record);
        person.died_tick = Some(tick);
        person.death_reason = Some(reason);
    }

    /// Whether `id` already has a `FoundingMember` entry (shapes the
    /// formation entry only; never read by simulation logic).
    pub fn has_founding_member(&self, id: AgentId) -> bool {
        self.of_agent(id)
            .any(|entry| entry.kind == ChronicleKind::FoundingMember && entry.subject == Some(id))
    }

    /// Every entry in append order.
    pub fn entries(&self) -> &[ChronicleEntry] {
        &self.entries
    }

    /// Entries naming person `id`, in append order.
    pub fn of_agent(&self, id: AgentId) -> impl Iterator<Item = &ChronicleEntry> + '_ {
        self.by_agent
            .get(&id)
            .into_iter()
            .flatten()
            .filter_map(|&index| self.entries.get(index as usize))
    }

    /// Entries naming settlement `id`, in append order.
    pub fn of_settlement(&self, id: SettlementId) -> impl Iterator<Item = &ChronicleEntry> + '_ {
        self.by_settlement
            .get(&id)
            .into_iter()
            .flatten()
            .filter_map(|&index| self.entries.get(index as usize))
    }

    /// The record of person `id`, alive or dead.
    pub fn person(&self, id: AgentId) -> Option<&PersonRecord> {
        self.people.get(&id)
    }

    /// Every person record in AgentId order.
    pub fn people(&self) -> impl Iterator<Item = (AgentId, &PersonRecord)> + '_ {
        self.people.iter().map(|(id, person)| (*id, person))
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no entry was recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of person records.
    pub fn person_count(&self) -> usize {
        self.people.len()
    }

    /// Size estimate in bytes (Pass 19 C5): entries times
    /// `size_of::<ChronicleEntry>()`, plus the two indexes (keys, `Vec`
    /// headers and `u32` positions), plus the person table (keys and
    /// records). Allocator slack and B-tree node overhead are not counted.
    pub fn approx_bytes(&self) -> usize {
        use std::mem::size_of;
        let vec_header = size_of::<Vec<u32>>();
        let agent_index: usize = self
            .by_agent
            .values()
            .map(|refs| size_of::<AgentId>() + vec_header + refs.len() * size_of::<u32>())
            .sum();
        let settlement_index: usize = self
            .by_settlement
            .values()
            .map(|refs| size_of::<SettlementId>() + vec_header + refs.len() * size_of::<u32>())
            .sum();
        self.entries.len() * size_of::<ChronicleEntry>()
            + agent_index
            + settlement_index
            + self.people.len() * (size_of::<AgentId>() + size_of::<PersonRecord>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(female: bool) -> PersonRecord {
        PersonRecord {
            female: Some(female),
            born_tick: Some(-100),
            temperament: Some([1, 2, 3, 4]),
            ..PersonRecord::default()
        }
    }

    #[test]
    fn discriminants_are_stable() {
        for (index, kind) in ChronicleKind::ALL.iter().enumerate() {
            assert_eq!(usize::from(kind.as_u8()), index);
        }
        for (index, cause) in DepartCause::ALL.iter().enumerate() {
            assert_eq!(usize::from(cause.as_u8()), index);
        }
        assert_eq!(ChronicleKind::SettlementDissolved.as_u8(), 16);
        // Pass 20: kinds 17–21 appended; 0–16 unchanged.
        assert_eq!(ChronicleKind::PorterDeparted.as_u8(), 17);
        assert_eq!(ChronicleKind::PorterDelivered.as_u8(), 18);
        assert_eq!(ChronicleKind::Quarrel.as_u8(), 19);
        assert_eq!(ChronicleKind::VillagesEstranged.as_u8(), 20);
        assert_eq!(ChronicleKind::VillagesReconciled.as_u8(), 21);
        // M1: kind 22 appended; 0–21 unchanged.
        assert_eq!(ChronicleKind::HutBuilt.as_u8(), 22);
        assert_eq!(ChronicleKind::ALL.len(), 23);
        let names: std::collections::BTreeSet<&str> =
            ChronicleKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(names.len(), 23);
        assert_eq!(DepartCause::TopUp.as_u8(), 4);
    }

    #[test]
    fn push_indexes_every_distinct_agent_and_settlement_once() {
        let mut chronicle = Chronicle::new();
        chronicle.push(
            ChronicleEntry::new(5, ChronicleKind::Born)
                .subject(10)
                .other(Some(3))
                .third(Some(4))
                .settlement(Some(1)),
        );
        chronicle.push(
            ChronicleEntry::new(6, ChronicleKind::Settled)
                .subject(3)
                .other(Some(3))
                .settlement(Some(2))
                .settlement2(Some(2)),
        );
        chronicle.push(
            ChronicleEntry::new(7, ChronicleKind::SettlementFormed)
                .settlement(Some(1))
                .value(9),
        );
        assert_eq!(chronicle.len(), 3);
        assert!(!chronicle.is_empty());
        let ticks =
            |entries: Vec<&ChronicleEntry>| entries.iter().map(|e| e.tick).collect::<Vec<_>>();
        assert_eq!(ticks(chronicle.of_agent(3).collect()), vec![5, 6]);
        assert_eq!(ticks(chronicle.of_agent(4).collect()), vec![5]);
        assert_eq!(ticks(chronicle.of_agent(10).collect()), vec![5]);
        assert_eq!(ticks(chronicle.of_agent(99).collect()), Vec::<u64>::new());
        assert_eq!(ticks(chronicle.of_settlement(1).collect()), vec![5, 7]);
        assert_eq!(ticks(chronicle.of_settlement(2).collect()), vec![6]);
        assert_eq!(chronicle.by_agent[&3], vec![0, 1]);
        assert_eq!(chronicle.by_settlement[&2], vec![1]);
    }

    #[test]
    fn entries_keep_append_order() {
        let mut chronicle = Chronicle::new();
        for tick in [9, 3, 7] {
            chronicle.push(ChronicleEntry::new(tick, ChronicleKind::CameOfAge).subject(1));
        }
        assert_eq!(
            chronicle
                .entries()
                .iter()
                .map(|e| e.tick)
                .collect::<Vec<_>>(),
            vec![9, 3, 7]
        );
    }

    #[test]
    fn person_table_is_created_once_and_outlives_death() {
        let mut chronicle = Chronicle::new();
        chronicle.ensure_person(1, || Some(person(true)));
        // A second ensure never calls the closure or overwrites.
        chronicle.ensure_person(1, || panic!("record must not be rebuilt"));
        chronicle.ensure_person(2, || None);
        assert!(chronicle.person(2).is_none());
        chronicle.close_person(1, 50, 3, || panic!("record exists"));
        let one = chronicle.person(1).expect("record");
        assert_eq!(
            (one.female, one.died_tick, one.death_reason),
            (Some(true), Some(50), Some(3))
        );
        // A death without a prior record creates it from the components.
        chronicle.close_person(7, 60, 0, || person(false));
        let seven = chronicle.person(7).expect("created at death");
        assert_eq!(
            (seven.female, seven.born_tick, seven.died_tick),
            (Some(false), Some(-100), Some(60))
        );
        assert_eq!(
            chronicle.people().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![1, 7]
        );
        assert_eq!(chronicle.person_count(), 2);
    }

    #[test]
    fn founding_member_lookup_reads_only_that_kind() {
        let mut chronicle = Chronicle::new();
        chronicle.push(
            ChronicleEntry::new(1, ChronicleKind::Joined)
                .subject(4)
                .settlement(Some(1)),
        );
        assert!(!chronicle.has_founding_member(4));
        chronicle.push(
            ChronicleEntry::new(2, ChronicleKind::FoundingMember)
                .subject(4)
                .settlement(Some(1)),
        );
        assert!(chronicle.has_founding_member(4));
        assert!(!chronicle.has_founding_member(5));
    }

    #[test]
    fn approx_bytes_counts_entries_indexes_and_people() {
        let mut chronicle = Chronicle::new();
        assert_eq!(chronicle.approx_bytes(), 0);
        chronicle.push(ChronicleEntry::new(1, ChronicleKind::CameOfAge).subject(4));
        chronicle.ensure_person(4, || Some(PersonRecord::default()));
        let expected = std::mem::size_of::<ChronicleEntry>()
            + std::mem::size_of::<AgentId>()
            + std::mem::size_of::<Vec<u32>>()
            + 4
            + std::mem::size_of::<AgentId>()
            + std::mem::size_of::<PersonRecord>();
        assert_eq!(chronicle.approx_bytes(), expected);
    }
}

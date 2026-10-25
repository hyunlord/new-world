//! Personal knowledge entries and cached practitioner bits; saved layout is unchanged.
use super::{bit, day_of, defs, KnowledgeDefs, KnowledgeId, Source};
use serde::{Deserialize, Serialize};

/// One thing a person knows (goal 3.2). 6 bytes (M4 DECISIONS D10: the last
/// use is kept as a day index, which is all forgetting and the aggregate read;
/// the node id and the source share a byte; the part gained by observation is
/// kept in 5 ‰ steps, the size of one observation gain).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// The day index (tick / 1,440) it was last used (or gained).
    pub last_used: u16,
    /// Proficiency ‰ (0..=1000).
    pub proficiency: u16,
    /// The node id (low 5 bits) and `Source::code` (high 3 bits).
    tag: u8,
    /// The part gained by observation only, in 5 ‰ steps.
    observed5: u8,
}

impl KnowledgeEntry {
    /// A new entry.
    pub const fn new(id: KnowledgeId, source: Source, proficiency: u16, day: u16) -> Self {
        Self {
            last_used: day,
            proficiency,
            tag: (id & 31) | (source.code() << 5),
            observed5: 0,
        }
    }

    /// The node.
    pub const fn id(&self) -> KnowledgeId {
        self.tag & 31
    }

    /// How it was gained.
    pub const fn source(&self) -> Source {
        Source::from_code(self.tag >> 5)
    }

    fn set_source(&mut self, source: Source) {
        self.tag = (self.tag & 31) | (source.code() << 5);
    }

    /// The part gained by observation only (‰, a multiple of 5).
    pub const fn observed(&self) -> u16 {
        self.observed5 as u16 * 5
    }
}

/// What a person knows (an ECS component, M4 ADR 0024): entries sorted by id
/// and the cached practitioner bitset. Colony founders get it at worldgen;
/// anyone else gets it with their first entry (a missing component = knows nothing).
///
/// A load rebuilds the bitset from the entries (`serde(try_from)`), so a save made
/// before a threshold changed in `knowledge.ron` gates by the new thresholds
/// (code review F8); the saved `practice` is kept in the format but not trusted.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedKnowledge")]
pub struct Knowledge {
    entries: Box<[KnowledgeEntry]>,
    practice: u32,
}

/// The saved form of [`Knowledge`] (the same fields).
#[derive(Deserialize)]
struct SavedKnowledge {
    entries: Vec<KnowledgeEntry>,
    #[serde(rename = "practice")]
    _practice: u32,
}

impl TryFrom<SavedKnowledge> for Knowledge {
    type Error = String;

    fn try_from(saved: SavedKnowledge) -> Result<Self, Self::Error> {
        let catalog = crate::colony::content::installed().map_err(|e| e.to_string())?;
        Ok(Self::from_entries_with_defs(
            saved.entries,
            catalog.knowledge_defs(),
        ))
    }
}

impl Knowledge {
    /// Every entry, in id order.
    pub fn entries(&self) -> &[KnowledgeEntry] {
        &self.entries
    }

    /// The entry of node `id`.
    pub fn get(&self, id: KnowledgeId) -> Option<&KnowledgeEntry> {
        self.entries
            .binary_search_by_key(&id, |e| e.id())
            .ok()
            .map(|i| &self.entries[i])
    }

    /// The proficiency of `id` (0 when unknown).
    pub fn proficiency(&self, id: KnowledgeId) -> u16 {
        self.get(id).map_or(0, |e| e.proficiency)
    }

    /// The practitioner bitset (bit `i` = at or above node `i`'s threshold).
    pub const fn practice(&self) -> u32 {
        self.practice
    }

    /// Whether the person practises every node of `mask`.
    pub const fn practises_all(&self, mask: u32) -> bool {
        self.practice & mask == mask
    }

    /// Whether the person practises node `id`.
    pub const fn practises(&self, id: KnowledgeId) -> bool {
        self.practice & bit(id) != 0
    }

    fn refresh(&mut self) {
        self.refresh_with_defs(defs());
    }

    fn refresh_with_defs(&mut self, defs: &KnowledgeDefs) {
        self.practice = self
            .entries
            .iter()
            .filter(|e| e.proficiency >= defs.threshold(e.id()))
            .fold(0, |m, e| m | bit(e.id()));
    }

    /// Knowledge holding exactly `entries` (sorted here by id; a node listed
    /// twice keeps its last entry), boxed once at its final size — the bulk
    /// constructor for worldgen, where entry-by-entry `set` re-boxed the slice
    /// for every node (M4 DECISIONS D23).
    pub fn from_entries(entries: Vec<KnowledgeEntry>) -> Self {
        Self::from_entries_with_defs(entries, defs())
    }

    fn from_entries_with_defs(mut entries: Vec<KnowledgeEntry>, defs: &KnowledgeDefs) -> Self {
        entries.reverse();
        entries.sort_by_key(KnowledgeEntry::id);
        entries.dedup_by_key(|e| e.id());
        entries.shrink_to_fit();
        let mut k = Self {
            entries: entries.into_boxed_slice(),
            practice: 0,
        };
        k.refresh_with_defs(defs);
        k
    }

    /// Set node `id` to `proficiency` from `source` at `tick` (inserted in id
    /// order when new). The cached bitset is refreshed.
    pub fn set(&mut self, id: KnowledgeId, proficiency: u16, source: Source, tick: u64) {
        let p = proficiency.min(1000);
        match self.entries.binary_search_by_key(&id, |e| e.id()) {
            Ok(i) => {
                let e = &mut self.entries[i];
                e.proficiency = p;
                e.set_source(source);
                e.last_used = day_of(tick);
            }
            Err(i) => {
                let mut v = std::mem::take(&mut self.entries).into_vec();
                v.insert(i, KnowledgeEntry::new(id, source, p, day_of(tick)));
                self.entries = v.into_boxed_slice();
            }
        }
        self.refresh();
    }

    /// Raise node `id` to at least `proficiency` from `source` (a lower or equal
    /// value changes nothing); returns whether it rose. M5 (the owner's M4
    /// decision, M5 DECISIONS D5): an entry the person still holds keeps the
    /// source it was first held by — and so its forgetting floor — when it is
    /// learned again; `source` labels only a new entry.
    pub fn raise_to(
        &mut self,
        id: KnowledgeId,
        proficiency: u16,
        source: Source,
        tick: u64,
    ) -> bool {
        if self.proficiency(id) >= proficiency && self.get(id).is_some() {
            return false;
        }
        let source = self.get(id).map_or(source, |e| e.source());
        self.set(id, proficiency, source, tick);
        true
    }

    /// Add `delta` ‰ to node `id` (capped at 1000), keeping its source; the
    /// entry is created with `source` when missing. Stamps the last use.
    pub fn add(&mut self, id: KnowledgeId, delta: u16, source: Source, tick: u64) {
        let (p, s) = self
            .get(id)
            .map_or((0, source), |e| (e.proficiency, e.source()));
        self.set(id, p.saturating_add(delta).min(1000), s, tick);
    }

    /// A completed gated job (goal 3.2): every node of `mask` the person holds
    /// gains `gain` ‰ and is stamped used at `tick`.
    pub fn use_mask(&mut self, mask: u32, gain: u16, tick: u64) {
        for e in self.entries.iter_mut().filter(|e| mask & bit(e.id()) != 0) {
            e.proficiency = e.proficiency.saturating_add(gain).min(1000);
            e.last_used = day_of(tick);
        }
        self.refresh();
    }

    /// Stamp node `id` used at `tick` without a gain.
    pub fn touch(&mut self, id: KnowledgeId, tick: u64) {
        if let Ok(i) = self.entries.binary_search_by_key(&id, |e| e.id()) {
            self.entries[i].last_used = day_of(tick);
        }
    }

    /// Add `delta` ‰ gained by observation to node `id`, never past `cap`
    /// (the threshold − 50); creates an `Observed` entry when missing.
    pub fn observe(&mut self, id: KnowledgeId, delta: u16, cap: u16, tick: u64) -> bool {
        let before = self.proficiency(id);
        if before >= cap {
            return false;
        }
        let gain = delta.min(cap - before);
        let source = self.get(id).map_or(Source::Observed, |e| e.source());
        self.set(id, before + gain, source, tick);
        if let Ok(i) = self.entries.binary_search_by_key(&id, |e| e.id()) {
            let steps = u8::try_from(gain.div_ceil(5)).unwrap_or(u8::MAX);
            self.entries[i].observed5 = self.entries[i].observed5.saturating_add(steps);
        }
        true
    }

    /// Apply forgetting (goal 3.3): every entry unused for more than
    /// `unused_days` whole days before `tick`'s day loses `rate(source)` ‰,
    /// never below `floor(source)`; an entry at 0 is removed. Returns whether
    /// anything changed.
    pub fn decay(
        &mut self,
        tick: u64,
        unused_days: u64,
        rate: impl Fn(Source) -> u16,
        floor: impl Fn(Source) -> u16,
    ) -> bool {
        let mut changed = false;
        for e in &mut self.entries {
            if u64::from(day_of(tick).saturating_sub(e.last_used)) <= unused_days {
                continue;
            }
            let low = floor(e.source());
            let next = e
                .proficiency
                .saturating_sub(rate(e.source()))
                .max(low.min(e.proficiency));
            if next != e.proficiency {
                e.proficiency = next;
                changed = true;
            }
        }
        if changed {
            if self.entries.iter().any(|e| e.proficiency == 0) {
                let mut v = std::mem::take(&mut self.entries).into_vec();
                v.retain(|e| e.proficiency > 0);
                self.entries = v.into_boxed_slice();
            }
            self.refresh();
        }
        changed
    }

    /// Bytes this component holds (inline + entries), for the memory report.
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.entries.len() * std::mem::size_of::<KnowledgeEntry>()
    }
}

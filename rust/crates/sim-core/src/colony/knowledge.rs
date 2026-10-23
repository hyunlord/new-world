//! Knowledge (M4, ADR 0024–0026): the knowledge map, a person's `Knowledge`,
//! the gate masks and the settlement's technology aggregate. Data only — the
//! channels, forgetting, discovery and the aggregate live in
//! `sim_systems::runtime::knowledge`, the gating in the job layer.
//!
//! The map is data (`data/colony/knowledge.ron`, embedded and parsed once by
//! [`defs`]); a node's [`KnowledgeId`] is its index in file order. A person is
//! a **practitioner** of a node when their proficiency (‰) is at or above the
//! node's threshold; [`Knowledge::practice`] caches that as a bitset (bit `i` =
//! node `i`), refreshed by every write. A job is gated by the nodes whose gates
//! cover its recipe, blueprint or designation ([`gates`], precomputed masks).
//!
//! Integers only (ADR 0016 C4): proficiencies in permille, ticks as `u32`.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use super::{BlueprintKind, Designation, ItemKind, RecipeKind, SkillKind};
use crate::components::AgentId;

/// A node of the knowledge map: its index in `knowledge.ron` (file order).
pub type KnowledgeId = u8;

/// Most nodes the map may hold (the practitioner bitset is a `u32`).
pub const MAX_NODES: usize = 32;

/// The quality ceiling of a craft by proficiency decile (goal 3.2): the VISION's
/// `0.3 + 0.7·p²` shape as an integer table, index `proficiency / 100`.
pub const QUAL_CAP: [u8; 11] = [0, 20, 35, 45, 55, 65, 75, 85, 92, 97, 100];

/// The quality cap of a craft made at `proficiency` ‰.
pub const fn quality_cap(proficiency: u16) -> u8 {
    let i = (proficiency / 100) as usize;
    QUAL_CAP[if i > 10 { 10 } else { i }]
}

/// A knowledge lane (the tech map's rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Lane {
    /// Fire, food, water, tracking.
    Survival,
    /// Stone and wood crafts.
    Craft,
    /// Building.
    Build,
    /// Quarrying and digging.
    Mine,
    /// Clay and pots.
    Pottery,
    /// Teaching and telling.
    Social,
}

impl Lane {
    /// Every lane in display order.
    pub const ALL: [Self; 6] = [
        Self::Survival,
        Self::Craft,
        Self::Build,
        Self::Mine,
        Self::Pottery,
        Self::Social,
    ];

    /// Stable code (0 survival … 5 social).
    pub const fn code(self) -> u8 {
        match self {
            Self::Survival => 0,
            Self::Craft => 1,
            Self::Build => 2,
            Self::Mine => 3,
            Self::Pottery => 4,
            Self::Social => 5,
        }
    }

    /// Stable lowercase name (`knowledge.ron`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Craft => "craft",
            Self::Build => "build",
            Self::Mine => "mine",
            Self::Pottery => "pottery",
            Self::Social => "social",
        }
    }

    /// Parse [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.name() == name)
    }
}

/// The era a node belongs to (the tech map's columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Era {
    /// The Old Stone Age.
    Paleolithic,
    /// The New Stone Age.
    Neolithic,
}

impl Era {
    /// Stable code (0 paleolithic, 1 neolithic).
    pub const fn code(self) -> u8 {
        match self {
            Self::Paleolithic => 0,
            Self::Neolithic => 1,
        }
    }

    /// Parse the `knowledge.ron` name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "paleolithic" => Some(Self::Paleolithic),
            "neolithic" => Some(Self::Neolithic),
            _ => None,
        }
    }
}

/// How a person came by an entry (goal 3.2); sets its forgetting rate and floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Known from the start (worldgen).
    Seeded,
    /// Found by the person (`DiscoverySystem`).
    Discovered,
    /// Heard at the evening gathering.
    Oral,
    /// Picked up by watching a practitioner at work.
    Observed,
    /// Taught in apprenticeship sessions.
    Apprenticed,
}

impl Source {
    /// Every source in code order.
    pub const ALL: [Self; 5] = [
        Self::Seeded,
        Self::Discovered,
        Self::Oral,
        Self::Observed,
        Self::Apprenticed,
    ];

    /// Stable code (0 seeded … 4 apprenticed); index into the rule's decay tables.
    pub const fn code(self) -> u8 {
        match self {
            Self::Seeded => 0,
            Self::Discovered => 1,
            Self::Oral => 2,
            Self::Observed => 3,
            Self::Apprenticed => 4,
        }
    }

    /// The source of [`Self::code`] (`Seeded` for an unknown code).
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Discovered,
            2 => Self::Oral,
            3 => Self::Observed,
            4 => Self::Apprenticed,
            _ => Self::Seeded,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Seeded => "seeded",
            Self::Discovered => "discovered",
            Self::Oral => "oral",
            Self::Observed => "observed",
            Self::Apprenticed => "apprenticed",
        }
    }
}

/// A settlement condition of a discovery (goal 3.1 `discover`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// At least `n` items of the kind on stockpile cells.
    Stockpile(ItemKind, u16),
    /// Someone has the skill at the level or above.
    Skill(SkillKind, u8),
    /// Someone holds the node at the proficiency (‰) or above.
    Knows(KnowledgeId, u16),
    /// A clay bank lies within this Chebyshev distance of the camp hearth.
    ClayBankNear(u32),
}

/// The apprenticeship and oral channels a node may unlock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Teaching at full fidelity (`instruction`).
    Apprenticeship,
    /// A better chance at the evening telling (`storytelling`).
    Oral,
}

/// Everyday lore a node stands for: no job is gated; daily life uses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lore {
    /// Keeping the hearth (used at the evening gathering).
    Hearth,
    /// Finding food (used while foraging or eating).
    Forage,
    /// Finding water (used while drinking).
    Water,
}

/// What only a node's practitioners may do (goal 3.1 `gates`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// Craft the recipe.
    Recipe(RecipeKind),
    /// Build the blueprint's site.
    Blueprint(BlueprintKind),
    /// Build the workshop's site.
    Workshop(BlueprintKind),
    /// Work the designation.
    Designation(Designation),
    /// A transmission channel.
    Channel(Channel),
    /// Everyday lore (gates no job).
    Lore(Lore),
}

/// Who knows a node at the start (goal 3.1 "seeded").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seeding {
    /// No one.
    Nobody,
    /// Every founder at 600 ‰.
    All,
    /// `n` of every 10 founders at 500–700 ‰.
    Count(u8),
}

/// One node of the knowledge map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeDef {
    /// Its id (index in the file).
    pub id: KnowledgeId,
    /// Stable name (`knowledge.ron` id).
    pub name: String,
    /// Lane.
    pub lane: Lane,
    /// Era.
    pub era: Era,
    /// Proficiency ‰ at which a person is a practitioner.
    pub threshold: u16,
    /// Nodes a discoverer must practise first.
    pub prereqs: Vec<KnowledgeId>,
    /// Discovery chance class 1–3.
    pub difficulty: u8,
    /// Settlement conditions of a discovery (all must hold; empty = never discovered).
    pub discover: Vec<Cond>,
    /// What only practitioners may do.
    pub gates: Vec<Gate>,
    /// Era-marker weight (0 = not a marker).
    pub marker_weight: u16,
    /// Who knows it at the start.
    pub seed: Seeding,
    /// Locale key of its name.
    pub name_key: String,
}

/// The parsed knowledge map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnowledgeDefs {
    /// Every node in id order.
    pub nodes: Vec<KnowledgeDef>,
    /// Era markers M4 does not have yet, by name and weight (they count in the denominator).
    pub absent_markers: Vec<(String, u16)>,
}

impl KnowledgeDefs {
    /// The node `id`.
    pub fn get(&self, id: KnowledgeId) -> Option<&KnowledgeDef> {
        self.nodes.get(usize::from(id))
    }

    /// The id of the node named `name`.
    pub fn id_of(&self, name: &str) -> Option<KnowledgeId> {
        self.nodes
            .iter()
            .position(|n| n.name == name)
            .and_then(|i| u8::try_from(i).ok())
    }

    /// The threshold of `id` (‰; 1000 for an unknown id).
    pub fn threshold(&self, id: KnowledgeId) -> u16 {
        self.get(id).map_or(1000, |n| n.threshold)
    }

    /// The total marker weight, absent markers included (the era denominator).
    pub fn marker_total(&self) -> u32 {
        self.nodes
            .iter()
            .map(|n| u32::from(n.marker_weight))
            .chain(self.absent_markers.iter().map(|(_, w)| u32::from(*w)))
            .sum()
    }
}

#[derive(Deserialize)]
struct FileRaw {
    schema_version: u32,
    absent_markers: Vec<(String, u16)>,
    knowledge: Vec<NodeRaw>,
}

#[derive(Deserialize)]
enum CondRaw {
    Stockpile(String, u16),
    Skill(String, u8),
    Knows(String, u16),
    ClayBankNear(u32),
}

#[derive(Deserialize)]
enum GateRaw {
    Recipe(String),
    Blueprint(String),
    Workshop(String),
    Designation(String),
    Channel(String),
    Lore(String),
}

#[derive(Deserialize)]
enum SeedRaw {
    None,
    All,
    Count(u8),
}

#[derive(Deserialize)]
struct NodeRaw {
    id: String,
    lane: String,
    era: String,
    threshold: u16,
    prereqs: Vec<String>,
    difficulty: u8,
    discover: Vec<CondRaw>,
    gates: Vec<GateRaw>,
    marker_weight: u16,
    seed: SeedRaw,
    name_key: String,
}

fn designation_from_name(name: &str) -> Option<Designation> {
    match name {
        "chop" => Some(Designation::Chop),
        "quarry" => Some(Designation::Quarry),
        "dig" => Some(Designation::Dig),
        "deconstruct" => Some(Designation::Deconstruct),
        "clay_pit" => Some(Designation::ClayPit),
        "hunt" => Some(Designation::Hunt),
        _ => None,
    }
}

/// Parse a knowledge file (schema 1); `None` on any error, unknown name,
/// duplicate id, bad threshold or difficulty, or more than [`MAX_NODES`] nodes.
pub fn parse_defs(text: &str) -> Option<KnowledgeDefs> {
    let file: FileRaw = ron::from_str(text).ok()?;
    if file.schema_version != 1 || file.knowledge.len() > MAX_NODES {
        return None;
    }
    let names: Vec<&str> = file.knowledge.iter().map(|n| n.id.as_str()).collect();
    let id_of = |name: &str| -> Option<KnowledgeId> {
        names
            .iter()
            .position(|n| *n == name)
            .and_then(|i| u8::try_from(i).ok())
    };
    let mut nodes = Vec::with_capacity(file.knowledge.len());
    for (i, raw) in file.knowledge.iter().enumerate() {
        if names[..i].contains(&raw.id.as_str())
            || raw.threshold == 0
            || raw.threshold > 1000
            || !(1..=3).contains(&raw.difficulty)
        {
            return None;
        }
        let prereqs = raw
            .prereqs
            .iter()
            .map(|p| id_of(p))
            .collect::<Option<Vec<_>>>()?;
        let discover = raw
            .discover
            .iter()
            .map(|c| match c {
                CondRaw::Stockpile(item, n) => {
                    Some(Cond::Stockpile(ItemKind::from_name(item)?, *n))
                }
                CondRaw::Skill(skill, l) => Some(Cond::Skill(SkillKind::from_name(skill)?, *l)),
                CondRaw::Knows(node, p) => Some(Cond::Knows(id_of(node)?, *p)),
                CondRaw::ClayBankNear(d) => Some(Cond::ClayBankNear(*d)),
            })
            .collect::<Option<Vec<_>>>()?;
        let gates = raw
            .gates
            .iter()
            .map(|g| match g {
                GateRaw::Recipe(r) => Some(Gate::Recipe(RecipeKind::from_name(r)?)),
                GateRaw::Blueprint(b) => Some(Gate::Blueprint(BlueprintKind::from_name(b)?)),
                GateRaw::Workshop(b) => Some(Gate::Workshop(BlueprintKind::from_name(b)?)),
                GateRaw::Designation(d) => Some(Gate::Designation(designation_from_name(d)?)),
                GateRaw::Channel(c) => match c.as_str() {
                    "apprenticeship" => Some(Gate::Channel(Channel::Apprenticeship)),
                    "oral" => Some(Gate::Channel(Channel::Oral)),
                    _ => None,
                },
                GateRaw::Lore(l) => match l.as_str() {
                    "hearth" => Some(Gate::Lore(Lore::Hearth)),
                    "forage" => Some(Gate::Lore(Lore::Forage)),
                    "water" => Some(Gate::Lore(Lore::Water)),
                    _ => None,
                },
            })
            .collect::<Option<Vec<_>>>()?;
        nodes.push(KnowledgeDef {
            id: u8::try_from(i).ok()?,
            name: raw.id.clone(),
            lane: Lane::from_name(&raw.lane)?,
            era: Era::from_name(&raw.era)?,
            threshold: raw.threshold,
            prereqs,
            difficulty: raw.difficulty,
            discover,
            gates,
            marker_weight: raw.marker_weight,
            seed: match raw.seed {
                SeedRaw::None => Seeding::Nobody,
                SeedRaw::All => Seeding::All,
                SeedRaw::Count(n) => Seeding::Count(n),
            },
            name_key: raw.name_key.clone(),
        });
    }
    Some(KnowledgeDefs {
        nodes,
        absent_markers: file.absent_markers,
    })
}

const KNOWLEDGE_RON: &str = include_str!("../../data/colony/knowledge.ron");

/// The knowledge map, parsed once from the embedded `data/colony/knowledge.ron`
/// (empty only if the file failed to parse; a unit test rules it out).
pub fn defs() -> &'static KnowledgeDefs {
    static TABLE: OnceLock<KnowledgeDefs> = OnceLock::new();
    TABLE.get_or_init(|| parse_defs(KNOWLEDGE_RON).unwrap_or_default())
}

/// The id of the node named `name` in the embedded map.
pub fn id(name: &str) -> Option<KnowledgeId> {
    defs().id_of(name)
}

/// The bit of node `id` in a practitioner bitset.
pub const fn bit(id: KnowledgeId) -> u32 {
    1u32 << (id as u32 & 31)
}

/// The precomputed gate masks of the embedded map (goal 3.2, DECISIONS D2):
/// for each recipe, blueprint (its site's building) and designation, the
/// bitset of nodes a worker must practise. 0 = open to everyone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gates {
    recipe: Vec<u32>,
    build: Vec<u32>,
    designation: Vec<u32>,
    apprenticeship: u32,
    oral: u32,
    lore: [u32; 3],
}

impl Gates {
    fn from_defs(defs: &KnowledgeDefs) -> Self {
        let mut g = Self {
            recipe: vec![0; RecipeKind::ALL.len()],
            build: vec![0; BlueprintKind::ALL.len()],
            designation: vec![0; 8],
            ..Self::default()
        };
        for node in &defs.nodes {
            let b = bit(node.id);
            for gate in &node.gates {
                match *gate {
                    Gate::Recipe(r) => g.recipe[usize::from(r.code())] |= b,
                    Gate::Blueprint(k) | Gate::Workshop(k) => g.build[usize::from(k.code())] |= b,
                    Gate::Designation(d) => {
                        if let Some(slot) = g.designation.get_mut(usize::from(d.code())) {
                            *slot |= b;
                        }
                    }
                    Gate::Channel(Channel::Apprenticeship) => g.apprenticeship |= b,
                    Gate::Channel(Channel::Oral) => g.oral |= b,
                    Gate::Lore(l) => g.lore[l as usize] |= b,
                }
            }
        }
        g
    }

    /// Nodes gating a craft of `recipe`.
    pub fn recipe(&self, recipe: RecipeKind) -> u32 {
        self.recipe
            .get(usize::from(recipe.code()))
            .copied()
            .unwrap_or(0)
    }

    /// Nodes gating building a site of `kind` (its blueprint or workshop gates).
    pub fn build(&self, kind: BlueprintKind) -> u32 {
        self.build
            .get(usize::from(kind.code()))
            .copied()
            .unwrap_or(0)
    }

    /// Nodes gating work on a designation.
    pub fn designation(&self, d: Designation) -> u32 {
        self.designation
            .get(usize::from(d.code()))
            .copied()
            .unwrap_or(0)
    }

    /// The node that teaches at full fidelity (`instruction`).
    pub const fn apprenticeship(&self) -> u32 {
        self.apprenticeship
    }

    /// The node that tells better (`storytelling`).
    pub const fn oral(&self) -> u32 {
        self.oral
    }

    /// The node standing for everyday lore `l`.
    pub const fn lore(&self, l: Lore) -> u32 {
        self.lore[l as usize]
    }
}

/// The gate masks of the embedded map, computed once.
pub fn gates() -> &'static Gates {
    static TABLE: OnceLock<Gates> = OnceLock::new();
    TABLE.get_or_init(|| Gates::from_defs(defs()))
}

/// The ids set in `mask`, ascending.
pub fn ids_of(mask: u32) -> impl Iterator<Item = KnowledgeId> {
    (0..32u8).filter(move |i| mask & bit(*i) != 0)
}

/// Ticks per day of the knowledge clock (1 tick = 1 minute in every world).
pub const DAY_TICKS: u64 = 1_440;

/// The day index of `tick` as an entry stores it (saturating).
pub fn day_of(tick: u64) -> u16 {
    u16::try_from(tick / DAY_TICKS).unwrap_or(u16::MAX)
}

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
/// A load rebuilds the bitset from the entries (`serde(from)`), so a save made
/// before a threshold changed in `knowledge.ron` gates by the new thresholds
/// (code review F8); the saved `practice` is kept in the format but not trusted.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "SavedKnowledge")]
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

impl From<SavedKnowledge> for Knowledge {
    fn from(saved: SavedKnowledge) -> Self {
        Self::from_entries(saved.entries)
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
        let defs = defs();
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
    pub fn from_entries(mut entries: Vec<KnowledgeEntry>) -> Self {
        entries.reverse();
        entries.sort_by_key(KnowledgeEntry::id);
        entries.dedup_by_key(|e| e.id());
        entries.shrink_to_fit();
        let mut k = Self {
            entries: entries.into_boxed_slice(),
            practice: 0,
        };
        k.refresh();
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

/// The settlement-level state of one node (goal 3.4, ADR 0026).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum TechStatus {
    /// No one has ever held it.
    #[default]
    Unknown,
    /// Exactly one person holds it (the warning "⚠ 문: Fox만 앎").
    AtRisk,
    /// Two or more hold it, fewer than three practise it.
    Known,
    /// Three or more practise it.
    Stable,
    /// It was held and its last holder is gone.
    Lost,
}

impl TechStatus {
    /// Stable code (0 unknown, 1 at risk, 2 known, 3 stable, 4 lost).
    pub const fn code(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::AtRisk => 1,
            Self::Known => 2,
            Self::Stable => 3,
            Self::Lost => 4,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::AtRisk => "at_risk",
            Self::Known => "known",
            Self::Stable => "stable",
            Self::Lost => "lost",
        }
    }
}

/// One node's aggregate (goal 3.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TechState {
    /// Adults at threshold who used it lately (or gained it lately).
    pub practitioners: u16,
    /// People at threshold, used or not.
    pub holders: u16,
    /// The state.
    pub status: TechStatus,
    /// Tick the status last changed.
    pub last_change_tick: u64,
    /// Up to five practitioners (holders when none practise), by `AgentId`.
    pub names: Vec<AgentId>,
}

/// The settlement's technology (goal 3.4; `ColonyState::tech`): one state per
/// node in id order, the era progress ‰ and a version for renderers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TechAggregate {
    /// One per node, in id order (empty before the first aggregate).
    pub states: Vec<TechState>,
    /// Era progress ‰ (Σ weight · coverage / Σ weight).
    pub progress_permille: u16,
    /// Adults counted at the last aggregate.
    pub adults: u16,
    /// Bumped whenever a state or the progress changes.
    pub version: u64,
}

impl TechAggregate {
    /// Nodes at risk (the HUD badge).
    pub fn at_risk(&self) -> usize {
        self.states
            .iter()
            .filter(|s| s.status == TechStatus::AtRisk)
            .count()
    }
}

/// The era label of `progress` ‰ (goal 3.4): 0 구석기 (< 250), 1 신석기 전환기
/// (250–749), 2 신석기 시대 (≥ 750).
pub const fn era_label(progress: u16) -> u8 {
    if progress < 250 {
        0
    } else if progress < 750 {
        1
    } else {
        2
    }
}

/// One apprenticeship order (goal 3.3; `ColonyState::teach_orders`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeachOrder {
    /// Its id (from 1).
    pub id: u32,
    /// The teacher.
    pub teacher: AgentId,
    /// The learner.
    pub learner: AgentId,
    /// What is taught.
    pub knowledge: KnowledgeId,
    /// Sessions held so far.
    pub sessions: u32,
    /// The day index of `sessions_today`.
    pub day: u64,
    /// Sessions held on `day`.
    pub sessions_today: u8,
    /// Tick the order was placed.
    pub placed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_parses_to_24_nodes_with_an_acyclic_prereq_graph() {
        let d = defs();
        assert_eq!(d.nodes.len(), 24);
        assert!(d.nodes.len() <= MAX_NODES);
        // Every prereq points at another node, and the graph has no cycle
        // (a depth-first search from each node never revisits its path).
        fn cyclic(d: &KnowledgeDefs, at: KnowledgeId, path: &mut Vec<KnowledgeId>) -> bool {
            if path.contains(&at) {
                return true;
            }
            path.push(at);
            let bad = d.nodes[usize::from(at)]
                .prereqs
                .iter()
                .any(|p| cyclic(d, *p, path));
            path.pop();
            bad
        }
        for n in &d.nodes {
            assert!(
                !cyclic(d, n.id, &mut Vec::new()),
                "{} is in a cycle",
                n.name
            );
            assert!(!n.prereqs.contains(&n.id));
        }
        // Spot values from the goal table (3.1).
        let knap = id("flint_knapping").and_then(|i| d.get(i)).cloned();
        assert_eq!(
            knap.as_ref().map(|k| (k.threshold, k.seed)),
            Some((300, Seeding::Count(3)))
        );
        let fire = id("pot_firing").and_then(|i| d.get(i)).cloned();
        assert_eq!(
            fire.map(|k| (k.marker_weight, k.era, k.discover)),
            Some((
                3,
                Era::Neolithic,
                vec![Cond::Stockpile(ItemKind::RawPot, 2)]
            ))
        );
        // Markers 3 + 3 + 2 + 1 present, farming 3 and weaving 2 absent: 14.
        assert_eq!(d.marker_total(), 14);
        let present: u32 = d.nodes.iter().map(|n| u32::from(n.marker_weight)).sum();
        assert_eq!(present, 9);
        assert!(parse_defs("(schema_version: 2, absent_markers: [], knowledge: [])").is_none());
        let bad = KNOWLEDGE_RON.replace("Recipe(\"haft_axe\")", "Recipe(\"bronze\")");
        assert!(parse_defs(&bad).is_none());
        let dup = KNOWLEDGE_RON.replace("(id: \"foraging\"", "(id: \"fire_keeping\"");
        assert!(parse_defs(&dup).is_none());
    }

    #[test]
    fn gate_masks_cover_the_goal_table() {
        let g = gates();
        let b = |name: &str| id(name).map_or(0, bit);
        assert_eq!(g.recipe(RecipeKind::KnapBlade), b("flint_knapping"));
        assert_eq!(g.recipe(RecipeKind::HaftAxe), b("hafting"));
        assert_eq!(g.recipe(RecipeKind::FirePot), b("pot_firing"));
        assert_eq!(g.build(BlueprintKind::Door), b("doorway"));
        assert_eq!(g.build(BlueprintKind::KnappingSpot), b("flint_knapping"));
        assert_eq!(g.build(BlueprintKind::Hut), b("hut_building"));
        assert_eq!(g.designation(Designation::Quarry), b("stone_quarrying"));
        assert_eq!(g.designation(Designation::Dig), b("digging"));
        assert_eq!(g.designation(Designation::ClayPit), b("clay_digging"));
        assert_eq!(g.designation(Designation::Chop), 0);
        assert_eq!(g.apprenticeship(), b("instruction"));
        assert_eq!(g.lore(Lore::Hearth), b("fire_keeping"));
    }

    #[test]
    fn entries_stay_sorted_and_the_bitset_follows_the_thresholds() {
        let knap = id("flint_knapping").unwrap_or(0);
        let door = id("doorway").unwrap_or(0);
        let mut k = Knowledge::default();
        k.set(knap, 299, Source::Oral, 5);
        k.set(door, 200, Source::Seeded, 6);
        assert_eq!(k.entries().iter().map(|e| e.id()).collect::<Vec<_>>(), {
            let mut v = vec![knap, door];
            v.sort_unstable();
            v
        });
        assert!(!k.practises(knap) && k.practises(door));
        assert!(k.raise_to(knap, 350, Source::Oral, 7));
        assert!(!k.raise_to(knap, 300, Source::Oral, 8), "never lowers");
        assert!(k.practises(knap));
        // Days: tick 100 is day 0; tick 1,440 × d is day d.
        let day = |d: u64| d * DAY_TICKS;
        k.use_mask(bit(knap), 2, day(10));
        assert_eq!(
            (k.proficiency(knap), k.get(knap).map(|e| e.last_used)),
            (352, Some(10))
        );
        // Forgetting: unused > 3 days, oral 2 ‰ a step to 0, seeded 1 ‰ to 100.
        let rate = |s: Source| if s == Source::Oral { 2 } else { 1 };
        let floor = |s: Source| if s == Source::Seeded { 100 } else { 0 };
        // On day 13 only the doorway (last used on day 0) fades; knapping was used on day 10.
        assert!(k.decay(day(13), 3, rate, floor));
        assert_eq!((k.proficiency(knap), k.proficiency(door)), (352, 199));
        assert!(!k.practises(door), "199 is below the doorway's 200");
        assert!(k.decay(day(14), 3, rate, floor));
        assert_eq!((k.proficiency(knap), k.proficiency(door)), (350, 198));
        for t in 0..400 {
            k.decay(day(20) + t, 3, rate, floor);
        }
        assert_eq!(k.get(knap), None, "an oral entry at 0 is removed");
        assert_eq!(
            k.proficiency(door),
            100,
            "a seeded entry stops at its floor"
        );
        assert_eq!(k.entries().len(), k.entries().iter().count());
        assert_eq!(std::mem::size_of::<KnowledgeEntry>(), 6);
        assert!(std::mem::size_of::<Knowledge>() <= 24);
        // Observation stops at its cap.
        assert!(k.observe(knap, 5, 250, 900));
        for _ in 0..100 {
            k.observe(knap, 5, 250, 901);
        }
        assert_eq!(k.proficiency(knap), 250);
        assert_eq!(
            k.get(knap).map(|e| (e.source(), e.observed())),
            Some((Source::Observed, 250))
        );
    }

    #[test]
    fn a_load_rebuilds_the_bitset_from_the_entries() {
        let knap = id("flint_knapping").expect("knapping");
        let mut k = Knowledge::default();
        k.set(knap, 600, Source::Seeded, 0);
        let text = ron::to_string(&k).expect("save");
        // A save whose bitset disagrees with its entries (a threshold changed since).
        let tampered = text.replace(&format!("practice:{}", k.practice()), "practice:0");
        assert_ne!(tampered, text, "the bitset is in the saved form");
        let loaded: Knowledge = ron::from_str(&tampered).expect("load");
        assert_eq!(loaded.practice(), bit(knap));
        assert_eq!(loaded, k);
    }

    #[test]
    fn from_entries_sorts_keeps_the_last_of_a_node_and_sets_the_bitset() {
        let knap = id("flint_knapping").expect("knapping");
        let fire = id("fire_keeping").expect("fire");
        let door = id("doorway").expect("doorway");
        let k = Knowledge::from_entries(vec![
            KnowledgeEntry::new(door, Source::Seeded, 150, 0),
            KnowledgeEntry::new(knap, Source::Seeded, 100, 0),
            KnowledgeEntry::new(fire, Source::Seeded, 600, 0),
            KnowledgeEntry::new(knap, Source::Oral, 350, 2),
        ]);
        let ids: Vec<KnowledgeId> = k.entries().iter().map(KnowledgeEntry::id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "sorted by id");
        assert_eq!(ids.len(), 3, "one entry per node");
        // The later knapping entry wins: 350 ‰ (threshold 300), heard.
        assert_eq!(
            k.get(knap).map(|e| (e.proficiency, e.source())),
            Some((350, Source::Oral))
        );
        // Practises knapping (350 ≥ 300) and fire keeping (600 ≥ 300), not the doorway (150 < 200).
        assert_eq!(k.practice(), bit(knap) | bit(fire));
        // The same as entry-by-entry `set`.
        let mut by_set = Knowledge::default();
        by_set.set(door, 150, Source::Seeded, 0);
        by_set.set(fire, 600, Source::Seeded, 0);
        by_set.set(knap, 350, Source::Oral, 2 * DAY_TICKS);
        assert_eq!(k, by_set);
    }

    #[test]
    fn quality_caps_and_era_labels_follow_their_tables() {
        assert_eq!(
            (
                quality_cap(0),
                quality_cap(299),
                quality_cap(350),
                quality_cap(1000)
            ),
            (0, 35, 45, 100)
        );
        assert!(QUAL_CAP.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(
            (
                era_label(249),
                era_label(250),
                era_label(749),
                era_label(750)
            ),
            (0, 1, 1, 2)
        );
        let codes: Vec<u8> = Source::ALL.iter().map(|s| s.code()).collect();
        assert_eq!(codes, vec![0, 1, 2, 3, 4]);
        let lanes: Vec<u8> = Lane::ALL.iter().map(|l| l.code()).collect();
        assert_eq!(lanes, vec![0, 1, 2, 3, 4, 5]);
    }
}

//! Knowledge categories and authored definition records.
use super::KnowledgeId;
use crate::colony::{BlueprintKind, Designation, ItemKind, RecipeKind, SkillKind};
use serde::{Deserialize, Serialize};

/// A lane index in the selected data registry. Legacy constants remain aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Lane(pub u8);

#[allow(non_upper_case_globals)]
impl Lane {
    /// Legacy survival lane, now food.
    pub const Survival: Self = Self(0);
    /// Legacy craft lane, now tools.
    pub const Craft: Self = Self(3);
    /// Legacy building lane.
    pub const Build: Self = Self(4);
    /// Legacy mining lane, now tools.
    pub const Mine: Self = Self(3);
    /// Legacy pottery lane, now heat.
    pub const Pottery: Self = Self(5);
    /// Legacy social lane, now society.
    pub const Social: Self = Self(7);
    /// The ten bundled lanes; renderers enumerate `KnowledgeDefs::eras.lanes` instead.
    pub const ALL: [Self; 10] = [
        Self(0),
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
        Self(9),
    ];
    /// Registry display index.
    pub const fn code(self) -> u8 {
        self.0
    }
    /// Canonical name from selected data.
    pub fn name(self) -> &'static str {
        super::defs()
            .eras
            .lanes
            .get(usize::from(self.0))
            .map_or("unknown", |l| l.id.as_str())
    }
    /// Resolve against the selected lane registry and its legacy aliases.
    pub fn from_name(name: &str) -> Option<Self> {
        super::defs().eras.lane_order(name).map(Self)
    }
}

/// Whether authored content has runtime behaviour wired.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    /// Runtime targets must all resolve and participate in simulation.
    #[default]
    Connected,
    /// Authored, visible and recorded as debt; never enables behaviour.
    Inert,
}
impl ContentStatus {
    /// Stable bridge spelling.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::Inert => "Inert",
        }
    }
}

pub use crate::colony::eras::Era;

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
    /// M5.5 (ADR 0031): brought by a migrant from a neighbour; forgotten no lower than an apprenticed entry.
    Migrant,
}

impl Source {
    /// Every source in code order.
    pub const ALL: [Self; 6] = [
        Self::Seeded,
        Self::Discovered,
        Self::Oral,
        Self::Observed,
        Self::Apprenticed,
        Self::Migrant,
    ];

    /// Stable code (0 seeded … 4 apprenticed, 5 migrant; the entry keeps 3 bits for it).
    pub const fn code(self) -> u8 {
        match self {
            Self::Seeded => 0,
            Self::Discovered => 1,
            Self::Oral => 2,
            Self::Observed => 3,
            Self::Apprenticed => 4,
            Self::Migrant => 5,
        }
    }

    /// The index into the rule's decay and floor tables (`KnowledgeRule::decay`, `floor`): the code, except a
    /// migrant's entry, which fades and floors as an apprenticed one (goal M5.5 §3.2).
    pub const fn table_index(self) -> usize {
        match self {
            Self::Migrant => 4,
            other => other.code() as usize,
        }
    }

    /// The source of [`Self::code`] (`Seeded` for an unknown code).
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Discovered,
            2 => Self::Oral,
            3 => Self::Observed,
            4 => Self::Apprenticed,
            5 => Self::Migrant,
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
            Self::Migrant => "migrant",
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
    /// Someone completed this many jobs of the authored work type.
    Practice(u8, u32),
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
    /// Authored progression tier, 1–4.
    pub tier: u8,
    /// Whether runtime behaviour is wired.
    pub status: ContentStatus,
    /// Missing behaviour and planned content pass, for the generated ledger.
    pub debt: Option<String>,
    /// Authored gates retained for inert debt, including unresolved targets.
    pub inert_gates: Vec<String>,
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
    /// The selected era ladder and lane registry.
    pub eras: crate::colony::eras::EraCatalog,
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

    /// Legacy authored marker total including historical absent-marker metadata.
    /// Era progress uses only present markers grouped by their authored era.
    pub fn marker_total(&self) -> u32 {
        self.nodes
            .iter()
            .map(|n| u32::from(n.marker_weight))
            .chain(self.absent_markers.iter().map(|(_, w)| u32::from(*w)))
            .sum()
    }
}

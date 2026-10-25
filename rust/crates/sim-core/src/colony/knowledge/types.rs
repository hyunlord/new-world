//! Knowledge categories and authored definition records.
use super::KnowledgeId;
use crate::colony::{BlueprintKind, Designation, ItemKind, RecipeKind, SkillKind};
use serde::{Deserialize, Serialize};

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

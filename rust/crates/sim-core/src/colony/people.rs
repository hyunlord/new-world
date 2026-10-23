//! M5 components on people: illness (ADR 0027), drafting (ADR 0028), apparel (ADR 0029) and mood (ADR 0030).
use serde::{Deserialize, Serialize};

/// A sick person (M5 illness, ADR 0027): slower work and more sleep need until `until`; never fatal by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sick {
    /// The tick the illness ends.
    pub until: u64,
}

/// A top a person can wear (M5 apparel, ADR 0029); no `Apparel` component = the loincloth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Top {
    /// A wrap of cured hides.
    HideWrap,
    /// A cloak of wolf fur (droppable 8.5).
    FurCloak,
}

impl Top {
    /// Stable code for the snapshot's apparel column (0 = loincloth).
    pub const fn code(self) -> u8 {
        match self {
            Self::HideWrap => 1,
            Self::FurCloak => 2,
        }
    }

    /// Cold exposure damage while worn, ‰ (goal 3.3: hide wrap ×700, fur cloak ×500).
    pub const fn cold_permille(self) -> u32 {
        match self {
            Self::HideWrap => 700,
            Self::FurCloak => 500,
        }
    }

    /// The item it is made of.
    pub const fn item(self) -> super::ItemKind {
        match self {
            Self::HideWrap => super::ItemKind::HideWrap,
            Self::FurCloak => super::ItemKind::FurCloak,
        }
    }

    /// The top an item makes, if it is one.
    pub const fn of_item(kind: super::ItemKind) -> Option<Self> {
        match kind {
            super::ItemKind::HideWrap => Some(Self::HideWrap),
            super::ItemKind::FurCloak => Some(Self::FurCloak),
            _ => None,
        }
    }
}

/// What a person wears (M5 apparel, ADR 0029): the top and the day it was put on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Apparel {
    /// The top worn.
    pub top: Top,
    /// The display day it was put on.
    pub since_day: u16,
}

/// What a drafted person was told to do (M5 drafting, ADR 0028).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftOrder {
    /// Stand where they are.
    Hold,
    /// Walk to a tile and stand there.
    MoveTo((u32, u32)),
    /// Attack an animal or a person.
    Attack(super::wildlife::Fighter),
}

/// A drafted person (M5, ADR 0028): takes only direct orders; the decision system skips them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drafted {
    /// The tick they were drafted.
    pub since: u64,
    /// The standing order.
    pub order: DraftOrder,
    /// A weapon stack they walk to first (auto-arm), if any.
    pub arming: Option<(u32, u32)>,
}

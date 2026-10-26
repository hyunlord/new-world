//! Concrete order identities captured at application, never inferred from command history.
use super::mind_records::MindFactSource;
use super::{ItemKind, JobKind};
use serde::{Deserialize, Serialize};

/// Identity of the live order that supplied a job target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MindOrderKey {
    /// A tile designation, including its stable kind code.
    Designation {
        /// Target cell coordinates.
        tile: (u32, u32),
        /// Stable designation code.
        kind: u8,
    },
    /// One concrete construction entity, distinct from later sites at the same tile.
    Site {
        /// Complete hecs entity bits, including generation, preserved by saves.
        entity: u64,
        /// Site centre used by build and delivery jobs.
        centre: (u32, u32),
    },
    /// One cell of a particular stockpile zone.
    Stockpile {
        /// Monotonic stockpile zone identity.
        zone: u32,
        /// Target cell coordinates.
        tile: (u32, u32),
    },
    /// One production bill in its owning workshop.
    Bill {
        /// Owning workshop identity.
        workshop: u32,
        /// Concrete bill identity, not a recipe code.
        bill: u32,
    },
    /// One apprenticeship order.
    Teach {
        /// Concrete apprenticeship order identity.
        order: u32,
    },
    /// A direct hunting designation for an animal.
    HuntAnimal {
        /// Stable animal identity.
        animal: u32,
    },
    /// A hunting rectangle with inclusive endpoints.
    HuntZone {
        /// Inclusive minimum and maximum tile coordinates.
        rect: (u32, u32, u32, u32),
    },
    /// Goods owed to a specific caravan from a particular stack.
    Trade {
        /// Caravan visit identity.
        caravan: u32,
        /// Stack from which payment is collected.
        from: (u32, u32),
        /// Item identity of the promised goods.
        item: ItemKind,
    },
    /// The settlement standing order to equip clothing.
    Dress,
}

/// Assignment-time origin, valid only for the exact currently held job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindJobOrigin {
    /// Exact job kind and target captured on assignment or command retargeting.
    pub kind: JobKind,
    /// Original assignment tick; combined with kind to reject stale origins.
    pub taken_at: u64,
    /// Actual accepted command or chronicle reference.
    pub source: MindFactSource,
}

/// Command-mutated draft identity; autonomous arming progress is not an order change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindDraftOrigin {
    /// Tick this draft enrollment began.
    pub since: u64,
    /// Direct order captured at the command mutation boundary.
    pub order: super::people::DraftOrder,
    /// Actual accepted command or chronicle reference.
    pub source: MindFactSource,
}

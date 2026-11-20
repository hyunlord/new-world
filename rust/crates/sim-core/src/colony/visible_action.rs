//! Transient presentation values for current agent actions and physical item transfers.

use super::{ItemKind, RecipeKind, Tool};
use crate::{
    components::{AgentId, ResourceKind},
    MaterialId,
};
use serde::Deserialize;
use std::collections::VecDeque;

type Tile = (u32, u32);

macro_rules! action_codes {
    ($($name:ident = $code:literal),+ $(,)?) => {
        /// Stable presentation codes. Recipe specializations are resolved from data.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
        #[repr(u8)]
        pub enum VisibleActionKind { $(#[doc = concat!("Mechanical action ", stringify!($name), ".")] $name = $code),+ }

        impl VisibleActionKind {
            /// All supported mechanical actions.
            pub const ALL: &[Self] = &[$(Self::$name),+];
        }
    };
}

action_codes! {
    Idle = 0, TendFire = 1, RoutineHandwork = 2, Sit = 3, Explore = 4,
    Play = 5, Chop = 6, Carry = 7, Build = 8, Dismantle = 9, Dig = 10,
    Mine = 11, Craft = 12, Teach = 13, Learn = 14, Hunt = 15, Strike = 16,
    Flee = 17, DraftMove = 18, Equip = 19, Harvest = 20, Till = 21,
    Sow = 22, TendCrop = 23, Capture = 24, Milk = 25, Slaughter = 26,
    Eat = 27, Drink = 28, Talk = 29, Sleep = 30,
}

/// Execution phase at a completed simulation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    /// No owned pose.
    Idle = 0,
    /// Moving to a target or intermediate stop.
    Approach = 1,
    /// Actively performing the action.
    Perform = 2,
    /// Moving a physical load to its destination.
    Transport = 3,
}

/// Target namespaces. Tile-addressed objects do not require a numeric ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VisibleTargetKind {
    /// A tile without a more specific physical object.
    Tile = 0,
    /// An occupied assigned sleep fixture.
    Bed = 1,
    /// A physical hearth.
    Hearth = 2,
    /// A workshop keyed by WorkshopId.
    Workshop = 3,
    /// A work site, optionally keyed by its template BuildingSiteId.
    Site = 4,
    /// A live wild animal keyed by AnimalId.
    WildAnimal = 5,
    /// A domesticated animal keyed by LivestockId.
    Livestock = 6,
    /// A live person keyed by AgentId.
    Agent = 7,
    /// A tile-addressed item source or deposit.
    Stack = 8,
    /// A tree on the given tile.
    Tree = 9,
    /// A harvestable plant on the given tile.
    Plant = 10,
    /// A cultivated tile.
    Field = 11,
    /// A geological source on its tile.
    Deposit = 12,
    /// A livestock enclosure keyed by BuildingSiteId.
    Pen = 13,
    /// A legacy tile-based food source.
    FoodStore = 14,
    /// A physical food stack carried by the identified agent.
    PersonalFood = 15,
    /// A tile-based water source.
    Water = 16,
    /// The delivery point of a trading caravan.
    Trade = 17,
    /// A stockpile cell keyed by its zone.
    Stockpile = 18,
    /// A deployed pot keyed by its object ID.
    Pot = 19,
    /// A legacy stockpile keyed by its settlement.
    SettlementStore = 20,
}

/// A reference to the physical subject of the action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleTarget {
    /// Object identity namespace.
    pub kind: VisibleTargetKind,
    /// Current physical location.
    pub tile: Option<Tile>,
    /// ID in the target kind’s namespace.
    pub object_id: Option<u64>,
}

impl VisibleTarget {
    /// Tile-addressed target.
    pub const fn at(kind: VisibleTargetKind, tile: Tile) -> Self {
        Self {
            kind,
            tile: Some(tile),
            object_id: None,
        }
    }

    /// Target with a typed identity.
    pub const fn identified(kind: VisibleTargetKind, tile: Tile, id: u64) -> Self {
        Self {
            kind,
            tile: Some(tile),
            object_id: Some(id),
        }
    }
}

/// Exact item and quantity represented in a hand or a physical transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarriedItemView {
    /// Item catalogue identity.
    pub item: ItemKind,
    /// Material when the physical item has one.
    pub material: Option<MaterialId>,
    /// Exact represented quantity.
    pub qty: u16,
}

impl CarriedItemView {
    /// Project the physical spec and count.
    pub const fn of(spec: super::ItemSpec, qty: u16) -> Self {
        Self {
            item: spec.kind,
            material: spec.material,
            qty,
        }
    }

    /// Translate counted legacy cargo through the installed catalogue's roles.
    pub fn from_legacy_resource(kind: ResourceKind, count: u32) -> Option<Self> {
        let qty = u16::try_from(count).ok().filter(|qty| *qty > 0)?;
        let role = match kind {
            ResourceKind::Food => "legacy_food",
            ResourceKind::Wood => "deadwood_output",
            ResourceKind::Stone => "boulder_output",
            ResourceKind::Water => return None,
        };
        Some(Self {
            item: super::content::installed_or_bundled().item_role(role)?,
            material: None,
            qty,
        })
    }
}

/// The actual tool component supplies kind, material, quality and uses.
pub type ToolView = Tool;

/// One row per live agent. Timing is absent when the owner has no exact clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleActionRow {
    /// Stable agent identity.
    pub agent_id: AgentId,
    /// Current mechanical action code.
    pub action: VisibleActionKind,
    /// Current execution phase.
    pub phase: Phase,
    /// Current movement goal.
    pub goal: Option<Tile>,
    /// Subject of the current execution leg.
    pub target: Option<VisibleTarget>,
    /// Live partner bound to this interaction.
    pub partner: Option<AgentId>,
    /// Physical hand cargo.
    pub held: Option<CarriedItemView>,
    /// Equipped tool.
    pub tool: Option<ToolView>,
    /// Admitted recipe for active craft work.
    pub recipe: Option<RecipeKind>,
    /// Actual recorded start of the action.
    pub started_at_tick: Option<u64>,
    /// Exclusive deadline if tracked by the owner.
    pub until_tick: Option<u64>,
}

/// Successful item operations in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[repr(u8)]
pub enum VisibleCueKind {
    /// An actual pickup into carried inventory.
    PickedUp = 0,
    /// An actual transfer from carried inventory to its destination.
    Deposited = 1,
    /// A tool or apparel became equipped.
    Equipped = 2,
}

impl VisibleCueKind {
    /// All supported cue kinds.
    pub const ALL: &[Self] = &[Self::PickedUp, Self::Deposited, Self::Equipped];
}

/// A one-shot cue at its committing simulation tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleActionCue {
    /// Committed operation.
    pub kind: VisibleCueKind,
    /// Execution tick of the operation.
    pub tick: u64,
    /// Zero-based execution order within the tick.
    pub sequence: u32,
    /// Stable agent identity.
    pub agent_id: AgentId,
    /// Object involved in the transfer.
    pub target: VisibleTarget,
    /// Physical item and quantity transferred.
    pub item: CarriedItemView,
}

/// Read-side history. Appended only by existing successful physical operations.
#[derive(Debug, Default)]
pub struct VisibleActionCues {
    recent: VecDeque<VisibleActionCue>,
    evicted_through_tick: Option<u64>,
}

impl VisibleActionCues {
    /// Maximum retained cue count.
    pub const CAPACITY: usize = 256;

    /// Append a successful transfer at the current execution tick.
    pub fn record(
        &mut self,
        tick: u64,
        agent_id: AgentId,
        kind: VisibleCueKind,
        target: VisibleTarget,
        item: CarriedItemView,
    ) {
        let sequence = self
            .recent
            .back()
            .filter(|last| last.tick == tick)
            .map_or(0, |last| last.sequence + 1);
        if self.recent.len() == Self::CAPACITY {
            self.evicted_through_tick = self.recent.pop_front().map(|cue| cue.tick);
        }
        self.recent.push_back(VisibleActionCue {
            kind,
            tick,
            sequence,
            agent_id,
            target,
            item,
        });
    }

    /// Record a successful counted-resource transfer if it has an item identity.
    pub fn record_legacy(
        &mut self,
        tick: u64,
        agent: AgentId,
        kind: VisibleCueKind,
        target: VisibleTarget,
        resource: ResourceKind,
        count: u32,
    ) {
        if let Some(item) = CarriedItemView::from_legacy_resource(resource, count) {
            self.record(tick, agent, kind, target, item);
        }
    }

    /// Latest tick with at least one evicted cue.
    pub const fn evicted_through_tick(&self) -> Option<u64> {
        self.evicted_through_tick
    }

    /// Exclusive tick cursor. None replays all retained cues.
    pub fn since_tick(&self, since_tick: Option<u64>) -> impl Iterator<Item = &VisibleActionCue> {
        self.recent
            .iter()
            .filter(move |cue| since_tick.is_none_or(|since| cue.tick > since))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_overflow_preserves_same_tick_sequence_and_reports_consumed_history() {
        let mut ring = VisibleActionCues::default();
        let target = VisibleTarget::at(VisibleTargetKind::Stack, (1, 1));
        let item = CarriedItemView {
            item: ItemKind::role("deadwood_output"),
            material: None,
            qty: 1,
        };
        for _ in 0..=VisibleActionCues::CAPACITY {
            ring.record(4, 1, VisibleCueKind::PickedUp, target, item);
        }
        assert_eq!(ring.evicted_through_tick(), Some(4));
        let cues: Vec<_> = ring.since_tick(Some(3)).collect();
        assert_eq!(cues.len(), VisibleActionCues::CAPACITY);
        assert_eq!(cues[0].sequence, 1);
        assert!(cues
            .windows(2)
            .all(|pair| pair[0].sequence + 1 == pair[1].sequence));
        assert_eq!(ring.since_tick(Some(4)).count(), 0);

        ring.record(5, 1, VisibleCueKind::Deposited, target, item);
        let latest = ring.since_tick(Some(4)).next().expect("new tick");
        assert_eq!((latest.tick, latest.sequence), (5, 0));
        assert_eq!(ring.evicted_through_tick(), Some(4));
    }
}

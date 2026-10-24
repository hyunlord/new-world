//! Player orders as recorded commands (M1, ADR 0012): the command, its result and the log entry.
use serde::{Deserialize, Serialize};

use super::{BillId, BillMode, BlueprintKind, RecipeKind, WorkType, WorkshopId};
use crate::components::AgentId;

/// One player order. Applied only through `sim_systems::commands::apply_command`
/// and recorded with its tick and result, so a run replays exactly.
/// Rectangles are inclusive tile corners in any order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Mark every tree in the rectangle for felling.
    DesignateChop {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Mark the trees on exactly these tiles (scripts and tools); same effect
    /// per listed tree tile as [`Command::DesignateChop`].
    DesignateChopTiles {
        /// Tiles `(x, y)`, in the order given.
        tiles: Vec<(u32, u32)>,
    },
    /// Remove chop marks and blueprints in the rectangle.
    CancelArea {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Add the rectangle's valid tiles as one stockpile zone.
    AddStockpile {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Remove the rectangle's tiles from every zone.
    RemoveZoneArea {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Place a blueprint with its footprint's top-left at `(x, y)`.
    PlaceBlueprint {
        /// The blueprint.
        kind: BlueprintKind,
        /// Footprint top-left x.
        x: u32,
        /// Footprint top-left y.
        y: u32,
    },
    /// Mark every resource node (boulder, flint outcrop) in the rectangle for quarrying (M2).
    DesignateQuarry {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Add a bill at the end of a workshop's list (M2 goal 3.5).
    AddBill {
        /// The workshop.
        workshop: WorkshopId,
        /// What to make.
        recipe: RecipeKind,
        /// Make n / maintain n.
        mode: BillMode,
    },
    /// Remove a bill (its crafts and supplies in progress are cancelled).
    RemoveBill {
        /// The workshop.
        workshop: WorkshopId,
        /// The bill.
        bill: BillId,
    },
    /// Change a bill's mode (its count of crafts done restarts at 0).
    SetBillMode {
        /// The workshop.
        workshop: WorkshopId,
        /// The bill.
        bill: BillId,
        /// The new mode.
        mode: BillMode,
    },
    /// Move a bill one place up (`up`) or down in its workshop's list.
    MoveBill {
        /// The workshop.
        workshop: WorkshopId,
        /// The bill.
        bill: BillId,
        /// Towards the front.
        up: bool,
    },
    /// Suspend or resume a bill.
    SetBillSuspended {
        /// The workshop.
        workshop: WorkshopId,
        /// The bill.
        bill: BillId,
        /// Suspended or not.
        suspended: bool,
    },
    /// Set one person's priority for one work type (0 = never, 1 highest … 4 lowest; M2 goal 3.6).
    SetWorkPriority {
        /// The person.
        agent: AgentId,
        /// The work type.
        work: WorkType,
        /// The level 0–4.
        level: u8,
    },
    /// Place one 1×1 space blueprint (wall, door, floor, bedroll) on every
    /// valid tile of the Bresenham line from `(x0, y0)` to `(x1, y1)` (M3).
    PlaceLine {
        /// The blueprint (a space kind).
        kind: BlueprintKind,
        /// Start x.
        x0: u32,
        /// Start y.
        y0: u32,
        /// End x.
        x1: u32,
        /// End y.
        y1: u32,
    },
    /// Place one 1×1 space blueprint on every valid tile of the rectangle, or
    /// only of its border when `hollow` (M3).
    PlaceRect {
        /// The blueprint (a space kind).
        kind: BlueprintKind,
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
        /// Only the border.
        hollow: bool,
    },
    /// Mark every undug hill tile of the rectangle (not on the map edge) for digging (M3).
    DesignateDig {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Mark every wall, door, packed floor and bedroll of the rectangle for deconstruction (M3).
    DesignateDeconstruct {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Give a room a role, or (`None`) let the colony decide again (M3, ADR 0023).
    SetRoomRole {
        /// The room.
        room: u32,
        /// The role; `None` = automatic.
        role: Option<crate::rooms::RoomRole>,
    },
    /// Mark every clay bank in the rectangle for digging clay (M4).
    DesignateClay {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Order `teacher` to teach `learner` the knowledge node `knowledge` in
    /// apprenticeship sessions (M4 goal 3.3).
    OrderTeach {
        /// The teacher (a practitioner of the node).
        teacher: AgentId,
        /// The learner (an adult or a child of at least 10 years, below the threshold).
        learner: AgentId,
        /// The node (`KnowledgeId`).
        knowledge: u8,
    },
    /// Cancel the apprenticeship order `order` (M4).
    CancelTeach {
        /// The order id.
        order: u32,
    },
    /// Test and probe only (M4 F10, never reachable from the game bridge):
    /// the person dies at once.
    DebugKill {
        /// The person.
        agent: AgentId,
    },
    /// Test and probe only (M5.5, the eureka fixture; never reachable from the game bridge): `count` items of `kind`
    /// at quality 50 appear on tile `(x, y)` (ledger created).
    DebugGive {
        /// The item kind.
        kind: super::ItemKind,
        /// How many.
        count: u16,
        /// The tile.
        x: u32,
        /// See [`Self::DebugGive::x`].
        y: u32,
    },
    /// Draft or release a person (M5, ADR 0028; rejected before M5).
    SetDrafted {
        /// The person.
        agent: AgentId,
        /// Drafted or not.
        drafted: bool,
    },
    /// Order a drafted person to walk to `(x, y)` and stand there (M5).
    DraftMove {
        /// The person.
        agent: AgentId,
        /// Target x.
        x: u32,
        /// Target y.
        y: u32,
    },
    /// Order a drafted person to attack the animal `animal` (M5).
    DraftAttack {
        /// The person.
        agent: AgentId,
        /// The animal (`AnimalId`).
        animal: u32,
    },
    /// Order a drafted person to attack the person `target` (M5; goal 3.5 right-click on a person).
    DraftAttackPerson {
        /// The drafted person.
        agent: AgentId,
        /// The person attacked (`AgentId`; not `agent`).
        target: AgentId,
    },
    /// Mark the deer `animal` to be hunted (M5).
    DesignateHunt {
        /// The deer (`AnimalId`).
        animal: u32,
    },
    /// Add a hunt zone: every deer inside the rect may be hunted (M5).
    AddHuntZone {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Remove the hunt zones overlapping the rect and the hunt marks of the deer inside it (M5).
    CancelHunt {
        /// Corner x.
        x0: u32,
        /// Corner y.
        y0: u32,
        /// Opposite corner x.
        x1: u32,
        /// Opposite corner y.
        y1: u32,
    },
    /// Turn the standing order "everyone in hides" on or off (M5, ADR 0029; on by default).
    SetDressOrder {
        /// On or off.
        on: bool,
    },
    /// M5.5 (ADR 0031): take migrant wave `wave` (its number) into the colony.
    AcceptMigrants {
        /// The wave number (`Wave::id`).
        wave: u32,
    },
    /// M5.5 (ADR 0031): send migrant wave `wave` away.
    DeclineMigrants {
        /// The wave number (`Wave::id`).
        wave: u32,
    },
    /// M5.5 (ADR 0032): barter with caravan `caravan`: hand over `give` (stockpiled stacks: `(x, y, count)`) for
    /// `take` (`(goods index, count)` of the caravan's goods); a gift when `take` is empty.
    Trade {
        /// The caravan number (`Caravan::id`).
        caravan: u32,
        /// What the colony gives: `(x, y, count)` of stacks on stockpile cells.
        give: Vec<(u32, u32, u16)>,
        /// What the colony takes: `(goods index, count)`.
        take: Vec<(u16, u16)>,
    },
    /// Player-selected automatic order policy.
    SetSteward {
        /// Whether automatic planning is enabled.
        on: bool,
        /// Allowed order families.
        scope: super::steward::StewardScope,
    },
    /// Ordinary command issued by the steward at the after-assignment phase.
    Steward {
        /// Versioned data rule id.
        rule: u16,
        /// Ordinary command, never another envelope or policy change.
        command: Box<Command>,
    },
}

impl Command {
    /// Stable snake-case name used by command scripts and logs.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DesignateChop { .. } => "designate_chop",
            Self::DesignateChopTiles { .. } => "designate_chop_tiles",
            Self::CancelArea { .. } => "cancel_area",
            Self::AddStockpile { .. } => "add_stockpile",
            Self::RemoveZoneArea { .. } => "remove_zone_area",
            Self::PlaceBlueprint { .. } => "place_blueprint",
            Self::DesignateQuarry { .. } => "designate_quarry",
            Self::AddBill { .. } => "add_bill",
            Self::RemoveBill { .. } => "remove_bill",
            Self::SetBillMode { .. } => "set_bill_mode",
            Self::MoveBill { .. } => "move_bill",
            Self::SetBillSuspended { .. } => "set_bill_suspended",
            Self::SetWorkPriority { .. } => "set_work_priority",
            Self::SetDrafted { .. } => "set_drafted",
            Self::PlaceLine { .. } => "place_line",
            Self::PlaceRect { .. } => "place_rect",
            Self::DesignateDig { .. } => "designate_dig",
            Self::DesignateDeconstruct { .. } => "designate_deconstruct",
            Self::SetRoomRole { .. } => "set_room_role",
            Self::DesignateClay { .. } => "designate_clay",
            Self::OrderTeach { .. } => "order_teach",
            Self::CancelTeach { .. } => "cancel_teach",
            Self::DebugKill { .. } => "debug_kill",
            Self::DebugGive { .. } => "debug_give",
            Self::DraftMove { .. } => "draft_move",
            Self::DraftAttack { .. } => "draft_attack",
            Self::DraftAttackPerson { .. } => "draft_attack_person",
            Self::DesignateHunt { .. } => "designate_hunt",
            Self::AddHuntZone { .. } => "add_hunt_zone",
            Self::CancelHunt { .. } => "cancel_hunt",
            Self::SetDressOrder { .. } => "set_dress_order",
            Self::AcceptMigrants { .. } => "accept_migrants",
            Self::DeclineMigrants { .. } => "decline_migrants",
            Self::Trade { .. } => "trade",
            Self::SetSteward { .. } => "set_steward",
            Self::Steward { .. } => "steward",
        }
    }
}

/// Why a command did nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RejectReason {
    /// The world has no colony rule (the baseline world takes no orders).
    NoColony,
    /// Part or all of the area is outside the map.
    OutOfMap,
    /// A footprint tile is water.
    Water,
    /// A footprint tile is mountain, a tree, a building, a site, a zone or reserved.
    Blocked,
    /// Valid, but there was nothing to change.
    Nothing,
    /// The command exists but is not part of this milestone (or rule set).
    NotYet,
    /// The workshop does not make that recipe (M2).
    WrongWorkshop,
    /// No such workshop, bill or person (M2).
    Unknown,
    /// A person stands on the tile (M3: a wall cannot be placed on them).
    Person,
    /// Undug hill: it must be dug first (M3).
    Hill,
    /// The map edge (M3: digging never opens it).
    Edge,
    /// Something is already built there (M3).
    Built,
    /// No one from the camp can walk there (M4 goal 3.6: across the river
    /// without a ford); the bridge shows "닿을 수 없음: 강 건너".
    Unreachable,
    /// The teacher does not practise what it should teach (M4).
    NoPractitioner,
    /// The learner is younger than the rule's learner age (M4); M5: a child cannot be drafted.
    TooYoung,
    /// The person is not drafted (M5: a draft order to someone not drafted).
    NotDrafted,
    /// M5.5: no migrant wave or caravan of that number is at the hearth.
    NoVisitor,
    /// M5.5: the traders refuse — what the colony gives is worth less than 80 % … 120 % of what it takes.
    Unbalanced,
}

impl RejectReason {
    /// Stable snake-case name for the bridge and logs.
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoColony => "no_colony",
            Self::OutOfMap => "out_of_map",
            Self::Water => "water",
            Self::Blocked => "blocked",
            Self::Nothing => "nothing",
            Self::NotYet => "not_yet",
            Self::WrongWorkshop => "wrong_workshop",
            Self::Unknown => "unknown",
            Self::Person => "person",
            Self::Hill => "hill",
            Self::Edge => "edge",
            Self::Built => "built",
            Self::Unreachable => "unreachable",
            Self::NoPractitioner => "no_practitioner",
            Self::TooYoung => "too_young",
            Self::NotDrafted => "not_drafted",
            Self::NoVisitor => "no_visitor",
            Self::Unbalanced => "unbalanced",
        }
    }
}

/// What a command did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether it changed anything.
    pub accepted: bool,
    /// Trees designated, tiles zoned or removed, things cancelled, sites placed.
    pub count: u32,
    /// Why not, when not accepted.
    pub reason: Option<RejectReason>,
}

impl CommandResult {
    /// Accepted with `count` changes.
    pub const fn ok(count: u32) -> Self {
        Self {
            accepted: true,
            count,
            reason: None,
        }
    }

    /// Rejected for `reason`.
    pub const fn rejected(reason: RejectReason) -> Self {
        Self {
            accepted: false,
            count: 0,
            reason: Some(reason),
        }
    }
}

/// One entry of the command log: applied before the systems of `tick` ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRecord {
    /// The tick whose systems ran after the command (the engine's next tick).
    pub tick: u64,
    /// The command.
    pub command: Command,
    /// What it did.
    pub result: CommandResult,
}

//! Frozen colony field order through format17, with versioned nested physical leaves.
use super::*;

/// Historical colony state with byte-sized touched pressure codes.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    deserialize = "S: serde::Deserialize<'de> + Default, I: serde::Deserialize<'de>, W: serde::Deserialize<'de>"
))]
pub struct LegacyColonyState<
    S = story::legacy_v7::LegacyStoryteller,
    I = legacy_items_v16::ItemStackV16,
    W = legacy_items_v16::WorkshopV16,
> {
    /// Standing trees by tile.
    pub trees: BTreeMap<(u32, u32), Tree>,
    /// Resource nodes (boulders, flint outcrops) by tile (M2).
    #[serde(default)]
    pub nodes: BTreeMap<(u32, u32), ResourceNode>,
    /// The colony's material table (M2; built at colony worldgen).
    #[serde(default)]
    pub materials: ColonyMaterials,
    /// Finished workshops by id (M2).
    #[serde(default)]
    pub workshops: BTreeMap<WorkshopId, W>,
    /// Next workshop id (from 1).
    #[serde(default)]
    pub next_workshop_id: WorkshopId,
    /// Next bill id (from 1).
    #[serde(default)]
    pub next_bill_id: BillId,
    /// The future work tile of each workshop blueprint site, by site centre
    /// (M2 review m5: later blueprints may not cover it).
    #[serde(default)]
    pub site_work_tiles: BTreeMap<(u32, u32), (u32, u32)>,
    /// M3 (ADR 0022): the kind of every 1×1 space site (wall, door, floor,
    /// bedroll) by tile, while it is a blueprint; read by the nav map.
    #[serde(default)]
    pub space_sites: BTreeMap<(u32, u32), BlueprintKind>,
    /// M3 (ADR 0023): every finished bedroll by tile, with its material (a bed).
    #[serde(default)]
    pub beds: BTreeMap<(u32, u32), crate::material::MaterialId>,
    /// M3 (ADR 0023): the rooms by id (`RoomSystem`), each tile's room, the
    /// next fresh id, the player's role overrides, the rooms announced in the
    /// chronicle, and whether walls, doors or digging changed since the last
    /// detection.
    #[serde(default)]
    pub rooms: BTreeMap<crate::rooms::RoomId, crate::rooms::Room>,
    /// The room of each room tile.
    #[serde(default)]
    pub room_of_tile: BTreeMap<(u32, u32), crate::rooms::RoomId>,
    /// Next fresh room id (from 1).
    #[serde(default)]
    pub next_room_id: crate::rooms::RoomId,
    /// Roles the player set (`SetRoomRole`).
    #[serde(default)]
    pub room_overrides: BTreeMap<crate::rooms::RoomId, crate::rooms::RoomRole>,
    /// Rooms already logged as formed.
    #[serde(default)]
    pub rooms_announced: std::collections::BTreeSet<crate::rooms::RoomId>,
    /// A wall, door or dug tile changed since the last room detection.
    #[serde(default)]
    pub rooms_dirty: bool,
    /// Bumped whenever the rooms or their roles change (renderers).
    #[serde(default)]
    pub rooms_version: u64,
    /// M3 (ADR 0021): targets a person found unreachable, by `(person, tile)`,
    /// with the tick until which the job assignment skips them for that person.
    #[serde(default)]
    pub unreachable: BTreeMap<(AgentId, (u32, u32)), u64>,
    /// Bumped whenever a tree is added or removed (renderers rebuild on change).
    pub trees_version: u64,
    /// Ground stacks, one per tile.
    pub ground_items: BTreeMap<(u32, u32), I>,
    /// Zones by id.
    pub zones: BTreeMap<ZoneId, Zone>,
    /// The zone each zoned tile belongs to.
    pub zone_of_tile: BTreeMap<(u32, u32), ZoneId>,
    /// Next zone id to hand out (starts at 1).
    pub next_zone_id: ZoneId,
    /// Player marks by tile.
    pub designations: BTreeMap<(u32, u32), Designation>,
    /// Footprint tiles of blueprint sites and finished huts.
    pub occupied: BTreeMap<(u32, u32), Occupant>,
    /// Who holds each reserved thing.
    pub reservations: BTreeMap<ReserveKey, AgentId>,
    /// Every command applied, in order.
    pub command_log: Vec<CommandRecord>,
    /// Items created and consumed.
    pub item_ledger: ItemLedger,
    /// Bumped by any change to zones, designations, ground items, sites or trees.
    pub overlay_version: u64,
    /// Counters.
    pub tallies: ColonyTallies,
    /// M4 (ADR 0026): the settlement's technology (`TechAggregateSystem`).
    #[serde(default)]
    pub tech: LegacyTechAggregate,
    /// M4 (goal 3.3): open apprenticeship orders in id order.
    #[serde(default)]
    pub teach_orders: Vec<knowledge::TeachOrder>,
    /// Next apprenticeship order id (from 1).
    #[serde(default)]
    pub next_teach_id: u32,
    /// M5 (ADR 0027): the storyteller — planned, active and past pressures.
    #[serde(default)]
    pub story: S,
    /// M5 (ADR 0028): animals, hunt marks and zones, meat lots.
    #[serde(default)]
    pub wildlife: wildlife::Wildlife,
    /// M5 (ADR 0030): quarrels, fights and breakdowns so far.
    #[serde(default)]
    pub mood_tallies: people::MoodTallies,
    /// M5 (ADR 0030): person-pair relations by `(lower id, higher id)`; a missing pair stands at its kin base.
    #[serde(default)]
    pub bonds: BTreeMap<(AgentId, AgentId), people::Bond>,
    /// M5 (ADR 0029): uses of each workshop's knife (the flint blade on its shelf) so far.
    #[serde(default)]
    pub knife_wear: BTreeMap<WorkshopId, u32>,
    /// M5 (ADR 0029): the standing order "everyone in hides" is off (it is on by default).
    #[serde(default)]
    pub dress_off: bool,
    /// M5 (ADR 0028): one side's attack in a fight, by `(attacker, defender)`;
    /// resolved by `CombatSystem`.
    #[serde(default)]
    pub fights: BTreeMap<(wildlife::Fighter, wildlife::Fighter), wildlife::Fight>,
    /// M5.5 (ADR 0031, 0032): the two neighbours, their migrant waves and caravans.
    #[serde(default)]
    pub neighbours: neighbours::Neighbours,
    /// Measurement only (M1.5 goal 8.2.1): jobs the decision seam released to
    /// a need, counted by [`JobReleaseKey`]. No system reads it, and no
    /// fingerprint or summary includes it.
    pub release_counts: BTreeMap<JobReleaseKey, u64>,
}
impl<S, I, W> From<LegacyColonyState<S, I, W>> for ColonyState
where
    S: Into<story::Storyteller>,
    I: Into<ItemStack>,
    W: Into<Workshop>,
{
    fn from(old: LegacyColonyState<S, I, W>) -> Self {
        Self {
            buildings: Default::default(),
            trees: old.trees,
            nodes: old.nodes,
            materials: old.materials,
            workshops: old
                .workshops
                .into_iter()
                .map(|(id, shop)| (id, shop.into()))
                .collect(),
            next_workshop_id: old.next_workshop_id,
            next_bill_id: old.next_bill_id,
            site_work_tiles: old.site_work_tiles,
            space_sites: old.space_sites,
            beds: old.beds,
            rooms: old.rooms,
            room_of_tile: old.room_of_tile,
            next_room_id: old.next_room_id,
            room_overrides: old.room_overrides,
            rooms_announced: old.rooms_announced,
            rooms_dirty: old.rooms_dirty,
            rooms_version: old.rooms_version,
            unreachable: old.unreachable,
            trees_version: old.trees_version,
            ground_items: old
                .ground_items
                .into_iter()
                .map(|(tile, stack)| (tile, stack.into()))
                .collect(),
            zones: old.zones,
            zone_of_tile: old.zone_of_tile,
            next_zone_id: old.next_zone_id,
            designations: old.designations,
            occupied: old.occupied,
            reservations: old.reservations,
            command_log: old.command_log,
            item_ledger: old.item_ledger,
            overlay_version: old.overlay_version,
            tallies: old.tallies,
            tech: old.tech.into(),
            teach_orders: old.teach_orders,
            next_teach_id: old.next_teach_id,
            story: old.story.into(),
            wildlife: old.wildlife,
            mood_tallies: old.mood_tallies,
            bonds: old.bonds,
            knife_wear: old.knife_wear,
            dress_off: old.dress_off,
            fights: old.fights,
            neighbours: old.neighbours,
            release_counts: old.release_counts,
        }
    }
}

/// Published formats9–16 use the widened storyteller and the old physical leaves.
pub type LegacyColonyStateV16 = LegacyColonyState<story::Storyteller>;

/// Format17 retained the same colony field order with physical garment wear.
pub type LegacyColonyStateV17 = LegacyColonyState<story::Storyteller, ItemStack, Workshop>;

impl<I, W> TryFrom<ColonyState> for LegacyColonyState<story::Storyteller, I, W>
where
    I: TryFrom<ItemStack>,
    W: TryFrom<Workshop>,
{
    type Error = &'static str;
    fn try_from(old: ColonyState) -> Result<Self, Self::Error> {
        Ok(Self {
            trees: old.trees,
            nodes: old.nodes,
            materials: old.materials,
            workshops: old
                .workshops
                .into_iter()
                .map(|(key, value)| {
                    value
                        .try_into()
                        .map(|value| (key, value))
                        .map_err(|_| "physical item cannot be represented by legacy save")
                })
                .collect::<Result<_, _>>()?,
            next_workshop_id: old.next_workshop_id,
            next_bill_id: old.next_bill_id,
            site_work_tiles: old.site_work_tiles,
            space_sites: old.space_sites,
            beds: old.beds,
            rooms: old.rooms,
            room_of_tile: old.room_of_tile,
            next_room_id: old.next_room_id,
            room_overrides: old.room_overrides,
            rooms_announced: old.rooms_announced,
            rooms_dirty: old.rooms_dirty,
            rooms_version: old.rooms_version,
            unreachable: old.unreachable,
            trees_version: old.trees_version,
            ground_items: old
                .ground_items
                .into_iter()
                .map(|(key, value)| {
                    value
                        .try_into()
                        .map(|value| (key, value))
                        .map_err(|_| "physical item cannot be represented by legacy save")
                })
                .collect::<Result<_, _>>()?,
            zones: old.zones,
            zone_of_tile: old.zone_of_tile,
            next_zone_id: old.next_zone_id,
            designations: old.designations,
            occupied: old.occupied,
            reservations: old.reservations,
            command_log: old.command_log,
            item_ledger: old.item_ledger,
            overlay_version: old.overlay_version,
            tallies: old.tallies,
            tech: old.tech.into(),
            teach_orders: old.teach_orders,
            next_teach_id: old.next_teach_id,
            story: old.story,
            wildlife: old.wildlife,
            mood_tallies: old.mood_tallies,
            bonds: old.bonds,
            knife_wear: old.knife_wear,
            dress_off: old.dress_off,
            fights: old.fights,
            neighbours: old.neighbours,
            release_counts: old.release_counts,
        })
    }
}

/// Exact technology aggregate through format17, before era vectors.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LegacyTechAggregate {
    /// Node aggregates in historical id order.
    pub states: Vec<knowledge::TechState>,
    /// Historical combined progress.
    pub progress_permille: u16,
    /// Adult census.
    pub adults: u16,
    /// Renderer revision.
    pub version: u64,
}
impl From<LegacyTechAggregate> for knowledge::TechAggregate {
    fn from(old: LegacyTechAggregate) -> Self {
        Self {
            states: old.states,
            progress_permille: old.progress_permille,
            adults: old.adults,
            version: old.version,
            era_progress: Vec::new(),
            current_era: (0, 0),
        }
    }
}
impl From<knowledge::TechAggregate> for LegacyTechAggregate {
    fn from(current: knowledge::TechAggregate) -> Self {
        Self {
            states: current.states,
            progress_permille: current.progress_permille,
            adults: current.adults,
            version: current.version,
        }
    }
}

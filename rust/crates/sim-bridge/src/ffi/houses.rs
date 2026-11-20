//! HOUSE1 read-only house and household snapshots (`get_house_snapshot`,
//! `get_household_snapshot`). Rows in id order; empty in a baseline world.
use godot::prelude::*;
use sim_core::colony::household::HouseView;
use sim_engine::SimResources;

/// The derived house rows (house id order), from the shared sim index.
pub fn collect_houses(world: &hecs::World, res: &SimResources) -> Vec<HouseView> {
    sim_systems::runtime::housing::house_views(world, res)
}

/// One household row; ids are `HouseholdId` / `AgentId` / house (root site) id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseholdRow {
    /// Household id.
    pub id: u32,
    /// Head (oldest adult).
    pub head: u64,
    /// Members in AgentId order.
    pub members: Vec<u64>,
    /// Formation tick.
    pub formed_tick: u64,
    /// Parent household, if any.
    pub parent: Option<u32>,
    /// Owned houses in id order.
    pub owned_houses: Vec<u32>,
}

/// Living households in id order.
pub fn collect_households(res: &SimResources) -> Vec<HouseholdRow> {
    let state = &res.colony.buildings.housing;
    state
        .households
        .values()
        .map(|household| HouseholdRow {
            id: household.id.0,
            head: household.head,
            members: household.members.iter().copied().collect(),
            formed_tick: household.formed_tick,
            parent: household.parent.map(|parent| parent.0),
            owned_houses: state
                .owned_by(household.id)
                .into_iter()
                .map(|house| house.0)
                .collect(),
        })
        .collect()
}

fn ids(values: impl IntoIterator<Item = u64>) -> PackedInt64Array {
    let values: Vec<i64> = values
        .into_iter()
        .map(|v| i64::try_from(v).unwrap_or(-1))
        .collect();
    PackedInt64Array::from(values.as_slice())
}

fn cells(values: &[(u32, u32)]) -> PackedInt32Array {
    let flat: Vec<i32> = values
        .iter()
        .flat_map(|&(x, y)| [x, y])
        .map(|v| i32::try_from(v).unwrap_or(-1))
        .collect();
    PackedInt32Array::from(flat.as_slice())
}

/// `{tick, revision, rows}`; cells are flat `[x0, y0, x1, y1, …]`, nullable ints are -1.
pub(super) fn houses_to_dict(res: &SimResources, rows: &[HouseView]) -> VarDictionary {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let mut out = VarDictionary::new();
    out.set("tick", i64::try_from(res.current_tick).unwrap_or(-1));
    // Moves with households, ownership and bed assignments; with `geometry_revision`
    // (rooms) and the overlay version it is the cache key for this read.
    out.set(
        "revision",
        i64::try_from(res.colony.buildings.housing.revision).unwrap_or(-1),
    );
    let mut list: Array<VarDictionary> = Array::new();
    for view in rows {
        let mut row = VarDictionary::new();
        row.set("house_id", i64::from(view.id.0));
        row.set("root_site_id", i64::from(view.id.0));
        row.set(
            "geometry_revision",
            i64::try_from(res.colony.rooms_version).unwrap_or(-1),
        );
        row.set("settlement_id", view.settlement.map_or(-1, i64::from));
        row.set("template_key", view.template.as_str());
        row.set(
            "template_name_key",
            catalog
                .building_templates()
                .get(&view.template)
                .map_or("UI_UNKNOWN", |template| template.name_key.as_str()),
        );
        row.set("era_key", view.era.as_str());
        row.set("tier", i64::from(view.tier));
        row.set("yard_stage", i64::from(view.yard_stage));
        row.set("occupied_days", i64::from(view.occupied_days));
        row.set("status", view.status.name());
        row.set(
            "first_completed_tick",
            view.first_completed_tick
                .and_then(|tick| i64::try_from(tick).ok())
                .unwrap_or(-1),
        );
        row.set("last_extended_tick", -1i64);
        row.set("completed_cells", cells(&view.completed_cells));
        row.set("roof_cells", cells(&view.roof_cells));
        row.set(
            "room_ids",
            ids(view.room_ids.iter().map(|id| u64::from(*id))),
        );
        let beds: Vec<_> = view.bed_refs.iter().map(|bed| bed.tile).collect();
        row.set("bed_cells", cells(&beds));
        row.set(
            "bed_slots",
            ids(view.bed_refs.iter().map(|bed| u64::from(bed.slots))),
        );
        row.set(
            "bed_reachable",
            ids(view.bed_refs.iter().map(|bed| u64::from(bed.reachable))),
        );
        row.set("hearth_cells", cells(&view.hearth_refs));
        row.set("resident_ids", ids(view.resident_ids.iter().copied()));
        row.set("household_groups", ids(view.household_groups.iter().copied()));
        row.set(
            "primary_household",
            view.primary_household
                .and_then(|key| i64::try_from(key).ok())
                .unwrap_or(-1),
        );
        row.set(
            "resident_count",
            i64::try_from(view.resident_ids.len()).unwrap_or(-1),
        );
        row.set("capacity", i64::from(view.capacity));
        row.set(
            "owner_household",
            view.owner_household.map_or(-1, |owner| i64::from(owner.0)),
        );
        let members = view
            .owner_household
            .and_then(|owner| res.colony.buildings.housing.households.get(&owner))
            .map(|household| household.members.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        row.set("household_member_ids", ids(members));
        list.push(&row);
    }
    out.set("rows", list);
    out
}

/// `{tick, rows}` with `id, head, members, formed_tick, parent (-1), owned_houses`.
pub(super) fn households_to_dict(res: &SimResources, rows: &[HouseholdRow]) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("tick", i64::try_from(res.current_tick).unwrap_or(-1));
    let mut list: Array<VarDictionary> = Array::new();
    for household in rows {
        let mut row = VarDictionary::new();
        row.set("id", i64::from(household.id));
        row.set("head", i64::try_from(household.head).unwrap_or(-1));
        row.set("members", ids(household.members.iter().copied()));
        row.set(
            "formed_tick",
            i64::try_from(household.formed_tick).unwrap_or(-1),
        );
        row.set("parent", household.parent.map_or(-1, i64::from));
        row.set(
            "owned_houses",
            ids(household.owned_houses.iter().map(|id| u64::from(*id))),
        );
        list.push(&row);
    }
    out.set("rows", list);
    out
}

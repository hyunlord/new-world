//! M1 colony bridge: order commands and colony reads behind `WorldSimNode`.
//!
//! Commands (`cmd_*`) go through [`submit_rect`] / [`submit_tiles`] /
//! [`submit_blueprint`] to `sim_systems::commands::submit`, which applies the
//! order at once and logs it with the engine's next tick. Reads are pure
//! collectors: [`collect_trees`] (tile order `(x, y)`), [`collect_colony_overlay`]
//! (each column in its map's key order) and [`collect_agent_jobs`] (the agent
//! snapshot's row order). Versions ([`collect_colony_versions`]) let GDScript
//! rebuild a layer only when its data changed.
mod tree_conversion;

use godot::prelude::*;
use sim_core::colony::{BlueprintKind, Carry, Command, CommandResult, Job, RejectReason};
use sim_core::components::{Agent, Position};
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::{check_blueprint, submit};

/// The four rectangle commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectCommand {
    /// `DesignateChop`.
    DesignateChop,
    /// `CancelArea`.
    CancelArea,
    /// `AddStockpile`.
    AddStockpile,
    /// `RemoveZoneArea`.
    RemoveZoneArea,
    /// `DesignateQuarry` (M2).
    DesignateQuarry,
}

/// Clip a rectangle given in signed tile coordinates: a corner below 0 is
/// moved to 0; `None` when the whole rectangle lies left of or above the map.
pub fn clip_rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Option<(u32, u32, u32, u32)> {
    if x0.max(x1) < 0 || y0.max(y1) < 0 {
        return None;
    }
    let c = |v: i32| v.max(0) as u32;
    Some((c(x0), c(y0), c(x1), c(y1)))
}

/// Apply and log a rectangle command (the body of the `cmd_*` rectangle funcs).
pub fn submit_rect(
    engine: &mut SimEngine,
    kind: RectCommand,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> CommandResult {
    let Some((x0, y0, x1, y1)) = clip_rect(x0, y0, x1, y1) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    let cmd = match kind {
        RectCommand::DesignateChop => Command::DesignateChop { x0, y0, x1, y1 },
        RectCommand::CancelArea => Command::CancelArea { x0, y0, x1, y1 },
        RectCommand::AddStockpile => Command::AddStockpile { x0, y0, x1, y1 },
        RectCommand::RemoveZoneArea => Command::RemoveZoneArea { x0, y0, x1, y1 },
        RectCommand::DesignateQuarry => Command::DesignateQuarry { x0, y0, x1, y1 },
    };
    submit(engine, cmd)
}

/// Apply and log `DesignateChopTiles` for the paired columns (negative tiles dropped).
pub fn submit_tiles(engine: &mut SimEngine, xs: &[i32], ys: &[i32]) -> CommandResult {
    let tiles: Vec<(u32, u32)> = xs
        .iter()
        .zip(ys)
        .filter(|(x, y)| **x >= 0 && **y >= 0)
        .map(|(x, y)| (*x as u32, *y as u32))
        .collect();
    if tiles.is_empty() {
        return CommandResult::rejected(RejectReason::OutOfMap);
    }
    submit(engine, Command::DesignateChopTiles { tiles })
}

/// Apply and log `PlaceBlueprint` (`kind` by name, `"hut"`); an unknown name
/// or a negative corner is rejected without a log entry.
pub fn submit_blueprint(engine: &mut SimEngine, kind: &str, x: i32, y: i32) -> CommandResult {
    let Some(kind) = BlueprintKind::from_name(kind) else {
        return CommandResult::rejected(RejectReason::Nothing);
    };
    if x < 0 || y < 0 {
        return CommandResult::rejected(RejectReason::OutOfMap);
    }
    submit(
        engine,
        Command::PlaceBlueprint {
            kind,
            x: x as u32,
            y: y as u32,
        },
    )
}

/// Whether a blueprint named `kind` fits with its top-left at `(x, y)`.
pub fn can_place(resources: &SimResources, kind: &str, x: i32, y: i32) -> bool {
    BlueprintKind::from_name(kind)
        .filter(|_| x >= 0 && y >= 0)
        .is_some_and(|kind| check_blueprint(resources, kind, x as u32, y as u32).is_ok())
}

/// `{accepted, count, reason}`; `reason` is `""` when accepted, else a
/// `RejectReason` name (`out_of_map`, `water`, `blocked`, `nothing`, `no_colony`, `not_yet`).
pub(super) fn result_to_dict(result: CommandResult) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("accepted", result.accepted);
    dict.set("count", i64::from(result.count));
    dict.set("reason", result.reason.map_or("", |r| r.name()));
    dict
}

/// Change counters of the colony layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColonyVersions {
    /// `colony.trees_version`.
    pub trees: u64,
    /// `colony.overlay_version`.
    pub overlay: u64,
    /// `colony.rooms_version` (M3).
    pub rooms: u64,
}

/// The colony change counters (cheap; poll every frame).
pub fn collect_colony_versions(resources: &SimResources) -> ColonyVersions {
    ColonyVersions {
        trees: resources.colony.trees_version,
        overlay: resources.colony.overlay_version,
        rooms: resources.colony.rooms_version,
    }
}

pub(super) fn versions_to_dict(v: ColonyVersions) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("trees_version", v.trees as i64);
    dict.set("overlay_version", v.overlay as i64);
    dict.set("rooms_version", v.rooms as i64);
    dict
}

/// Every standing tree: `(version, tiles in (x, y) order)`.
pub fn collect_trees(resources: &SimResources) -> (u64, Vec<(u32, u32)>) {
    (
        resources.colony.trees_version,
        resources.colony.trees.keys().copied().collect(),
    )
}

/// Authoritative standing trees in stable tile order, with aligned saved material names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSnapshot {
    /// Tree change counter.
    pub version: u64,
    /// Standing tile coordinates in `(x, y)` order.
    pub tiles: Vec<(u32, u32)>,
    /// Canonical saved material names; empty exposes invalid state without guessing.
    pub species: Vec<String>,
}

/// Read persisted tree identities, never a current distribution or renderer hash.
pub fn collect_tree_snapshot(resources: &SimResources) -> TreeSnapshot {
    let mut tiles = Vec::with_capacity(resources.colony.trees.len());
    let mut species = Vec::with_capacity(resources.colony.trees.len());
    for &tile in resources.colony.trees.keys() {
        tiles.push(tile);
        species.push(
            resources
                .content_state
                .tree_material(tile)
                .map_or("", |material| resources.colony.materials.name(material))
                .to_owned(),
        );
    }
    TreeSnapshot {
        version: resources.colony.trees_version,
        tiles,
        species,
    }
}

pub(super) fn trees_to_dict(trees: TreeSnapshot) -> VarDictionary {
    tree_conversion::tree_snapshot_to_dict(trees)
}

/// One colony site as the overlay shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlaySite {
    /// Footprint top-left x.
    pub x: u32,
    /// Footprint top-left y.
    pub y: u32,
    /// Footprint width.
    pub w: u32,
    /// Footprint height.
    pub h: u32,
    /// `BlueprintKind::code` (0 = hut).
    pub kind: u32,
    /// Logs needed.
    pub needed: u16,
    /// Logs delivered.
    pub delivered: u16,
    /// Work done.
    pub progress: u32,
    /// Work needed.
    pub required: u32,
}

/// Everything the order overlay draws (goal 6.5), each list in its map's key order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColonyOverlay {
    /// `colony.overlay_version`.
    pub version: u64,
    /// Marked tiles (chop and quarry marks).
    pub designations: Vec<(u32, u32)>,
    /// The mark of each `designations` entry (0 chop, 1 quarry; M2; 2 dig, 3 deconstruct; M3).
    pub designation_kinds: Vec<u8>,
    /// Resource nodes: tile, `NodeKind::code`, items left (M2).
    pub nodes: Vec<((u32, u32), u8, u8)>,
    /// Quality of each ground stack (same order as `items`; M2).
    pub item_qualities: Vec<u8>,
    /// Zoned tiles with their zone id.
    pub zones: Vec<(u32, (u32, u32))>,
    /// Ground stacks: tile, `ItemKind::code`, count.
    pub items: Vec<((u32, u32), u32, u16)>,
    /// Blueprint sites, by centre tile.
    pub sites: Vec<OverlaySite>,
    /// M3: whether each ground stack (same order as `items`) lies under a
    /// storehouse roof ("covered").
    pub item_covered: Vec<bool>,
}

/// Collect the order overlay (designations, zones, ground items, sites).
pub fn collect_colony_overlay(world: &hecs::World, resources: &SimResources) -> ColonyOverlay {
    let colony = &resources.colony;
    let sites = sim_systems::runtime::jobs::items::colony_sites(world)
        .values()
        .map(|view| OverlaySite {
            x: view.site.top_left.0,
            y: view.site.top_left.1,
            w: view.site.size.0,
            h: view.site.size.1,
            kind: view.site.kind.code(),
            needed: view.needs.needed_total(),
            delivered: view.needs.delivered_total(),
            progress: view.progress,
            required: view.required,
        })
        .collect();
    ColonyOverlay {
        version: colony.overlay_version,
        designations: colony.designations.keys().copied().collect(),
        designation_kinds: colony.designations.values().map(|d| d.code()).collect(),
        nodes: colony
            .nodes
            .iter()
            .map(|(tile, node)| (*tile, node.kind.code(), node.left))
            .collect(),
        item_qualities: colony.ground_items.values().map(|s| s.quality).collect(),
        item_covered: colony
            .ground_items
            .keys()
            .map(|t| sim_systems::runtime::rooms::covered(resources, *t))
            .collect(),
        zones: colony
            .zone_of_tile
            .iter()
            .map(|(tile, id)| (*id, *tile))
            .collect(),
        items: colony
            .ground_items
            .iter()
            .map(|(tile, stack)| (*tile, stack.kind.code(), stack.count))
            .collect(),
        sites,
    }
}

fn ids<T: Copy>(rows: &[T], f: impl Fn(T) -> i64) -> PackedInt64Array {
    PackedInt64Array::from(rows.iter().map(|r| f(*r)).collect::<Vec<_>>().as_slice())
}

fn ints<T: Copy>(rows: &[T], f: impl Fn(T) -> i64) -> PackedInt32Array {
    let values: Vec<i32> = rows.iter().map(|r| f(*r) as i32).collect();
    PackedInt32Array::from(values.as_slice())
}

pub(super) fn overlay_to_dict(o: &ColonyOverlay) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("version", o.version as i64);
    dict.set("designation_xs", ints(&o.designations, |t| i64::from(t.0)));
    dict.set("designation_ys", ints(&o.designations, |t| i64::from(t.1)));
    dict.set("designation_kinds", ints(&o.designation_kinds, i64::from));
    dict.set("node_xs", ints(&o.nodes, |n| i64::from(n.0 .0)));
    dict.set("node_ys", ints(&o.nodes, |n| i64::from(n.0 .1)));
    dict.set("node_kinds", ints(&o.nodes, |n| i64::from(n.1)));
    dict.set("node_left", ints(&o.nodes, |n| i64::from(n.2)));
    dict.set("item_qualities", ints(&o.item_qualities, i64::from));
    dict.set("zone_ids", ints(&o.zones, |z| i64::from(z.0)));
    dict.set("zone_xs", ints(&o.zones, |z| i64::from(z.1 .0)));
    dict.set("zone_ys", ints(&o.zones, |z| i64::from(z.1 .1)));
    dict.set("item_xs", ints(&o.items, |i| i64::from(i.0 .0)));
    dict.set("item_ys", ints(&o.items, |i| i64::from(i.0 .1)));
    dict.set("item_kinds", ids(&o.items, |i| i64::from(i.1)));
    dict.set("item_counts", ints(&o.items, |i| i64::from(i.2)));
    dict.set("site_xs", ints(&o.sites, |s| i64::from(s.x)));
    dict.set("site_ys", ints(&o.sites, |s| i64::from(s.y)));
    dict.set("site_ws", ints(&o.sites, |s| i64::from(s.w)));
    dict.set("site_hs", ints(&o.sites, |s| i64::from(s.h)));
    dict.set("site_kinds", ids(&o.sites, |s| i64::from(s.kind)));
    dict.set("site_needed", ints(&o.sites, |s| i64::from(s.needed)));
    dict.set("site_delivered", ints(&o.sites, |s| i64::from(s.delivered)));
    dict.set("site_progress", ints(&o.sites, |s| i64::from(s.progress)));
    dict.set("site_required", ints(&o.sites, |s| i64::from(s.required)));
    dict.set("item_covered", ints(&o.item_covered, i64::from));
    dict
}

/// Per agent snapshot row (same `(Agent, Position)` query order as
/// `collect_agent_snapshot`): `(job code (0 none, JobKind::code), carry kind
/// (-1 none, ItemKind::code), carry count)`.
pub fn collect_agent_jobs(world: &hecs::World) -> Vec<(u8, i64, u16)> {
    world
        .query::<(&Agent, &Position, Option<&Job>, Option<&Carry>)>()
        .iter()
        .map(|(_, (_, _, job, carry))| {
            (
                job.map_or(0, |job| job.kind.code()),
                carry.map_or(-1, |c| i64::from(c.kind.code())),
                carry.map_or(0, |c| c.count),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangles_clip_negative_corners_and_reject_off_map() {
        assert_eq!(clip_rect(-3, 2, 4, -1), Some((0, 2, 4, 0)));
        assert_eq!(clip_rect(-3, 2, -1, 5), None);
        assert_eq!(clip_rect(1, -2, 3, -1), None);
    }
}

//! M3 bridge: built space and rooms behind `WorldSimNode` (ADR 0022, 0023).
//!
//! Commands: [`submit_space`] turns the `cmd_place_line`, `cmd_place_rect`,
//! `cmd_designate_dig`, `cmd_designate_deconstruct` and `cmd_set_room_role`
//! arguments into a `Command` and applies it through
//! `sim_systems::commands::submit` (logged with the next tick, ADR 0012).
//! Reads (pure; rows in a fixed order): [`collect_place_preview`] (the drag
//! ghost: each tile of a placement and why it would be refused, tile order of
//! the shape), [`collect_space`] (walls, doors, floors, bedrolls and dug tiles,
//! `(y, x)` order), [`collect_rooms`] (rooms by id, their tiles in `(x, y)`
//! order) and [`collect_room_detail`] (one room's panel). Ids: `RoomId` (u32).
use godot::prelude::*;
use sim_core::colony::{BlueprintKind, Command, CommandResult, RejectReason};
use sim_core::components::Position;
use sim_core::rooms::RoomRole;
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::{check_blueprint, check_space, line_tiles, submit};
use sim_systems::runtime::jobs::space::{built_at, Built};
use std::collections::BTreeSet;

/// One M3 order as the bridge receives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceCommand {
    /// `PlaceLine` of a space kind by name.
    Line {
        /// Blueprint name (`log_wall`, `stone_wall`, `door`, `packed_floor`, `bedroll`).
        kind: String,
        /// Corners.
        rect: (i32, i32, i32, i32),
    },
    /// `PlaceRect` of a space kind by name.
    Rect {
        /// Blueprint name.
        kind: String,
        /// Corners.
        rect: (i32, i32, i32, i32),
        /// Only the border.
        hollow: bool,
    },
    /// `DesignateDig` over the rectangle.
    Dig((i32, i32, i32, i32)),
    /// `DesignateDeconstruct` over the rectangle.
    Deconstruct((i32, i32, i32, i32)),
    /// `SetRoomRole` (`role` by name, `"auto"` = automatic).
    RoomRole {
        /// The room.
        room: i64,
        /// The role name.
        role: String,
    },
}

fn clip(rect: (i32, i32, i32, i32)) -> Option<(u32, u32, u32, u32)> {
    super::colony::clip_rect(rect.0, rect.1, rect.2, rect.3)
}

/// Apply and log one M3 order; an unknown name, a non-space kind or an off-map
/// rectangle is rejected without a log entry.
pub fn submit_space(engine: &mut SimEngine, cmd: SpaceCommand) -> CommandResult {
    let space_kind = |name: &str| BlueprintKind::from_name(name).filter(|k| k.is_space());
    let command = match cmd {
        SpaceCommand::Line { kind, rect } => {
            let (Some(kind), Some((x0, y0, x1, y1))) = (space_kind(&kind), clip(rect)) else {
                return CommandResult::rejected(RejectReason::Nothing);
            };
            Command::PlaceLine {
                kind,
                x0,
                y0,
                x1,
                y1,
            }
        }
        SpaceCommand::Rect { kind, rect, hollow } => {
            let (Some(kind), Some((x0, y0, x1, y1))) = (space_kind(&kind), clip(rect)) else {
                return CommandResult::rejected(RejectReason::Nothing);
            };
            Command::PlaceRect {
                kind,
                x0,
                y0,
                x1,
                y1,
                hollow,
            }
        }
        SpaceCommand::Dig(rect) => {
            let Some((x0, y0, x1, y1)) = clip(rect) else {
                return CommandResult::rejected(RejectReason::OutOfMap);
            };
            Command::DesignateDig { x0, y0, x1, y1 }
        }
        SpaceCommand::Deconstruct(rect) => {
            let Some((x0, y0, x1, y1)) = clip(rect) else {
                return CommandResult::rejected(RejectReason::OutOfMap);
            };
            Command::DesignateDeconstruct { x0, y0, x1, y1 }
        }
        SpaceCommand::RoomRole { room, role } => {
            let Ok(room) = u32::try_from(room) else {
                return CommandResult::rejected(RejectReason::Unknown);
            };
            let role = match role.as_str() {
                "auto" => None,
                name => match RoomRole::from_name(name) {
                    Some(r) => Some(r),
                    None => return CommandResult::rejected(RejectReason::Unknown),
                },
            };
            Command::SetRoomRole { room, role }
        }
    };
    submit(engine, command)
}

/// The drag ghost: every tile of a placement and its refusal (`""` = fits).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlacePreview {
    /// Tiles in the shape's order (a building: its top-left only).
    pub tiles: Vec<(u32, u32)>,
    /// `RejectReason::name` per tile, `""` when it fits.
    pub reasons: Vec<&'static str>,
}

/// The preview of placing `kind` (a blueprint name) over the drag `(x0, y0,
/// x1, y1)` (`(x0, y0)` where it started): `shape` 0 = one tile (or a building's top-left),
/// 1 = a line, 2 = a hollow rectangle, 3 = a filled rectangle. Space kinds are
/// checked tile by tile (`check_space`), buildings once (`check_blueprint`).
/// Read-only.
pub fn collect_place_preview(
    world: &hecs::World,
    res: &SimResources,
    kind: &str,
    drag: (i32, i32, i32, i32),
    shape: i64,
) -> PlacePreview {
    let Some(kind) = BlueprintKind::from_name(kind) else {
        return PlacePreview::default();
    };
    let Some((x0, y0, x1, y1)) = clip(drag) else {
        return PlacePreview::default();
    };
    if !kind.is_space() {
        let reason = check_blueprint(res, kind, x0, y0)
            .err()
            .map_or("", |r| r.name());
        return PlacePreview {
            tiles: vec![(x0, y0)],
            reasons: vec![reason],
        };
    }
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let (lx, hx) = (x0.min(x1), x0.max(x1).min(w.saturating_sub(1)));
    let (ly, hy) = (y0.min(y1), y0.max(y1).min(h.saturating_sub(1)));
    let tiles: Vec<(u32, u32)> = match shape {
        1 => line_tiles((x0, y0), (x1, y1)),
        2 | 3 => (ly..=hy)
            .flat_map(|y| (lx..=hx).map(move |x| (x, y)))
            .filter(|t| shape == 3 || t.0 == lx || t.0 == hx || t.1 == ly || t.1 == hy)
            .collect(),
        _ => vec![(x0, y0)],
    };
    let people: BTreeSet<(u32, u32)> = world
        .query::<&Position>()
        .iter()
        .map(|(_, p)| (p.x, p.y))
        .collect();
    let reasons = tiles
        .iter()
        .map(|t| {
            check_space(res, &people, kind, *t)
                .err()
                .map_or("", |r| r.name())
        })
        .collect();
    PlacePreview { tiles, reasons }
}

pub(super) fn preview_to_dict(p: &PlacePreview) -> VarDictionary {
    let xs: Vec<i32> = p.tiles.iter().map(|t| t.0 as i32).collect();
    let ys: Vec<i32> = p.tiles.iter().map(|t| t.1 as i32).collect();
    let ok: Vec<u8> = p.reasons.iter().map(|r| u8::from(r.is_empty())).collect();
    let reasons: Vec<GString> = p.reasons.iter().map(|r| GString::from(*r)).collect();
    let mut dict = VarDictionary::new();
    dict.set("xs", PackedInt32Array::from(xs.as_slice()));
    dict.set("ys", PackedInt32Array::from(ys.as_slice()));
    dict.set("ok", PackedByteArray::from(ok.as_slice()));
    dict.set("reasons", PackedStringArray::from(reasons.as_slice()));
    dict
}

/// The built space of the colony (`collect_space`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpaceView {
    /// `colony.overlay_version` (every built change bumps it).
    pub version: u64,
    /// Walls and doors: tile and kind (0 log wall, 1 stone wall, 2 door).
    pub walls: Vec<((u32, u32), u8)>,
    /// Floors: tile and kind (0 packed earth, 1 dug rock).
    pub floors: Vec<((u32, u32), u8)>,
    /// Bedrolls.
    pub beds: Vec<(u32, u32)>,
    /// Dug hill tiles.
    pub dug: Vec<(u32, u32)>,
}

/// Every wall, door, floor, bedroll and dug tile, `(y, x)` order (empty in the baseline).
pub fn collect_space(res: &SimResources) -> SpaceView {
    let mut view = SpaceView {
        version: res.colony.overlay_version,
        ..SpaceView::default()
    };
    if res.rules.colony.is_none() {
        return view;
    }
    let g = &res.tile_grid;
    for y in 0..g.height {
        for x in 0..g.width {
            let i = g.idx(x, y);
            let t = (x, y);
            if g.wall_material[i].is_some() {
                let code = match built_at(res, t) {
                    Some(Built::Door) => 2,
                    Some(Built::StoneWall) => 1,
                    _ => 0,
                };
                view.walls.push((t, code));
            }
            if g.is_dug(i) {
                view.dug.push(t);
                view.floors.push((t, 1));
            } else if g.floor_material[i].is_some() {
                view.floors.push((t, 0));
            }
        }
    }
    view.beds = res.colony.beds.keys().copied().collect();
    view.beds.sort_by_key(|t| (t.1, t.0));
    view
}

fn int_col(values: impl Iterator<Item = u32>) -> PackedInt32Array {
    let v: Vec<i32> = values.map(|x| x as i32).collect();
    PackedInt32Array::from(v.as_slice())
}

pub(super) fn space_to_dict(v: &SpaceView) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("version", v.version as i64);
    dict.set("wall_xs", int_col(v.walls.iter().map(|w| w.0 .0)));
    dict.set("wall_ys", int_col(v.walls.iter().map(|w| w.0 .1)));
    dict.set(
        "wall_kinds",
        int_col(v.walls.iter().map(|w| u32::from(w.1))),
    );
    dict.set("floor_xs", int_col(v.floors.iter().map(|f| f.0 .0)));
    dict.set("floor_ys", int_col(v.floors.iter().map(|f| f.0 .1)));
    dict.set(
        "floor_kinds",
        int_col(v.floors.iter().map(|f| u32::from(f.1))),
    );
    dict.set("bed_xs", int_col(v.beds.iter().map(|b| b.0)));
    dict.set("bed_ys", int_col(v.beds.iter().map(|b| b.1)));
    dict.set("dug_xs", int_col(v.dug.iter().map(|d| d.0)));
    dict.set("dug_ys", int_col(v.dug.iter().map(|d| d.1)));
    dict
}

/// One room row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomRow {
    /// `RoomId`.
    pub id: u32,
    /// Its role.
    pub role: RoomRole,
    /// Whether the player set the role.
    pub overridden: bool,
    /// Tile count.
    pub tiles: u32,
    /// Beds in it.
    pub beds: u32,
    /// Roofed.
    pub roofed: bool,
    /// Dug out of a hill.
    pub dug: bool,
    /// Doors on its border.
    pub doors: u32,
    /// The tile nearest the mean of its tiles (for the label), `(y, x)` tie.
    pub centre: (u32, u32),
}

/// The colony's rooms (`collect_rooms`): rows by id, and each room tile with its id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoomsView {
    /// `colony.rooms_version`.
    pub version: u64,
    /// Rooms in id order.
    pub rows: Vec<RoomRow>,
    /// Room tiles in `(x, y)` order with their room id.
    pub tiles: Vec<((u32, u32), u32)>,
}

/// Every room (empty in the baseline).
pub fn collect_rooms(res: &SimResources) -> RoomsView {
    let c = &res.colony;
    let rows = c
        .rooms
        .values()
        .map(|r| {
            let n = r.tiles.len().max(1) as u64;
            let (sx, sy) = r.tiles.iter().fold((0u64, 0u64), |(a, b), t| {
                (a + u64::from(t.0), b + u64::from(t.1))
            });
            let mean = ((sx / n) as u32, (sy / n) as u32);
            let centre = r
                .tiles
                .iter()
                .copied()
                .min_by_key(|t| (t.0.abs_diff(mean.0).max(t.1.abs_diff(mean.1)), t.1, t.0))
                .unwrap_or(mean);
            RoomRow {
                id: r.id,
                role: r.role,
                overridden: c.room_overrides.contains_key(&r.id),
                tiles: r.tiles.len() as u32,
                beds: sim_systems::runtime::rooms::beds_in(res, r),
                roofed: r.roofed,
                dug: r.dug,
                doors: r.doors.len() as u32,
                centre,
            }
        })
        .collect();
    RoomsView {
        version: c.rooms_version,
        rows,
        tiles: c.room_of_tile.iter().map(|(t, id)| (*t, *id)).collect(),
    }
}

pub(super) fn rooms_to_dict(v: &RoomsView) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("version", v.version as i64);
    dict.set("ids", int_col(v.rows.iter().map(|r| r.id)));
    dict.set(
        "roles",
        int_col(v.rows.iter().map(|r| u32::from(r.role.code()))),
    );
    let names: Vec<GString> = v
        .rows
        .iter()
        .map(|r| GString::from(r.role.name()))
        .collect();
    dict.set("role_names", PackedStringArray::from(names.as_slice()));
    let overridden: Vec<u8> = v.rows.iter().map(|r| u8::from(r.overridden)).collect();
    dict.set("overridden", PackedByteArray::from(overridden.as_slice()));
    dict.set("tiles", int_col(v.rows.iter().map(|r| r.tiles)));
    dict.set("beds", int_col(v.rows.iter().map(|r| r.beds)));
    let roofed: Vec<u8> = v.rows.iter().map(|r| u8::from(r.roofed)).collect();
    dict.set("roofed", PackedByteArray::from(roofed.as_slice()));
    let dug: Vec<u8> = v.rows.iter().map(|r| u8::from(r.dug)).collect();
    dict.set("dug", PackedByteArray::from(dug.as_slice()));
    dict.set("doors", int_col(v.rows.iter().map(|r| r.doors)));
    dict.set("centre_xs", int_col(v.rows.iter().map(|r| r.centre.0)));
    dict.set("centre_ys", int_col(v.rows.iter().map(|r| r.centre.1)));
    dict.set("tile_xs", int_col(v.tiles.iter().map(|t| t.0 .0)));
    dict.set("tile_ys", int_col(v.tiles.iter().map(|t| t.0 .1)));
    dict.set("tile_rooms", int_col(v.tiles.iter().map(|t| t.1)));
    dict
}

/// One room's panel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoomDetail {
    /// Whether the room exists.
    pub found: bool,
    /// The room row.
    pub row: Option<RoomRow>,
    /// Items lying in it (on its tiles), all kinds.
    pub items: u32,
    /// Of them, items under a storehouse roof ("covered").
    pub covered: u32,
    /// People asleep in it now.
    pub sleepers: u32,
}

/// One room's panel (`found: false` when there is no such room).
pub fn collect_room_detail(world: &hecs::World, res: &SimResources, room: i64) -> RoomDetail {
    let Some(r) = u32::try_from(room)
        .ok()
        .and_then(|id| res.colony.rooms.get(&id))
    else {
        return RoomDetail::default();
    };
    let row = collect_rooms(res)
        .rows
        .into_iter()
        .find(|row| row.id == r.id);
    let items: u32 = r
        .tiles
        .iter()
        .filter_map(|t| res.colony.ground_items.get(t))
        .map(|s| u32::from(s.count))
        .sum();
    let covered = if r.roofed && r.role == RoomRole::Storehouse {
        items
    } else {
        0
    };
    let sleepers = world
        .query::<(&Position, &sim_core::components::AgentState)>()
        .iter()
        .filter(|(_, (p, s))| {
            r.tiles.contains(&(p.x, p.y))
                && matches!(
                    s,
                    sim_core::components::AgentState::Consuming {
                        target: sim_core::components::TargetKind::Sleep
                    }
                )
        })
        .count() as u32;
    RoomDetail {
        found: true,
        row,
        items,
        covered,
        sleepers,
    }
}

pub(super) fn room_detail_to_dict(d: &RoomDetail) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("found", d.found);
    let row = d.row.clone();
    dict.set("id", row.as_ref().map_or(-1, |r| i64::from(r.id)));
    dict.set(
        "role",
        row.as_ref().map_or(-1, |r| i64::from(r.role.code())),
    );
    dict.set(
        "role_name",
        GString::from(row.as_ref().map_or("", |r| r.role.name())),
    );
    dict.set("overridden", row.as_ref().is_some_and(|r| r.overridden));
    dict.set("tiles", row.as_ref().map_or(0, |r| i64::from(r.tiles)));
    dict.set("beds", row.as_ref().map_or(0, |r| i64::from(r.beds)));
    dict.set("roofed", row.as_ref().is_some_and(|r| r.roofed));
    dict.set("dug", row.as_ref().is_some_and(|r| r.dug));
    dict.set("doors", row.as_ref().map_or(0, |r| i64::from(r.doors)));
    dict.set("items", i64::from(d.items));
    dict.set("covered", i64::from(d.covered));
    dict.set("sleepers", i64::from(d.sleepers));
    dict
}

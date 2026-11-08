//! Physical, derived livestock enclosure geometry; no persisted pen cache.
use sim_core::colony::building_site::{BuildingSiteId, BuildingStatus, Tile};
use sim_core::colony::content::{installed_or_bundled, CompletionCategory};
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::ReserveKey;
use sim_core::components::AgentId;
use sim_core::TerrainType;
use sim_engine::SimResources;
use std::collections::BTreeSet;

/// Current physical enclosure and natural grazing cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PenLayout {
    /// Persistent template site.
    pub site: BuildingSiteId,
    /// Authored hard animal capacity, independent of forage.
    pub capacity: u16,
    /// Grazeable cells in row-major order.
    pub interior: Vec<Tile>,
    /// Physically completed door cells in row-major order.
    pub gates: Vec<Tile>,
}

/// An enclosure's current physical admission failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenUnavailable {
    /// No template site exists.
    UnknownSite,
    /// The site is unfinished or cancelled.
    NotComplete,
    /// The template has no enabled livestock capability.
    NotLivestock,
    /// A physical constituent or perimeter segment is missing.
    BrokenBoundary,
    /// No physically completed gate exists.
    NoGate,
    /// No natural grazing cell remains.
    NoInterior,
    /// Footprint arithmetic or map bounds are invalid.
    OutOfBounds,
}

/// Derive an intact enclosure from the actual rotated template footprint.
pub fn pen_layout(res: &SimResources, site: BuildingSiteId) -> Result<PenLayout, PenUnavailable> {
    let saved = res
        .colony
        .buildings
        .sites
        .get(&site)
        .ok_or(PenUnavailable::UnknownSite)?;
    let template = installed_or_bundled()
        .building_templates()
        .get(&saved.template)
        .ok_or(PenUnavailable::NotLivestock)?;
    if template.status != ContentStatus::Connected || template.function.livestock_capacity == 0 {
        return Err(PenUnavailable::NotLivestock);
    }
    if saved.status != BuildingStatus::Complete {
        return Err(PenUnavailable::NotComplete);
    }
    if !crate::runtime::building_templates::is_intact(res, saved) {
        return Err(PenUnavailable::BrokenBoundary);
    }
    if saved.rotation > 3
        || template.rotated_footprint(saved.rotation).ok() != Some(saved.footprint)
    {
        return Err(PenUnavailable::OutOfBounds);
    }
    let (width, height) = saved.footprint;
    let right = saved
        .anchor
        .0
        .checked_add(width)
        .ok_or(PenUnavailable::OutOfBounds)?;
    let bottom = saved
        .anchor
        .1
        .checked_add(height)
        .ok_or(PenUnavailable::OutOfBounds)?;
    if width < 3 || height < 3 || right > res.tile_grid.width || bottom > res.tile_grid.height {
        return Err(PenUnavailable::OutOfBounds);
    }
    let mut boundary = BTreeSet::new();
    let mut gates = BTreeSet::new();
    for piece in &saved.pieces {
        let def = piece.kind.definition();
        // Fence uses the door projection for people, but remains an animal barrier.
        let door = def.completion == CompletionCategory::Door && !def.blocks_livestock;
        if !def.blocks_livestock && !door {
            continue;
        }
        let end_x = piece
            .tile
            .0
            .checked_add(piece.size.0)
            .ok_or(PenUnavailable::OutOfBounds)?;
        let end_y = piece
            .tile
            .1
            .checked_add(piece.size.1)
            .ok_or(PenUnavailable::OutOfBounds)?;
        if piece.tile.0 < saved.anchor.0
            || piece.tile.1 < saved.anchor.1
            || end_x > right
            || end_y > bottom
        {
            return Err(PenUnavailable::BrokenBoundary);
        }
        for y in piece.tile.1..end_y {
            for x in piece.tile.0..end_x {
                if x == saved.anchor.0 || y == saved.anchor.1 || x == right - 1 || y == bottom - 1 {
                    if !boundary.insert((x, y)) {
                        return Err(PenUnavailable::BrokenBoundary);
                    }
                    if door {
                        if !res.tile_grid.is_door(res.tile_grid.idx(x, y)) {
                            return Err(PenUnavailable::BrokenBoundary);
                        }
                        gates.insert((x, y));
                    }
                }
            }
        }
    }
    let mut interior = Vec::new();
    for y in saved.anchor.1..bottom {
        for x in saved.anchor.0..right {
            let edge =
                x == saved.anchor.0 || y == saved.anchor.1 || x == right - 1 || y == bottom - 1;
            if edge && !boundary.contains(&(x, y)) {
                return Err(PenUnavailable::BrokenBoundary);
            }
            if !edge && grazeable(res, (x, y)) {
                interior.push((x, y));
            }
        }
    }
    if gates.is_empty() {
        return Err(PenUnavailable::NoGate);
    }
    if interior.is_empty() {
        return Err(PenUnavailable::NoInterior);
    }
    let mut gates: Vec<_> = gates.into_iter().collect();
    gates.sort_by_key(|&(x, y)| (y, x));
    Ok(PenLayout {
        site,
        capacity: template.function.livestock_capacity,
        interior,
        gates,
    })
}

// Natural open/cleared forest/tundra ground, never floors, dug rock, storage or crops.
fn grazeable(res: &SimResources, tile: Tile) -> bool {
    let grid = &res.tile_grid;
    let index = grid.idx(tile.0, tile.1);
    matches!(
        grid.terrain_type[index],
        TerrainType::Plain | TerrainType::Forest | TerrainType::Tundra
    ) && grid.outdoor[index]
        && grid.wall_material[index].is_none()
        && grid.floor_material[index].is_none()
        && grid.ground_resource[index].is_none()
        && !grid.is_dug(index)
        && crate::runtime::nav::passable(res, tile)
        && !res.colony.trees.contains_key(&tile)
        && !res.colony.nodes.contains_key(&tile)
        && !crate::runtime::jobs::items::under_building(res, tile)
        && !res.colony.zone_of_tile.contains_key(&tile)
        && !crate::runtime::cultivation::occupies(res, tile)
}

/// Check hard capacity and the one canonical in-flight capture owner.
pub fn has_capture_slot(res: &SimResources, pen: &PenLayout, owner: Option<AgentId>) -> bool {
    let occupied = res
        .livestock
        .animals
        .values()
        .filter(|a| a.pen_site == pen.site)
        .count();
    occupied < usize::from(pen.capacity)
        && res
            .colony
            .reservations
            .get(&ReserveKey::PenCapture(pen.site))
            .is_none_or(|holder| Some(*holder) == owner)
}

/// Nearest reachable available gate, ordered by distance, site, then row-major tile.
pub fn nearest_pen_with_room(
    res: &SimResources,
    from: Tile,
    worker_region: u32,
) -> Option<(BuildingSiteId, Tile)> {
    res.colony
        .buildings
        .sites
        .keys()
        .filter_map(|&site| pen_layout(res, site).ok())
        .filter(|pen| has_capture_slot(res, pen, None))
        .flat_map(|pen| pen.gates.into_iter().map(move |gate| (pen.site, gate)))
        .filter(|(_, gate)| res.nav.passable(*gate) && res.nav.reaches(worker_region, *gate))
        .min_by_key(|&(site, gate)| {
            (
                crate::runtime::jobs::items::chebyshev(from, gate),
                site,
                gate.1,
                gate.0,
            )
        })
}

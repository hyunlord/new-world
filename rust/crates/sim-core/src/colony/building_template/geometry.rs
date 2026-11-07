//! Deterministic template selectors and exact material sums over existing blueprints.
use super::{BuildingTemplate, Direction, PieceAt};
use crate::colony::content::{BlueprintKind, Catalog, PlacementCategory};
use crate::colony::knowledge::ContentStatus;
use crate::colony::workshop::work_tile_of;
use std::collections::{BTreeMap, BTreeSet};

/// One physical plan entry; overlapping layers activate sequentially at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPiece {
    /// Resolved existing blueprint identity.
    pub kind: BlueprintKind,
    /// Top-left offset from the building anchor.
    pub offset: (u32, u32),
    /// Physical existing footprint (workshop orientation stays unchanged).
    pub size: (u32, u32),
}

impl BuildingTemplate {
    /// External dimensions after zero to three clockwise quarter-turns.
    pub fn rotated_footprint(&self, rotation: u8) -> Result<(u32, u32), String> {
        if self.footprint.0 == 0
            || self.footprint.1 == 0
            || self.footprint.0 > 64
            || self.footprint.1 > 64
        {
            return Err(format!("building {}: invalid footprint", self.id));
        }
        match rotation {
            0 | 2 => Ok(self.footprint),
            1 | 3 => Ok((self.footprint.1, self.footprint.0)),
            _ => Err(format!("building {}: rotation must be0..3", self.id)),
        }
    }

    /// Expand the whole plan on the command tick, preserving piece-file order.
    /// Floor/furnishing overlap describes separate pending layers, never duplicate live sites.
    pub fn expand(
        &self,
        catalog: &Catalog,
        station: Option<&str>,
        rotation: u8,
    ) -> Result<Vec<PlannedPiece>, String> {
        if self.status != ContentStatus::Connected {
            return Err(format!("building {}: template is inert", self.id));
        }
        let rotated = self.rotated_footprint(rotation)?;
        let mut plan = Vec::new();
        let mut seen = BTreeSet::new();
        let edge_overrides = self
            .pieces
            .iter()
            .filter_map(|piece| match piece.at {
                PieceAt::Edge(edge) => Some(edge_tile(self.footprint, edge)),
                PieceAt::Anchor | PieceAt::Ring | PieceAt::Interior => None,
            })
            .collect::<BTreeSet<_>>();
        for piece in &self.pieces {
            let kind = catalog
                .blueprint_by_name(&piece.kind)
                .ok_or_else(|| format!("building {}: missing blueprint {}", self.id, piece.kind))?;
            let definition = catalog
                .blueprint(kind)
                .ok_or_else(|| format!("building {}: unresolved blueprint", self.id))?;
            if !definition.placeable || piece.count == Some(0) {
                return Err(format!(
                    "building {}: unplaceable/zero-count piece {}",
                    self.id, piece.kind
                ));
            }
            let limit = usize::try_from(piece.count.unwrap_or(u32::MAX)).unwrap_or(usize::MAX);
            let mut offsets = Vec::new();
            match piece.at {
                PieceAt::Anchor => offsets.push((0, 0)),
                PieceAt::Edge(edge) => offsets.push(edge_tile(self.footprint, edge)),
                PieceAt::Ring | PieceAt::Interior => {
                    for y in 0..self.footprint.1 {
                        for x in 0..self.footprint.0 {
                            let border = x == 0
                                || y == 0
                                || x + 1 == self.footprint.0
                                || y + 1 == self.footprint.1;
                            let selected = match piece.at {
                                PieceAt::Ring => border && !edge_overrides.contains(&(x, y)),
                                PieceAt::Interior => !border,
                                PieceAt::Anchor | PieceAt::Edge(_) => false,
                            };
                            if selected {
                                offsets.push((x, y));
                            }
                        }
                    }
                }
            }
            if piece
                .count
                .is_some_and(|count| u64::from(count) > offsets.len() as u64)
            {
                return Err(format!(
                    "building {}: selected area has too few tiles",
                    self.id
                ));
            }
            for offset in offsets.into_iter().take(limit) {
                let offset = if piece.at == PieceAt::Anchor {
                    offset
                } else {
                    rotate(offset, self.footprint, rotation)
                };
                let size = definition.footprint;
                if offset.0.checked_add(size.0).is_none_or(|x| x > rotated.0)
                    || offset.1.checked_add(size.1).is_none_or(|y| y > rotated.1)
                {
                    return Err(format!("building {}: piece leaves footprint", self.id));
                }
                let work_tile =
                    (!definition.workshop_tags.is_empty()).then(|| work_tile_of(offset, size));
                if work_tile.is_some_and(|tile| tile.0 >= rotated.0 || tile.1 >= rotated.1) {
                    return Err(format!(
                        "building {}: station work tile leaves footprint",
                        self.id
                    ));
                }
                reserve_piece(&mut seen, offset, size, definition.placement, work_tile)
                    .map_err(|error| format!("building {}: {error}", self.id))?;
                plan.push(PlannedPiece { kind, offset, size });
            }
        }
        let selected_station = match (station, self.stations.first()) {
            (Some(name), _) if self.stations.iter().any(|candidate| candidate == name) => {
                Some(name)
            }
            (Some(_), _) => {
                return Err(format!(
                    "building {}: station is not an authored choice",
                    self.id
                ))
            }
            (None, first) => first.map(String::as_str),
        };
        if let Some(name) = selected_station {
            let kind = catalog
                .blueprint_by_name(name)
                .ok_or_else(|| format!("building {}: unknown station {name}", self.id))?;
            let definition = catalog
                .blueprint(kind)
                .ok_or_else(|| format!("building {}: unresolved station", self.id))?;
            if definition.workshop_tags.is_empty() || !definition.placeable {
                return Err(format!("building {}: invalid station {name}", self.id));
            }
            let size = definition.footprint;
            if size.0 >= rotated.0 || size.1 >= rotated.1 {
                return Err(format!("building {}: station exceeds room", self.id));
            }
            // Existing workshop commands have fixed orientation and a south work tile.
            // Re-fit inside the rotated room rather than inventing rotated workshop mechanics.
            let offset = (1..rotated.1.saturating_sub(1))
                .flat_map(|y| (1..rotated.0.saturating_sub(1)).map(move |x| (x, y)))
                .find(|&(x, y)| {
                    x + size.0 < rotated.0
                        && y + size.1 + 1 < rotated.1
                        && footprint_is_free(&seen, (x, y), size, definition.placement)
                        && nonfloor_is_free(&seen, work_tile_of((x, y), size))
                })
                .ok_or_else(|| {
                    format!(
                        "building {}: station footprint/work tile blocked or outside interior",
                        self.id
                    )
                })?;
            reserve_piece(
                &mut seen,
                offset,
                size,
                definition.placement,
                Some(work_tile_of(offset, size)),
            )
            .map_err(|error| format!("building {}: {error}", self.id))?;
            plan.push(PlannedPiece { kind, offset, size });
        }
        if plan.is_empty() {
            return Err(format!(
                "building {}: connected template needs pieces",
                self.id
            ));
        }
        Ok(plan)
    }
}

// Footprints are bounded by the validated <=64x64 template. Floors form the only
// stackable layer; every other physical piece and station work tile excludes all others.
const FLOOR: u8 = 0;
const NONFLOOR: u8 = 1;
const WORK: u8 = 2;

fn nonfloor_is_free(seen: &BTreeSet<((u32, u32), u8)>, tile: (u32, u32)) -> bool {
    !seen.contains(&(tile, NONFLOOR)) && !seen.contains(&(tile, WORK))
}

fn footprint_is_free(
    seen: &BTreeSet<((u32, u32), u8)>,
    offset: (u32, u32),
    size: (u32, u32),
    placement: PlacementCategory,
) -> bool {
    (offset.1..offset.1 + size.1).all(|y| {
        (offset.0..offset.0 + size.0).all(|x| {
            nonfloor_is_free(seen, (x, y))
                && (placement != PlacementCategory::Floor || !seen.contains(&((x, y), FLOOR)))
        })
    })
}

fn reserve_piece(
    seen: &mut BTreeSet<((u32, u32), u8)>,
    offset: (u32, u32),
    size: (u32, u32),
    placement: PlacementCategory,
    work_tile: Option<(u32, u32)>,
) -> Result<(), String> {
    let floor = placement == PlacementCategory::Floor;
    // Check the complete piece/work area before mutating; rejected plans never leave partial state.
    for y in offset.1..offset.1 + size.1 {
        for x in offset.0..offset.0 + size.0 {
            if !nonfloor_is_free(seen, (x, y)) {
                let why = if floor {
                    "floor must precede overlapping non-floor piece/work tile"
                } else {
                    "overlapping physical footprint/work tile"
                };
                return Err(format!("{why} at ({x}, {y})"));
            }
            if floor && seen.contains(&((x, y), FLOOR)) {
                return Err(format!("overlapping floor at ({x}, {y})"));
            }
        }
    }
    if let Some(tile) = work_tile {
        if !nonfloor_is_free(seen, tile) {
            return Err(format!(
                "station work tile overlaps physical footprint/work tile at ({}, {})",
                tile.0, tile.1
            ));
        }
    }
    for y in offset.1..offset.1 + size.1 {
        for x in offset.0..offset.0 + size.0 {
            seen.insert(((x, y), if floor { FLOOR } else { NONFLOOR }));
        }
    }
    if let Some(tile) = work_tile {
        seen.insert((tile, WORK));
    }
    Ok(())
}

fn edge_tile((width, height): (u32, u32), edge: Direction) -> (u32, u32) {
    match edge {
        Direction::North => (width / 2, 0),
        Direction::East => (width.saturating_sub(1), height / 2),
        Direction::South => (width / 2, height.saturating_sub(1)),
        Direction::West => (0, height / 2),
    }
}
fn rotate((x, y): (u32, u32), (width, height): (u32, u32), rotation: u8) -> (u32, u32) {
    match rotation {
        1 => (height - 1 - y, x),
        2 => (width - 1 - x, height - 1 - y),
        3 => (y, width - 1 - x),
        _ => (x, y),
    }
}

/// Exact authored material sum; runtime affordability uses the same expanded physical plans.
pub fn material_cost(
    plan: &[PlannedPiece],
    catalog: &Catalog,
) -> Result<BTreeMap<String, u32>, String> {
    let mut materials = BTreeMap::<String, u32>::new();
    for piece in plan {
        let definition = catalog
            .blueprint(piece.kind)
            .ok_or_else(|| "unresolved piece kind".to_string())?;
        for need in &definition.needs {
            let item = catalog
                .item(need.item)
                .ok_or_else(|| "unresolved piece item".to_string())?;
            let total = materials.entry(item.identity.id.clone()).or_default();
            *total = total
                .checked_add(u32::from(need.count))
                .ok_or_else(|| "material total overflow".to_string())?;
        }
    }
    Ok(materials)
}

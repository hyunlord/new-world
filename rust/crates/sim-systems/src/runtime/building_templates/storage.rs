//! Storage functions materialize through the existing stockpile pipeline at completion.
use sim_core::colony::building_site::BuildingSite;
use sim_core::colony::content::{installed_or_bundled, CompletionCategory};
use sim_core::colony::RejectReason;
use sim_engine::SimResources;

/// Add storage cells once, before the aggregate completion event is committed.
/// Existing stockpile cells retain their original zone identities.
pub fn complete(
    world: &hecs::World,
    res: &mut SimResources,
    site: &BuildingSite,
) -> Result<(), RejectReason> {
    let template = installed_or_bundled()
        .building_templates()
        .get(&site.template)
        .ok_or(RejectReason::Unknown)?;
    match template.function.storage.as_deref() {
        None => return Ok(()),
        Some("all" | "grain") => {}
        Some(_) => return Err(RejectReason::Unknown),
    }
    let right = site
        .anchor
        .0
        .checked_add(site.footprint.0)
        .ok_or(RejectReason::OutOfMap)?;
    let bottom = site
        .anchor
        .1
        .checked_add(site.footprint.1)
        .ok_or(RejectReason::OutOfMap)?;
    if site.footprint.0 < 3
        || site.footprint.1 < 3
        || right > res.tile_grid.width
        || bottom > res.tile_grid.height
    {
        return Err(RejectReason::OutOfMap);
    }
    let mut tiles = Vec::new();
    for y in site.anchor.1 + 1..bottom - 1 {
        for x in site.anchor.0 + 1..right - 1 {
            let occupied_piece = site.pieces.iter().any(|piece| {
                piece.kind.definition().completion != CompletionCategory::Floor
                    && x >= piece.tile.0
                    && y >= piece.tile.1
                    && x < piece.tile.0.saturating_add(piece.size.0)
                    && y < piece.tile.1.saturating_add(piece.size.1)
            });
            if !occupied_piece {
                tiles.push((x, y));
            }
        }
    }
    crate::commands::add_template_stockpile(res, &tiles)?;
    crate::runtime::rooms::assign_roles(world, res);
    Ok(())
}

/// Authored filter of a physical room associated with a completed template.
pub fn selector(res: &SimResources, tile: (u32, u32)) -> Option<&'static str> {
    let room = crate::runtime::rooms::room_at(res, tile)?;
    let template = installed_or_bundled()
        .building_templates()
        .get(room.dwelling_kind.as_deref()?)?;
    template.function.storage.as_deref()
}

/// A specialized stockpile accepts its nutritional group; ordinary cells accept all items.
pub fn accepts(res: &SimResources, tile: (u32, u32), kind: sim_core::colony::ItemKind) -> bool {
    match selector(res, tile) {
        None | Some("all") => true,
        Some(group) => kind
            .definition()
            .food
            .as_ref()
            .is_some_and(|food| food.groups.iter().any(|candidate| candidate == group)),
    }
}

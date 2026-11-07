//! Structural validation before restored building provenance can affect the runtime.
use sim_core::colony::building_site::BuildingState;
use std::collections::BTreeSet;

pub(super) fn validate(state: &BuildingState) -> Result<(), String> {
    let mut pieces = BTreeSet::new();
    for (&id, site) in &state.sites {
        if id != site.id
            || id >= state.next_site_id
            || site.rotation > 3
            || site.footprint.0 == 0
            || site.footprint.1 == 0
            || site.template.is_empty()
            || site.station.as_ref().is_some_and(String::is_empty)
        {
            return Err(format!("invalid building site identity/shape {id}"));
        }
        for piece in &site.pieces {
            if piece.id >= state.next_piece_id
                || !pieces.insert(piece.id)
                || piece.size.0 == 0
                || piece.size.1 == 0
            {
                return Err(format!(
                    "invalid building piece identity/shape {}",
                    piece.id
                ));
            }
        }
    }
    Ok(())
}

//! Template lifecycle atop existing jobs, placement validation, and completion identity.
mod association;
pub mod commands;
mod history_validation;
pub mod materials;
mod preview;
pub mod storage;
pub mod system;
pub mod upgrade;
mod upgrade_plan;
mod validation;
pub use association::associate_rooms;
use hecs::World;
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::building_site::{
    BuildCause, BuildingPiece, BuildingSite, BuildingStatus, PiecePhase,
};
use sim_core::colony::content::{installed_or_bundled, CompletionCategory};
use sim_core::colony::{ColonySite, Command, Occupant, RejectReason};
use sim_engine::SimResources;
pub use system::BuildingTemplateSystem;
pub use validation::{validate_state, validate_state_at_boundary};

/// Exact physical callback; historical huts do not have CompletedIdentity entries.
/// Call only at successful physical completion, before the entity is despawned.
pub fn observe_completion(res: &mut SimResources, completed: &ColonySite) {
    for site in res.colony.buildings.sites.values_mut() {
        if site.status != BuildingStatus::Building {
            continue;
        }
        for piece in &mut site.pieces {
            if piece.phase == PiecePhase::Active
                && piece.kind == completed.kind
                && piece.tile == completed.top_left
                && piece.size == completed.size
            {
                piece.phase = PiecePhase::Complete;
            }
        }
    }
}

/// Read the existing physical ledger; an absent ECS site is never a completion witness.
fn physically_complete(res: &SimResources, piece: &BuildingPiece) -> bool {
    let layer = crate::runtime::jobs::space::completion_layer(piece.kind);
    if crate::runtime::jobs::space::completed_at(res, piece.tile, layer) == Some(piece.kind) {
        return true;
    }
    match piece.kind.definition().completion {
        CompletionCategory::Workshop => res.colony.workshops.values().any(|workshop| {
            workshop.kind == piece.kind
                && workshop.top_left == piece.tile
                && workshop.size == piece.size
        }),
        // Historical huts have no authoritative identity. The exact callback marks
        // Complete first; then the existing hut occupancy confirms it still exists.
        CompletionCategory::Dwelling => {
            piece.phase == PiecePhase::Complete
                && (0..piece.size.1).all(|dy| {
                    (0..piece.size.0).all(|dx| {
                        res.colony
                            .occupied
                            .get(&(piece.tile.0 + dx, piece.tile.1 + dy))
                            == Some(&Occupant::Hut(piece.centre()))
                    })
                })
        }
        _ => false,
    }
}

/// One deterministic pass in stable site and authored piece order.
/// Register after physical completion systems and RoomSystem (priority 153).
pub fn advance(world: &mut World, res: &mut SimResources) {
    if res.rules.colony.is_none() || res.colony.buildings.sites.is_empty() {
        return;
    }
    let ids: Vec<_> = res.colony.buildings.sites.keys().copied().collect();
    for id in ids {
        let Some(mut site) = res.colony.buildings.sites.get(&id).cloned() else {
            continue;
        };
        if site.status == BuildingStatus::Complete {
            if site
                .pieces
                .iter()
                .any(|piece| !physically_complete(res, piece))
            {
                for piece in &mut site.pieces {
                    if !physically_complete(res, piece) {
                        piece.phase = PiecePhase::Cancelled;
                    }
                }
                site.status = BuildingStatus::Cancelled;
                res.colony.touch(false);
                res.colony.buildings.sites.insert(id, site);
            }
            continue;
        }
        if site.status != BuildingStatus::Building {
            continue;
        }
        let physical = crate::runtime::jobs::items::colony_sites(world);
        let mut lost = false;
        for piece in &mut site.pieces {
            if physically_complete(res, piece) {
                piece.phase = PiecePhase::Complete;
            } else if piece.phase == PiecePhase::Complete {
                // A completed constituent was removed while other work remained.
                piece.phase = PiecePhase::Cancelled;
                lost = true;
            } else if piece.phase == PiecePhase::Replacing
                && crate::runtime::jobs::space::completed_at(
                    res,
                    piece.tile,
                    sim_core::colony::content_state::CompletedLayer::Wall,
                )
                .is_some()
                && res.colony.designations.get(&piece.tile)
                    != Some(&sim_core::colony::Designation::Deconstruct)
            {
                lost = true;
            } else if piece.phase == PiecePhase::Active
                && !physical.get(&piece.centre()).is_some_and(|active| {
                    active.site.kind == piece.kind
                        && active.site.top_left == piece.tile
                        && active.site.size == piece.size
                })
            {
                piece.phase = PiecePhase::Cancelled;
                lost = true;
            }
        }
        if lost {
            // Refund/cancel remaining physical work and release pending commitments.
            commands::cancel_plan(world, res, &mut site);
            res.colony.buildings.sites.insert(id, site);
            continue;
        }
        for index in 0..site.pieces.len() {
            if site.pieces[index].phase == PiecePhase::Replacing {
                let piece = &site.pieces[index];
                let layer = crate::runtime::jobs::space::completion_layer(piece.kind);
                if crate::runtime::jobs::space::completed_at(res, piece.tile, layer).is_none() {
                    site.pieces[index].phase = PiecePhase::Pending;
                }
            }
            if site.pieces[index].phase != PiecePhase::Pending {
                continue;
            }
            let piece = &site.pieces[index];
            if site.pieces[..index]
                .iter()
                .any(|prior| prior.overlaps(piece) && prior.phase != PiecePhase::Complete)
            {
                continue;
            }
            let command = Command::PlaceBlueprint {
                kind: piece.kind,
                x: piece.tile.0,
                y: piece.tile.1,
            };
            let result = crate::commands::apply_template_piece(world, res, command);
            if result.accepted {
                site.pieces[index].phase = PiecePhase::Active;
            } else if result.reason == Some(RejectReason::Unreachable) {
                site.pieces[index].phase = PiecePhase::Cancelled;
                site.status = BuildingStatus::Cancelled;
                break;
            }
        }
        if site.status == BuildingStatus::Cancelled {
            commands::cancel_plan(world, res, &mut site);
        }
        if site
            .pieces
            .iter()
            .all(|piece| piece.phase == PiecePhase::Complete)
        {
            site.status = BuildingStatus::Complete;
            if site.completion_entry.is_none() {
                let mut entry =
                    ChronicleEntry::new(res.current_tick, ChronicleKind::BuildingComplete);
                entry.cause = match site.cause {
                    BuildCause::Command => 0,
                    BuildCause::Zone(_) => 1,
                    BuildCause::Steward(_) => 2,
                };
                // History pins the completed template ordinal, never the mutable current site.
                let Some(template_value) = installed_or_bundled()
                    .building_templates()
                    .buildings
                    .iter()
                    .position(|template| template.id == site.template)
                    .and_then(|index| i64::try_from(index).ok())
                else {
                    site.status = BuildingStatus::Cancelled;
                    res.colony.buildings.sites.insert(id, site);
                    continue;
                };
                if storage::complete(world, res, &site).is_err() {
                    commands::cancel_plan(world, res, &mut site);
                    res.colony.buildings.sites.insert(id, site);
                    continue;
                }
                entry.value = template_value;
                site.completion_entry = Some(res.chronicle.push_caused(
                    entry,
                    Some(CauseRef::Building {
                        site: id,
                        record: site.source_record,
                        origin: site.cause,
                    }),
                ));
            }
        }
        res.colony.buildings.sites.insert(id, site);
    }
    associate_rooms(res);
}

/// Whether a completed aggregate still has every physical piece, for pure world-art reads.
pub fn is_intact(res: &SimResources, site: &BuildingSite) -> bool {
    site.status == BuildingStatus::Complete
        && !site.pieces.is_empty()
        && site
            .pieces
            .iter()
            .all(|piece| piece.phase == PiecePhase::Complete && physically_complete(res, piece))
}

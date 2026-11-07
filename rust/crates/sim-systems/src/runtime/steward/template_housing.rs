//! Template adapters around the established house safety/clearance planner.
use super::*;
use sim_core::colony::building_template::{BuildingTemplate, PieceAt, PlannedPiece};
use sim_core::colony::content::{installed_or_bundled, CompletionCategory};

pub(super) fn covered(res: &SimResources) -> u32 {
    let room_beds = res
        .colony
        .beds
        .keys()
        .filter(|tile| {
            res.colony
                .room_of_tile
                .get(tile)
                .and_then(|id| res.colony.rooms.get(id))
                .is_some_and(|room| super::super::housing::constructed(res, room))
        })
        .count() as u32;
    let huts = res
        .building_kinds
        .values()
        .filter(|kind| **kind == sim_core::components::BuildingKind::Hut)
        .count() as u32;
    room_beds.saturating_add(huts.saturating_mul(res.rules.colony.map_or(0, |rule| rule.hut_beds)))
}

pub(super) fn pending(res: &SimResources) -> bool {
    res.colony
        .buildings
        .sites
        .values()
        .any(|site| site.status == sim_core::colony::building_site::BuildingStatus::Building)
}

pub(super) fn ring_action(template: &BuildingTemplate, original: &Action) -> Option<Action> {
    let catalog = installed_or_bundled();
    let wall = template
        .pieces
        .iter()
        .find(|piece| piece.at == PieceAt::Ring)
        .and_then(|piece| catalog.blueprint_by_name(&piece.kind))?;
    let door = template
        .pieces
        .iter()
        .find(|piece| matches!(piece.at, PieceAt::Edge(_)))
        .and_then(|piece| catalog.blueprint_by_name(&piece.kind))?;
    let bed = template
        .pieces
        .iter()
        .filter_map(|piece| catalog.blueprint_by_name(&piece.kind))
        .find(|kind| {
            catalog
                .blueprint(*kind)
                .is_some_and(|def| def.completion == CompletionCategory::Bed)
        })?;
    let Action::House {
        max_beds,
        completion_role,
        ..
    } = *original
    else {
        return None;
    };
    Some(Action::House {
        wall,
        door,
        bed,
        max_beds: max_beds.min(u32::from(template.function.beds)),
        service_cells: 0,
        completion_role,
    })
}

pub(super) fn rotations(template: &BuildingTemplate, anchor: Tile, size: Tile) -> Vec<(Tile, u8)> {
    let catalog = installed_or_bundled();
    (0..4)
        .filter_map(|rotation| {
            if template.rotated_footprint(rotation).ok()? != size {
                return None;
            }
            let plan = template.expand(catalog, None, rotation).ok()?;
            let door = plan
                .iter()
                .find(|piece| piece.kind.definition().completion == CompletionCategory::Door)?;
            Some((
                (anchor.0 + door.offset.0, anchor.1 + door.offset.1),
                rotation,
            ))
        })
        .collect()
}

pub(super) fn expanded(template: &BuildingTemplate, rotation: u8) -> Option<Vec<PlannedPiece>> {
    template.expand(installed_or_bundled(), None, rotation).ok()
}

pub(super) fn piece_tiles(plan: &[PlannedPiece], anchor: Tile) -> BTreeSet<Tile> {
    plan.iter()
        .flat_map(|piece| {
            (0..piece.size.1).flat_map(move |y| {
                (0..piece.size.0)
                    .map(move |x| (anchor.0 + piece.offset.0 + x, anchor.1 + piece.offset.1 + y))
            })
        })
        .collect()
}

pub(super) fn preview(
    res: &SimResources,
    people: &BTreeSet<Tile>,
    plan: &[PlannedPiece],
    anchor: Tile,
) -> bool {
    plan.iter().all(|piece| {
        let tile = (anchor.0 + piece.offset.0, anchor.1 + piece.offset.1);
        if piece.kind.is_space() {
            crate::commands::check_space_after_tree_clear(
                res,
                people,
                piece.kind,
                tile,
                res.colony.trees.contains_key(&tile),
            )
            .is_ok()
        } else {
            crate::commands::check_blueprint(res, piece.kind, tile.0, tile.1).is_ok()
        }
    })
}

// Single-Anchor dwellings retain the existing hut physical blueprint and sleep model.
// Ring templates use house_geometry below; no second room/nav model is introduced here.
pub(super) fn anchor_house(
    world: &World,
    res: &SimResources,
    hearth: Tile,
    template: &BuildingTemplate,
    reserve_only: bool,
) -> HousePlan {
    if pending(res)
        || !res.colony.space_sites.is_empty()
        || (!reserve_only && world.query::<&ColonySite>().iter().next().is_some())
        || covered(res) >= world.query::<&Agent>().iter().count() as u32
    {
        return HousePlan::default();
    }
    let Some(plan) = expanded(template, 0) else {
        return HousePlan::default();
    };
    if plan.len() != 1
        || !matches!(
            template.pieces.first().map(|piece| piece.at),
            Some(PieceAt::Anchor)
        )
    {
        return HousePlan::default();
    }
    let piece = &plan[0];
    let Some(rule) = res.rules.colony else {
        return HousePlan::default();
    };
    let size = piece.kind.plan(&rule).0;
    if size != template.footprint {
        return HousePlan::default();
    }
    let def = piece.kind.definition();
    if def.planned_nav != sim_core::colony::content::PlannedNav::Unchanged
        || !def.placeable
        || (def.requires_economy && rule.economy.is_none())
        || (def.requires_knowledge && rule.knowledge.is_none())
        || (def.requires_space && rule.space.is_none())
    {
        return HousePlan::default();
    }
    let Ok(costs) = template_selection::costs(res, &plan) else {
        return HousePlan::default();
    };
    let data = sim_core::colony::steward::rules().ok();
    let camp_radius = data
        .as_ref()
        .and_then(|data| {
            data.rules.iter().find_map(|rule| match rule.action {
                Action::Wood { camp_radius, .. } => Some(camp_radius),
                _ => None,
            })
        })
        .unwrap_or(0);
    let can_chop = crate::runtime::knowledge::anyone_practises(
        world,
        knowledge::gates().designation(sim_core::colony::Designation::Chop),
    );
    let people: BTreeSet<_> = world
        .query::<&Position>()
        .iter()
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();
    let work: BTreeSet<_> = res
        .colony
        .workshops
        .values()
        .map(|shop| shop.work_tile())
        .chain(res.colony.site_work_tiles.values().copied())
        .collect();
    let reserved: BTreeSet<_> = res
        .colony
        .reservations
        .keys()
        .map(|key| key.tile())
        .collect();
    for anchor in candidates(res, hearth) {
        let Some(right) = anchor.0.checked_add(size.0) else {
            continue;
        };
        let Some(bottom) = anchor.1.checked_add(size.1) else {
            continue;
        };
        if right > res.tile_grid.width
            || bottom > res.tile_grid.height
            || !template_selection::inside_housing(res, anchor, size)
        {
            continue;
        }
        let tiles: BTreeSet<_> = (anchor.1..bottom)
            .flat_map(|y| (anchor.0..right).map(move |x| (x, y)))
            .collect();
        // These are check_blueprint's unchanged footprint blockers, except trees,
        // which may pass only the exact established clearance policy below.
        if tiles.iter().any(|tile| {
            !permitted(res, *tile)
                || crate::runtime::cultivation::occupies(res, *tile)
                || !crate::runtime::nav::passable(res, *tile)
                || crate::runtime::jobs::items::is_water(res, *tile)
                || crate::runtime::jobs::items::has_wall(res, *tile)
                || crate::runtime::jobs::items::under_building(res, *tile)
                || res.colony.nodes.contains_key(tile)
                || res.colony.beds.contains_key(tile)
                || res.colony.zone_of_tile.contains_key(tile)
                || work.contains(tile)
                || reserved.contains(tile)
                || res.food.pots.values().any(|pot| pot.tile == *tile)
        }) {
            continue;
        }
        let clearing: Vec<_> = tiles
            .iter()
            .copied()
            .filter(|tile| res.colony.trees.contains_key(tile))
            .collect();
        if clearing.iter().any(|tile| {
            !can_chop
                || distance(*tile, hearth) <= camp_radius
                || res
                    .colony
                    .designations
                    .get(tile)
                    .is_some_and(|mark| *mark != sim_core::colony::Designation::Chop)
        }) {
            continue;
        }
        if let Some(data) = &data {
            let budget = data.rules.iter().find_map(|rule| {
                let Action::Wood {
                    items,
                    cap_log_margin: Some(margin),
                    ..
                } = &rule.action
                else {
                    return None;
                };
                if !res.steward.enabled
                    || !res.steward.scope.permits(rule.scope)
                    || (rule.extended && !res.steward.extended)
                {
                    return None;
                }
                Some((items, *margin))
            });
            if let Some((items, margin)) = budget {
                let (have, marked) = wood_supply(world, res, items, true);
                let added: u32 = clearing
                    .iter()
                    .filter(|tile| !res.colony.designations.contains_key(tile))
                    .filter_map(|tile| res.colony.trees.get(tile))
                    .map(|tree| u32::from(tree.logs))
                    .sum();
                let cost: u32 = costs
                    .iter()
                    .filter(|(item, _)| items.contains(item))
                    .map(|(_, count)| *count)
                    .sum();
                let target = building_log_need(world, items)
                    .max(cost)
                    .saturating_add(margin)
                    .saturating_add(fuel_input_need(world, res, items));
                if added > 0 && have.saturating_add(marked).saturating_add(added) > target {
                    continue;
                }
            }
        }
        // Hut planned_nav is unchanged, so a hut cannot introduce a new wall/roof
        // component. Keep all current access valid and require reachability to its cells.
        let nav = if res.nav.is_built() {
            res.nav.map().clone()
        } else {
            crate::runtime::nav::build(res)
        };
        let mut preview = sim_core::nav::NavState::default();
        preview.install(nav);
        if !preview.preview_reaches_all(hearth, tiles.iter().copied()) {
            continue;
        }
        let commands = if !clearing.is_empty() {
            let fresh: Vec<_> = clearing
                .into_iter()
                .filter(|tile| !res.colony.designations.contains_key(tile))
                .collect();
            if fresh.is_empty() {
                Vec::new()
            } else {
                vec![Command::DesignateChopTiles { tiles: fresh }]
            }
        } else if check_blueprint(res, piece.kind, anchor.0, anchor.1).is_err()
            || tiles.iter().any(|tile| people.contains(tile))
        {
            // Occupancy waits at the selected geometry; never redirects the home.
            Vec::new()
        } else {
            vec![Command::BuildTemplate {
                id: template.id.clone(),
                anchor,
                rotation: 0,
                station: None,
            }]
        };
        return HousePlan {
            commands,
            reserved: tiles.clone(),
            covered: tiles,
        };
    }
    HousePlan::default()
}

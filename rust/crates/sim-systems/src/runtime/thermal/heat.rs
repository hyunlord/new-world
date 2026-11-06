//! Integer room heat from authoritative geometry and actual paid receipts.
use super::*;
pub(super) fn remap(
    res: &SimResources,
    state: &ThermalState,
    outdoor: i32,
) -> BTreeMap<u32, RoomThermal> {
    res.colony
        .rooms
        .values()
        .filter(|room| room.roofed)
        .map(|room| {
            let sum: i64 = room
                .tiles
                .iter()
                .map(|tile| {
                    state
                        .rooms
                        .values()
                        .find(|old| old.tiles.contains(tile))
                        .map_or(i64::from(outdoor), |old| i64::from(old.temperature_d10))
                })
                .sum();
            let count = i64::try_from(room.tiles.len()).unwrap_or(1).max(1);
            (
                room.id,
                RoomThermal {
                    temperature_d10: i32::try_from(sum.div_euclid(count)).unwrap_or(outdoor),
                    tiles: room.tiles.clone(),
                    last_integrated_tick: state
                        .rooms
                        .get(&room.id)
                        .filter(|old| old.tiles == room.tiles)
                        .and_then(|old| old.last_integrated_tick),
                },
            )
        })
        .collect()
}

fn loss(res: &SimResources, state: &ThermalState, tiles: &std::collections::BTreeSet<Tile>) -> i64 {
    let rules = &state.rules;
    let mut total =
        i64::from(rules.roof_floor_loss_per_tile) * i64::try_from(tiles.len()).unwrap_or(0);
    for &(x, y) in tiles {
        for (dx, dy) in [(0_i64, -1_i64), (-1, 0), (1, 0), (0, 1)] {
            let (Ok(nx), Ok(ny)) = (
                u32::try_from(i64::from(x) + dx),
                u32::try_from(i64::from(y) + dy),
            ) else {
                continue;
            };
            let tile = (nx, ny);
            if tiles.contains(&tile) {
                continue;
            }
            if nx >= res.tile_grid.width || ny >= res.tile_grid.height {
                total += i64::from(rules.natural_boundary_loss);
                continue;
            }
            let index = res.tile_grid.idx(nx, ny);
            total += if res.tile_grid.is_door(index) {
                let open = state
                    .doors
                    .get(&tile)
                    .map_or(0, |door| door.open_ticks_in_hour.min(60));
                i64::from(rules.closed_door_loss)
                    + (i64::from(rules.open_door_loss - rules.closed_door_loss) * i64::from(open)
                        + 59)
                        / 60
            } else if let Some(material) = res.tile_grid.wall_material[index] {
                i64::from(
                    1000 - state
                        .rules
                        .material_insulation_permille
                        .get(&material)
                        .copied()
                        .unwrap_or(0),
                )
            } else {
                i64::from(rules.natural_boundary_loss)
            };
        }
    }
    total
}

pub(super) fn integrate(world: &World, res: &SimResources, state: &mut ThermalState, outdoor: i32) {
    let mut bodies: BTreeMap<u32, i64> = BTreeMap::new();
    for (_, (_, pos, child)) in world.query::<(&Agent, &Position, Option<&Child>)>().iter() {
        if let Some(room) = res.colony.room_of_tile.get(&(pos.x, pos.y)) {
            *bodies.entry(*room).or_default() += i64::from(if child.is_some() {
                state.rules.child_body_heat
            } else {
                state.rules.adult_body_heat
            });
        }
    }
    let mut temperatures = Vec::new();
    for (&id, room) in &state.rooms {
        let capacity =
            i64::from(state.rules.capacity_per_tile) * i64::try_from(room.tiles.len()).unwrap_or(0);
        let conductance = loss(res, state, &room.tiles);
        let heaters = state
            .paid_hearths
            .values()
            .filter(|paid| {
                paid.tick == res.current_tick
                    && paid.requested_wood > 0
                    && paid.debited_wood == paid.requested_wood
                    && room.tiles.contains(&paid.tile)
                    && res.building_kinds.get(&paid.tile)
                        == Some(&sim_core::components::BuildingKind::Hearth)
            })
            .map(|paid| paid.tile)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let heat = i64::from(state.rules.paid_hearth_heat) * i64::try_from(heaters).unwrap_or(0)
            + bodies.get(&id).copied().unwrap_or(0);
        let numerator = capacity
            .checked_mul(i64::from(room.temperature_d10))
            .and_then(|v| {
                conductance
                    .checked_mul(i64::from(outdoor))
                    .and_then(|o| v.checked_add(o))
            })
            .and_then(|v| v.checked_add(heat));
        if let Some(value) = numerator
            .and_then(|n| n.checked_div_euclid(capacity + conductance))
            .and_then(|n| i32::try_from(n).ok())
        {
            temperatures.push((id, value));
        }
    }
    for (id, value) in temperatures {
        if let Some(room) = state.rooms.get_mut(&id) {
            room.temperature_d10 = value;
            room.last_integrated_tick = Some(res.current_tick);
        }
    }
    for door in state.doors.values_mut() {
        door.open_ticks_in_hour = 0;
    }
    state.last_integrated_tick = Some(res.current_tick);
}

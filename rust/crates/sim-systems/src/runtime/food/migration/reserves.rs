//! Preserve finite legacy owned reserves without minting old supply-map counters.
use super::super::items::{create_ground, FoodSource};
use hecs::World;
use sim_core::colony::{content, ItemSpec};
use sim_core::components::{Agent, Inventory, Position, ResourceKind};
use sim_engine::SimResources;

type Tile = (u32, u32);

pub(super) fn convert(world: &mut World, res: &mut SimResources, day: u32) -> Result<(), String> {
    let kind = content::installed_or_bundled()
        .item_role("legacy_food")
        .ok_or("missing legacy_food migration role")?;
    let spec = ItemSpec::raw(kind, None);
    if kind.definition().food.as_ref().map(|food| food.units) != Some(1) {
        return Err("legacy_food must preserve exactly one unit per piece".into());
    }
    let mut stores: Vec<_> = res
        .settlements
        .iter()
        .map(|(id, settlement)| {
            (
                *id,
                settlement.formation_tile,
                settlement
                    .stockpile
                    .get(&ResourceKind::Food)
                    .copied()
                    .unwrap_or(0),
            )
        })
        .collect();
    stores.sort_by_key(|(id, _, _)| *id);
    for (id, tile, count) in stores {
        place(res, spec, (tile, count, day))?;
        if let Some(settlement) = res.settlements.get_mut(&id) {
            settlement.stockpile.remove(&ResourceKind::Food);
        }
    }
    let mut cargo = Vec::new();
    for (entity, (agent, inventory)) in world.query::<(&Agent, &Inventory)>().iter() {
        let count = inventory.get(ResourceKind::Food);
        if count == 0 {
            continue;
        }
        let position = world
            .get::<&Position>(entity)
            .map_err(|_| format!("legacy food owner {} lacks a position", agent.id))?;
        cargo.push((agent.id, entity, (position.x, position.y), count));
    }
    cargo.sort_by_key(|(id, _, _, _)| *id);
    for (_, entity, tile, count) in cargo {
        place(res, spec, (tile, count, day))?;
        world
            .get::<&mut Inventory>(entity)
            .map_err(|_| "legacy food owner lost its inventory")?
            .remove(ResourceKind::Food, count);
    }
    Ok(())
}

pub(super) fn place(
    res: &mut SimResources,
    spec: ItemSpec,
    source: (Tile, u32, u32),
) -> Result<(), String> {
    let (tile, mut count, day) = source;
    if count == u32::MAX {
        return Err("legacy owned food reserve uses an unsupported sentinel".into());
    }
    if tile.0 >= res.tile_grid.width || tile.1 >= res.tile_grid.height {
        return Err("legacy food owner is outside the map".into());
    }
    let capacity = spec.kind.default_stack_max();
    if capacity == 0 {
        return Err("legacy food stack capacity is zero".into());
    }
    while count > 0 {
        let amount = u16::try_from(count.min(u32::from(capacity)))
            .map_err(|_| "legacy food chunk exceeds item capacity")?;
        create_ground(res, tile, spec, amount, day, FoodSource::Migration)
            .map_err(|error| format!("legacy food placement: {error}"))?;
        count -= u32::from(amount);
    }
    Ok(())
}

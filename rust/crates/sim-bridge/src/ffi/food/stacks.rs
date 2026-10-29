//! Physical stack storage and age explanations for a selected tile.
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_engine::SimResources;

/// Real food stack and the exact storage class used by expiry mechanics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoodStackView {
    /// Item registry code.
    pub kind: u32,
    /// Physical pieces.
    pub count: u16,
    /// Stable mechanical storage label.
    pub storage: &'static str,
    /// Complete days remaining; none exposes missing or invalid age explicitly.
    pub remaining: Option<u32>,
}
/// Collect ground, workshop shelf and actual pot contents at one tile.
pub fn collect_tile_food(
    world: &hecs::World,
    res: &SimResources,
    tile: (u32, u32),
) -> Vec<FoodStackView> {
    if res.rules.colony.is_none() {
        return Vec::new();
    }
    let mut owners = vec![FoodOwner::Ground(tile)];
    for (&id, pot) in &res.food.pots {
        if pot.tile == tile {
            owners.extend(pot.contents.iter().map(|stack| FoodOwner::Pot {
                id,
                key: stack.spec().into(),
            }));
        }
    }
    for (&id, shop) in &res.colony.workshops {
        if shop.work_tile() == tile {
            owners.extend(res.food.ages.keys().copied().filter(
                |owner| matches!(owner, FoodOwner::Shelf { workshop, .. } if *workshop == id),
            ));
        }
    }
    let day = sim_systems::runtime::food::day(res);
    owners
        .into_iter()
        .filter_map(|owner| {
            let stack = sim_systems::runtime::food::items::stack(world, res, owner)?;
            stack.kind.definition().food.as_ref()?;
            let storage = match sim_systems::runtime::food::storage::class(res, owner) {
                FoodStorageClass::Ground => "ground",
                FoodStorageClass::Stockpile => "stockpile",
                FoodStorageClass::RoofedStore => "roofed_store",
                FoodStorageClass::Pot => "pot",
            };
            Some(FoodStackView {
                kind: stack.kind.code(),
                count: stack.count,
                storage,
                remaining: sim_systems::runtime::food::storage::remaining_days(
                    world, res, owner, day,
                )
                .ok(),
            })
        })
        .collect()
}

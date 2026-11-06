//! Stored edible food within the authored dwelling navigation distance.
use hecs::World;
use sim_core::colony::thermal::DwellingStageDef;
use sim_core::rooms::Tile;
use sim_engine::SimResources;
pub(super) fn food_near(
    world: &World,
    res: &SimResources,
    bed: Tile,
    rule: &DwellingStageDef,
    adults: u64,
) -> bool {
    use crate::runtime::food;
    use sim_core::colony::content::FoodStorageClass;
    use sim_core::colony::food::FoodOwner;
    if rule.nearby_food_adult_days == 0 {
        return true;
    }
    let daily = sim_core::colony::steward::rules().ok().and_then(|file| {
        file.rules.iter().find_map(|rule| match &rule.action {
            sim_core::colony::steward::Action::FoodPlan(plan) => Some(plan.adult_daily_milli),
            _ => None,
        })
    });
    let Some(daily) = daily else {
        return false;
    };
    let required = u64::from(daily) * adults.max(1) * u64::from(rule.nearby_food_adult_days);
    let mut units = 0_u64;
    for &owner in res.food.ages.keys() {
        let tile = match owner {
            FoodOwner::Ground(tile) => Some(tile),
            FoodOwner::Shelf { workshop, .. } => res
                .colony
                .workshops
                .get(&workshop)
                .map(|shop| shop.work_tile()),
            FoodOwner::Pot { id, .. } => res.food.pots.get(&id).map(|pot| pot.tile),
            FoodOwner::Carried(_) => None,
        };
        let Some(tile) = tile else {
            continue;
        };
        if food::storage::class(res, owner) == FoodStorageClass::Ground {
            continue;
        }
        if bed.0.abs_diff(tile.0).max(bed.1.abs_diff(tile.1)) > rule.food_nav_distance {
            continue;
        }
        let Some(path) = res.nav.preview_path(bed, tile) else {
            continue;
        };
        if path.len() > usize::try_from(rule.food_nav_distance).unwrap_or(0) {
            continue;
        }
        let Some(stack) = food::items::stack(world, res, owner) else {
            continue;
        };
        if !food::choice::edible(res, owner, stack.kind) {
            continue;
        }
        if let Some(food) = &stack.kind.definition().food {
            units += u64::from(stack.count) * u64::from(food.units);
        }
    }
    units * 1000 >= required
}

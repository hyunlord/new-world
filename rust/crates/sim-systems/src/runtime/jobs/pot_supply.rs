//! Physical pot ingredients available to ordinary, reserved Supply jobs.
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::{ItemKind, ItemStack, ReserveKey};
use sim_engine::SimResources;

/// Mirror existing transfer admission: the source must be fresh in Pot and
/// the unchanged birth day must also be fresh in Ground (the carrying class).
/// This does not grant a new lifetime when removing preserved food from a pot.
fn portable(res: &SimResources, owner: FoodOwner, stack: ItemStack) -> bool {
    let Some(food) = &stack.kind.definition().food else {
        return false;
    };
    let Some(age) = res.food.ages.get(&owner) else {
        return false;
    };
    let day = crate::runtime::food::day(res);
    stack.count > 0
        && age.born_day <= day
        && age.remaining(day, food.lifetime_days(FoodStorageClass::Pot)) > 0
        && age.remaining(day, food.lifetime_days(FoodStorageClass::Ground)) > 0
}

/// Stable contents ordering independent of insertion order and prior merges.
pub(super) fn in_pot(
    res: &SimResources,
    id: u32,
    kind: ItemKind,
) -> Option<(FoodOwner, ItemStack)> {
    res.food
        .pots
        .get(&id)?
        .contents
        .iter()
        .copied()
        .filter(|stack| stack.kind == kind)
        .map(|stack| {
            (
                FoodOwner::Pot {
                    id,
                    key: FoodStackKey::from(stack.spec()),
                },
                stack,
            )
        })
        .filter(|(owner, stack)| portable(res, *owner, *stack))
        .min_by_key(|(owner, _)| *owner)
}

/// Tile ownership stays compatible with existing Supply and Stack reservations.
/// Ground wins; otherwise pot id then stack key gives one deterministic source.
pub(super) fn source(
    res: &SimResources,
    tile: (u32, u32),
    kind: ItemKind,
) -> Option<(FoodOwner, ItemStack)> {
    if let Some(stack) = res
        .colony
        .ground_items
        .get(&tile)
        .copied()
        .filter(|stack| stack.kind == kind && stack.count > 0)
    {
        return Some((FoodOwner::Ground(tile), stack));
    }
    res.food
        .pots
        .iter()
        .filter(|(_, pot)| pot.tile == tile)
        .find_map(|(&id, _)| in_pot(res, id, kind))
}

/// Unreserved portable pieces, counted once by the bill scheduler.
pub(super) fn available(res: &SimResources, kind: ItemKind) -> u32 {
    res.food
        .pots
        .iter()
        .filter(|(_, pot)| {
            !res.colony
                .reservations
                .contains_key(&ReserveKey::Stack(pot.tile))
        })
        .flat_map(|(&id, pot)| pot.contents.iter().copied().map(move |stack| (id, stack)))
        .filter(|(_, stack)| stack.kind == kind)
        .filter(|(id, stack)| {
            portable(
                res,
                FoodOwner::Pot {
                    id: *id,
                    key: FoodStackKey::from(stack.spec()),
                },
                *stack,
            )
        })
        .map(|(_, stack)| u32::from(stack.count))
        .sum()
}

/// Positions are stable during a JobExecute pass; contents remain live reads.
type PotTiles = std::collections::BTreeMap<(u32, u32), Vec<u32>>;

#[derive(Default)]
pub(super) struct SourceIndex(std::cell::OnceCell<PotTiles>);

impl SourceIndex {
    pub(super) fn exists(&self, res: &SimResources, tile: (u32, u32), kind: ItemKind) -> bool {
        if res
            .colony
            .ground_items
            .get(&tile)
            .is_some_and(|stack| stack.kind == kind && stack.count > 0)
        {
            return true;
        }
        if kind.definition().food.is_none() {
            return false;
        }
        let tiles = self.0.get_or_init(|| {
            let mut tiles = std::collections::BTreeMap::<_, Vec<_>>::new();
            for (&id, pot) in &res.food.pots {
                tiles.entry(pot.tile).or_default().push(id);
            }
            tiles
        });
        tiles
            .get(&tile)
            .is_some_and(|ids| ids.iter().any(|&id| in_pot(res, id, kind).is_some()))
    }
}

//! Derived spatial food candidates; authored mind policy supplies all ranking.
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::ItemKind;
use sim_core::spatial::ChunkIndex;
use sim_engine::SimResources;
use std::collections::BTreeMap;

type Tile = (u32, u32);

/// Caller-owned index of accessible public food, grouped by immutable item data.
#[derive(Debug, Default)]
pub struct FoodChoiceIndex {
    generation: Option<(u64, u64, u32)>,
    kinds: BTreeMap<ItemKind, ChunkIndex<FoodOwner>>,
}
impl FoodChoiceIndex {
    /// Rebuild only after item/zone changes or the day boundary.
    pub fn refresh(&mut self, res: &SimResources) -> bool {
        let generation = (
            res.food.version,
            res.colony.overlay_version,
            super::day(res),
        );
        if self.generation == Some(generation) {
            return false;
        }
        self.kinds.clear();
        for (&tile, stack) in &res.colony.ground_items {
            if edible(res, FoodOwner::Ground(tile), stack.kind) {
                self.kinds
                    .entry(stack.kind)
                    .or_default()
                    .insert(tile, FoodOwner::Ground(tile));
            }
        }
        for (&id, pot) in &res.food.pots {
            for stack in &pot.contents {
                let owner = FoodOwner::Pot {
                    id,
                    key: FoodStackKey::from(stack.spec()),
                };
                if edible(res, owner, stack.kind) {
                    self.kinds
                        .entry(stack.kind)
                        .or_default()
                        .insert(pot.tile, owner);
                }
            }
        }
        self.generation = Some(generation);
        true
    }

    /// Same actionable food availability for mind policy and reservation admission.
    pub fn available_to(
        &self,
        res: &SimResources,
        origin: Tile,
        agent: sim_core::components::AgentId,
        carry: Option<sim_core::colony::Carry>,
    ) -> bool {
        if carry.is_some_and(|carry| {
            let owner = FoodOwner::Carried(agent);
            carry.count > 0
                && carry.kind.definition().food.as_ref().is_some_and(|food| {
                    food.raw_edible
                        && res.food.ages.get(&owner).is_some_and(|age| {
                            age.born_day <= super::day(res)
                                && age.remaining(
                                    super::day(res),
                                    food.lifetime_days(super::storage::class(res, owner)),
                                ) > 0
                        })
                })
        }) {
            return true;
        }
        self.kinds().any(|kind| {
            self.nearest(res, kind, origin, u32::MAX, |_, tile| {
                res.nav.reachable(origin, tile)
            })
            .is_some()
        })
    }

    /// Kinds represented by candidates, ordered by stable item identity.
    pub fn kinds(&self) -> impl Iterator<Item = ItemKind> + '_ {
        self.kinds.keys().copied()
    }

    /// Nearest eligible owner of one kind, using deterministic spatial tie-breaking.
    pub fn nearest(
        &self,
        res: &SimResources,
        kind: ItemKind,
        origin: Tile,
        radius: u32,
        eligible: impl Fn(FoodOwner, Tile) -> bool,
    ) -> Option<(Tile, FoodOwner)> {
        self.kinds
            .get(&kind)?
            .nearest_where(origin, radius, |tile, owner| {
                edible(res, *owner, kind) && eligible(*owner, tile)
            })
    }

    /// All fresh public owners on this tile; caller chooses by authored score.
    pub fn at<'a>(
        &'a self,
        res: &'a SimResources,
        tile: Tile,
    ) -> impl Iterator<Item = FoodOwner> + 'a {
        self.kinds.iter().flat_map(move |(kind, index)| {
            index
                .within_square(tile, 0)
                .filter_map(move |(_, owner)| edible(res, *owner, *kind).then_some(*owner))
        })
    }
}

/// Data eligibility and age check, independent of policy preference.
pub fn edible(res: &SimResources, owner: FoodOwner, kind: ItemKind) -> bool {
    kind.definition()
        .food
        .as_ref()
        .is_some_and(|food| food.raw_edible)
        && fresh(res, owner, kind)
}

/// Physical freshness independent of raw edibility, so grain remains tradable.
pub fn fresh(res: &SimResources, owner: FoodOwner, kind: ItemKind) -> bool {
    let Some(food) = &kind.definition().food else {
        return false;
    };
    let physical = match owner {
        FoodOwner::Ground(tile) => res.colony.ground_items.get(&tile),
        FoodOwner::Pot { id, key } => res.food.pots.get(&id).and_then(|pot| {
            pot.contents
                .iter()
                .find(|s| FoodStackKey::from(s.spec()) == key)
        }),
        FoodOwner::Carried(_) | FoodOwner::Shelf { .. } => None,
    };
    physical.is_some_and(|stack| stack.kind == kind && stack.count > 0)
        && res.food.ages.get(&owner).is_some_and(|age| {
            age.born_day <= super::day(res)
                && age.remaining(
                    super::day(res),
                    food.lifetime_days(super::storage::class(res, owner)),
                ) > 0
        })
}

/// Whether any public physical food owner can provide a fresh edible item now.
pub fn available(res: &SimResources) -> bool {
    res.colony
        .ground_items
        .iter()
        .any(|(&tile, stack)| edible(res, FoodOwner::Ground(tile), stack.kind))
        || res.food.pots.iter().any(|(&id, pot)| {
            pot.contents.iter().any(|stack| {
                edible(
                    res,
                    FoodOwner::Pot {
                        id,
                        key: FoodStackKey::from(stack.spec()),
                    },
                    stack.kind,
                )
            })
        })
}

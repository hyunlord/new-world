//! Choose existing food storage without mixing production dates when separate room exists.
use crate::runtime::food::storage;
use sim_core::colony::food::{FoodAge, FoodOwner, FoodStackKey};
use sim_core::colony::{ItemSpec, ReserveKey};
use sim_core::spatial::chebyshev;
use sim_engine::SimResources;

type Tile = (u32, u32);

/// Clean capacity uses only empty owners or the exact same production day.
pub(super) fn clean_owners(
    res: &SimResources,
    tile: Tile,
    spec: ItemSpec,
    born: u32,
) -> Vec<(FoodOwner, u16)> {
    owners(res, tile, spec, born, true)
}

fn owners(
    res: &SimResources,
    tile: Tile,
    spec: ItemSpec,
    born: u32,
    clean: bool,
) -> Vec<(FoodOwner, u16)> {
    let Some(food) = &spec.kind.definition().food else {
        return Vec::new();
    };
    let day = crate::runtime::food::day(res);
    let usable = |owner| {
        born <= day
            && FoodAge { born_day: born }
                .remaining(day, food.lifetime_days(storage::class(res, owner)))
                > 0
    };
    let same_age = |owner| {
        res.food.ages.get(&owner).is_some_and(|age| {
            age.born_day <= day
                && (!clean || age.born_day == born)
                && age.remaining(day, food.lifetime_days(storage::class(res, owner))) > 0
        })
    };
    let mut owners = Vec::new();
    for (&id, pot) in &res.food.pots {
        if pot.tile != tile {
            continue;
        }
        let owner = FoodOwner::Pot {
            id,
            key: FoodStackKey::from(spec),
        };
        let existing = pot
            .contents
            .iter()
            .find(|stack| FoodStackKey::from(stack.spec()) == FoodStackKey::from(spec));
        if !usable(owner)
            || existing.is_some_and(|stack| !stack.spec().merges_with(&spec) || !same_age(owner))
        {
            continue;
        }
        let used: u32 = pot
            .contents
            .iter()
            .map(|stack| u32::from(stack.count))
            .sum();
        let room = u32::from(pot.vessel.kind.definition().pot_capacity).saturating_sub(used);
        if let Ok(room) = u16::try_from(room) {
            if room > 0 {
                owners.push((owner, room));
            }
        }
    }
    let owner = FoodOwner::Ground(tile);
    if usable(owner)
        && res
            .colony
            .ground_items
            .get(&tile)
            .is_none_or(|stack| stack.spec().merges_with(&spec) && same_age(owner))
    {
        let room = super::items::room_on(res, tile, spec);
        if room > 0 {
            owners.push((owner, room));
        }
    }
    owners
}

fn clean_room(res: &SimResources, tile: Tile, spec: ItemSpec, born: u32) -> u16 {
    clean_owners(res, tile, spec, born)
        .iter()
        .fold(0u16, |sum, (_, room)| sum.saturating_add(*room))
}

fn choose(
    res: &SimResources,
    region: u32,
    from: Tile,
    covered_only: bool,
    room: impl Fn(Tile) -> u16,
) -> Option<(Tile, u16)> {
    let open = |tile| {
        room(tile) > 0
            && !res
                .colony
                .reservations
                .contains_key(&ReserveKey::Cell(tile))
            && res.nav.reaches(region, tile)
            && (!covered_only || crate::runtime::rooms::covered(res, tile))
    };
    let (nearest, ()) = res
        .spatial
        .colony
        .cells
        .nearest_where(from, u32::MAX, |tile, _| open(tile))?;
    let Some(bonus) = res
        .rules
        .colony
        .and_then(|rule| rule.space)
        .map(|space| space.storehouse_bonus_tiles)
    else {
        return Some((nearest, room(nearest)));
    };
    let to = if !covered_only
        && !crate::runtime::rooms::covered(res, nearest)
        && crate::runtime::rooms::storehouse_exists(res)
    {
        res.spatial
            .colony
            .cells
            .nearest_where(
                from,
                chebyshev(from, nearest).saturating_add(bonus),
                |tile, _| crate::runtime::rooms::covered(res, tile) && open(tile),
            )
            .map_or(nearest, |(tile, ())| tile)
    } else {
        nearest
    };
    Some((to, room(to)))
}

/// No retained absence memo: different ages of one item have different clean capacity.
pub(super) fn clean_destination(
    res: &SimResources,
    region: u32,
    from: Tile,
    spec: ItemSpec,
    born: u32,
    covered_only: bool,
) -> Option<(Tile, u16)> {
    spec.kind.definition().food.as_ref()?;
    choose(res, region, from, covered_only, |tile| {
        clean_room(res, tile, spec, born)
    })
}

/// Actual reachable, unreserved storage capacity; caller refreshes the colony spatial index.
/// Prefer age-preserving room, retaining ordinary mixed-age delivery when none exists.
pub fn destination(
    res: &SimResources,
    region: u32,
    from: Tile,
    spec: ItemSpec,
    born: u32,
    covered_only: bool,
) -> Option<(Tile, u16)> {
    clean_destination(res, region, from, spec, born, covered_only).or_else(|| {
        choose(res, region, from, covered_only, |tile| {
            if spec.kind.definition().food.is_some() {
                owners(res, tile, spec, born, false)
                    .iter()
                    .fold(0u16, |sum, (_, room)| sum.saturating_add(*room))
            } else {
                super::food_transfers::haul_room(res, tile, spec)
            }
        })
    })
}

/// Source dates come only from the physical owner; missing ages keep legacy admission.
pub(super) fn clean_from(
    res: &SimResources,
    region: u32,
    from: Tile,
    spec: ItemSpec,
    covered_only: bool,
) -> Option<(Tile, u16)> {
    let born = res.food.ages.get(&FoodOwner::Ground(from))?.born_day;
    clean_destination(res, region, from, spec, born, covered_only)
}

//! Proportional field-food attribution through physical transfers, recipes and meals.
use sim_core::colony::cultivation::OriginShare;
use sim_core::colony::food::FoodOwner;
use sim_engine::SimResources;

/// Missing entries are known nonfield food, including migrated pre-farming saves.
pub fn share(res: &SimResources, owner: FoodOwner) -> OriginShare {
    res.cultivation
        .food_origins
        .get(&owner)
        .copied()
        .unwrap_or_default()
}

/// Weighted fractions round outward, retaining uncertainty instead of inventing precision.
pub fn weighted(parts: impl IntoIterator<Item = (OriginShare, u64)>) -> OriginShare {
    let (mut lower, mut upper, mut weight) = (0_u128, 0_u128, 0_u128);
    for (part, count) in parts {
        let count = u128::from(count);
        lower += u128::from(part.lower) * count;
        upper += u128::from(part.upper) * count;
        weight += count;
    }
    if weight == 0 {
        return OriginShare::default();
    }
    OriginShare {
        lower: u32::try_from(lower / weight).unwrap_or(OriginShare::SCALE),
        upper: u32::try_from(upper.div_ceil(weight)).unwrap_or(OriginShare::SCALE),
    }
}

/// Merge after physical admission; existing_count refers to the destination before placement.
pub fn merge(
    res: &mut SimResources,
    owner: FoodOwner,
    existing_count: u16,
    added: OriginShare,
    count: u16,
) {
    let merged = weighted([
        (share(res, owner), u64::from(existing_count)),
        (added, u64::from(count)),
    ]);
    if merged.upper == 0 {
        res.cultivation.food_origins.remove(&owner);
    } else {
        res.cultivation.food_origins.insert(owner, merged);
    }
}

/// Only a completed meal contributes to intake; years use the world's actual calendar.
pub fn eaten(res: &mut SimResources, day: u32, units: u64, origin: OriginShare) {
    let year = u32::try_from(u64::from(day) / res.rules.calendar.days_per_year().max(1))
        .unwrap_or(u32::MAX);
    let tally = res.cultivation.eaten_by_year.entry(year).or_default();
    tally.units += units;
    tally.field_lower += units * u64::from(origin.lower);
    tally.field_upper += units * u64::from(origin.upper);
}

/// Keep the existing drying queue and its version-16 sidecar aligned, including old saves.
pub fn push_drying(
    res: &mut SimResources,
    batch: sim_core::colony::food::DryingBatch,
    origin: OriginShare,
) {
    res.cultivation
        .drying_origins
        .resize(res.food.pending_drying.len(), OriginShare::default());
    res.food.pending_drying.push(batch);
    res.cultivation.drying_origins.push(origin);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_recipe_rounds_outward_and_pure_inputs_stay_exact() {
        assert_eq!(weighted([(OriginShare::FIELD, 2)]), OriginShare::FIELD);
        assert_eq!(
            weighted([(OriginShare::default(), 2)]),
            OriginShare::default()
        );
        assert_eq!(
            weighted([(OriginShare::FIELD, 1), (OriginShare::default(), 2)]),
            OriginShare {
                lower: 333_333,
                upper: 333_334
            }
        );
        let mixed = weighted([(OriginShare::FIELD, 1), (OriginShare::default(), 2)]);
        assert_eq!(weighted([(mixed, 2), (mixed, 1)]), mixed);
    }
}

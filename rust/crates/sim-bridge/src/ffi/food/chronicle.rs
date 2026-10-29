//! Historical demand is projected only from the actual saved winter-plan evidence.
use crate::ffi::{chronicle::ChronicleRows, story::WhyRows};
use godot::prelude::*;
use sim_engine::SimResources;

fn demand(res: &SimResources, index: i64) -> i64 {
    u32::try_from(index)
        .ok()
        .and_then(|index| res.food.winter_plans.get(&index))
        .map_or(-1, |row| {
            i64::try_from(row.daily_demand_milli).unwrap_or(i64::MAX)
        })
}

pub(in crate::ffi) fn food_chronicle_rows_to_dict(
    rows: ChronicleRows,
    page: bool,
    res: &SimResources,
) -> VarDictionary {
    let values: Vec<i64> = rows.index.iter().map(|index| demand(res, *index)).collect();
    let mut result = if page {
        crate::ffi::chronicle::rows_to_dict(rows, true)
    } else {
        crate::ffi::chronicle::rows_to_dict(rows, false)
    };
    result.set(
        "food_daily_milli",
        PackedInt64Array::from(values.as_slice()),
    );
    result
}

pub(in crate::ffi) fn food_why_to_dict(rows: &WhyRows, res: &SimResources) -> VarDictionary {
    let values: Vec<i64> = rows
        .link
        .iter()
        .enumerate()
        .map(|(i, link)| {
            if *link == 0 {
                rows.code.get(i).map_or(-1, |index| demand(res, *index))
            } else {
                -1
            }
        })
        .collect();
    let mut result = crate::ffi::story::why_to_dict(rows);
    result.set(
        "food_daily_milli",
        PackedInt64Array::from(values.as_slice()),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{colony::food::WinterPlanEvidence, material::MaterialRegistry};

    #[test]
    fn historical_demand_uses_entry_identity_and_preserves_missing_and_zero() {
        let mut res = SimResources::new(8, 8, MaterialRegistry::new());
        res.food.winter_plans.insert(
            12,
            WinterPlanEvidence {
                tick: 77,
                daily_demand_milli: 2400,
                stored_units: 12,
                required_units: 15,
            },
        );
        res.food.winter_plans.insert(
            19,
            WinterPlanEvidence {
                tick: 90,
                daily_demand_milli: 0,
                stored_units: 0,
                required_units: 0,
            },
        );
        let indexes = [19, -1, 12, 1, 12];
        let observed: Vec<_> = indexes.iter().map(|index| demand(&res, *index)).collect();
        assert_eq!(observed, [0, -1, 2400, -1, 2400]);
        assert_eq!(observed.len(), indexes.len());
        res.current_tick = 100_000;
        assert_eq!(demand(&res, 12), 2400);
    }
}

//! Daily winter records use actual post-expiry stores at calendar boundaries.
use hecs::World;
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::steward::{rules, Action};
use sim_engine::SimResources;

/// Observe once after daily expiry; this never issues an order or changes policy.
pub fn observe(world: &World, res: &mut SimResources) {
    if res.rules.colony.is_none() {
        return;
    }
    let Ok(rules) = rules() else {
        return;
    };
    let Some(policy) = rules.rules.iter().find_map(|rule| match &rule.action {
        Action::FoodPlan(policy) => Some(policy),
        _ => None,
    }) else {
        return;
    };
    let day = crate::runtime::food::day(res);
    if res.food.firsts.last_plan_day == Some(day) {
        return;
    }
    res.food.firsts.last_plan_day = Some(day);
    let facts = super::summary(world, res, policy);
    let calendar = &res.rules.calendar;
    let year_days = calendar
        .days_per_season
        .saturating_mul(calendar.seasons_per_year);
    let Some(year) = day.checked_div(year_days) else {
        return;
    };
    let year_day = day % year_days;
    let winter_start = calendar
        .days_per_season
        .saturating_mul(policy.winter_season);
    let need = super::winter_need(facts.adults);
    if year_day == winter_start && facts.adults > 0 && facts.stored_units >= need {
        res.food.firsts.winter_started_with_stores.insert(year);
    }
    let settlement = res.settlements.keys().min().copied();
    // Winter-start evidence is retained even when automatic orders are disabled.
    if (facts.preparing && res.steward.enabled) || year_day == winter_start {
        let index = res.chronicle.push_caused(
            ChronicleEntry::new(res.current_tick, ChronicleKind::WinterFoodPlan)
                .settlement(settlement)
                .cause(u32::try_from(need).unwrap_or(u32::MAX))
                .value(i64::try_from(facts.stored_units).unwrap_or(i64::MAX)),
            None,
        );
        res.food.winter_plans.insert(
            index,
            sim_core::colony::food::WinterPlanEvidence {
                tick: res.current_tick,
                daily_demand_milli: facts.daily_milli,
                stored_units: facts.stored_units,
                required_units: need,
            },
        );
    }
    if year_day == 0
        && year > 0
        && facts.adults > 0
        && facts.stored_units > 0
        && res
            .food
            .firsts
            .winter_started_with_stores
            .contains(&(year - 1))
        && res.food.firsts.survived_winters.is_empty()
    {
        let start_day = u64::from(
            (year - 1)
                .saturating_mul(year_days)
                .saturating_add(winter_start),
        );
        let start = res
            .chronicle
            .entries()
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, entry)| {
                (entry.kind == ChronicleKind::WinterFoodPlan
                    && entry.settlement == settlement
                    && crate::runtime::resource_regen::day_index_at(res, entry.tick) == start_day
                    && entry.value >= i64::from(entry.cause))
                .then(|| u32::try_from(index).ok())
                .flatten()
            });
        let Some(start) = start else {
            return;
        };
        // The milestone proves a living colony and stores at both boundaries,
        // not that every person survived or that no hunger death occurred.
        res.food.firsts.survived_winters.insert(year - 1);
        res.chronicle.push_caused(
            ChronicleEntry::new(res.current_tick, ChronicleKind::WinterStoredSurvived)
                .settlement(settlement)
                .cause(year - 1)
                .value(i64::try_from(facts.stored_units).unwrap_or(i64::MAX)),
            Some(CauseRef::Entry(start)),
        );
    }
}

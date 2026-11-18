//! Settlement reputation (M-GROW 3.6): recomputed once per "month" (`periods_per_season` per season) at the
//! neighbours' day boundary from facts already in the sim; read-only except its own state and a chronicle line.
//!
//! Reads: colony food summary, covered beds, people, wildlife `bitten`, neighbour `trades`.
//! Writes: `ColonyState::reputation`; Events: `ReputationChanged`.
//! Feeds the seasonal migrant roll ([`roll_adjusted`]) and refuses waves below `refuse_below` ([`refuses`]).

use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::reputation::{self, CauseFact, ReputationData};
use sim_core::colony::steward::{rules, Action};
use sim_engine::SimResources;

/// Each cause's score in ‰ (-1000..=1000), in data order.
pub fn scores(world: &World, res: &SimResources, data: &ReputationData) -> Vec<i32> {
    let state = &res.colony.reputation;
    data.causes
        .iter()
        .map(|cause| match cause.fact {
            CauseFact::Food => food_score(world, res, data),
            CauseFact::Shelter => {
                let people = u64::from(super::neighbours::colony_people(world));
                if people == 0 {
                    0
                } else {
                    let beds = u64::from(super::steward::planning::covered_beds(res));
                    (beds * 2000 / people).min(2000) as i32 - 1000
                }
            }
            CauseFact::Safety => {
                let bites = res
                    .colony
                    .wildlife
                    .tallies
                    .bitten
                    .saturating_sub(state.last_bitten);
                1000 - (bites.saturating_mul(u64::from(data.bite_penalty_permille))).min(2000)
                    as i32
            }
            CauseFact::Trade => {
                let trades = res
                    .colony
                    .neighbours
                    .tallies
                    .trades
                    .saturating_sub(state.last_trades);
                trades
                    .saturating_mul(u64::from(data.trade_bonus_permille))
                    .min(1000) as i32
            }
        })
        .collect()
}

fn food_score(world: &World, res: &SimResources, data: &ReputationData) -> i32 {
    let Some(policy) = rules().ok().and_then(|registry| {
        registry.rules.iter().find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy),
            _ => None,
        })
    }) else {
        return 0;
    };
    let food = super::steward::food::summary(world, res, policy);
    if food.daily_milli == 0 {
        return 0;
    }
    let days_milli = food.stored_units.saturating_mul(1_000_000) / food.daily_milli;
    (days_milli.saturating_mul(2) / u64::from(data.food_good_days)).min(2000) as i32 - 1000
}

/// Contributions (score × weight / 1000) in data order, and the value base + their sum.
pub fn contributions(data: &ReputationData, scores: &[i32]) -> (Vec<i32>, u8) {
    let parts: Vec<i32> = data
        .causes
        .iter()
        .zip(scores)
        .map(|(cause, score)| score * i32::from(cause.weight) / 1000)
        .collect();
    // parse() guarantees base ± Σweight stays within 0..=100, so this never clamps.
    let value = (i32::from(data.base) + parts.iter().sum::<i32>()).clamp(0, 100) as u8;
    (parts, value)
}

/// Recompute when a new period began (called once per day by the neighbours system).
pub fn update(world: &World, res: &mut SimResources, day: u32) {
    let Some(data) = reputation::data() else {
        return;
    };
    let period_days = (res.rules.season_days() / data.periods_per_season).max(1);
    let period = day / period_days;
    if period == 0 || period <= res.colony.reputation.period {
        return;
    }
    let scores = scores(world, res, data);
    let (parts, value) = contributions(data, &scores);
    let state = &mut res.colony.reputation;
    state.period = period;
    state.value = Some(value);
    state.contributions = parts;
    state.last_bitten = res.colony.wildlife.tallies.bitten;
    state.last_trades = res.colony.neighbours.tallies.trades;
    let moved = state
        .chronicled
        .is_none_or(|last| last.abs_diff(value) >= data.chronicle_step);
    if !moved {
        return;
    }
    state.chronicled = Some(value);
    let mut cause = 0u32;
    for (slot, index) in state.ranked().into_iter().take(2).enumerate() {
        let part = state.contributions[index];
        if part == 0 {
            break;
        }
        let byte = ((index as u32 + 1) << 1) | u32::from(part < 0);
        cause |= byte << (8 * slot);
    }
    let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::ReputationChanged);
    entry.cause = cause;
    entry.value = i64::from(value);
    super::chronicle_log::record(world, res, entry);
}

/// A seasonal wave roll ‰ moved by the reputation (unchanged before the first value).
pub fn roll_adjusted(res: &SimResources, chance: u32, cap: u32) -> u32 {
    let (Some(data), Some(value)) = (reputation::data(), res.colony.reputation.value) else {
        return chance;
    };
    let delta = (i64::from(value) - i64::from(data.base)) * i64::from(data.roll_permille_per_point);
    (i64::from(chance) + delta).clamp(0, i64::from(cap)) as u32
}

/// Whether arriving migrant waves are refused now.
pub fn refuses(res: &SimResources) -> bool {
    match (reputation::data(), res.colony.reputation.value) {
        (Some(data), Some(value)) => value < data.refuse_below,
        _ => false,
    }
}

/// Whether the low-reputation card should be offered.
pub fn low(res: &SimResources) -> bool {
    match (reputation::data(), res.colony.reputation.value) {
        (Some(data), Some(value)) => value < data.card_below,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributions_sum_to_the_value_without_clamping() {
        let data = reputation::data().expect("reputation.ron");
        for scores in [
            [1000; 4],
            [-1000; 4],
            [1000, -1000, 0, 500],
            [-999, 1, -1, 999],
        ] {
            let (parts, value) = contributions(data, &scores);
            assert_eq!(
                i32::from(value),
                i32::from(data.base) + parts.iter().sum::<i32>()
            );
        }
    }

    #[test]
    fn reputation_moves_the_roll_and_refuses_when_low() {
        let data = reputation::data().expect("reputation.ron");
        let mut engine = sim_engine::SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
        let res = &mut engine.resources;
        assert_eq!(roll_adjusted(res, 300, 800), 300, "no value yet: unchanged");
        assert!(!refuses(res));
        res.colony.reputation.value = Some(data.base + 10);
        assert_eq!(
            roll_adjusted(res, 300, 800),
            300 + 10 * data.roll_permille_per_point
        );
        res.colony.reputation.value = Some(data.base - 10);
        assert_eq!(
            roll_adjusted(res, 300, 800),
            300 - 10 * data.roll_permille_per_point
        );
        res.colony.reputation.value = Some(100);
        assert_eq!(roll_adjusted(res, 790, 800), 800, "capped");
        res.colony.reputation.value = Some(data.refuse_below - 1);
        assert!(refuses(res) && low(res));
        res.colony.reputation.value = Some(data.refuse_below);
        assert!(!refuses(res));
    }
}

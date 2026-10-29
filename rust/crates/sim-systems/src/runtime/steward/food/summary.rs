//! Physical food summary shared by planning, alerts and food acceptance observers.
use hecs::World;
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::steward::FoodPlan;
use sim_core::colony::ItemKind;
use sim_core::components::{Agent, Child, Cold};
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

/// Fresh reachable food only; ingredient potential never inflates edible stores.
#[derive(Debug, Default)]
pub struct FoodSummary {
    /// Actual fresh edible units in storage.
    pub stored_units: u64,
    /// Actual fresh edible units in all reachable owners, including hands.
    pub usable_units: u64,
    /// Cooked units available now.
    pub cooked_units: u64,
    /// Stored edible units lasting through the authored winter keep horizon.
    pub winter_units: u64,
    /// Stored units expiring within two days.
    pub expiring_units: u64,
    /// Fresh reachable pieces by item, including inedible ingredients.
    pub pieces: BTreeMap<ItemKind, u64>,
    /// Nutritional groups currently supplied by fresh reachable items.
    pub groups: BTreeSet<String>,
    /// Living adult count.
    pub adults: u64,
    /// All-member daily demand in thousandths, including current cold.
    pub daily_milli: u64,
    /// Days until the current year's winter begins, zero during winter.
    pub winter_in: u32,
    /// Whether the authored winter preparation window is active.
    pub preparing: bool,
    /// Calendar days in winter.
    pub winter_days: u32,
}

/// Frozen owner bar: warm adults times six days times 12/5 units per day.
pub fn winter_need(adults: u64) -> u64 {
    adults.saturating_mul(72).div_ceil(5)
}

/// Build one food planning snapshot, never once per person or per tick.
pub fn summary(world: &World, res: &SimResources, policy: &FoodPlan) -> FoodSummary {
    let mut facts = FoodSummary::default();
    for (_, (_, child, cold)) in world
        .query::<(&Agent, Option<&Child>, Option<&Cold>)>()
        .iter()
    {
        let rate = if child.is_some() {
            policy.child_daily_milli
        } else {
            facts.adults += 1;
            policy.adult_daily_milli
        };
        let multiplier = if cold.is_some() {
            policy.cold_multiplier
        } else {
            1000
        };
        facts.daily_milli += u64::from(rate) * u64::from(multiplier) / 1000;
    }
    let day = crate::runtime::food::day(res);
    let year_days = res
        .rules
        .calendar
        .days_per_season
        .saturating_mul(res.rules.calendar.seasons_per_year);
    let year_day = day.checked_rem(year_days).unwrap_or(0);
    let winter_start = res
        .rules
        .calendar
        .days_per_season
        .saturating_mul(policy.winter_season);
    facts.winter_in = winter_start.saturating_sub(year_day);
    facts.winter_days = res.rules.calendar.days_per_season;
    facts.preparing = year_day >= policy.reserve_start_day && year_day < winter_start;
    let positions: BTreeMap<_, _> = world
        .query::<(&Agent, &sim_core::components::Position)>()
        .iter()
        .map(|(_, (agent, position))| (agent.id, (position.x, position.y)))
        .collect();
    for owner in res.food.ages.keys().copied() {
        let Some(stack) = crate::runtime::food::items::stack(world, res, owner) else {
            continue;
        };
        let Some(food) = &stack.kind.definition().food else {
            continue;
        };
        let Ok(remaining) = crate::runtime::food::storage::remaining_days(world, res, owner, day)
        else {
            continue;
        };
        if remaining == 0 || !reachable(&positions, res, owner) {
            continue;
        }
        *facts.pieces.entry(stack.kind).or_default() += u64::from(stack.count);
        facts.groups.extend(food.groups.iter().cloned());
        if !food.raw_edible {
            continue;
        }
        let units = u64::from(stack.count) * u64::from(food.units);
        facts.usable_units += units;
        if food.cooked {
            facts.cooked_units += units;
        }
        if crate::runtime::food::storage::class(res, owner) == FoodStorageClass::Ground {
            continue;
        }
        facts.stored_units += units;
        if remaining <= 2 {
            facts.expiring_units += units;
        }
        if covers_winter(remaining, facts.winter_in, policy.winter_keep_days) {
            facts.winter_units += units;
        }
    }
    facts
}

fn reachable(positions: &BTreeMap<u64, (u32, u32)>, res: &SimResources, owner: FoodOwner) -> bool {
    let tile = match owner {
        FoodOwner::Ground(tile) => Some(tile),
        FoodOwner::Shelf { workshop, .. } => res
            .colony
            .workshops
            .get(&workshop)
            .map(|shop| shop.work_tile()),
        FoodOwner::Pot { id, .. } => res.food.pots.get(&id).map(|pot| pot.tile),
        FoodOwner::Carried(id) => positions.get(&id).copied(),
    };
    tile.is_some_and(|tile| crate::commands::camp_reaches(res, tile))
}

/// Expiry on the exclusive winter end is sufficient; expired food never qualifies.
pub(super) const fn covers_winter(remaining: u32, winter_in: u32, keep_days: u32) -> bool {
    remaining > 0 && remaining >= winter_in.saturating_add(keep_days)
}

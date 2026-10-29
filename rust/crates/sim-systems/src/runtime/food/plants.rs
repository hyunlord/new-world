//! Calendar-boundary plant growth; harvesting never creates an ECS entity.
//! Reads: plant content, calendar, tile-indexed plants. Writes: ripe yields and food version.
use sim_core::colony::content::{installed_or_bundled, PlantDef, PlantPhase};
use sim_core::colony::food::PlantState;
use sim_core::rules::CalendarRule;
use sim_engine::SimResources;

/// Authored phase on a display day, including the within-season ripe window.
pub fn phase(def: &PlantDef, calendar: &CalendarRule, day: u32) -> PlantPhase {
    let (_, season, offset, _) = calendar.date_of_day(u64::from(day));
    let authored = def
        .phases
        .get(season as usize)
        .copied()
        .unwrap_or(PlantPhase::Dormant);
    if authored != PlantPhase::Ripe {
        return authored;
    }
    if offset < def.ripe_start_day {
        PlantPhase::Growing
    } else if def.ripe_end_day.is_some_and(|end| offset >= end) {
        PlantPhase::Spent
    } else {
        PlantPhase::Ripe
    }
}

/// First day of this continuous ripe window, bounded by the authored year.
fn window_start(def: &PlantDef, calendar: &CalendarRule, day: u32) -> u32 {
    let year_days = u32::try_from(calendar.days_per_year()).unwrap_or(u32::MAX);
    let mut first = day;
    for _ in 0..year_days {
        if first == 0 || phase(def, calendar, first - 1) != PlantPhase::Ripe {
            break;
        }
        first -= 1;
    }
    first
}

/// Advance one plant to a daily boundary without banking missed yields.
pub fn advance_plant(
    plant: &mut PlantState,
    def: &PlantDef,
    date: (&CalendarRule, u32, Option<u32>),
) -> bool {
    let (calendar, day, previous_day) = date;
    let before = *plant;
    if phase(def, calendar, day) != PlantPhase::Ripe {
        plant.units_left = 0;
        plant.next_regrow_day = 0;
    } else {
        let entering =
            previous_day.is_none_or(|previous| previous < window_start(def, calendar, day));
        if entering || (plant.units_left == 0 && day >= plant.next_regrow_day) {
            plant.units_left = def.yield_count;
            plant.next_regrow_day = u32::MAX;
        }
    }
    before != *plant
        || previous_day
            .is_some_and(|previous| phase(def, calendar, previous) != phase(def, calendar, day))
}

/// Update plants once per day; the enclosing food system owns the day stamp.
pub fn advance_day(res: &mut SimResources, day: u32, previous_day: Option<u32>) -> bool {
    if res.rules.colony.is_none() || previous_day == Some(day) {
        return false;
    }
    let catalog = installed_or_bundled();
    let delay = regrowth_delay(res, day, previous_day);
    let mut changed = false;
    for plant in res.food.plants.values_mut() {
        if let Some(def) = catalog.plant(plant.species) {
            if delay > 0
                && plant.units_left == 0
                && plant.next_regrow_day != u32::MAX
                && def.regrow_days > 0
                && def.yield_item.definition().food.is_some()
                && previous_day
                    .is_some_and(|previous| previous >= window_start(def, &res.rules.calendar, day))
                && phase(def, &res.rules.calendar, day) == PlantPhase::Ripe
            {
                let deadline = plant
                    .next_regrow_day
                    .saturating_add(delay)
                    .min(u32::MAX - 1);
                changed |= deadline != plant.next_regrow_day;
                plant.next_regrow_day = deadline;
            }
            changed |= advance_plant(plant, def, (&res.rules.calendar, day, previous_day));
        }
    }
    if changed {
        res.food.touch();
    }
    changed
}

/// Whole growth days withheld by actual active pressure; no fractional state or item yield.
fn regrowth_delay(res: &SimResources, day: u32, previous_day: Option<u32>) -> u32 {
    let Some(previous) = previous_day else {
        return 0;
    };
    let Some(drought) = crate::runtime::story::active_mechanic(
        res,
        sim_core::colony::content::PressureMechanic::Drought,
    ) else {
        return 0;
    };
    let Some(pressure) = res
        .colony
        .story
        .active
        .iter()
        .find(|p| p.kind == drought.kind)
    else {
        return 0;
    };
    // Story begins at its own daily tick, after the plant boundary. Count
    // completed intervals at start+1 through end, before Story removes it.
    let first = previous
        .saturating_add(1)
        .max(pressure.start_day.saturating_add(1));
    let last = day.min(pressure.end_day);
    if first > last || pressure.start_day >= pressure.end_day {
        return 0;
    }
    let elapsed_before = u64::from(first - pressure.start_day - 1);
    let elapsed_after = u64::from(last - pressure.start_day);
    let rate = u64::from(drought.rule.drought_regen_permille.min(1000));
    let earned = elapsed_after * rate / 1000 - elapsed_before * rate / 1000;
    u32::try_from(elapsed_after - elapsed_before - earned).unwrap_or(u32::MAX)
}

/// Present a depleted ripe plant as spent until its next actual regrowth.
pub fn phase_at(res: &SimResources, tile: (u32, u32), day: u32) -> Option<PlantPhase> {
    let plant = res.food.plants.get(&tile)?;
    let def = installed_or_bundled().plant(plant.species)?;
    let current = phase(def, &res.rules.calendar, day);
    Some(if current == PlantPhase::Ripe && plant.units_left == 0 {
        PlantPhase::Spent
    } else {
        current
    })
}

/// Whether a plant currently has a ripe yield, independently of designation.
pub fn harvestable(res: &SimResources, tile: (u32, u32)) -> bool {
    if res.rules.colony.is_none() {
        return false;
    }
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    phase_at(res, tile, day) == Some(PlantPhase::Ripe)
}

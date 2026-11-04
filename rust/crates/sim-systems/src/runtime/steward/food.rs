//! Demand-limited food orders and observable physical winter stores.
mod cooking;
mod dependencies;
mod harvest;
mod observe;
mod preservation;
mod storage;
mod summary;
use hecs::World;
pub use observe::observe;
use sim_core::colony::steward::FoodPlan;
use sim_core::colony::Command;
use sim_engine::SimResources;
pub use summary::{summary, winter_need, FoodSummary};

/// Plan normal replayable commands from one current food snapshot.
pub fn plan(world: &World, res: &SimResources, policy: &FoodPlan) -> Vec<Command> {
    let facts = summary(world, res, policy);
    let mut commands = field_orders(world, res, policy, &facts);
    commands.extend(storage::plan(res, policy, &facts));
    let roofed_store = res.colony.rooms.values().any(|room| {
        room.role == sim_core::rooms::RoomRole::Storehouse && super::housing::constructed(res, room)
    });
    if roofed_store {
        commands.extend(harvest::plan(world, res, policy, &facts));
    } else {
        // Keep immediate meals available while the real storage enclosure is built.
        let mut immediate = policy.clone();
        immediate.reserve_days = 1;
        let mut immediate_facts = summary(world, res, &immediate);
        immediate_facts.preparing = false;
        commands.extend(harvest::plan(world, res, &immediate, &immediate_facts));
    }
    commands.extend(cooking::plan(world, res, policy, &facts));
    let reserved_pots = dependencies::reserved_cooking_pots(res, policy);
    if let Some((&tile, _)) = res.colony.ground_items.iter().find(|(tile, stack)| {
        stack.kind.definition().pot_capacity > 0
            && res.colony.zone_of_tile.contains_key(tile)
            && !reserved_pots.contains(*tile)
            && !res.food.pots.values().any(|pot| pot.tile == **tile)
            && !res.steward.vetoed(
                &sim_core::colony::steward::Target::Tile(tile.0, tile.1),
                res.current_tick,
            )
    }) {
        commands.push(Command::DeployFoodPot { tile });
    }
    commands
}

/// Effective harvest snapshot and (immediate, durable, credited immediate, pending durable) units.
/// Immediate credit includes pending output and executable net cooking conversion.
pub fn budget_observation(
    world: &World,
    res: &SimResources,
) -> Option<(FoodSummary, (u64, u64, u64, u64))> {
    let data = sim_core::colony::steward::rules().ok()?;
    let mut policy = data.rules.iter().find_map(|rule| match &rule.action {
        sim_core::colony::steward::Action::FoodPlan(policy) => Some(policy.clone()),
        _ => None,
    })?;
    let roofed = res.colony.rooms.values().any(|room| {
        room.role == sim_core::rooms::RoomRole::Storehouse && super::housing::constructed(res, room)
    });
    if !roofed {
        policy.reserve_days = 1;
    }
    let mut facts = summary(world, res, &policy);
    if !roofed {
        facts.preparing = false;
    }
    let (immediate, durable) = harvest::targets(&policy, &facts);
    let (pending_immediate, pending_durable) = harvest::pending_output(world, res, &policy, &facts);
    let pending_immediate = pending_immediate
        .saturating_add(cooking::convertible_immediate(world, res, &policy, &facts));
    Some((
        facts,
        (immediate, durable, pending_immediate, pending_durable),
    ))
}

fn seed_reserve(
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> Option<(sim_core::colony::ItemKind, u64)> {
    let plan = policy.cultivation.as_ref()?;
    let crop = sim_core::colony::content::installed_or_bundled().crop(plan.crop)?;
    let year_days = res
        .rules
        .calendar
        .days_per_season
        .saturating_mul(res.rules.calendar.seasons_per_year);
    let (target, _) = cultivation_area(plan, annual_food_milli(res, policy, facts, year_days));
    let accepted = res
        .cultivation
        .fields
        .values()
        .filter(|field| field.crop == plan.crop && field.active)
        .count() as u64;
    Some((
        crop.seed_item,
        target
            .max(accepted)
            .saturating_mul(u64::from(crop.seeds_per_tile)),
    ))
}

fn field_orders(
    world: &World,
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
) -> Vec<Command> {
    // Pending wall/bed pieces do not describe their eventual roof interior.
    if !res.colony.space_sites.is_empty() {
        return Vec::new();
    }
    let Some(plan) = &policy.cultivation else {
        return Vec::new();
    };
    let Some(hearth) = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()
    else {
        return Vec::new();
    };
    let year_days = res
        .rules
        .calendar
        .days_per_season
        .saturating_mul(res.rules.calendar.seasons_per_year);
    let (target, cap) = cultivation_area(plan, annual_food_milli(res, policy, facts, year_days));
    let accepted = res
        .cultivation
        .fields
        .values()
        .filter(|field| {
            field.crop == plan.crop
                && (field.active
                    || matches!(
                        field.phase,
                        sim_core::colony::cultivation::FieldPhase::Sown { .. }
                            | sim_core::colony::cultivation::FieldPhase::Growing { .. }
                            | sim_core::colony::cultivation::FieldPhase::Ripe { .. }
                    ))
        })
        .count() as u64;
    let needed = target
        .saturating_sub(accepted)
        .min(cap.saturating_sub(accepted))
        .min(u64::from(plan.batch));
    if needed == 0 {
        return Vec::new();
    }
    let mut reserved = super::planning::reserved_field_house_tiles(world, res, hearth);
    for (_, site) in world.query::<&sim_core::colony::ColonySite>().iter() {
        reserved.extend(site.tiles());
    }
    reserved.extend(res.colony.site_work_tiles.values().copied());
    reserved.extend(res.colony.workshops.values().map(|shop| shop.work_tile()));
    let mut tiles = Vec::new();
    for y in 0..res.tile_grid.height {
        for x in 0..res.tile_grid.width {
            if x.abs_diff(hearth.0).max(y.abs_diff(hearth.1)) <= policy.forage_radius
                && !res.steward.vetoed(
                    &sim_core::colony::steward::Target::Tile(x, y),
                    res.current_tick,
                )
                && !reserved.contains(&(x, y))
                && crate::runtime::cultivation::commands::zoneable(res, (x, y))
            {
                tiles.push((x, y));
            }
        }
    }
    tiles.sort_by_key(|&(x, y)| (x.abs_diff(hearth.0).max(y.abs_diff(hearth.1)), y, x));
    tiles
        .into_iter()
        .take(usize::try_from(needed).unwrap_or(usize::MAX))
        .map(|tile| Command::ZoneField {
            min: tile,
            max: tile,
            crop: plan.crop,
        })
        .collect()
}

/// Population demand across the authored warm and winter seasons, in thousandths.
fn annual_food_milli(
    res: &SimResources,
    policy: &FoodPlan,
    facts: &FoodSummary,
    year_days: u32,
) -> u64 {
    let winter = res.rules.calendar.days_per_season.min(year_days);
    let warm = facts
        .warm_daily_milli
        .saturating_mul(u64::from(year_days - winter));
    let cold = facts
        .warm_daily_milli
        .saturating_mul(u64::from(winter))
        .saturating_mul(u64::from(policy.cold_multiplier))
        .div_ceil(1000);
    warm.saturating_add(cold)
}

/// Target and hard area cap in tiles; physical seeds limit jobs, never this replacement goal.
pub fn cultivation_area(
    policy: &sim_core::colony::steward::CultivationPlan,
    annual_milli: u64,
) -> (u64, u64) {
    let target = annual_milli
        .saturating_mul(u64::from(policy.replacement_permille))
        .div_ceil(1_000_000u64.saturating_mul(u64::from(policy.net_units_per_tile).max(1)));
    (
        target,
        target.saturating_mul(u64::from(policy.area_cap_permille)) / 1000,
    )
}

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod cooking_bill_tests;

#[cfg(test)]
mod horizon_tests;
#[cfg(test)]
mod preservation_tests;

//! Successful whole-item meals: hunger relief, bounded dietary history and raw sickness.
use super::items::{self, FoodError};
use hecs::{Entity, World};
use sim_core::colony::food::{FoodOwner, MealRecord};
use sim_core::colony::people::Sick;
use sim_core::colony::ItemKind;
use sim_core::components::{Agent, Hunger, Lineage};
use sim_core::rng::{keyed_below, STREAM_FOOD_SICK};
use sim_engine::SimResources;

/// Evidence from one completed item intake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealOutcome {
    /// Item actually consumed.
    pub item: ItemKind,
    /// Nominal units, before the hunger clamp.
    pub units: u32,
    /// Data-defined preparation state.
    pub cooked: bool,
    /// Ingredient groups actually eaten.
    pub groups: Vec<String>,
    /// Actual relief, rounded down to tenths for telemetry.
    pub relief_tenths: u64,
}

/// Consume one fresh edible piece; failed admission changes neither food nor hunger.
pub fn consume(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    owner: FoodOwner,
    day: u32,
) -> Result<MealOutcome, FoodError> {
    if res.rules.colony.is_none() {
        return Err(FoodError::Invalid);
    }
    let agent = world
        .get::<&Agent>(entity)
        .map_err(|_| FoodError::MissingPerson)?
        .id;
    let before = world
        .get::<&Hunger>(entity)
        .map_err(|_| FoodError::MissingPerson)?
        .value;
    let stack = items::stack(world, res, owner).ok_or(FoodError::MissingOwner)?;
    let food = stack
        .kind
        .definition()
        .food
        .as_ref()
        .ok_or(FoodError::Inedible)?;
    if !food.raw_edible || matches!(owner, FoodOwner::Carried(id) if id != agent) {
        return Err(FoodError::Inedible);
    }
    let lineage = if food.sick_permille > 0 {
        Some(
            world
                .get::<&Lineage>(entity)
                .map_err(|_| FoodError::MissingPerson)?
                .key,
        )
    } else {
        None
    };
    let born_day = res
        .food
        .ages
        .get(&owner)
        .ok_or(FoodError::MissingAge)?
        .born_day;
    let storage_lifetime_days = food.lifetime_days(super::storage::class(res, owner));
    let source_choice = res.mind.history.as_ref().and_then(|history| {
        let choice = history.latest(agent)?;
        let running = *res.mind.running.get(&agent)?;
        choice
            .matches_running(agent, running)
            .ok()
            .filter(|matched| *matched)
            .map(|_| choice.id())
    });
    let batch = items::take(world, res, owner, 1, day)?;
    // Hunger remains the existing f32 component; new persisted accounting is integer.
    let relief = crate::runtime::agent_actions::needs::HUNGER_CONSUME_AMOUNT * food.units as f32;
    let after = (before - relief).max(0.0);
    world
        .get::<&mut Hunger>(entity)
        .map_err(|_| FoodError::MissingPerson)?
        .value = after;
    let relief_tenths = ((before - after) * 10.0).max(0.0) as u64;
    let nominal_tenths = (relief * 10.0).max(0.0) as u64;
    res.colony.item_ledger.consume(batch.spec.kind, 1);
    let tally = &mut res.food.tallies;
    *tally.eaten.entry(batch.spec.kind).or_default() += 1;
    tally.eaten_units += u64::from(food.units);
    tally.meals += 1;
    tally.cooked_meals += u64::from(food.cooked);
    tally.relief_tenths += relief_tenths;
    tally.wasted_tenths += nominal_tenths.saturating_sub(relief_tenths);
    res.tallies.meals_eaten += 1;
    res.tallies.ground_meals += u64::from(matches!(owner, FoodOwner::Ground(_)));
    let history = res.food.meals.entry(agent).or_default();
    history.prune(res.current_tick, res.rules.calendar.days_to_ticks(3));
    history.record(
        MealRecord {
            tick: res.current_tick,
            item: stack.kind,
            pieces: 1,
            units: food.units,
            cooked: food.cooked,
            source_owner: owner,
            born_day,
            storage_lifetime_days,
            source_choice,
        },
        &food.groups,
    );
    if let Some(lineage) = lineage {
        if keyed_below(
            res.world_seed,
            STREAM_FOOD_SICK,
            lineage,
            res.current_tick,
            stack.kind.code(),
            0,
            1000,
        ) < u64::from(food.sick_permille)
        {
            let sick_days = res
                .rules
                .colony
                .and_then(|rule| rule.story)
                .map_or(sim_core::rules::StoryRule::M5.sick_days, |story| {
                    story.sick_days
                });
            let until = res
                .current_tick
                .saturating_add(res.rules.calendar.days_to_ticks(u64::from(sick_days)));
            let until = world
                .get::<&Sick>(entity)
                .map_or(until, |sick| sick.until.max(until));
            world
                .insert_one(entity, Sick { until })
                .map_err(|_| FoodError::MissingPerson)?;
        }
    }
    if food.cooked {
        super::events::cooked_meal(
            res,
            agent,
            stack.kind,
            food.units,
            (owner, born_day, storage_lifetime_days),
        );
    }
    super::evidence::clear_person(res, agent);
    res.food.touch();
    Ok(MealOutcome {
        item: stack.kind,
        units: food.units,
        cooked: food.cooked,
        groups: food.groups.clone(),
        relief_tenths,
    })
}

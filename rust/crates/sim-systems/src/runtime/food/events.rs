//! Once-only food milestones emitted only after their physical effects succeed.
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::ItemKind;
use sim_engine::SimResources;

/// Record the first completed cooked meal, never a bill or interrupted intent.
pub fn cooked_meal(
    res: &mut SimResources,
    agent: u64,
    item: ItemKind,
    units: u32,
    source: (FoodOwner, u32, u32),
) {
    if res.food.firsts.cooked_meal.is_some() {
        return;
    }
    let entry = ChronicleEntry::new(res.current_tick, ChronicleKind::FirstCookedMeal)
        .subject(agent)
        .cause(item.code())
        .value(i64::from(units));
    let parent = super::evidence::operation(
        res,
        Some(agent),
        sim_core::colony::food_evidence::OperationEvidence::Intake {
            owner: source.0,
            item,
            born_day: source.1,
            lifetime_days: source.2,
        },
    );
    let index = res
        .chronicle
        .push_caused(entry, Some(sim_core::chronicle::CauseRef::Entry(parent)));
    res.food.firsts.cooked_meal = Some(u64::from(index));
}

/// Record preserved food only after a real transfer into actual storage.
pub fn stored(res: &mut SimResources, owner: FoodOwner, kind: ItemKind, count: u16) {
    if res.food.firsts.dried_store.is_some()
        || !kind.definition().tags.iter().any(|tag| tag == "preserved")
    {
        return;
    }
    if matches!(super::storage::class(res, owner), FoodStorageClass::Ground) {
        return;
    }
    let entry = ChronicleEntry::new(res.current_tick, ChronicleKind::FirstDriedStore)
        .cause(kind.code())
        .value(i64::from(count));
    let Some(age) = res.food.ages.get(&owner).copied() else {
        return;
    };
    let parent = super::evidence::operation(
        res,
        None,
        sim_core::colony::food_evidence::OperationEvidence::Stored {
            owner,
            item: kind,
            pieces: count,
            born_day: age.born_day,
        },
    );
    let index = res
        .chronicle
        .push_caused(entry, Some(sim_core::chronicle::CauseRef::Entry(parent)));
    res.food.firsts.dried_store = Some(u64::from(index));
}

/// Capture facts before death cleanup drops cargo or removes the person.
/// Stages describe observations now, not an inferred historical harvest failure.
pub fn hunger_death(
    world: &hecs::World,
    res: &mut SimResources,
    entity: hecs::Entity,
    agent: u64,
    position: (u32, u32),
    tick: u64,
) -> Option<u32> {
    use sim_core::chronicle::{CauseRef, NeedSample};
    use sim_core::colony::people::{Modifier, Mood};
    use sim_core::components::{Hunger, Thirst};
    res.rules.colony?;
    let hunger = world.get::<&Hunger>(entity).ok()?.value;
    if hunger < Hunger::SATURATION {
        return None;
    }
    let mut stored_units = 0_u64;
    for owner in res.food.ages.keys().copied() {
        if matches!(super::storage::class(res, owner), FoodStorageClass::Ground) {
            continue;
        }
        let tile = match owner {
            FoodOwner::Ground(tile) => Some(tile),
            FoodOwner::Pot { id, .. } => res.food.pots.get(&id).map(|pot| pot.tile),
            FoodOwner::Shelf { workshop, .. } => res
                .colony
                .workshops
                .get(&workshop)
                .map(|shop| shop.work_tile()),
            FoodOwner::Carried(_) => None,
        };
        if !tile.is_some_and(|tile| res.nav.reachable(position, tile)) {
            continue;
        }
        let Some(stack) = super::items::stack(world, res, owner) else {
            continue;
        };
        let Some(food) = &stack.kind.definition().food else {
            continue;
        };
        if food.raw_edible
            && super::storage::remaining_days(world, res, owner, super::day(res))
                .is_ok_and(|days| days > 0)
        {
            stored_units += u64::from(stack.count) * u64::from(food.units);
        }
    }
    let catalog = sim_core::colony::content::installed_or_bundled();
    let mut food_plants = 0_u64;
    let mut ripe_units = 0_u64;
    let mut reachable_units = 0_u64;
    for (&tile, plant) in &res.food.plants {
        let Some(def) = catalog.plant(plant.species) else {
            continue;
        };
        let Some(food) = &def.yield_item.definition().food else {
            continue;
        };
        food_plants += 1;
        if !super::plants::harvestable(res, tile) {
            continue;
        }
        let units = u64::from(plant.units_left) * u64::from(food.units);
        ripe_units += units;
        if res.nav.reachable(position, tile) {
            reachable_units += units;
        }
    }
    let reason = if food_plants == 0 {
        3
    } else if ripe_units == 0 {
        4
    } else if reachable_units == 0 {
        5
    } else {
        6
    };
    let observation = |stage, value| {
        ChronicleEntry::new(tick, ChronicleKind::HungerFoodChain)
            .subject(agent)
            .cause(stage)
            .value(value)
    };
    if let Ok(job) = world.get::<&sim_core::colony::Job>(entity) {
        if let sim_core::colony::JobKind::Harvest { plant } = job.kind {
            if job.item.definition().food.is_some() {
                super::evidence::record_failure(
                    res,
                    agent,
                    sim_core::colony::food_evidence::AcquisitionReceipt {
                        tick,
                        target: Some(plant),
                        job_started: Some(job.taken_at),
                        reason: sim_core::colony::food_evidence::AcquisitionReason::Unfinished,
                    },
                );
            }
        }
    }
    let historical = super::evidence::death_parent(res, agent);
    let reason_entry = res.chronicle.push_caused(
        observation(reason, i64::try_from(ripe_units).unwrap_or(i64::MAX)),
        historical.map(CauseRef::Entry),
    );
    let harvest_entry = res.chronicle.push_caused(
        observation(2, i64::try_from(reachable_units).unwrap_or(i64::MAX)),
        Some(CauseRef::Entry(reason_entry)),
    );
    let stores_entry = res.chronicle.push_caused(
        observation(1, i64::try_from(stored_units).unwrap_or(i64::MAX)),
        Some(CauseRef::Entry(harvest_entry)),
    );
    let sample = NeedSample {
        agent,
        tick,
        hunger: (hunger.clamp(0.0, 100.0) * 10.0) as u16,
        thirst: world
            .get::<&Thirst>(entity)
            .map_or(0, |thirst| (thirst.value.clamp(0.0, 100.0) * 10.0) as u16),
        nights: world.get::<&Mood>(entity).map_or(0, |mood| mood.nights & 7),
        entry: Some(stores_entry),
    };
    let hunger_tenths = sample.hunger;
    let sample_id = res.chronicle.push_sample(sample);
    Some(res.chronicle.push_caused(
        observation(0, i64::from(hunger_tenths)),
        Some(CauseRef::Sample {
            code: Modifier::Hungry.code(),
            sample: sample_id,
        }),
    ))
}

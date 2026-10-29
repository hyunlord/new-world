//! Raw daily food observations; no thresholds or policy mutations.
use hecs::World;
use serde::Serialize;
use sim_core::colony::content::FoodStorageClass;
use sim_core::colony::food::{FoodOwner, FoodTallies};
use sim_core::components::{Agent, Child, Position};
use sim_engine::SimResources;
use sim_systems::runtime::food::{items, storage};
use std::collections::BTreeMap;

#[derive(Serialize)]
pub(super) struct Adult {
    pub id: u64,
    pub tile: (u32, u32),
    pub region: u32,
    pub groups: BTreeMap<String, u64>,
}

#[derive(Serialize)]
pub(super) struct Stored {
    pub region: u32,
    pub owner: FoodOwner,
    pub tile: (u32, u32),
    pub item: String,
    pub pieces: u16,
    pub units_per_piece: u32,
    pub born_day: u32,
    pub remaining_days: u32,
    pub storage: &'static str,
    pub reachable_adults: Vec<u64>,
}

#[derive(Serialize)]
pub(super) struct SettlementStore {
    pub id: u32,
    pub home: (u32, u32),
    pub region: u32,
    pub runtime_units: u64,
}

#[derive(Serialize)]
pub(super) struct Physical {
    pub owner: FoodOwner,
    pub item: String,
    pub pieces: u16,
    pub units_per_piece: u32,
    pub born_day: u32,
    pub lifetime_days: u32,
}

#[derive(Serialize)]
pub(super) struct Daily {
    pub day: u32,
    pub phase: &'static str,
    pub physical: Vec<Physical>,
    pub tick: u64,
    pub alive: usize,
    pub adults: Vec<Adult>,
    pub settlements: BTreeMap<u32, Vec<u64>>,
    pub settlement_stores: Vec<SettlementStore>,
    pub stored: Vec<Stored>,
    pub tallies: FoodTallies,
    pub physical_food_units: u64,
    pub harvested_edible_units: u64,
    pub harvested_convertible_units: u64,
    pub errors: Vec<String>,
}

pub(super) fn daily(world: &World, res: &SimResources) -> Daily {
    let day = sim_systems::runtime::food::day(res);
    let mut adults: Vec<_> = world
        .query::<(&Agent, &Position)>()
        .without::<&Child>()
        .iter()
        .map(|(_, (agent, pos))| Adult {
            id: agent.id,
            tile: (pos.x, pos.y),
            region: res.nav.region_of((pos.x, pos.y)),
            groups: res
                .food
                .meals
                .get(&agent.id)
                .map(|m| m.last_group_tick.clone())
                .unwrap_or_default(),
        })
        .collect();
    adults.sort_by_key(|adult| adult.id);
    let settlements = res
        .settlements
        .iter()
        .map(|(&id, settlement)| {
            let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
            members.sort_unstable();
            (id, members)
        })
        .collect();
    let mut snapshot = Daily {
        day,
        phase: "after_expiry",
        physical: Vec::new(),
        tick: res.current_tick,
        alive: world.query::<&Agent>().iter().count(),
        adults,
        settlements,
        settlement_stores: res
            .settlements
            .iter()
            .map(|(&id, settlement)| SettlementStore {
                id,
                home: settlement.formation_tile,
                region: res.nav.region_of(settlement.formation_tile),
                runtime_units: sim_systems::runtime::food::reserves::units(world, res, id, false),
            })
            .collect(),
        stored: Vec::new(),
        tallies: res.food.tallies.clone(),
        physical_food_units: 0,
        harvested_edible_units: res
            .food
            .tallies
            .harvested
            .iter()
            .filter_map(|(kind, count)| {
                kind.definition()
                    .food
                    .as_ref()
                    .filter(|food| food.raw_edible)
                    .map(|food| count * u64::from(food.units))
            })
            .sum(),
        harvested_convertible_units: res
            .food
            .tallies
            .harvested
            .iter()
            .filter_map(|(kind, count)| {
                kind.definition()
                    .food
                    .as_ref()
                    .filter(|food| !food.raw_edible)
                    .map(|food| count * u64::from(food.units))
            })
            .sum(),
        errors: Vec::new(),
    };
    for (&owner, age) in &res.food.ages {
        let Some(stack) = items::stack(world, res, owner) else {
            snapshot
                .errors
                .push(format!("missing physical food owner {owner:?}"));
            continue;
        };
        let Some(food) = &stack.kind.definition().food else {
            snapshot.errors.push(format!("nonfood age {owner:?}"));
            continue;
        };
        snapshot.physical_food_units += u64::from(stack.count) * u64::from(food.units);
        let class = storage::class(res, owner);
        snapshot.physical.push(Physical {
            owner,
            item: stack.kind.name().into(),
            pieces: stack.count,
            units_per_piece: food.units,
            born_day: age.born_day,
            lifetime_days: food.lifetime_days(class),
        });
        let storage = match class {
            FoodStorageClass::Ground => continue,
            FoodStorageClass::Stockpile => "stockpile",
            FoodStorageClass::RoofedStore => "roofed_store",
            FoodStorageClass::Pot => "pot",
        };
        let tile = match owner {
            FoodOwner::Ground(tile) => Some(tile),
            FoodOwner::Shelf { workshop, .. } => {
                res.colony.workshops.get(&workshop).map(|s| s.work_tile())
            }
            FoodOwner::Pot { id, .. } => res.food.pots.get(&id).map(|pot| pot.tile),
            FoodOwner::Carried(_) => None,
        };
        let Some(tile) = tile else {
            continue;
        };
        let remaining_days = age.remaining(day, food.lifetime_days(class));
        if !food.raw_edible || remaining_days == 0 || age.born_day > day {
            continue;
        }
        snapshot.stored.push(Stored {
            region: res.nav.region_of(tile),
            owner,
            tile,
            item: stack.kind.name().into(),
            pieces: stack.count,
            units_per_piece: food.units,
            born_day: age.born_day,
            remaining_days,
            storage,
            reachable_adults: snapshot
                .adults
                .iter()
                .filter(|adult| res.nav.reaches(adult.region, tile))
                .map(|adult| adult.id)
                .collect(),
        });
    }
    snapshot
}

//! Measurement-only boundary snapshots and completed meal/death evidence.
//! Priority: 120. Interval: calendar day. Reads: world and food state. Writes: observer buffer only.
use super::metrics::{self, Daily};
use hecs::World;
use serde::Serialize;
use sim_core::colony::food::FoodTallies;
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use std::io::Write;
use std::sync::{Arc, Mutex};

pub(super) type Boundaries = Arc<Mutex<Vec<Daily>>>;
pub(super) struct BoundaryObserver {
    pub interval: u64,
    pub before_expiry: bool,
    pub boundaries: Boundaries,
}
impl RuntimeSystem for BoundaryObserver {
    fn name(&self) -> &str {
        if self.before_expiry {
            "M6aMeasurementBeforeExpiry"
        } else {
            "M6aMeasurementAfterExpiry"
        }
    }
    fn priority(&self) -> u32 {
        if self.before_expiry {
            118
        } else {
            120
        }
    }
    fn tick_interval(&self) -> u64 {
        self.interval
    }
    fn tick_offset(&self, res: &SimResources) -> u64 {
        (self.interval - res.rules.clock_offset_ticks % self.interval) % self.interval
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        if let Ok(mut rows) = self.boundaries.lock() {
            let mut row = metrics::daily(world, res);
            if self.before_expiry {
                row.phase = "before_expiry";
            }
            rows.push(row);
        }
    }
    fn save_state(&self) -> Result<Vec<u8>, String> {
        Err(
            "M6a measurement-only worlds cannot be saved; use the uninstrumented save fixture"
                .into(),
        )
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum Record<'a> {
    Daily {
        snapshot: &'a Daily,
    },
    Meal {
        tick: u64,
        day: u64,
        agent: u64,
        item: &'a str,
        pieces: u16,
        units: u32,
        cooked: bool,
        groups: &'a [String],
        source_owner: sim_core::colony::food::FoodOwner,
        born_day: u32,
        storage_lifetime_days: u32,
        choice: serde_json::Value,
    },
    HungerDeath {
        tick: u64,
        day: u64,
        count: u64,
    },
    FoodFlow {
        tick: u64,
        tallies: &'a FoodTallies,
    },
    /// R2-2: food spoiled this tick and how many settlement members were hungry (above the seek
    /// threshold) at that moment.
    SpoilMoment {
        tick: u64,
        day: u64,
        spoiled_units: u64,
        hungry_members: u64,
    },
    /// R2-2: food units that entered storage this tick, split by covered (roofed store or pot)
    /// placement and, for open placement, whether covered room for the stack existed.
    Placement {
        tick: u64,
        day: u64,
        covered: u64,
        open_with_covered_room: u64,
        open_without_covered_room: u64,
    },
    Violation {
        tick: u64,
        kind: &'a str,
        detail: &'a str,
    },
}

pub(super) fn write(out: &mut impl Write, row: &impl Serialize) -> Result<(), String> {
    serde_json::to_writer(&mut *out, row).map_err(|e| e.to_string())?;
    out.write_all(b"\n").map_err(|e| e.to_string())
}

#[derive(Default)]
pub(super) struct Watch {
    pub observed_ticks: u64,
    pub invariant_checks: u64,
    pub violations: u64,
    pub observed_meals: u64,
    pub previous: FoodTallies,
    pub starvation: u64,
    /// Stored count per storage owner after the previous tick (R2-2 placement).
    pub stored: std::collections::BTreeMap<sim_core::colony::food::FoodOwner, u16>,
    pub placed_covered: u64,
    pub placed_open_with_room: u64,
    pub placed_open_without_room: u64,
    pub spoil_moments: u64,
    pub spoil_moments_hungry: u64,
}
impl Watch {
    /// R2-2: count units that entered stockpile, roofed-store or pot owners this tick.
    fn placement(
        &mut self,
        engine: &SimEngine,
        day: u64,
        out: &mut impl Write,
    ) -> Result<(), String> {
        use sim_core::colony::content::FoodStorageClass as Class;
        let (world, r) = (&engine.world, &engine.resources);
        let mut now = std::collections::BTreeMap::new();
        let (mut covered, mut open_room, mut open_none) = (0u64, 0u64, 0u64);
        for (&owner, age) in &r.food.ages {
            let class = sim_systems::runtime::food::storage::class(r, owner);
            if !matches!(class, Class::Stockpile | Class::RoofedStore | Class::Pot) {
                continue;
            }
            let Some(stack) = sim_systems::runtime::food::items::stack(world, r, owner) else {
                continue;
            };
            now.insert(owner, stack.count);
            let added = stack
                .count
                .saturating_sub(self.stored.get(&owner).copied().unwrap_or(0));
            if added == 0 {
                continue;
            }
            let units = u64::from(added)
                * u64::from(stack.kind.definition().food.as_ref().map_or(1, |f| f.units));
            if matches!(class, Class::RoofedStore | Class::Pot) {
                covered += units;
                continue;
            }
            let sim_core::colony::food::FoodOwner::Ground(tile) = owner else {
                continue;
            };
            let room = sim_systems::runtime::jobs::food_storage::destination(
                r,
                r.nav.region_of(tile),
                tile,
                sim_core::colony::ItemSpec::raw(stack.kind, None),
                age.born_day,
                true,
            );
            if room.is_some() {
                open_room += units;
            } else {
                open_none += units;
            }
        }
        self.stored = now;
        if covered + open_room + open_none > 0 {
            self.placed_covered += covered;
            self.placed_open_with_room += open_room;
            self.placed_open_without_room += open_none;
            write(
                out,
                &Record::Placement {
                    tick: r.current_tick,
                    day,
                    covered,
                    open_with_covered_room: open_room,
                    open_without_covered_room: open_none,
                },
            )?;
        }
        Ok(())
    }

    pub fn after_tick(&mut self, engine: &SimEngine, out: &mut impl Write) -> Result<(), String> {
        let r = &engine.resources;
        if r.current_tick != self.observed_ticks || engine.current_tick() != self.observed_ticks + 1
        {
            return Err("M6a observer skipped or repeated a tick".into());
        }
        self.observed_ticks += 1;
        let day = sim_systems::runtime::resource_regen::day_index(r);
        for (&agent, history) in &r.food.meals {
            for meal in history
                .last_meals
                .iter()
                .filter(|meal| meal.tick == r.current_tick)
            {
                let def = meal.item.definition();
                let food = def
                    .food
                    .as_ref()
                    .ok_or("nonfood in completed meal record")?;
                let choice = match meal.source_choice {
                    None => serde_json::Value::Null,
                    Some(id) => {
                        let history = r
                            .mind
                            .history
                            .as_ref()
                            .ok_or("missing meal choice history")?;
                        let packed = history
                            .find(agent, id)
                            .ok_or("missing actual meal choice")?;
                        let display = packed
                            .decode_display(history.dictionary())
                            .map_err(|e| format!("meal choice decode: {e:?}"))?;
                        serde_json::json!({"id":display.id,"parent":display.parent,"agent":display.agent,"tick":display.tick,
                            "node":display.selected.labels.name,"behaviour":display.selected.labels.behaviour,
                            "target":display.selected.labels.target,"facts":display.facts.iter().map(|fact| serde_json::json!({"name":fact.name,"value":fact.value,"cause":fact.cause})).collect::<Vec<_>>()})
                    }
                };
                write(
                    out,
                    &Record::Meal {
                        tick: meal.tick,
                        day,
                        agent,
                        item: &def.identity.id,
                        pieces: meal.pieces,
                        units: meal.units,
                        cooked: meal.cooked,
                        groups: &food.groups,
                        source_owner: meal.source_owner,
                        born_day: meal.born_day,
                        storage_lifetime_days: meal.storage_lifetime_days,
                        choice,
                    },
                )?;
                self.observed_meals += u64::from(meal.pieces);
            }
        }
        if self.observed_meals != r.food.tallies.meals {
            return Err("completed meal records do not reconcile with meal tally".into());
        }
        let deaths = r
            .tallies
            .deaths_starvation
            .checked_sub(self.starvation)
            .ok_or("death tally decreased")?;
        if deaths > 0 {
            write(
                out,
                &Record::HungerDeath {
                    tick: r.current_tick,
                    day,
                    count: deaths,
                },
            )?;
        }
        self.starvation = r.tallies.deaths_starvation;
        let spoiled = r
            .food
            .tallies
            .spoiled_units
            .saturating_sub(self.previous.spoiled_units);
        if spoiled > 0 {
            let members: std::collections::BTreeSet<u64> = r
                .settlements
                .values()
                .flat_map(|s| s.member_agents.iter().copied())
                .collect();
            let hungry = engine
                .world
                .query::<(&sim_core::components::Agent, &sim_core::components::Hunger)>()
                .iter()
                .filter(|(_, (a, h))| {
                    members.contains(&a.id)
                        && h.value > sim_systems::runtime::agent_actions::HUNGER_THRESHOLD
                })
                .count() as u64;
            self.spoil_moments += 1;
            self.spoil_moments_hungry += u64::from(hungry > 0);
            write(
                out,
                &Record::SpoilMoment {
                    tick: r.current_tick,
                    day,
                    spoiled_units: spoiled,
                    hungry_members: hungry,
                },
            )?;
        }
        self.placement(engine, day, out)?;
        if self.previous != r.food.tallies {
            write(
                out,
                &Record::FoodFlow {
                    tick: r.current_tick,
                    tallies: &r.food.tallies,
                },
            )?;
            self.previous = r.food.tallies.clone();
        }
        for (kind, result) in [
            (
                "item_reservations",
                sim_systems::runtime::jobs::invariants::check(&engine.world, r),
            ),
            (
                "food",
                sim_systems::runtime::food::validate(&engine.world, r),
            ),
        ] {
            self.invariant_checks += 1;
            if let Err(detail) = result {
                self.violations += 1;
                write(
                    out,
                    &Record::Violation {
                        tick: r.current_tick,
                        kind,
                        detail: &detail,
                    },
                )?;
            }
        }
        Ok(())
    }
}

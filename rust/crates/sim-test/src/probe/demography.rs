//! Colony demography (M1.5 goal 9, bars D1–D5), measurement only.
//!
//! Every day at 12:00 (the tick `FertilitySystem` runs) the observer
//! counts the woman-days of exposure: women of fertile age
//! (`[fertile_from_years, fertile_to_years)` of the rule) with a living
//! partner in a settlement whose members' mean hunger and thirst are within
//! the rule's limits ("fertile, paired, fed"), pregnant or not. D1 is
//! births / (woman-days / days per year). At every year boundary it records
//! the living population. At the end it reads the chronicle: old-age deaths
//! with their age in years (founders = the agents alive at tick 0), and for
//! each child born in the first year its coming of age (tick, age in years
//! and ticks), plus the number of living people at or past the child age
//! who still carry `Child` (D5 wants 0).
use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_core::components::{Agent, AgentId, BodyHealth, Child, Hunger, Kin, Life, Sex, Thirst};
use sim_core::rules::BirthRule;
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

/// The observer (see the module header).
#[derive(Debug, Clone, Default)]
pub struct Demography {
    founders: BTreeSet<AgentId>,
    woman_days: u64,
    population_by_year: BTreeMap<u64, u64>,
}

fn fed(
    engine: &SimEngine,
    members: &std::collections::HashSet<AgentId>,
    max_h: f64,
    max_t: f64,
) -> bool {
    let rows: Vec<(f64, f64)> = engine
        .world
        .query::<(&Agent, &Hunger, &Thirst)>()
        .iter()
        .filter(|(_, (a, _, _))| members.contains(&a.id))
        .map(|(_, (_, h, t))| (f64::from(h.value), t.value))
        .collect();
    if rows.is_empty() {
        return false;
    }
    let n = rows.len() as f64;
    rows.iter().map(|r| r.0).sum::<f64>() / n <= max_h
        && rows.iter().map(|r| r.1).sum::<f64>() / n <= max_t
}

impl Demography {
    /// Start observing `engine` at tick 0.
    pub fn new(engine: &SimEngine) -> Self {
        Self {
            founders: engine
                .world
                .query::<&Agent>()
                .iter()
                .map(|(_, a)| a.id)
                .collect(),
            ..Self::default()
        }
    }

    /// Observe the state after a tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let r = &engine.resources;
        let now = engine.current_tick();
        let tpd = r.rules.calendar.ticks_per_day.max(1);
        let tpy = r.rules.calendar.ticks_per_year().max(1);
        if now.is_multiple_of(tpy) {
            let alive = engine.world.query::<&Agent>().iter().count() as u64;
            self.population_by_year.insert(now / tpy, alive);
        }
        let BirthRule::Fertility {
            fertile_from_years,
            fertile_to_years,
            max_mean_hunger,
            max_mean_thirst,
            ..
        } = r.rules.society.birth
        else {
            return;
        };
        // The tick that has just run is FertilitySystem's midday tick.
        let minute = sim_systems::runtime::agent::fertility::FERTILITY_MINUTE;
        if (now - 1 + r.rules.clock_offset_ticks) % tpd != minute * tpd / 1440 {
            return;
        }
        let at = now - 1;
        let alive: BTreeSet<AgentId> = engine
            .world
            .query::<(&Agent, Option<&BodyHealth>)>()
            .iter()
            .filter(|(_, (_, h))| h.is_none_or(|h| !h.is_dead()))
            .map(|(_, (a, _))| a.id)
            .collect();
        let from = r
            .rules
            .calendar
            .years_to_ticks(u64::from(fertile_from_years)) as i64;
        let to = r.rules.calendar.years_to_ticks(u64::from(fertile_to_years)) as i64;
        let mut fed_cache: BTreeMap<u32, bool> = BTreeMap::new();
        let mut women: Vec<AgentId> = engine
            .world
            .query::<(&Agent, &Sex, &Kin, &Life, Option<&Child>)>()
            .iter()
            .filter(|(_, (a, sex, kin, life, child))| {
                sex.female
                    && child.is_none()
                    && alive.contains(&a.id)
                    && (from..to).contains(&life.age_ticks(at))
                    && kin.partner.is_some_and(|p| alive.contains(&p))
            })
            .map(|(_, (a, ..))| a.id)
            .collect();
        women.sort_unstable();
        for id in women {
            let sid = sim_systems::runtime::agent::fertility::settlement_of(r, id);
            let Some(sid) = sid else { continue };
            let ok = *fed_cache.entry(sid).or_insert_with(|| {
                r.settlements.get(&sid).is_some_and(|s| {
                    fed(engine, &s.member_agents, max_mean_hunger, max_mean_thirst)
                })
            });
            if ok {
                self.woman_days += 1;
            }
        }
    }

    /// The D1–D5 measurements of the run.
    pub fn report(&self, engine: &SimEngine) -> Value {
        let r = &engine.resources;
        let cal = r.rules.calendar;
        let tpy = cal.ticks_per_year().max(1);
        let now = engine.current_tick();
        let births = r.tallies.births;
        let woman_years = self.woman_days as f64 / cal.days_per_year().max(1) as f64;
        let entries = r.chronicle.entries();
        let mut born_at: BTreeMap<AgentId, u64> = BTreeMap::new();
        let mut came_of_age: BTreeMap<AgentId, u64> = BTreeMap::new();
        let mut old_age: Vec<Value> = Vec::new();
        let mut births_by_year: BTreeMap<u64, u64> = BTreeMap::new();
        for e in entries {
            match (e.kind, e.subject) {
                (ChronicleKind::Born, Some(id)) => {
                    born_at.insert(id, e.tick);
                    *births_by_year.entry(e.tick / tpy).or_insert(0) += 1;
                }
                (ChronicleKind::CameOfAge, Some(id)) => {
                    came_of_age.entry(id).or_insert(e.tick);
                }
                (ChronicleKind::Died, Some(id)) if e.cause == 3 => old_age.push(json!({
                    "agent": id, "tick": e.tick, "founder": self.founders.contains(&id),
                    "age_years": cal.whole_years(e.value), "age_ticks": e.value,
                })),
                _ => {}
            }
        }
        let child_ticks = r.rules.society.life_stage.map_or(0, |l| l.child_ticks);
        let year_one: Vec<Value> = born_at
            .iter()
            .filter(|(_, &t)| t < tpy)
            .map(|(id, &t)| {
                let coa = came_of_age.get(id).copied();
                json!({"agent": id, "born_tick": t, "came_of_age_tick": coa,
                       "age_ticks_at_coming_of_age": coa.map(|c| c - t),
                       "age_years_at_coming_of_age": coa.map(|c| cal.whole_years((c - t) as i64)),
                       "reached_child_age_in_run": t + child_ticks <= now})
            })
            .collect();
        let child_after_age = engine
            .world
            .query::<(&Agent, &Child, &Life)>()
            .iter()
            .filter(|(_, (_, _, life))| life.age_ticks(now) >= child_ticks as i64)
            .count();
        let alive = engine.world.query::<&Agent>().iter().count();
        let mut ages: Vec<u64> = engine
            .world
            .query::<&Life>()
            .iter()
            .map(|(_, l)| cal.whole_years(l.age_ticks(now)))
            .collect();
        ages.sort_unstable();
        json!({
            "births": births,
            "births_by_year": births_by_year,
            "woman_days": self.woman_days,
            "woman_years": woman_years,
            "births_per_woman_year": if woman_years > 0.0 { births as f64 / woman_years } else { 0.0 },
            "population_by_year": self.population_by_year,
            "alive_end": alive,
            "extinct": alive == 0,
            "old_age_deaths": old_age,
            "year_one_children": year_one,
            "child_marker_past_child_age": child_after_age,
            "ages_years_end": ages,
            "pregnant_end": engine.world.query::<&sim_core::components::Pregnant>().iter().count(),
        })
    }
}

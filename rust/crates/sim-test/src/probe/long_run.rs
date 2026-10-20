//! Additive v9 economy, housing and life-stage observations.
use serde_json::{json, Value};
use sim_core::{
    components::{
        Agent, AgentState, BuildingKind, Child, Elder, Hunger, Life, Position, SeekTarget,
        TargetKind,
    },
    rules::{BirthRule, HousingCount},
};
use sim_engine::SimEngine;
use sim_systems::runtime::settlement::{
    food_economy::{need_per_day, source_supply_per_day, sustainable_adults},
    housing_capacity,
};
use std::collections::BTreeMap;

fn sustain_reporting(engine: &SimEngine) -> (f64, u32) {
    match engine.resources.rules.society.birth {
        BirthRule::Provisioned {
            sustain: Some(rule),
            ..
        } => (rule.utilization, rule.forage_radius),
        _ => (0.85, 16),
    }
}

/// Count each owned shelter once, using the production capacity helper.
pub(super) fn owned_capacity(engine: &SimEngine) -> u64 {
    let Some(mut rule) = engine.resources.rules.society.shelter_planning else {
        return 0;
    };
    rule.housing = HousingCount::Owned;
    engine
        .resources
        .settlements
        .keys()
        .map(|sid| u64::from(housing_capacity(&engine.resources, *sid, &rule)))
        .sum()
}

/// The fifteen schema-three keys; quantities unavailable under v8 are zero.
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let (utilization, forage_radius) = sustain_reporting(engine);
    let source_count = r
        .food_source_max
        .keys()
        .filter(|&&(x, y)| {
            r.settlements.values().any(|s| {
                x.abs_diff(s.formation_tile.0)
                    .max(y.abs_diff(s.formation_tile.1))
                    <= forage_radius
            })
        })
        .count();
    // Union of catchments: an overlapping v8 bush contributes only once globally.
    let supply = source_count as f64 * source_supply_per_day(r.ticks_per_day);
    // The production gate plans against the annual mean when seasons are on.
    let planning_supply = r.rules.seasons.map_or(supply, |rule| {
        supply
            * f64::from(
                rule.regen_permille
                    .iter()
                    .map(|p| u32::from(*p))
                    .sum::<u32>(),
            )
            / 4000.0
    });
    let mut homes = BTreeMap::new();
    let mut settlements: Vec<_> = r.settlements.values().collect();
    settlements.sort_by_key(|s| s.settlement_id);
    for settlement in settlements {
        for id in &settlement.member_agents {
            homes.entry(*id).or_insert(settlement.formation_tile);
        }
    }
    let mut query = engine.world.query::<(
        &Agent,
        &Position,
        Option<&Child>,
        Option<&Hunger>,
        Option<&AgentState>,
    )>();
    let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
    rows.sort_by_key(|row| row.0.id);
    let (mut adults, mut children, mut need, mut beds, mut sleeping, mut idle, mut near) =
        (0u64, 0u64, 0.0, 0u64, 0u64, 0u64, 0u64);
    for (agent, pos, child, hunger, state) in rows {
        children += u64::from(child.is_some());
        adults += u64::from(child.is_none());
        if let Some(hunger) = hunger {
            need += need_per_day(hunger, r.ticks_per_day);
        }
        let bed = r.bed_of.get(&agent.id);
        beds += u64::from(bed.is_some());
        sleeping += u64::from(
            matches!(
                state,
                Some(AgentState::Consuming {
                    target: TargetKind::Sleep
                })
            ) && bed == Some(&(pos.x, pos.y))
                && r.building_kinds.get(&(pos.x, pos.y)) == Some(&BuildingKind::Shelter),
        );
        if child.is_none() && state.is_none_or(|state| *state == AgentState::Idle) {
            idle += 1;
            near += u64::from(homes.get(&agent.id).is_some_and(|&(x, y)| {
                r.building_kinds.get(&(x, y)) == Some(&BuildingKind::Hearth)
                    && pos.x.abs_diff(x).max(pos.y.abs_diff(y)) <= 3
            }));
        }
    }
    json!({"sample_kind":"periodic","minute_of_day":if r.ticks_per_day == 0 {0} else {(engine.current_tick()+r.rules.clock_offset_ticks)%r.ticks_per_day},
        "adults":adults,"children":children,"housing_capacity_owned":owned_capacity(engine),
        "food_supply_per_day":supply,"food_need_per_day":need,
        "sustainable_adults":sustainable_adults(planning_supply,utilization,r.ticks_per_day),
        "food_harvested_total":r.tallies.food_harvested,"meals_eaten_total":r.tallies.meals_eaten,
        "meals_without_food_total":r.tallies.meals_without_food,
        "beds_assigned":beds,"sleeping_in_shelter":sleeping,"idle_adults":idle,"idle_adults_near_hearth":near})
}

/// Every-tick non-vacuous child-stage evidence, with deaths separated from maturation.
#[derive(Debug, Default)]
pub(super) struct MaturationMonitor {
    previous: BTreeMap<u64, u64>,
    /// Distinct children first observed alive.
    pub(super) observed: u64,
    /// Observed children later alive without Child.
    pub(super) matured: u64,
    /// Observed children that died before maturation.
    pub(super) died: u64,
}
impl MaturationMonitor {
    /// Observe at tick zero and each completed tick, and enforce the due-date bound.
    pub(super) fn observe(&mut self, engine: &SimEngine) -> Result<(), String> {
        check_maturation(engine)?;
        let live: BTreeMap<_, _> = engine
            .world
            .query::<(&Agent, Option<&Child>)>()
            .iter()
            .map(|(_, (agent, child))| (agent.id, child.map(|child| child.born_tick)))
            .collect();
        for (id, born_tick) in &self.previous {
            match live.get(id) {
                None => self.died += 1,
                Some(None) => {
                    if let Some(rule) = engine.resources.rules.society.life_stage {
                        let due = born_tick.saturating_add(rule.child_ticks);
                        let completed = engine.current_tick();
                        // Systems see internal t; the observer sees completed t + 1.
                        // Interval 60 permits internal due..=due+59, never earlier.
                        if completed <= due || completed > due.saturating_add(60) {
                            return Err(format!(
                                "invalid maturation age: agent={id} born={born_tick} completed={completed} due={due}"
                            ));
                        }
                    }
                    self.matured += 1;
                }
                Some(Some(_)) => {}
            }
        }
        for (id, child) in &live {
            if child.is_some() && !self.previous.contains_key(id) {
                self.observed += 1;
            }
        }
        self.previous = live
            .into_iter()
            .filter_map(|(id, born)| born.map(|born| (id, born)))
            .collect();
        Ok(())
    }
    /// Counts expose runs in which no child or maturation was actually observed.
    pub(super) fn report(&self) -> Value {
        json!({"children_observed":self.observed,"maturations_observed":self.matured,
            "child_deaths":self.died,"children_still_alive":self.previous.len(),"late_children":0})
    }
}

/// Reject Child at completed due+60: internal due+59 was the last valid run.
pub(super) fn check_maturation(engine: &SimEngine) -> Result<(), String> {
    if let Some(rule) = engine.resources.rules.society.life_stage {
        for (_, (agent, child)) in engine.world.query::<(&Agent, &Child)>().iter() {
            if engine.current_tick()
                >= child
                    .born_tick
                    .saturating_add(rule.child_ticks)
                    .saturating_add(60)
            {
                return Err(format!(
                    "late child maturation: agent={} born={} current={} child_ticks={}",
                    agent.id,
                    child.born_tick,
                    engine.current_tick(),
                    rule.child_ticks
                ));
            }
        }
    }
    Ok(())
}

/// Strong additive v9 comparison state, kept separate from the frozen fingerprint.
pub(super) fn trace(engine: &SimEngine) -> Value {
    let mut agents: Vec<_> = engine
        .world
        .query::<(
            &Agent,
            Option<&Child>,
            Option<&Life>,
            Option<&Elder>,
            Option<&SeekTarget>,
        )>()
        .iter()
        .map(|(_, (agent, child, life, elder, seek))| {
            (
                agent.id,
                child.map(|child| child.born_tick),
                life.map(|life| (life.born_tick, life.lifespan_ticks)),
                elder.is_some(),
                seek.map(|seek| seek.tile),
            )
        })
        .collect();
    agents.sort_by_key(|row| row.0);
    json!({"agents":agents,"beds":engine.resources.bed_of.iter().collect::<Vec<_>>(),
        "owners":engine.resources.building_owner.iter().collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maturation_completed_tick_bounds_and_death_are_distinct() {
        use sim_core::{material::MaterialRegistry, rules::LifeStageRule};
        // Due=10. Removal is valid at completed ticks 11..=70; a child
        // remaining at completed 70 is already late. Death is not maturation.
        for (completed, remove, death, valid) in [
            (10, true, false, false),
            (11, true, false, true),
            (70, true, false, true),
            (71, true, false, false),
            (69, false, false, true),
            (70, false, false, false),
            (11, false, true, true),
        ] {
            let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
            engine.resources.rules.society.life_stage = Some(LifeStageRule {
                child_ticks: 10,
                hunger_rate: 0.03,
                thirst_rate: 0.048,
                sleep_rate: 0.03,
                tether_radius: 3,
            });
            let entity = engine.spawn_agent(1, 1);
            engine
                .world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child");
            let mut monitor = MaturationMonitor::default();
            monitor.observe(&engine).expect("initial child");
            for _ in 0..completed {
                engine.tick();
            }
            if remove {
                engine.world.remove_one::<Child>(entity).expect("mature");
            }
            if death {
                engine.world.despawn(entity).expect("death");
            }
            let result = monitor.observe(&engine);
            assert_eq!(
                result.is_ok(),
                valid,
                "tick={completed} remove={remove} death={death}"
            );
            if valid {
                assert_eq!(monitor.matured, u64::from(remove));
                assert_eq!(monitor.died, u64::from(death));
            }
        }
    }

    #[test]
    fn owned_beds_children_food_and_idle_hearth_have_exact_values() {
        use sim_core::{
            components::{ResourceKind, Settlement},
            material::MaterialRegistry,
            rules::WorldRules,
        };
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        // Only the baseline's shelter rule (8 per shelter); no sustain rule
        // (reporting falls back to 0.85 utilization, radius 16), no seasons and
        // no life-stage rule (the child below matures at tick 0).
        engine.resources.rules = WorldRules::default();
        engine.resources.rules.society.shelter_planning =
            WorldRules::baseline(0).society.shelter_planning;
        let adult = engine.spawn_agent(10, 10);
        let child = engine.spawn_agent(11, 10);
        engine
            .world
            .insert(adult, (Hunger::new(0.0, 0.05), AgentState::Idle))
            .expect("adult");
        engine
            .world
            .insert(
                child,
                (
                    Child { born_tick: 0 },
                    Hunger::new(0.0, 0.03),
                    AgentState::Consuming {
                        target: TargetKind::Sleep,
                    },
                ),
            )
            .expect("child");
        let aid = engine.world.get::<&Agent>(adult).expect("adult id").id;
        let cid = engine.world.get::<&Agent>(child).expect("child id").id;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (10, 10);
        settlement.add_member_agent(aid);
        settlement.add_member_agent(cid);
        settlement.store(ResourceKind::Food, 20);
        engine.resources.settlements.insert(1, settlement);
        let mut overlap = Settlement::new_with_id(2, 0);
        overlap.formation_tile = (10, 11);
        engine.resources.settlements.insert(2, overlap);
        engine
            .resources
            .building_kinds
            .insert((10, 10), BuildingKind::Hearth);
        engine
            .resources
            .building_kinds
            .insert((11, 10), BuildingKind::Shelter);
        engine.resources.building_owner.insert((11, 10), 1);
        engine.resources.bed_of.insert(cid, (11, 10));
        engine.resources.food_source_max.insert((12, 10), 5);
        engine.resources.tallies.food_harvested = 7;
        engine.resources.tallies.meals_eaten = 8;
        engine.resources.tallies.meals_without_food = 2;
        let row = metrics(&engine);
        for (key, want) in [
            ("adults", 1),
            ("children", 1),
            ("housing_capacity_owned", 8),
            ("beds_assigned", 1),
            ("sleeping_in_shelter", 1),
            ("idle_adults", 1),
            ("idle_adults_near_hearth", 1),
            ("food_harvested_total", 7),
            ("meals_eaten_total", 8),
            ("meals_without_food_total", 2),
            ("sustainable_adults", 4),
        ] {
            assert_eq!(row[key], want, "{key}");
        }
        assert_eq!(row["food_supply_per_day"], 12.0); // Both catchments include the same bush.
        assert!((row["food_need_per_day"].as_f64().expect("need") - 3.84).abs() < 1e-5);
        let mut monitor = MaturationMonitor::default();
        monitor.observe(&engine).expect("initial");
        engine.world.remove_one::<Child>(child).expect("mature");
        monitor.observe(&engine).expect("matured");
        assert_eq!(monitor.report()["children_observed"], 1);
        assert_eq!(monitor.report()["maturations_observed"], 1);
    }

    #[test]
    fn reporting_is_additive_finite_and_partitions_population() {
        let engine = crate::scenes::baseline_scene(0);
        let row = metrics(&engine);
        assert_eq!(row.as_object().expect("object").len(), 15);
        assert!(row
            .as_object()
            .expect("object")
            .values()
            .all(|v| !v.is_null()));
        // Tick 0: every founder (the world's drawn `starting_agents`) is an adult.
        assert_eq!(
            row["adults"],
            sim_core::variety::WorldVariety::draw_v18(0).starting_agents
        );
        assert_eq!(row["children"], 0);
        for key in [
            "housing_capacity_owned",
            "food_supply_per_day",
            "food_need_per_day",
            "sustainable_adults",
            "food_harvested_total",
            "meals_eaten_total",
            "meals_without_food_total",
            "beds_assigned",
            "sleeping_in_shelter",
            "idle_adults",
            "idle_adults_near_hearth",
        ] {
            assert!(row[key].is_number(), "{key}");
        }
        assert_eq!(row["sample_kind"], "periodic");
        assert_eq!(row["minute_of_day"], 480);
        assert!(check_maturation(&engine).is_ok());
        assert_eq!(trace(&engine), trace(&crate::scenes::baseline_scene(0)));
    }
}

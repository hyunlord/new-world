//! Basic need-consumption actions called by AgentDecisionSystem.
//! Mutates the existing source maps, needs, state and food tallies; no scheduling or ECS ownership.
use sim_core::components::{AgentState, Hunger, Sleep, Thirst};
use sim_core::rules::NightRestRule;
use sim_engine::{SimResources, RESOURCE_SOURCE_INFINITE};

use super::needs::{
    NeedAvailability, FATIGUE_CONSUME_AMOUNT, HUNGER_CONSUME_AMOUNT, THIRST_CONSUME_AMOUNT,
};

/// Resolve a ground meal, preserving strict admission and the tick-local food version.
pub(in crate::runtime) fn consume_food(
    resources: &mut SimResources,
    key: (u32, u32),
    state: &mut AgentState,
    hunger: Option<&mut Hunger>,
    food_version: &mut u64,
) {
    // This scalar-source action belongs to baseline; colony intake requires an item owner.
    if resources.rules.colony.is_some() {
        *state = AgentState::Idle;
        return;
    }
    // V7 Section 16-α0 — source guard: a tile at the
    // RESOURCE_SOURCE_INFINITE sentinel never depletes.
    // Finite tiles keep decrement-and-remove (Assertions
    // 16/17). Legacy rules retain unconditional relief; strict rules require food.
    let food_present = resources
        .food_tiles
        .get(&key)
        .is_some_and(|counter| *counter > 0);
    if resources.rules.agent.strict_consume && !food_present {
        *state = AgentState::Idle;
        return;
    }
    if let Some(counter) = resources.food_tiles.get_mut(&key) {
        *food_version += 1;
        if *counter != RESOURCE_SOURCE_INFINITE {
            resources.tallies.food_harvested += u64::from(*counter > 0);
            *counter = counter.saturating_sub(1);
            if *counter == 0 {
                resources.food_tiles.remove(&key);
            }
        }
    }
    if let Some(h) = hunger {
        h.value = (h.value - HUNGER_CONSUME_AMOUNT).max(0.0);
        resources.tallies.meals_eaten += 1;
        resources.tallies.meals_without_food += u64::from(!food_present);
        // Pass 5, display-only: a resolved ground meal.
        resources.tallies.ground_meals += 1;
    }
    *state = AgentState::Idle;
}

/// Resolve drinking with the legacy missing-source relief rule.
pub(in crate::runtime) fn consume_water(
    resources: &mut SimResources,
    key: (u32, u32),
    state: &mut AgentState,
    thirst: Option<&mut Thirst>,
) {
    // V7 Section 16-α0 — source guard (mirror of Food).
    if let Some(counter) = resources.water_tiles.get_mut(&key) {
        if *counter != RESOURCE_SOURCE_INFINITE {
            *counter = counter.saturating_sub(1);
            if *counter == 0 {
                resources.water_tiles.remove(&key);
            }
        }
    }
    if let Some(t) = thirst {
        t.value = (t.value - THIRST_CONSUME_AMOUNT).max(0.0);
    }
    *state = AgentState::Idle;
}

/// Resolve nighttime recovery or a legacy one-shot rest using the caller's night window.
#[allow(clippy::too_many_arguments)] // Explicit borrowed inputs preserve the existing decision boundary.
pub(in crate::runtime) fn consume_sleep(
    resources: &mut SimResources,
    key: (u32, u32),
    state: &mut AgentState,
    hunger: Option<&Hunger>,
    thirst: Option<&Thirst>,
    sleep: Option<&mut Sleep>,
    night_rest: Option<NightRestRule>,
    availability: NeedAvailability,
) {
    if let Some(rule) = night_rest {
        let urgent_hunger = availability.food
            && hunger
                .as_ref()
                .is_some_and(|need| f64::from(need.value) > rule.wake_preempt);
        let urgent_thirst = availability.water
            && thirst
                .as_ref()
                .is_some_and(|need| need.value > rule.wake_preempt);
        if urgent_hunger || urgent_thirst {
            *state = AgentState::Idle;
        } else if let Some(sleep) = sleep {
            sleep.fatigue = (sleep.fatigue - rule.recovery_per_tick).max(0.0);
        }
        return;
    }
    // V7 Section 16-α0 — source guard (mirror of Food).
    if let Some(counter) = resources.sleep_tiles.get_mut(&key) {
        if *counter != RESOURCE_SOURCE_INFINITE {
            *counter = counter.saturating_sub(1);
            if *counter == 0 {
                resources.sleep_tiles.remove(&key);
            }
        }
    }
    if let Some(s) = sleep {
        s.fatigue = (s.fatigue - FATIGUE_CONSUME_AMOUNT).max(0.0);
    }
    *state = AgentState::Idle;
}

#[cfg(test)]
mod m6a_tests {
    use super::*;
    use sim_core::{rules::WorldRules, MaterialRegistry};

    #[test]
    fn colony_scalar_food_cannot_produce_a_meal_but_baseline_still_can() {
        for colony in [false, true] {
            let mut res = SimResources::new(8, 8, MaterialRegistry::new());
            res.rules = if colony {
                WorldRules::colony(0)
            } else {
                WorldRules::baseline(0)
            };
            res.food_tiles.insert((2, 2), 3);
            let mut state = AgentState::Consuming {
                target: sim_core::components::TargetKind::Food,
            };
            let mut hunger = Hunger::new(70.0, 0.05);
            let mut version = 0;
            consume_food(
                &mut res,
                (2, 2),
                &mut state,
                Some(&mut hunger),
                &mut version,
            );
            assert_eq!(hunger.value, if colony { 70.0 } else { 40.0 });
            assert_eq!(res.food_tiles[&(2, 2)], if colony { 3 } else { 2 });
            assert_eq!(res.tallies.meals_eaten, u64::from(!colony));
            assert_eq!(version, u64::from(!colony));
        }
    }
}

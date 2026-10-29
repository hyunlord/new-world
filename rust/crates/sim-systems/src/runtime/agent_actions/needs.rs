//! Survival-need selection and the clock helpers of the decision system:
//! resource availability, the actionable-need order (hunger, thirst, fatigue),
//! minute windows, the per-person night-rest window, the active fatigue
//! threshold, and the need thresholds and consume amounts.

use sim_core::causal::DecisionReason;
use sim_core::components::{Hunger, Sleep, TargetKind, Temperament, Thirst};
use sim_core::rules::{NightRestRule, TemperamentRule};
use sim_engine::SimResources;

/// Resource presence used to select actionable survival needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeedAvailability {
    /// Whether a breached hunger need may seek food.
    pub food: bool,
    /// Whether a breached thirst need may seek water.
    pub water: bool,
    /// Whether a breached fatigue need may seek sleep.
    pub sleep: bool,
}

impl NeedAvailability {
    /// Snapshot availability once per tick; legacy ignores source absence.
    pub fn from_resources(resources: &SimResources) -> Self {
        if resources.rules.agent.need_fallthrough {
            Self {
                food: if resources.rules.colony.is_some() {
                    crate::runtime::food::choice::available(resources)
                } else {
                    resources.food_tiles.values().any(|amount| *amount > 0)
                },
                water: resources.water_tiles.values().any(|amount| *amount > 0),
                sleep: resources.sleep_tiles.values().any(|amount| *amount > 0),
            }
        } else {
            Self {
                food: resources.rules.colony.is_none()
                    || crate::runtime::food::choice::available(resources),
                water: true,
                sleep: true,
            }
        }
    }
}

/// Highest-priority breached available survival need: hunger, thirst, fatigue.
pub fn actionable_need(
    hunger: Option<&Hunger>,
    thirst: Option<&Thirst>,
    sleep: Option<&Sleep>,
    availability: NeedAvailability,
) -> Option<(TargetKind, DecisionReason)> {
    actionable_need_with_fatigue_threshold(hunger, thirst, sleep, availability, FATIGUE_THRESHOLD)
}

/// Highest-priority actionable need using the caller's active fatigue threshold.
pub fn actionable_need_with_fatigue_threshold(
    hunger: Option<&Hunger>,
    thirst: Option<&Thirst>,
    sleep: Option<&Sleep>,
    availability: NeedAvailability,
    fatigue_threshold: f64,
) -> Option<(TargetKind, DecisionReason)> {
    if availability.food && hunger.is_some_and(|need| need.value > HUNGER_THRESHOLD) {
        Some((TargetKind::Food, DecisionReason::HungerThresholdBreach))
    } else if availability.water && thirst.is_some_and(|need| need.value > THIRST_THRESHOLD) {
        Some((TargetKind::Water, DecisionReason::ThirstThresholdBreach))
    } else if availability.sleep && sleep.is_some_and(|need| need.fatigue > fatigue_threshold) {
        Some((TargetKind::Sleep, DecisionReason::FatigueThresholdBreach))
    } else {
        None
    }
}

/// Whether an integer minute lies in a half-open, possibly midnight-wrapping window.
pub const fn minute_in_window(minute: u64, start: u32, end: u32) -> bool {
    if start < end {
        minute >= start as u64 && minute < end as u64
    } else if start > end {
        minute >= start as u64 || minute < end as u64
    } else {
        false
    }
}

/// Whether the simulation clock currently lies within a specified minute window.
pub fn clock_in_window(resources: &SimResources, start: u32, end: u32) -> bool {
    resources.ticks_per_day != 0
        && minute_in_window(
            resources
                .current_tick
                .saturating_add(resources.rules.clock_offset_ticks)
                % resources.ticks_per_day,
            start,
            end,
        )
}

/// Pass 16 (`game_v20`, HA → bedtime): the night-rest rule in force for one
/// person at day tick `day_tick` (`(tick + clock_offset_ticks) %
/// ticks_per_day`; None when the day has no ticks): `rule` while the clock
/// lies in the person's own window — their bedtime `lean.bedtime(start, t)` to
/// the unchanged end — else None. Without a lean or a temperament the window
/// is the rule's own, i.e. the global filtered rule.
pub fn person_night_rest(
    rule: Option<NightRestRule>,
    lean: Option<TemperamentRule>,
    temperament: Option<&Temperament>,
    day_tick: Option<u64>,
) -> Option<NightRestRule> {
    let rule = rule?;
    let start = match (lean, temperament) {
        (Some(lean), Some(t)) => lean.bedtime(rule.start_minute, t),
        _ => rule.start_minute,
    };
    day_tick
        .is_some_and(|minute| minute_in_window(minute, start, rule.end_minute))
        .then_some(rule)
}

/// Active fatigue threshold shared by simulation decisions and probe accounting.
pub fn fatigue_threshold_at(resources: &SimResources) -> f64 {
    resources
        .rules
        .agent
        .night_rest
        .filter(|rule| clock_in_window(resources, rule.start_minute, rule.end_minute))
        .map_or(FATIGUE_THRESHOLD, |rule| rule.fatigue_threshold)
}

/// Critical need level used by Pass 5 rules when a world sets no
/// `AgentRules::critical_preemption` (every Pass 5 world sets 75.0).
pub const CRITICAL_NEED: f64 = 75.0;

/// Hunger value strictly above which an Idle agent transitions to
/// `Seeking { Food }`. Phase 5-β scope: a flat constant. Per-agent
/// thresholds (temperament-driven) land in δ.
///
/// Typed `f32` to match [`Hunger::value`](sim_core::components::Hunger),
/// which itself remains `f32` per the locked α scope. Thirst-side
/// constants (and the rest of the new β math) use `f64` per the
/// project's "ALL f64 for simulation math" rule.
pub const HUNGER_THRESHOLD: f32 = 50.0;

/// Thirst value strictly above which an Idle agent transitions to
/// `Seeking { Water }` (only if Hunger has not already triggered).
///
/// `f64` to match the new β [`Thirst`] component and the project's
/// simulation-math determinism rule.
pub const THIRST_THRESHOLD: f64 = 50.0;

/// Amount subtracted from `Hunger.value` when a `Consuming { Food }`
/// step completes. Chosen so a single Consume cleanly drops a freshly
/// breached agent (value ≈ 51) back below `HUNGER_THRESHOLD`.
///
/// `f32` to match [`Hunger::value`](sim_core::components::Hunger).
pub const HUNGER_CONSUME_AMOUNT: f32 = 30.0;

/// Amount subtracted from `Thirst.value` when a `Consuming { Water }`
/// step completes. Mirrors `HUNGER_CONSUME_AMOUNT` (in `f64` to match
/// the new β [`Thirst`] component).
pub const THIRST_CONSUME_AMOUNT: f64 = 30.0;

/// Fatigue value strictly above which an Idle agent transitions to
/// `Seeking { Sleep }` (only if neither Hunger nor Thirst has already
/// triggered).
///
/// `f64` to match the new γ [`Sleep`] component (V7 Phase 5-γ / P5γ-8).
pub const FATIGUE_THRESHOLD: f64 = 50.0;

/// Amount subtracted from `Sleep.fatigue` when a `Consuming { Sleep }`
/// step completes. Mirrors `HUNGER_CONSUME_AMOUNT` / `THIRST_CONSUME_AMOUNT`
/// (in `f64` to match the new γ [`Sleep`] component, V7 Phase 5-γ / P5γ-8).
pub const FATIGUE_CONSUME_AMOUNT: f64 = 30.0;

/// Loneliness value strictly above which an Idle agent transitions to
/// `Seeking { Agent(other) }` (only if no higher-priority drive has
/// already triggered). V7 Phase 7-β / P7β-14.
///
/// `f64` to match the [`Social`] component.
pub const SOCIAL_THRESHOLD: f64 = 50.0;

/// Amount subtracted from `Social.loneliness` when a multi-tick social
/// interaction completes. Mirrors `HUNGER_CONSUME_AMOUNT` style.
/// V7 Phase 7-β / P7β-14.
pub const SOCIAL_CONSUME_AMOUNT: f64 = 30.0;

/// Number of ticks of mutual `Consuming { Agent(other) }` required before
/// a `SocialInteractionCompleted` is emitted. V7 Phase 7-β / P7β-14.
pub const REQUIRED_INTERACTION_PROGRESS: u32 = 3;

/// Amount added to `RelationshipState::familiarity` (saturating at 1.0)
/// on each completed interaction. V7 Phase 7-β / P7β-14.
pub const FAMILIARITY_BUMP: f64 = 0.1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn person_night_rest_is_the_own_window() {
        let rule = sim_core::rules::WorldRules::baseline(0).agent.night_rest;
        let lean = sim_core::rules::WorldRules::baseline(0).agent.temperament;
        let cautious = Temperament {
            ns: 500,
            ha: 1000,
            rd: 500,
            p: 500,
        };
        let bold = Temperament {
            ns: 500,
            ha: 0,
            rd: 500,
            p: 500,
        };
        // 20:30 — only the cautious (bedtime 20:00) are in their night.
        assert!(person_night_rest(rule, lean, Some(&cautious), Some(1230)).is_some());
        assert!(person_night_rest(rule, lean, Some(&bold), Some(1230)).is_none());
        assert!(person_night_rest(rule, lean, None, Some(1230)).is_none());
        // 21:30 — the bold (bedtime 22:00) are still up; the mean person is not.
        assert!(person_night_rest(rule, lean, Some(&bold), Some(1290)).is_none());
        assert!(person_night_rest(rule, lean, None, Some(1290)).is_some());
        // 22:00 to 06:00 everyone; the end is unchanged.
        for minute in [1320, 0, 359] {
            assert!(person_night_rest(rule, lean, Some(&bold), Some(minute)).is_some());
            assert!(person_night_rest(rule, lean, Some(&cautious), Some(minute)).is_some());
        }
        assert!(person_night_rest(rule, lean, Some(&cautious), Some(360)).is_none());
        // No day ticks, no rule: never.
        assert!(person_night_rest(rule, lean, Some(&cautious), None).is_none());
        assert!(person_night_rest(None, lean, Some(&cautious), Some(0)).is_none());
    }
}

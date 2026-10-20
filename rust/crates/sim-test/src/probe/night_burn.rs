//! Additive schema-eight observations: the night's burn and the day's winter fall.
use serde_json::{json, Value};
use sim_core::rules::WorldRules;
use sim_engine::{SimEngine, SimResources};
use sim_systems::runtime::settlement::firewood::{night_rate, winter_fall_at};

/// FireSystem's run interval; its runs fall on multiples of this tick.
const FIRE_INTERVAL: u64 = 60;

/// Schema-eight keys; every value is finite.
///
/// `night_burn_per_hour`: the rate `FireSystem` used at its last run (system
/// tick rounded down to a multiple of 60), 0 outside the burn window or
/// without the rule (`firewood::night_rate`). `winter_fall_today`: the winter
/// deadwood fall per day in force at the system tick, -1 outside a winter or
/// without the winter spread regrowth (`firewood::winter_fall_at`).
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let (burn, fall) = values(engine);
    json!({"night_burn_per_hour": burn, "winter_fall_today": fall})
}

/// `(night_burn_per_hour, winter_fall_today)` as [`metrics`] reports them; the
/// recorder reads the same function, so its hidden columns equal the probe keys.
pub(crate) fn values(engine: &SimEngine) -> (u32, i64) {
    let r = &engine.resources;
    r.rules.society.firewood.map_or((0, -1), |rule| {
        let last_run = r.current_tick - r.current_tick % FIRE_INTERVAL;
        (
            night_rate(r, &rule, last_run),
            winter_fall_at(r, &rule, r.current_tick).map_or(-1, i64::from),
        )
    })
}

/// The `rules` block of `summary.json`: the firewood rule's severity weights,
/// burns and falls and the Pass 8 fields, so evaluators read them from the run
/// (null without the firewood rule); schema 9 adds `keyed_draws` (0/1) and
/// `noise_replicate` (the replicate in force at the engine's tick, 0 unless a
/// noise fork set it).
pub(super) fn rules_block(resources: &SimResources) -> Value {
    let rules: &WorldRules = &resources.rules;
    json!({
        "keyed_draws": u8::from(rules.agent.keyed_draws),
        "noise_replicate": resources.noise_replicate_for(resources.current_tick),
        "firewood": rules.society.firewood.map(|rule| json!({
        "winter_severity_permille": rule.winter_severity_permille,
        "winter_severity_burn_per_hour": rule.winter_severity_burn_per_hour,
        "winter_severity_regen_units_per_day": rule.winter_severity_regen_units_per_day,
        "winter_night_burn_permille": rule.winter_night_burn_permille,
        "winter_night_burn_step": rule.winter_night_burn_step,
        "winter_fall_spread": rule.winter_fall_spread,
        "windfall_over_snow": rule.windfall_over_snow,
        "burn_hours": sim_systems::runtime::settlement::firewood::burn_hours(&rule),
    }))})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_world_reports_night_burn_values_outside_winter() {
        let mut engine = crate::scenes::baseline_scene(3);
        let row = metrics(&engine);
        assert_eq!(row.as_object().expect("object").len(), 2);
        // Tick 0 is 08:00 of the first spring day: outside the burn window.
        assert_eq!(row["night_burn_per_hour"], 0);
        assert_eq!(row["winter_fall_today"], -1);
        // 20:00 of day one: the rule's spring burn per hour.
        for _ in 0..(12 * 60 + 1) {
            engine.tick();
        }
        let rule = engine.resources.rules.society.firewood.expect("firewood");
        let spring = sim_systems::runtime::settlement::firewood::night_rate(
            &engine.resources,
            &rule,
            engine.resources.current_tick - engine.resources.current_tick % 60,
        );
        assert_eq!(spring, u32::from(rule.burn_per_hour[0])); // 2 per hour in spring.
        assert_eq!(metrics(&engine)["night_burn_per_hour"], spring);
        // Without the firewood rule both keys are neutral.
        let bare = SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
        assert_eq!(
            metrics(&bare),
            json!({"night_burn_per_hour": 0, "winter_fall_today": -1})
        );
    }

    #[test]
    fn winter_fall_and_rules_block_read_the_rule() {
        let mut engine = crate::scenes::baseline_scene(0);
        let days = engine
            .resources
            .rules
            .seasons
            .expect("seasons")
            .days_per_season;
        // 12:00 of the first winter day (display day 3 x days_per_season; tick =
        // day x 1440 + 720 - 480).
        engine.resources.current_tick = u64::from(3 * days) * 1440 + 240;
        let level = sim_systems::runtime::settlement::firewood::winter_severity_at(
            &engine.resources,
            engine.resources.current_tick,
        );
        let rule = WorldRules::baseline(0)
            .society
            .firewood
            .expect("baseline firewood");
        let want =
            rule.winter_severity_regen_units_per_day[usize::try_from(level).expect("winter")];
        assert_eq!(metrics(&engine)["winter_fall_today"], want);
        let block = rules_block(&crate::scenes::baseline_scene(0).resources);
        assert_eq!(
            block["firewood"]["winter_severity_permille"],
            json!(rule.winter_severity_permille)
        );
        assert_eq!(
            block["firewood"]["winter_night_burn_permille"],
            json!(rule.winter_night_burn_permille)
        );
        assert_eq!(
            block["firewood"]["burn_hours"],
            sim_systems::runtime::settlement::firewood::burn_hours(&rule)
        );
        assert_eq!(block["keyed_draws"], 1);
        assert_eq!(block["noise_replicate"], 0);
        let bare = SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
        assert_eq!(
            rules_block(&bare.resources),
            json!({"keyed_draws": 0, "noise_replicate": 0, "firewood": null})
        );
        engine.apply_noise_fork(3);
        assert_eq!(rules_block(&engine.resources)["noise_replicate"], 3);
    }
}

//! Per-world interpretation of validated selector fields.
use super::*;
use crate::rules::{StoryRule, WildlifeRule};
impl RuleU32 {
    /// Resolve this validated selector against the current world.
    pub fn story(&self, rule: &StoryRule) -> u32 {
        match self.rule.as_deref() {
            Some("first_winter_mercy_days") => rule.first_winter_mercy_days,
            Some("mercy_day") => rule.mercy_day,
            Some("beasts_gap_days") => rule.beasts_gap_days,
            Some("winter_end_mercy_days") => rule.winter_end_mercy_days,
            Some("illness_day") => rule.illness_day,
            Some("cold_due_days") => rule.cold_due_days,
            Some("beasts_due_days") => rule.beasts_due_days,
            Some("big_colony_adults") => rule.big_colony_adults,
            Some("wolves_base") => rule.wolves_base,
            Some("wolves_per_items") => rule.wolves_per_items,
            Some("wolves_max") => rule.wolves_max,
            Some("snap_burn_permille") => rule.snap_burn_permille,
            Some("snap_exposure_permille") => rule.snap_exposure_permille,
            Some("snap_outdoor_work_permille") => rule.snap_outdoor_work_permille,
            Some("drought_regen_permille") => rule.drought_regen_permille,
            Some("drought_thirst_permille") => rule.drought_thirst_permille,
            Some("sick_work_permille") => rule.sick_work_permille,
            Some("sick_sleep_permille") => rule.sick_sleep_permille,
            Some("sick_spread_permille") => rule.sick_spread_permille,
            Some("sick_days") => rule.sick_days,
            Some("beasts_max_days") => rule.beasts_max_days,
            _ => self.value,
        }
    }
}
impl RuleU32 {
    /// Resolve this validated selector against the current world.
    pub fn wildlife(&self, rule: &WildlifeRule) -> u32 {
        match self.rule.as_deref() {
            Some("deer_flee_tiles") => rule.deer_flee_tiles,
            Some("deer_flee_speed") => rule.deer_flee_speed,
            Some("deer_flee_steps") => rule.deer_flee_steps,
            Some("deer_rest_steps") => rule.deer_rest_steps,
            Some("deer_home_tiles") => rule.deer_home_tiles,
            Some("deer_min_hearth_distance") => rule.deer_min_hearth_distance,
            Some("wolf_sight") => rule.wolf_sight,
            Some("wolf_chase_tiles") => rule.wolf_chase_tiles,
            Some("hit_ticks") => rule.hit_ticks,
            _ => self.value,
        }
    }
}
impl RuleU16 {
    /// Resolve this validated selector against the current world.
    pub fn wildlife(&self, rule: &WildlifeRule) -> u16 {
        match self.rule.as_deref() {
            Some("deer_hp") => rule.deer_hp,
            Some("wolf_hp") => rule.wolf_hp,
            Some("wolf_damage") => rule.wolf_damage,
            Some("meat_with_knife") => rule.meat_with_knife,
            Some("meat_without_knife") => rule.meat_without_knife,
            _ => self.value,
        }
    }
}
impl RuleBool {
    /// Resolve this validated selector against the current world.
    pub fn wildlife(&self, rule: &WildlifeRule) -> bool {
        match self.rule.as_deref() {
            Some("deer_flee") => rule.deer_flee,
            Some("wolf_hides") => rule.wolf_hides,
            _ => self.value,
        }
    }
}
impl RuleSpan {
    /// Resolve duration without consuming a keyed draw.
    pub fn story(&self, rule: &StoryRule) -> (u32, u32) {
        match self.rule.as_deref() {
            Some("cold_snap_days") => rule.cold_snap_days,
            Some("drought_days") => rule.drought_days,
            Some("sick_days") => (rule.sick_days, rule.sick_days),
            Some("beasts_max_days") => (rule.beasts_max_days, rule.beasts_max_days),
            _ => self.value,
        }
    }
}
impl PressureDef {
    /// Whether the current per-world mechanic switch permits this pressure.
    pub fn enabled(&self, rule: &StoryRule) -> bool {
        match self.enabled_rule.as_deref() {
            Some("illness") => rule.illness,
            Some("beasts") => rule.beasts,
            _ => true,
        }
    }
    /// Duration bounds before the caller's existing keyed draw.
    pub fn duration_span(&self, rule: &StoryRule) -> (u32, u32) {
        self.duration.story(rule)
    }
    /// Project selected effects while preserving unrelated per-world policy.
    pub fn resolve_rule(&self, rule: &StoryRule) -> StoryRule {
        let mut out = *rule;
        if let Some(e) = &self.cold {
            out.snap_burn_permille = e.burn.story(rule);
            out.snap_exposure_permille = e.exposure.story(rule);
            out.snap_outdoor_work_permille = e.outdoor_work.story(rule);
        }
        if let Some(e) = &self.drought {
            out.drought_regen_permille = e.regrowth.story(rule);
            out.drought_thirst_permille = e.thirst.story(rule);
        }
        if let Some(e) = &self.illness {
            out.sick_work_permille = e.work.story(rule);
            out.sick_sleep_permille = e.sleep.story(rule);
            out.sick_spread_permille = e.spread.story(rule);
            out.sick_days = self.duration.story(rule).0;
        }
        if self.pack.is_some() {
            out.beasts_max_days = self.duration.story(rule).0;
        }
        out
    }
}
impl AnimalDef {
    /// Project species behavior onto the current world mechanics rule.
    pub fn resolve_rule(&self, rule: &WildlifeRule) -> WildlifeRule {
        let mut out = *rule;
        match self.mechanic {
            AnimalMechanic::Grazer => out.deer_hp = self.hp.wildlife(rule),
            AnimalMechanic::Predator => out.wolf_hp = self.hp.wildlife(rule),
        }
        if let Some(g) = &self.grazer {
            out.deer_flee = g.flee.wildlife(rule);
            out.deer_flee_tiles = g.flee_tiles.wildlife(rule);
            out.deer_flee_speed = g.flee_speed.wildlife(rule);
            out.deer_flee_steps = g.flee_steps.wildlife(rule);
            out.deer_rest_steps = g.rest_steps.wildlife(rule);
            out.deer_home_tiles = g.home_tiles.wildlife(rule);
            out.deer_min_hearth_distance = g.min_hearth_distance.wildlife(rule);
        }
        if let Some(p) = &self.predator {
            out.wolf_sight = p.sight.wildlife(rule);
            out.wolf_damage = p.damage.wildlife(rule);
            out.wolf_chase_tiles = p.chase_tiles.wildlife(rule);
            out.hit_ticks = p.hit_ticks.wildlife(rule);
        }
        if let Some(m) = &self.loot.meat {
            out.meat_with_knife = m.with_cutter.wildlife(rule);
            out.meat_without_knife = m.without_cutter.wildlife(rule);
        }
        if let Some(h) = &self.loot.hide {
            out.wolf_hides = h.enabled.wildlife(rule);
        }
        out
    }
}

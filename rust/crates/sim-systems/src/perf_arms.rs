//! Feature-gated decision-arm observations; inclusive timings never affect simulation state.
//! Drivers drain on the simulation thread at world boundaries and after each tick.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

/// Inclusive elapsed time and execution or explicitly named cardinality counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeTotal {
    /// Inclusive elapsed nanoseconds; nested scopes must not be summed.
    pub ns: u64,
    /// Scope entries, or units for `count.*` keys; these are not unique person-ticks.
    pub calls: u64,
}

thread_local! {
    static TOTALS: RefCell<BTreeMap<&'static str, ScopeTotal>> = const { RefCell::new(BTreeMap::new()) };
}

/// Records inclusive elapsed time even when a branch returns or continues early.
#[derive(Debug)]
pub struct Scope {
    name: &'static str,
    started: Instant,
}

impl Scope {
    /// Begin an observational scope, without borrowing any simulation data.
    pub fn start(name: &'static str) -> Self {
        Self {
            name,
            started: Instant::now(),
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        let ns = u64::try_from(self.started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        TOTALS.with(|totals| {
            let mut totals = totals.borrow_mut();
            let total = totals.entry(self.name).or_default();
            total.ns = total.ns.saturating_add(ns);
            total.calls = total.calls.saturating_add(1);
        });
    }
}

/// Add a cardinality or outcome count without manufacturing elapsed time.
pub fn count(name: &'static str, n: u64) {
    TOTALS.with(|totals| {
        let mut totals = totals.borrow_mut();
        let total = totals.entry(name).or_default();
        total.calls = total.calls.saturating_add(n);
    });
}

/// Record a selected need; non-need targets do not increment any need counter.
pub fn count_need(target: sim_core::components::TargetKind, names: [&'static str; 3]) {
    use sim_core::components::TargetKind;
    let name = match target {
        TargetKind::Food => Some(names[0]),
        TargetKind::Water => Some(names[1]),
        TargetKind::Sleep => Some(names[2]),
        TargetKind::GatherFood
        | TargetKind::GatherWood
        | TargetKind::ConstructionSite
        | TargetKind::Agent(_)
        | TargetKind::Activity
        | TargetKind::Job => None,
    };
    if let Some(name) = name {
        count(name, 1);
    }
}

/// Drain this thread's observations. Drivers fill missing keys from ALL_ARMS at export.
/// Call only when no observation scopes are alive (normally after engine.tick()).
pub fn take() -> BTreeMap<&'static str, ScopeTotal> {
    TOTALS.with(|totals| std::mem::take(&mut *totals.borrow_mut()))
}

/// Complete scope registry, including arms that never fire in a run.
pub const ALL_ARMS: &[&str] = &[
    "count.social.natural_candidate_success",
    "count.cascade_committed.target.food",
    "count.cascade_committed.target.water",
    "count.cascade_committed.target.sleep",
    "count.cascade_committed.target.gather_food",
    "count.cascade_committed.target.gather_wood",
    "count.cascade_committed.target.construction",
    "count.cascade_committed.target.agent",
    "count.cascade_committed.target.activity",
    "count.cascade_committed.target.job",
    "count.cascade_committed.reason.hunger",
    "count.cascade_committed.reason.thirst",
    "count.cascade_committed.reason.fatigue",
    "count.cascade_committed.reason.construction",
    "count.cascade_committed.reason.social",
    "count.cascade_committed.reason.memory",
    "count.cascade_committed.reason.combat",
    "count.cascade_committed.reason.settlement",
    "count.evaluation.primary",
    "count.evaluation.replay",
    "count.natural_committed.combat",
    "count.natural_committed.construction",
    "count.natural_committed.fatigue",
    "count.natural_committed.hunger",
    "count.natural_committed.memory",
    "count.natural_committed.settlement",
    "count.natural_committed.social",
    "count.natural_committed.thirst",
    "count.need.committed.food",
    "count.need.committed.sleep",
    "count.need.committed.water",
    "count.need.natural_candidate.food",
    "count.need.natural_candidate.sleep",
    "count.need.natural_candidate.water",
    "count.people.drafted",
    "count.people.fleeing",
    "count.seam.nonurgent",
    "count.seam.urgent",
    "eval.arrival.job",
    "eval.arrival.activity",
    "eval.arrival.food",
    "eval.arrival.gather_food",
    "eval.arrival.gather_wood",
    "eval.arrival.water",
    "eval.arrival.sleep",
    "eval.arrival.construction",
    "eval.arrival.agent",
    "selected.consume_job_noop",
    "selected.post_job_preserved",
    "eval.forage_selector",
    "eval.firewood_selector",
    "eval.job_hold",
    "eval.job_skip",
    "eval.founder_idle",
    "eval.porter_presence",
    "eval.meal_wait",
    "eval.social_chase_timeout",
    "eval.social_partner_exists",
    "eval.migration_exit",
    "eval.migration_need",
    "eval.memory_flip_margin",
    "eval.memory_combat",
    "eval.arrival",
    "eval.seam.critical",
    "eval.seam.crafting",
    "eval.seam.finish_leg",
    "eval.seam.restore",
    "eval.seam.stale",
    "eval.actionable_need_with_fatigue_threshold.L1144",
    "eval.actionable_need_with_fatigue_threshold.L1166",
    "eval.actionable_need_with_fatigue_threshold.L1185",
    "eval.actionable_need_with_fatigue_threshold.L1395",
    "eval.actionable_need_with_fatigue_threshold.L1406",
    "eval.actionable_need_with_fatigue_threshold.L1417",
    "eval.actionable_need_with_fatigue_threshold.L1904",
    "eval.actionable_need_with_fatigue_threshold.L1921",
    "eval.actionable_need_with_fatigue_threshold.L1977",
    "eval.actionable_need_with_fatigue_threshold.L2043",
    "eval.actionable_need_with_fatigue_threshold.L944",
    "eval.activity_interrupt",
    "eval.bush_exists.L984",
    "eval.choose_bush.L1667",
    "eval.choose_bush.L2951",
    "eval.choose_spot.L1710",
    "eval.choose_spot.L2978",
    "eval.gather_tile_exists.L1084",
    "eval.memory_weight_delta.L1260",
    "eval.memory_weight_delta.L1511",
    "eval.memory_weight_delta.L1517",
    "eval.nearest_gather_tile.L1761",
    "eval.nearest_gather_tile.L2735",
    "eval.nearest_gather_tile.L2840",
    "eval.nearest_local_food.L2851",
    "eval.nearest_resource_tile.L2823",
    "eval.nearest_resource_tile.L2831",
    "eval.nearest_resource_tile.L2856",
    "eval.spot_exists.L1035",
    "phase.apply_seek",
    "phase.arrival_predicate",
    "phase.construction_snapshot",
    "phase.forage_retarget",
    "phase.job_holders",
    "phase.job_settle",
    "phase.life_stages",
    "phase.local_food_snapshot",
    "phase.memory_eligibility",
    "phase.memory_score",
    "phase.migrant_markers",
    "phase.person_evaluation",
    "phase.person_thresholds",
    "phase.porter_snapshot",
    "phase.positions",
    "phase.post",
    "phase.post_positions",
    "phase.preempt_clear",
    "phase.prepare",
    "phase.quarrel_chronicle",
    "phase.replay_collect",
    "phase.round",
    "phase.routine_setup",
    "phase.social_retry",
    "phase.social_snapshots",
    "phase.tick",
    "phase.wood_retarget",
    "seam.craft_locked",
    "seam.critical",
    "seam.drafted_snapshot",
    "seam.finish_leg_locked",
    "seam.fleeing_snapshot",
    "seam.hold_for_decision",
    "seam.holders",
    "seam.may_yield",
    "seam.no_colony",
    "seam.no_margin",
    "seam.release",
    "seam.release_key",
    "seam.restore",
    "seam.settle",
    "seam.settle_person",
    "seam.skips_decision",
    "seam.stale_job",
    "selected.activity_arrival",
    "selected.activity_interrupt",
    "selected.activity_interrupt_check",
    "selected.activity_seek_end",
    "selected.activity_seeking",
    "selected.arrival",
    "selected.breached",
    "selected.builder_continuation",
    "selected.builder_planning",
    "selected.combat",
    "selected.combat_eligible",
    "selected.combat_event",
    "selected.construction_arrival",
    "selected.construction_natural",
    "selected.consume_activity",
    "selected.consume_activity_end",
    "selected.consume_agent_noop",
    "selected.consume_construction_noop",
    "selected.consume_food",
    "selected.consume_gather_food",
    "selected.consume_gather_wood",
    "selected.consume_sleep",
    "selected.consume_water",
    "selected.consuming",
    "selected.dispute_daily_cap",
    "selected.dispute_day_reset",
    "selected.dispute_uncosted",
    "selected.firewood",
    "selected.forage",
    "selected.forage_planning",
    "selected.forage_retarget",
    "selected.forage_retarget_found",
    "selected.founder_continuation",
    "selected.gather",
    "selected.gather_already_quarrelled",
    "selected.gather_critical_preempt",
    "selected.gather_depleted",
    "selected.gather_dispute",
    "selected.gather_finite",
    "selected.gather_infinite",
    "selected.gather_relations",
    "selected.gather_rival",
    "selected.gather_room",
    "selected.gather_source",
    "selected.idle_cascade",
    "selected.idle_founder",
    "selected.idle_porter",
    "selected.invitation",
    "selected.job_hold",
    "selected.job_skip",
    "selected.meal_snapshot",
    "selected.meal_wait",
    "selected.meal_wait_check",
    "selected.memory_construction_eligible",
    "selected.memory_flip",
    "selected.memory_hunger_eligible",
    "selected.memory_only_construction",
    "selected.memory_only_social",
    "selected.memory_sleep_eligible",
    "selected.memory_social_eligible",
    "selected.memory_thirst_eligible",
    "selected.migration_exit",
    "selected.migration_exit_check",
    "selected.migration_fallback",
    "selected.migration_need_preempt",
    "selected.migration_nonmember",
    "selected.migration_started",
    "selected.natural_choice",
    "selected.need_crossover",
    "selected.need_crossover_check",
    "selected.need_natural",
    "selected.porter_continuation",
    "selected.post_activity",
    "selected.post_agent",
    "selected.post_agent_missing",
    "selected.post_builder",
    "selected.post_clear",
    "selected.post_founder",
    "selected.post_gather_food",
    "selected.post_gather_food_empty",
    "selected.post_gather_wood",
    "selected.post_gather_wood_empty",
    "selected.post_porter",
    "selected.post_resource",
    "selected.post_resource_resolve",
    "selected.post_wood_claim",
    "selected.quarrel",
    "selected.quarrel_capped",
    "selected.relation_dispute_loss",
    "selected.relation_quarrel_loss",
    "selected.relations_snapshot",
    "selected.routine_entry",
    "selected.routine_finish",
    "selected.routine_started",
    "selected.seeking",
    "selected.social_arrival",
    "selected.social_chase_check",
    "selected.social_natural",
    "selected.social_need_preempt",
    "selected.social_partner_missing",
    "selected.social_tag",
    "selected.social_timeout",
    "selected.wood_pickup",
    "selected.wood_planning",
    "selected.wood_retarget",
    "selected.wood_retarget_found",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn early_return_records_scope_and_drain_resets_counts() {
        // Given an empty observation accumulator.
        let _ = take();
        fn observed_return() {
            let _scope = Scope::start("test.return");
        }
        // When a function exits with its scope alive.
        observed_return();
        let totals = take();
        // Then it records exactly once and the next drain has no stale calls.
        assert_eq!(totals["test.return"].calls, 1);
        assert!(take().values().all(|total| total.calls == 0));
    }

    #[test]
    fn nested_scopes_count_independently_without_materializing_unhit_rows() {
        // Given a clean accumulator.
        let _ = take();
        // When nested scopes complete.
        {
            let _outer = Scope::start("test.outer");
            let _inner = Scope::start("test.inner");
        }
        let totals = take();
        // Then inclusive parent time covers the child and unhit arms stay in the registry.
        assert_eq!(totals["test.outer"].calls, 1);
        assert_eq!(totals["test.inner"].calls, 1);
        assert!(totals["test.outer"].ns >= totals["test.inner"].ns);
        assert!(ALL_ARMS.iter().all(|name| !totals.contains_key(name)));
    }
    #[test]
    fn cardinality_counts_preserve_units_and_never_add_elapsed_time() {
        // Given empty observations and three need categories.
        let _ = take();
        let names = ["test.food", "test.water", "test.sleep"];
        // When observing people and an actual selected need.
        count("test.people", 7);
        count_need(sim_core::components::TargetKind::Water, names);
        count_need(sim_core::components::TargetKind::Job, names);
        let totals = take();
        // Then cardinalities are exact, non-needs do not masquerade as needs, and ns stays zero.
        assert_eq!(totals["test.people"], ScopeTotal { ns: 0, calls: 7 });
        assert_eq!(totals["test.water"], ScopeTotal { ns: 0, calls: 1 });
        assert!(!totals.contains_key("test.food"));
        assert!(!totals.contains_key("test.sleep"));
    }
    #[test]
    fn outcome_observer_evaluates_once_and_counts_only_the_actual_result() {
        // Given a fresh accumulator and an observable predicate evaluation.
        let _ = take();
        let mut evaluations = 0;
        // When an unsuccessful predicate is observed.
        let result = arm_outcome!("test.yes", "test.no", {
            evaluations += 1;
            false
        });
        let totals = take();
        // Then no duplicate evaluation or positive outcome is introduced.
        assert!(!result);
        assert_eq!(evaluations, 1);
        assert_eq!(totals["test.no"], ScopeTotal { ns: 0, calls: 1 });
        assert!(!totals.contains_key("test.yes"));
    }
}

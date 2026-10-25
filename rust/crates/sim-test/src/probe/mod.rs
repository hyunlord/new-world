//! Deterministic headless instrumentation; none of these measurements feeds simulation.
pub(crate) mod adult_work;
mod agent_trace;
pub(crate) mod area_floor;
mod args;
pub mod ascii;
pub mod b4;
pub mod chronicle_out;
pub mod churn;
pub mod colony;
mod deaths;
mod deaths_observer;
pub mod demography;
pub mod economy;
pub(crate) mod exposure_health;
mod family_events;
mod family_samples;
mod family_trace;
mod fingerprint;
pub(crate) mod fission_settlements;
/// M3 built space and rooms in a colony run (`m3` summary block).
pub mod knowledge;
mod long_run;
pub mod m5;
pub mod m55;
pub(crate) mod meals_firewood;
pub(crate) mod metrics;
mod mux2;
pub(crate) mod night_burn;
mod relation_caps;
mod relations_block;
mod routine_accounting;
mod routine_observer;
mod routine_trace;
mod runner;
mod seasons_forage;
pub mod space;
mod steward;
mod summary;
mod winter_severity;
mod wood_work;

pub use agent_trace::ProbeOptions;
pub use args::ProbeArgs;
pub use fingerprint::fingerprint;
pub(crate) use runner::scene_engine;
pub use runner::{run, run_with, run_with_chronicle};

#[cfg(test)]
mod tests {
    #[test]
    fn display_social_counter_does_not_change_the_fingerprint() {
        let mut engine = crate::scenes::baseline_scene(0);
        let before = super::fingerprint(&engine);
        engine.resources.tallies.social_completed = 9;
        assert_eq!(before, super::fingerprint(&engine));
        assert_eq!(
            super::metrics::sample(&engine, "baseline", 0, 0)["social_completed"],
            9
        );
    }

    #[test]
    fn schema_thirteen_is_exactly_additive() {
        // Schema 3 had 53 keys; schema 4 added the 15 Pass 4 keys; schema 5 the
        // 15 Pass 5 keys; schema 6 the 6 Pass 6 keys; schema 7 the 5 Pass 7
        // keys; schema 8 the 2 Pass 8 keys; schema 9 adds no row key (its two
        // keys live in the summary's rules block); schema 10 the 3 Pass 10 keys;
        // schema 11 the 8 Pass 11 keys; schema 12 the 14 Pass 12 keys; schema
        // 13 the 2 Pass 13 keys. V8-0 removed four god-hand keys (three of
        // Pass 7, `wet_burn_wood_total` of Pass 12), 0 in every natural run.
        let sample = super::metrics::sample(&crate::scenes::baseline_scene(0), "baseline", 0, 0);
        assert_eq!(sample["schema_version"], 13);
        assert_eq!(
            sample.as_object().expect("sample object").len(),
            53 + 15 + 15 + 6 + 5 + 2 + 3 + 8 + 14 + 2 - 4
        );
    }
}

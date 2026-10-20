//! Deterministic headless instrumentation; none of these measurements feeds simulation.
mod agent_trace;
mod args;
pub mod chronicle_out;
mod deaths;
mod deaths_observer;
mod family_events;
mod family_samples;
mod family_trace;
mod fingerprint;
mod long_run;
pub(crate) mod metrics;
mod pass4;
pub(crate) mod pass5;
mod pass6;
mod pass7;
pub(crate) mod pass8;
pub(crate) mod pass10;
pub(crate) mod pass11;
pub(crate) mod pass12;
pub(crate) mod pass13;
mod pass20;
mod pass21;
mod pass17;
mod pass17_activity;
mod routine_observer;
mod runner;
mod summary;

pub use agent_trace::ProbeOptions;
pub use args::ProbeArgs;
pub use fingerprint::fingerprint;
pub use runner::{run, run_with, run_with_chronicle};
pub(crate) use runner::scene_engine;

#[cfg(test)]
mod tests {
    #[test]
    fn display_social_counter_does_not_change_legacy_fingerprint() {
        let mut engine = crate::scenes::legacy_scene();
        let before = super::fingerprint(&engine);
        engine.resources.tallies.social_completed = 9;
        assert_eq!(before, super::fingerprint(&engine));
        assert_eq!(
            super::metrics::sample(&engine, "legacy", 0, 0)["social_completed"],
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
        let sample = super::metrics::sample(&crate::scenes::legacy_scene(), "legacy", 0, 0);
        assert_eq!(sample["schema_version"], 13);
        assert_eq!(sample.as_object().expect("sample object").len(), 53 + 15 + 15 + 6 + 5 + 2 + 3 + 8 + 14 + 2 - 4);
    }
}

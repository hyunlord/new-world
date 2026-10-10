//! Deterministic headless instrumentation; none of these measurements feeds simulation.
mod args;
mod deaths;
mod deaths_observer;
mod fingerprint;
mod long_run;
mod metrics;
mod runner;
mod summary;

pub use args::ProbeArgs;
pub use fingerprint::fingerprint;
pub use runner::run;

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
    fn schema_three_is_exactly_additive() {
        let sample = super::metrics::sample(&crate::scenes::legacy_scene(), "legacy", 0, 0);
        assert_eq!(sample["schema_version"], 3);
        assert_eq!(sample.as_object().expect("sample object").len(), 53);
    }
}

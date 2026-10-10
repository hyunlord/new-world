//! Deterministic headless instrumentation; none of these measurements feeds simulation.
mod args;
mod fingerprint;
mod metrics;
mod runner;
mod summary;

pub use args::ProbeArgs;
pub use fingerprint::fingerprint;
pub use runner::run;

#[cfg(test)]
mod tests {
    #[test]
    fn schema_remains_version_one() {
        let sample = super::metrics::sample(&crate::scenes::legacy_scene(), "legacy", 0, 0);
        assert_eq!(sample["schema_version"], 1);
        assert_eq!(sample.as_object().expect("sample object").len(), 34);
    }
}

//! Measurement-only M6a food-chain instrumentation and isolated content fixture.
mod cli;
mod data_only;
mod metrics;
mod observer;
pub use cli::run;
pub use data_only::run_data_only;

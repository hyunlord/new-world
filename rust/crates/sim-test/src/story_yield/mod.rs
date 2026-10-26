//! Read-only story-yield instrument over persistent chronicle cause links.

mod cli;
mod metric;
pub use cli::run;
pub use metric::{measure, YearYield};

#[cfg(test)]
mod tests;

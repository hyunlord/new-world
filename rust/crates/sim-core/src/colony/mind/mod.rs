//! Pure colony mind definitions, columnar inputs and deterministic selection.
//! No ECS access, effects, global catalogue bootstrap or causal-ID allocation.

mod compile;
mod continuation;
mod error;
mod facts;
mod model;
mod raw;
mod select;
mod selection;

pub use compile::compile;
pub use continuation::{
    guard_continuation, guard_continuation_cached, ContinuationPlan, ContinuationReader,
    GuardCache, GuardOutcome, GuardRead, GuardScratch, GuardWake,
};
pub use error::*;
pub use facts::{FactReader, FactRow, PersonFacts};
pub use model::*;
pub use select::{choose, choose_with_reader, EvalScratch, FactWitness, WitnessRole};
pub use selection::*;

#[cfg(test)]
mod tests;

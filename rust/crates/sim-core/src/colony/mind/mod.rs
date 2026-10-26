//! Pure colony mind definitions, columnar inputs and deterministic selection.
//! No ECS access, effects, global catalogue bootstrap or causal-ID allocation.

mod compile;
mod error;
mod facts;
mod model;
mod raw;
mod select;
mod selection;

pub use compile::compile;
pub use error::*;
pub use facts::{FactRow, PersonFacts};
pub use model::*;
pub use select::{choose, EvalScratch, FactWitness, WitnessRole};
pub use selection::*;

#[cfg(test)]
mod tests;

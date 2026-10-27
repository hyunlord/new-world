//! Colony mind: registered facts, pure data selection and explicit lifecycle effects.
//! Registration remains on the legacy decision until the measured switch commit.

mod behaviours;
pub mod context;
pub mod facts;
pub(crate) mod job_origins;
mod program;
pub(crate) mod record;
pub(in crate::runtime) mod scheduling;
#[cfg(feature = "mind_shadow")]
pub mod shadow;
mod system;
#[cfg(test)]
mod system_tests;

pub use program::{MindProgram, MindProgramError};
pub use system::{ColonyMindSystem, MindTickError};

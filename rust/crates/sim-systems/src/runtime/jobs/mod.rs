//! The M1 job layer (goal section 7, ADR 0013): every piece of job logic.
//!
//! - [`assign::JobAssignSystem`] (124) hands idle or leisured adults a job:
//!   Build (deliver logs, then build) > Chop > Haul, with reservations.
//! - [`execute::JobExecuteSystem`] (127) walks each job through its stages.
//! - [`seam`] is the narrow, documented hook the decision system (125) calls:
//!   job holders keep their job through leisure and idle choices and lose it
//!   to any survival choice.
//! - [`cancel`] ends jobs and releases reservations; [`items`] puts logs down
//!   (merge and spill) and indexes colony sites; [`invariants`] checks the
//!   log ledger, the reservations and the stacks.
//!
//! Data flow: command → designation / zone / site (`crate::commands`) →
//! assign (reserve) → execute (move logs, ledger) → `ConstructionSystem`
//! (hut progress and completion). Every system returns at once without
//! `rules.colony`, so the baseline world never runs any of this.
pub mod assign;
pub mod cancel;
pub mod execute;
pub mod invariants;
pub mod items;
pub mod seam;

pub use assign::{JobAssignSystem, JOB_ASSIGN_PRIORITY};
pub use cancel::{cancel, finish, on_death, release_job};
pub use execute::{JobExecuteSystem, JOB_EXECUTE_PRIORITY};
pub use seam::{hold_for_decision, holders, settle_after_decision, skips_decision, JobHolders};

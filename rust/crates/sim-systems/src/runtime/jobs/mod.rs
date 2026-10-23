//! The M1 job layer (goal section 7, ADR 0013): every piece of job logic.
//!
//! - [`bills::BillSystem`] (123, every 60 ticks) picks each workshop's active bill (M2).
//! - [`assign::JobAssignSystem`] (124) hands idle or leisured adults a job by
//!   work priority and type (M2 goal 3.6), with reservations.
//! - [`execute::JobExecuteSystem`] (127) walks each job through its stages.
//! - [`seam`] is the narrow, documented hook the decision system (125) calls:
//!   job holders keep their job through leisure and idle choices and lose it
//!   to any survival choice.
//! - [`cancel`] ends jobs and releases reservations; [`items`] puts items down
//!   (merge and spill) and indexes colony sites; [`craft`] starts and finishes
//!   crafts; [`invariants`] checks the per-kind ledger, the reservations and
//!   the stacks; [`space`] finishes, takes apart and digs built space (M3);
//!   [`teach`] runs apprenticeship orders as paired Teach / Learn jobs (M4).
//!
//! Data flow: command → designation / zone / site (`crate::commands`) →
//! assign (reserve) → execute (move logs, ledger) → `ConstructionSystem`
//! (hut progress and completion). Every system returns at once without
//! `rules.colony`, so the baseline world never runs any of this.
pub mod assign;
pub mod bills;
pub mod cancel;
pub mod craft;
pub mod execute;
pub mod hearth;
pub mod hunt;
pub mod invariants;
pub mod items;
pub mod seam;
pub mod space;
pub mod teach;

pub use assign::{JobAssignSystem, JOB_ASSIGN_PRIORITY};
pub use bills::{BillSystem, BILL_PRIORITY};
pub use cancel::{cancel, finish, on_death, release_job};
pub use execute::{JobExecuteSystem, JOB_EXECUTE_PRIORITY};
pub use hearth::{HearthFeedSystem, HEARTH_FEED_PRIORITY};
pub use seam::{hold_for_decision, holders, settle_after_decision, skips_decision, JobHolders};

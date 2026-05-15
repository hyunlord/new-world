//! V7 Phase 4-α — Canonical ECS components.
//!
//! First deliverable of the Agent Core phase (Week 7-8). Replaces the
//! Phase 2 local placeholder in `sim_systems::runtime::influence::agent_sample::Position`
//! per the self-documenting landmark at `agent_sample.rs:9-15`.
//!
//! Phase 5-α extends the surface with [`AgentId`] (re-exported here) and
//! the [`Hunger`] need component — the first daily-routine driver.
//!
//! See `.harness/plans/phase4.md` §2 and `.harness/plans/phase5.md` §2.1
//! for the full sub-stage decomposition.

pub mod agent;
pub mod hunger;
pub mod position;

pub use agent::{Agent, AgentId};
pub use hunger::Hunger;
pub use position::Position;

//! V7 Phase 4-α — Canonical ECS components.
//!
//! First deliverable of the Agent Core phase (Week 7-8). Replaces the
//! Phase 2 local placeholder in `sim_systems::runtime::influence::agent_sample::Position`
//! per the self-documenting landmark at `agent_sample.rs:9-15`.
//!
//! See `.harness/plans/phase4.md` §2 for the full sub-stage decomposition.

pub mod agent;
pub mod position;

pub use agent::Agent;
pub use position::Position;

//! V7 Phase 4-α — Canonical ECS components.
//!
//! First deliverable of the Agent Core phase (Week 7-8). Replaces the
//! Phase 2 local placeholder in `sim_systems::runtime::influence::agent_sample::Position`
//! per the self-documenting landmark at `agent_sample.rs:9-15`.
//!
//! Phase 5-α extends the surface with [`AgentId`] (re-exported here) and
//! the [`Hunger`] need component — the first daily-routine driver.
//!
//! Phase 5-β adds the second need ([`Thirst`]) and the agent-decision
//! FSM ([`AgentState`] + [`TargetKind`]) that consumes both needs and
//! emits the first agent-originated `CausalEvent::AgentDecision`.
//!
//! See `.harness/plans/phase4.md` §2 and `.harness/plans/phase5.md`
//! §2.1 / §2.β for the full sub-stage decomposition.

pub mod agent;
pub mod agent_state;
pub mod hunger;
pub mod position;
pub mod thirst;

pub use agent::{Agent, AgentId};
pub use agent_state::{AgentState, TargetKind};
pub use hunger::Hunger;
pub use position::Position;
pub use thirst::Thirst;

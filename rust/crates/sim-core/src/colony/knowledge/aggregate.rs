//! Settlement technology aggregates and apprenticeship orders.
use super::KnowledgeId;
use crate::components::AgentId;
use serde::{Deserialize, Serialize};

/// The settlement-level state of one node (goal 3.4, ADR 0026).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum TechStatus {
    /// No one has ever held it.
    #[default]
    Unknown,
    /// Exactly one person holds it (the warning "⚠ 문: Fox만 앎").
    AtRisk,
    /// Two or more hold it, fewer than three practise it.
    Known,
    /// Three or more practise it.
    Stable,
    /// It was held and its last holder is gone.
    Lost,
}

impl TechStatus {
    /// Stable code (0 unknown, 1 at risk, 2 known, 3 stable, 4 lost).
    pub const fn code(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::AtRisk => 1,
            Self::Known => 2,
            Self::Stable => 3,
            Self::Lost => 4,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::AtRisk => "at_risk",
            Self::Known => "known",
            Self::Stable => "stable",
            Self::Lost => "lost",
        }
    }
}

/// One node's aggregate (goal 3.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TechState {
    /// Adults at threshold who used it lately (or gained it lately).
    pub practitioners: u16,
    /// People at threshold, used or not.
    pub holders: u16,
    /// The state.
    pub status: TechStatus,
    /// Tick the status last changed.
    pub last_change_tick: u64,
    /// Up to five practitioners (holders when none practise), by `AgentId`.
    pub names: Vec<AgentId>,
}

/// The settlement's technology (goal 3.4; `ColonyState::tech`): one state per
/// node in id order, the era progress ‰ and a version for renderers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TechAggregate {
    /// One per node, in id order (empty before the first aggregate).
    pub states: Vec<TechState>,
    /// Era progress ‰ (Σ weight · coverage / Σ weight).
    pub progress_permille: u16,
    /// Adults counted at the last aggregate.
    pub adults: u16,
    /// Bumped whenever a state or the progress changes.
    pub version: u64,
}

impl TechAggregate {
    /// Nodes at risk (the HUD badge).
    pub fn at_risk(&self) -> usize {
        self.states
            .iter()
            .filter(|s| s.status == TechStatus::AtRisk)
            .count()
    }
}

/// The era label of `progress` ‰ (goal 3.4): 0 구석기 (< 250), 1 신석기 전환기
/// (250–749), 2 신석기 시대 (≥ 750).
pub const fn era_label(progress: u16) -> u8 {
    if progress < 250 {
        0
    } else if progress < 750 {
        1
    } else {
        2
    }
}

/// One apprenticeship order (goal 3.3; `ColonyState::teach_orders`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeachOrder {
    /// Its id (from 1).
    pub id: u32,
    /// The teacher.
    pub teacher: AgentId,
    /// The learner.
    pub learner: AgentId,
    /// What is taught.
    pub knowledge: KnowledgeId,
    /// Sessions held so far.
    pub sessions: u32,
    /// The day index of `sessions_today`.
    pub day: u64,
    /// Sessions held on `day`.
    pub sessions_today: u8,
    /// Tick the order was placed.
    pub placed: u64,
}

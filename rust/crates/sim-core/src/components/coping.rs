// TODO(v3.1): Convert remaining f32 fields to deterministic equivalents (f64).
//             HashMap → BTreeMap conversion completed in a10-coping-deterministic feature.
use crate::enums::CopingStrategyId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A delayed stress rebound entry (from C13 Substance Use — Cooper 1995).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopingRebound {
    /// Ticks remaining until rebound fires
    pub delay: i32,
    /// Stress rebound amount on 0..1 scale (applied as rebound * 300 / 2000 to stress level)
    pub stress_rebound: f32,
    /// Allostatic load increment on 0..1 scale (added directly to allostatic_load)
    pub allostatic_add: f32,
}

/// Coping strategy state
/// Lazarus & Folkman (1984) Transactional Model / Carver et al. (1989) COPE Scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coping {
    /// Currently active coping strategy (if any)
    pub active_strategy: Option<CopingStrategyId>,
    /// Cooldown ticks remaining per strategy
    pub strategy_cooldowns: BTreeMap<CopingStrategyId, u32>,
    /// Times each strategy has been used
    pub usage_counts: BTreeMap<CopingStrategyId, u32>,
    /// Per-strategy proficiency (0.0..=1.0, increases with use)
    pub proficiency: BTreeMap<CopingStrategyId, f32>,
    /// C05 Denial: accumulated hidden debt (Compas 2001)
    pub denial_accumulator: f32,
    /// C05 Denial: ticks until debt explosion (72-tick timer)
    pub denial_timer: u32,
    /// C13 Substance Use: dependency level (0.0..=1.0, Cooper 1995)
    pub dependency_score: f32,
    /// C11 Disengagement: learned helplessness score (Seligman 1975)
    pub helplessness_score: f32,
    /// Control appraisal cap (lowered by helplessness; starts at 1.0)
    pub control_appraisal_cap: f32,
    /// Total mental breaks experienced (drives K(n) learning pressure)
    pub break_count: u32,
    /// C13 Substance Use: ticks since last use (for withdrawal)
    pub substance_recent_timer: u32,
    /// Delayed stress rebound queue (from C13 Substance Use)
    pub rebound_queue: Vec<CopingRebound>,
}

impl Default for Coping {
    fn default() -> Self {
        Self {
            active_strategy: None,
            strategy_cooldowns: BTreeMap::new(),
            usage_counts: BTreeMap::new(),
            proficiency: BTreeMap::new(),
            denial_accumulator: 0.0,
            denial_timer: 0,
            dependency_score: 0.0,
            helplessness_score: 0.0,
            control_appraisal_cap: 1.0,
            break_count: 0,
            substance_recent_timer: 0,
            rebound_queue: Vec::new(),
        }
    }
}

impl Coping {
    pub fn is_on_cooldown(&self, strategy: CopingStrategyId) -> bool {
        self.strategy_cooldowns.get(&strategy).copied().unwrap_or(0) > 0
    }

    /// Get proficiency for a strategy (0.0 if not owned)
    pub fn get_proficiency(&self, strategy: CopingStrategyId) -> f32 {
        self.proficiency.get(&strategy).copied().unwrap_or(0.0)
    }
}

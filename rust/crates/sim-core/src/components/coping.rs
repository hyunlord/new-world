use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::enums::CopingStrategyId;

/// Coping strategy state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Coping {
    /// Currently active coping strategy (if any)
    pub active_strategy: Option<CopingStrategyId>,
    /// Cooldown ticks remaining per strategy
    pub strategy_cooldowns: HashMap<CopingStrategyId, u32>,
    /// Times each strategy has been used
    pub usage_counts: HashMap<CopingStrategyId, u32>,
}

impl Coping {
    pub fn is_on_cooldown(&self, strategy: CopingStrategyId) -> bool {
        self.strategy_cooldowns.get(&strategy).copied().unwrap_or(0) > 0
    }
}

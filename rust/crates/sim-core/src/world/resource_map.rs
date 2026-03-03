use serde::{Deserialize, Serialize};
use crate::enums::ResourceType;
use std::collections::HashMap;

/// Global resource tracking (settlement-level aggregates)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceMap {
    /// Resources stored per settlement: settlement_id → {resource_type → amount}
    pub settlement_stocks: HashMap<u64, HashMap<ResourceType, f64>>,
}

impl ResourceMap {
    pub fn get_stock(&self, settlement_id: u64, resource: ResourceType) -> f64 {
        self.settlement_stocks
            .get(&settlement_id)
            .and_then(|m| m.get(&resource))
            .copied()
            .unwrap_or(0.0)
    }

    pub fn add_stock(&mut self, settlement_id: u64, resource: ResourceType, amount: f64) {
        let stocks = self.settlement_stocks.entry(settlement_id).or_default();
        *stocks.entry(resource).or_insert(0.0) += amount;
    }

    pub fn take_stock(&mut self, settlement_id: u64, resource: ResourceType, amount: f64) -> f64 {
        let stocks = self.settlement_stocks.entry(settlement_id).or_default();
        let current = stocks.entry(resource).or_insert(0.0);
        let taken = amount.min(*current);
        *current -= taken;
        taken
    }
}

use crate::ids::TraitId;
use serde::{Deserialize, Serialize};

/// Active personality/archetype traits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Traits {
    /// Currently active trait IDs (from trait_definitions_fixed.json)
    pub active: Vec<TraitId>,
    /// Salience scores for potential trait activation
    pub salience_scores: Vec<(TraitId, f64)>,
}

impl Traits {
    pub fn has_trait(&self, id: &str) -> bool {
        self.active.iter().any(|t| t == id)
    }

    pub fn add_trait(&mut self, id: TraitId) {
        if !self.has_trait(&id) {
            self.active.push(id);
        }
    }

    pub fn remove_trait(&mut self, id: &str) {
        self.active.retain(|t| t != id);
    }
}

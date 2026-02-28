use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::ids::SkillId;

/// A single skill entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    /// Skill level (0-100)
    pub level: u16,
    /// Accumulated XP toward next level
    pub xp: f64,
}

impl Default for SkillEntry {
    fn default() -> Self { Self { level: 0, xp: 0.0 } }
}

/// All skills for one entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skills {
    pub entries: HashMap<SkillId, SkillEntry>,
}

impl Skills {
    pub fn get_level(&self, id: &str) -> u16 {
        self.entries.get(id).map(|e| e.level).unwrap_or(0)
    }

    pub fn best_skill_level(&self) -> u16 {
        self.entries.values().map(|e| e.level).max().unwrap_or(0)
    }

    pub fn add_xp(&mut self, id: &str, xp: f64) {
        let entry = self.entries.entry(id.to_string()).or_default();
        entry.xp += xp;
    }
}

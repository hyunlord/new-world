use serde::{Deserialize, Serialize};
use crate::enums::{Sex, GrowthStage};
use crate::ids::SettlementId;

/// Core identity for every entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub birth_tick: u64,
    pub sex: Sex,
    pub species_id: String,  // "human"
    pub settlement_id: Option<SettlementId>,
    /// Current growth stage (derived from age)
    pub growth_stage: GrowthStage,
    /// Zodiac sign (from birth day_of_year)
    pub zodiac_sign: String,
    /// Blood phenotype ("A", "B", "AB", "O")
    pub blood_type: String,
    /// Speech style
    pub speech_tone: String,
    pub speech_verbosity: String,
    pub speech_humor: String,
    /// Preferences (Layer 7)
    pub pref_food: String,
    pub pref_color: String,
    pub pref_season: String,
    pub dislikes: Vec<String>,
}

impl Default for Identity {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            birth_tick: 0,
            sex: Sex::Male,
            species_id: "human".to_string(),
            settlement_id: None,
            growth_stage: GrowthStage::Adult,
            zodiac_sign: "aries".to_string(),
            blood_type: "O".to_string(),
            speech_tone: "casual".to_string(),
            speech_verbosity: "normal".to_string(),
            speech_humor: "none".to_string(),
            pref_food: "food".to_string(),
            pref_color: "blue".to_string(),
            pref_season: "summer".to_string(),
            dislikes: Vec::new(),
        }
    }
}

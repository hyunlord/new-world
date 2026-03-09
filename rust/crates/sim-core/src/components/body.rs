// TODO(v3.1): Convert remaining f32 fields and helpers to f64 for determinism.
use serde::{Deserialize, Serialize};

/// Body attribute axes (potential/trainability/realized 3-layer system)
/// All values are in raw integer scale (potential 0-10000, realized 0-15000)
/// except health/attractiveness/height which are 0.0-1.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    // === Layer 1.5: Body Potential (0-10000 int scale) ===
    pub str_potential: i32,
    pub agi_potential: i32,
    pub end_potential: i32,
    pub tou_potential: i32,
    pub rec_potential: i32,
    pub dr_potential: i32,

    // === Trainability (0-1000 int scale) ===
    pub str_trainability: i32,
    pub agi_trainability: i32,
    pub end_trainability: i32,
    pub tou_trainability: i32,
    pub rec_trainability: i32,
    pub dr_trainability: i32,

    // === Realized (0-15000 int scale) ===
    pub str_realized: i32,
    pub agi_realized: i32,
    pub end_realized: i32,
    pub tou_realized: i32,
    pub rec_realized: i32,
    pub dr_realized: i32,

    // === Training XP accumulator (per axis) ===
    pub str_xp: f32,
    pub agi_xp: f32,
    pub end_xp: f32,
    pub tou_xp: f32,
    pub rec_xp: f32,
    pub dr_xp: f32,

    // === Innate immunity (0-1000 int scale) ===
    pub innate_immunity: i32,

    // === Appearance (0.0-1.0) ===
    pub attractiveness: f32,
    pub height: f32,

    // === Health (0.0-1.0) ===
    pub health: f32,

    // === Blood type genetics ===
    pub blood_genotype: String,  // "AA", "AO", "AB", "BB", "BO", "OO"

    // === Distinguishing mark ===
    pub distinguishing_mark: Option<String>,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            str_potential: 1050, agi_potential: 1050, end_potential: 1050,
            tou_potential: 1050, rec_potential: 1050, dr_potential: 1050,
            str_trainability: 500, agi_trainability: 500, end_trainability: 500,
            tou_trainability: 500, rec_trainability: 500, dr_trainability: 500,
            str_realized: 1000, agi_realized: 700, end_realized: 700,
            tou_realized: 700, rec_realized: 700, dr_realized: 500,
            str_xp: 0.0, agi_xp: 0.0, end_xp: 0.0,
            tou_xp: 0.0, rec_xp: 0.0, dr_xp: 0.0,
            innate_immunity: 500,
            attractiveness: 0.5,
            height: 0.5,
            health: 1.0,
            blood_genotype: "OO".to_string(),
            distinguishing_mark: None,
        }
    }
}

impl Body {
    /// Derived speed: float(agi_realized) * 0.0012 + 0.30
    pub fn speed(&self) -> f32 {
        self.agi_realized as f32 * 0.0012 + 0.30
    }

    /// Derived strength: float(str_realized) / 1000.0
    pub fn strength_norm(&self) -> f32 {
        self.str_realized as f32 / 1000.0
    }
}

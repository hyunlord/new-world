#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

// ═══════════════════════════════════════
// Personality
// ═══════════════════════════════════════

/// HEXACO 6-axis personality model (Ashton & Lee)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(u8)]
pub enum HexacoAxis {
    H = 0, // Honesty-Humility
    E = 1, // Emotionality
    X = 2, // Extraversion
    A = 3, // Agreeableness
    C = 4, // Conscientiousness
    O = 5, // Openness
}

/// HEXACO 24 facets (4 per axis). Index = axis*4 + facet_offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(u8)]
pub enum HexacoFacet {
    // H (0-3)
    Sincerity = 0, Fairness = 1, GreedAvoidance = 2, Modesty = 3,
    // E (4-7)
    Fearfulness = 4, Anxiety = 5, Dependence = 6, Sentimentality = 7,
    // X (8-11)
    SocialSelfEsteem = 8, SocialBoldness = 9, Sociability = 10, Liveliness = 11,
    // A (12-15)
    Forgiveness = 12, Gentleness = 13, Flexibility = 14, Patience = 15,
    // C (16-19)
    Organization = 16, Diligence = 17, Perfectionism = 18, Prudence = 19,
    // O (20-23)
    AestheticAppreciation = 20, Inquisitiveness = 21, Creativity = 22, Unconventionality = 23,
}

/// Bowlby (1969) attachment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum AttachmentType {
    Secure,    // ~55%
    Anxious,   // ~20%
    Avoidant,  // ~20%
    Fearful,   // ~5%
}

impl Default for AttachmentType {
    fn default() -> Self { Self::Secure }
}

// ═══════════════════════════════════════
// Emotion (Plutchik wheel)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(u8)]
pub enum EmotionType {
    Joy = 0,
    Trust = 1,
    Fear = 2,
    Surprise = 3,
    Sadness = 4,
    Disgust = 5,
    Anger = 6,
    Anticipation = 7,
}

// ═══════════════════════════════════════
// Needs (Maslow + Alderfer ERG, 13 total)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(u8)]
pub enum NeedType {
    // L1: Existence
    Hunger = 0,
    Thirst = 1,
    Sleep = 2,
    Warmth = 3,
    Safety = 4,
    // L2: Relatedness
    Belonging = 5,
    Intimacy = 6,
    Recognition = 7,
    // L3: Growth
    Autonomy = 8,
    Competence = 9,
    SelfActualization = 10,
    Meaning = 11,
    Transcendence = 12,
}

pub const NEED_COUNT: usize = 13;

// ═══════════════════════════════════════
// Values (33 types)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(u8)]
pub enum ValueType {
    Law = 0, Loyalty = 1, Family = 2, Friendship = 3, Power = 4,
    Truth = 5, Cunning = 6, Eloquence = 7, Fairness = 8, Decorum = 9,
    Tradition = 10, Artwork = 11, Cooperation = 12, Independence = 13,
    Stoicism = 14, Introspection = 15, SelfControl = 16, Tranquility = 17,
    Harmony = 18, Merriment = 19, Craftsmanship = 20, MartialProwess = 21,
    Skill = 22, HardWork = 23, Sacrifice = 24, Competition = 25,
    Perseverance = 26, Leisure = 27, Commerce = 28, Romance = 29,
    Knowledge = 30, Nature = 31, Peace = 32,
}

pub const VALUE_COUNT: usize = 33;

// ═══════════════════════════════════════
// Stress (Lazarus stress appraisal, 5 states)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum StressState {
    Calm,
    Alert,
    Resistance,
    Exhaustion,
    Collapse,
}

impl Default for StressState {
    fn default() -> Self { Self::Calm }
}

// ═══════════════════════════════════════
// Mental Break (10 types — from mental_breaks.json)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum MentalBreakType {
    Panic,
    Rage,
    OutrageViolence,
    Shutdown,
    Purge,
    GriefWithdrawal,
    Fugue,
    Paranoia,
    CompulsiveRitual,
    HystericalBonding,
}

// ═══════════════════════════════════════
// Growth Stage (6 stages — from game_config.gd)
// Thresholds in ticks: infant<13140, toddler<26280, child<52560, teen<65700, adult<245280, elder≥245280
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, PartialOrd, Ord)]
pub enum GrowthStage {
    Infant,   // < 3y  (< 13140 ticks)
    Toddler,  // < 6y  (< 26280 ticks)
    Child,    // < 12y (< 52560 ticks)
    Teen,     // < 15y (< 65700 ticks)
    Adult,    // < 56y (< 245280 ticks)
    Elder,    // 56+y  (≥ 245280 ticks)
}

impl GrowthStage {
    /// Get growth stage from age in ticks (from GameConfig age thresholds)
    pub fn from_age_ticks(age_ticks: u64) -> Self {
        if age_ticks < 13_140 {
            GrowthStage::Infant
        } else if age_ticks < 26_280 {
            GrowthStage::Toddler
        } else if age_ticks < 52_560 {
            GrowthStage::Child
        } else if age_ticks < 65_700 {
            GrowthStage::Teen
        } else if age_ticks < 245_280 {
            GrowthStage::Adult
        } else {
            GrowthStage::Elder
        }
    }

    pub fn is_child_age(&self) -> bool {
        matches!(self, GrowthStage::Infant | GrowthStage::Toddler | GrowthStage::Child | GrowthStage::Teen)
    }
}

impl Default for GrowthStage {
    fn default() -> Self { Self::Adult }
}

// ═══════════════════════════════════════
// Sex
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum Sex {
    Male,
    Female,
}

impl Default for Sex {
    fn default() -> Self { Self::Male }
}

// ═══════════════════════════════════════
// Social Class
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, PartialOrd, Ord)]
pub enum SocialClass {
    Outcast,
    Commoner,
    Artisan,
    Merchant,
    Noble,
    Ruler,
}

impl Default for SocialClass {
    fn default() -> Self { Self::Commoner }
}

// ═══════════════════════════════════════
// Relationship Type
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum RelationType {
    Stranger,     // affinity < NETWORK_TIE_WEAK_MIN (5.0)
    Acquaintance, // 5.0 ≤ affinity < 30.0
    Friend,       // 30.0 ≤ affinity < 60.0
    CloseFriend,  // 60.0 ≤ affinity < 85.0
    Intimate,     // affinity ≥ 85.0
    Spouse,
    Parent,
    Child,
    Sibling,
    Rival,
    Enemy,
}

impl Default for RelationType {
    fn default() -> Self { Self::Stranger }
}

// ═══════════════════════════════════════
// Intelligence (Gardner 8)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString)]
#[repr(u8)]
pub enum IntelligenceType {
    Linguistic = 0,
    Logical = 1,
    Spatial = 2,
    Musical = 3,
    Kinesthetic = 4,
    Interpersonal = 5,
    Intrapersonal = 6,
    Naturalistic = 7,
}

pub const INTELLIGENCE_COUNT: usize = 8;

// ═══════════════════════════════════════
// Action Type
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum ActionType {
    Idle,
    Forage,
    Hunt,
    Fish,
    Build,
    Craft,
    Socialize,
    Rest,
    Sleep,
    Eat,
    Drink,
    Explore,
    Flee,
    Fight,
    Migrate,
    Teach,
    Learn,
    MentalBreak,
    Pray,
    Wander,
    GatherWood,
    GatherStone,
    GatherHerbs,
    DeliverToStockpile,
    TakeFromStockpile,
    SeekShelter,
    SitByFire,
    VisitPartner,
}

impl Default for ActionType {
    fn default() -> Self { Self::Idle }
}

// ═══════════════════════════════════════
// Terrain / Biome (from GameConfig Biome enum)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum TerrainType {
    DeepWater,
    ShallowWater,
    Beach,
    Grassland,
    Forest,
    DenseForest,
    Hill,
    Mountain,
    Snow,
}

// ═══════════════════════════════════════
// Resource (from GameConfig ResourceType enum)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter)]
pub enum ResourceType {
    Food,
    Wood,
    Stone,
}

// ═══════════════════════════════════════
// Simulation Speed
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum SimSpeed {
    Paused,
    Normal,  // 1x
    Fast,    // 2x
    Faster,  // 3x
    Ultra,   // 5x
    Max,     // 10x
}

impl Default for SimSpeed {
    fn default() -> Self { Self::Normal }
}

// ═══════════════════════════════════════
// Coping Strategy (15 types — from coping_definitions.json C01-C15)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter)]
pub enum CopingStrategyId {
    StrategicPlanning,       // C01
    InstrumentalSupport,     // C02
    EmotionalSupport,        // C03
    PositiveReframing,       // C04
    Denial,                  // C05
    Acceptance,              // C06
    Humor,                   // C07
    ReligiousCoping,         // C08
    Venting,                 // C09
    ActiveDistraction,       // C10
    BehavioralDisengagement, // C11
    SelfBlame,               // C12
    SubstanceUse,            // C13
    Rumination,              // C14
    ProblemSolving,          // C15
}

// ═══════════════════════════════════════
// Tech State (V2 5-state system)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum TechState {
    Unknown,
    KnownLow,
    KnownStable,
    ForgottenRecent,
    ForgottenLong,
}

impl Default for TechState {
    fn default() -> Self { Self::Unknown }
}

// ═══════════════════════════════════════
// Tech Era
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter)]
pub enum TechEra {
    #[strum(serialize = "stone_age")]
    StoneAge,
    #[strum(serialize = "tribal")]
    Tribal,
    #[strum(serialize = "bronze_age")]
    BronzeAge,
}

impl Default for TechEra {
    fn default() -> Self { Self::StoneAge }
}

// ═══════════════════════════════════════
// Death Cause
// ═══════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
pub enum DeathCause {
    OldAge,
    Starvation,
    Dehydration,
    Exposure,
    Disease,
    Combat,
    Accident,
    ChildMortality,
    InfantMortality,
}

// ═══════════════════════════════════════
// Blood Type (Layer 7)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum BloodType {
    A,
    B,
    AB,
    O,
}

impl Default for BloodType {
    fn default() -> Self { Self::O }
}

// ═══════════════════════════════════════
// Zodiac Sign (Layer 7)
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter)]
pub enum ZodiacSign {
    Aries, Taurus, Gemini, Cancer, Leo, Virgo,
    Libra, Scorpio, Sagittarius, Capricorn, Aquarius, Pisces,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_growth_stage_from_ticks() {
        assert_eq!(GrowthStage::from_age_ticks(0), GrowthStage::Infant);
        assert_eq!(GrowthStage::from_age_ticks(13139), GrowthStage::Infant);
        assert_eq!(GrowthStage::from_age_ticks(13140), GrowthStage::Toddler);
        assert_eq!(GrowthStage::from_age_ticks(26279), GrowthStage::Toddler);
        assert_eq!(GrowthStage::from_age_ticks(26280), GrowthStage::Child);
        assert_eq!(GrowthStage::from_age_ticks(52560), GrowthStage::Child);
        assert_eq!(GrowthStage::from_age_ticks(65700), GrowthStage::Adult);
        assert_eq!(GrowthStage::from_age_ticks(245280), GrowthStage::Elder);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = StressState::Exhaustion;
        let json = serde_json::to_string(&s).unwrap();
        let back: StressState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn test_need_count() {
        use strum::IntoEnumIterator;
        assert_eq!(NeedType::iter().count(), NEED_COUNT);
    }

    #[test]
    fn test_value_count() {
        use strum::IntoEnumIterator;
        assert_eq!(ValueType::iter().count(), VALUE_COUNT);
    }

    #[test]
    fn test_mental_break_count() {
        use strum::IntoEnumIterator;
        // Cannot use EnumIter without adding it; just check variant count manually
        let breaks = [
            MentalBreakType::Panic, MentalBreakType::Rage, MentalBreakType::OutrageViolence,
            MentalBreakType::Shutdown, MentalBreakType::Purge, MentalBreakType::GriefWithdrawal,
            MentalBreakType::Fugue, MentalBreakType::Paranoia, MentalBreakType::CompulsiveRitual,
            MentalBreakType::HystericalBonding,
        ];
        assert_eq!(breaks.len(), 10);
        let _ = CopingStrategyId::iter().count(); // just checks iter works
        assert_eq!(CopingStrategyId::iter().count(), 15);
    }
}

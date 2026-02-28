use serde::{Deserialize, Serialize};

/// Unique entity ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

/// Unique settlement ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SettlementId(pub u64);

/// Unique building ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingId(pub u64);

/// Trait ID referencing trait_definitions_fixed.json (e.g., "f_sincere")
pub type TraitId = String;

/// Skill ID referencing tech catalog (e.g., "SKILL_FIRE_MAKING")
pub type SkillId = String;

/// Tech ID referencing tech JSON files (e.g., "TECH_FIRE_MAKING")
pub type TechId = String;

impl EntityId {
    pub const NONE: EntityId = EntityId(u64::MAX);
}

impl SettlementId {
    pub const NONE: SettlementId = SettlementId(u64::MAX);
}

impl BuildingId {
    pub const NONE: BuildingId = BuildingId(u64::MAX);
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

impl std::fmt::Display for SettlementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Settlement({})", self.0)
    }
}

impl std::fmt::Display for BuildingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Building({})", self.0)
    }
}

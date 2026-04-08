//! RON-backed schema definitions for v3.1 simulation content.

pub mod action;
pub mod furniture;
pub mod material;
pub mod recipe;
pub mod structure;
pub mod temperament;
pub mod world_rules;

pub use action::{ActionCategory, ActionCondition, ActionDef, ActionEffect};
pub use furniture::{FurnitureDef, InfluenceEmission};
pub use material::{MaterialCategory, MaterialDef, MaterialProperties};
pub use recipe::{RecipeDef, RecipeOutput, RecipeRequires, TagRequirement};
pub use structure::{RoleRecognition, StructureDef, StructureRequirement};
pub use temperament::{
    AxisShift, BiasMatrixRow, CauseTrigger, DerivedRuleOverride, ShiftCondition, TemperamentRules,
    TemperamentShiftRule,
};
pub use world_rules::{
    AgentConstants, GlobalConstants, InfluenceChannelRule, InfluenceClampPolicyDef,
    RuleResourceModifier, RuleResourceSpawn, RuleSpecialZone, WorldRuleset, ZoneResourceBoost,
};

#[cfg(test)]
mod tests {
    use super::material::MaterialCategory;

    #[test]
    fn exports_are_available() {
        assert!(matches!(MaterialCategory::Stone, MaterialCategory::Stone));
    }
}

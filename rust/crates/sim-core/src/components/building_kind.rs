//! Stable building categories, stored beside the legacy placement event.
use super::{BlueprintId, BuildingBlueprint};
use serde::{Deserialize, Serialize};
/// Player-visible building category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    /// Camp centre and source of warmth.
    Hearth,
    /// Resident housing and sleep source.
    Shelter,
    /// Settlement food storage marker.
    Stockpile,
}
impl BuildingKind {
    /// Stable packed-snapshot code.
    pub const fn code(self) -> u8 {
        match self {
            Self::Hearth => 0,
            Self::Shelter => 1,
            Self::Stockpile => 2,
        }
    }
}
/// Shared shelter blueprint identity.
pub const SHELTER_BLUEPRINT_ID: BlueprintId = 1;
/// Construct a one-tile shelter blueprint with the world's work requirement.
pub fn shelter_blueprint(required_progress: u32) -> BuildingBlueprint {
    BuildingBlueprint::new(SHELTER_BLUEPRINT_ID, 1, 1, required_progress)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_codes_and_shelter_work() {
        assert_eq!(
            [
                BuildingKind::Hearth.code(),
                BuildingKind::Shelter.code(),
                BuildingKind::Stockpile.code()
            ],
            [0, 1, 2]
        );
        assert_eq!(shelter_blueprint(240), BuildingBlueprint::new(1, 1, 1, 240));
    }
}

//! WorldSim Core — V7 Reset
//!
//! V7 Master Direction Section 7: Foundation systems.
//!
//! - Week 1~2: Material System Deep
//! - Week 3~4: Tile Grid + Influence System
//! - Week 5~6: Cause-Effect Tracking + "왜?" UI
//! - Week 7~8: Agent Core
//! - Week 9~10: First Daily Routine
//! - Week 11~12: Building System Deep

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// V7 version marker.
pub const V7_VERSION: &str = "0.7.0-init";

/// Material RON schema version. Files declaring a higher value are rejected
/// by the loader so older builds never silently load incompatible mod data.
pub const MATERIAL_SCHEMA_VERSION: u32 = 1;

pub mod material;

pub use material::{
    MaterialId,
    MaterialCategory,
    MaterialDef,
    MaterialProperties,
    MaterialRegistry,
    AutoDerivedStats,
    DerivedStatKind,
    MaterialError,
    Explanation,
    PropertyKind,
    TerrainType,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v7_version_present() {
        assert_eq!(V7_VERSION, "0.7.0-init");
    }

    #[test]
    fn material_schema_version_is_one() {
        assert_eq!(MATERIAL_SCHEMA_VERSION, 1);
    }
}

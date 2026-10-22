//! Material System (Week 1~2 of V7).
//!
//! Phase 0 design references:
//! - DF wiki `Material_science` (absolute physical values).
//! - RimWorld wiki `Stuff` / `StuffProperties` (factor multipliers).
//! - CRC Handbook of Chemistry and Physics, 102nd ed. (density, melting).
//! - Mohs scale (1812) for hardness.
//! - Callister, *Materials Science and Engineering* (yield, fracture).

pub mod category;
pub mod definition;
pub mod derivation;
pub mod error;
pub mod explanation;
pub mod id;
pub mod loader;
pub mod properties;
pub mod registry;
pub mod terrain;

pub use category::MaterialCategory;
pub use definition::MaterialDef;
pub use derivation::{
    AutoDerivedStats, DerivedStatKind, IRON_AXE_BLUNT, IRON_AXE_CUT, IRON_AXE_DURABILITY,
};
pub use error::MaterialError;
pub use explanation::{explain, Explanation, PropertyKind};
pub use id::MaterialId;
pub use loader::{
    load_directory, load_ron, parse_ron_str, MaterialDefRaw, MaterialFile, CURRENT_SCHEMA_VERSION,
};
pub use properties::MaterialProperties;
pub use registry::MaterialRegistry;
pub use terrain::TerrainType;

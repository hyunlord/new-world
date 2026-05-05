//! Material System (Week 1~2 of V7).
//!
//! Phase 0 design references:
//! - DF wiki `Material_science` (absolute physical values).
//! - RimWorld wiki `Stuff` / `StuffProperties` (factor multipliers).
//! - CRC Handbook of Chemistry and Physics, 102nd ed. (density, melting).
//! - Mohs scale (1812) for hardness.
//! - Callister, *Materials Science and Engineering* (yield, fracture).

pub mod id;
pub mod category;
pub mod terrain;
pub mod error;
pub mod properties;
pub mod definition;
pub mod derivation;
pub mod explanation;
pub mod registry;
pub mod loader;

pub use id::MaterialId;
pub use category::MaterialCategory;
pub use terrain::TerrainType;
pub use error::MaterialError;
pub use properties::MaterialProperties;
pub use definition::MaterialDef;
pub use derivation::{
    AutoDerivedStats, DerivedStatKind, IRON_AXE_BLUNT, IRON_AXE_CUT, IRON_AXE_DURABILITY,
};
pub use explanation::{Explanation, PropertyKind, explain};
pub use registry::MaterialRegistry;
pub use loader::{CURRENT_SCHEMA_VERSION, MaterialDefRaw, MaterialFile, load_directory, load_ron};

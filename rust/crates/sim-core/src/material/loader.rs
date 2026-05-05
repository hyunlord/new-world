//! RON loader for material files.
//!
//! Schema version is locked at compile time and re-checked per file. Each
//! parsed material's properties go through `MaterialProperties::validate()`
//! before conversion into `MaterialDef`.

use std::path::Path;

use serde::Deserialize;

use crate::material::category::MaterialCategory;
use crate::material::definition::MaterialDef;
use crate::material::error::MaterialError;
use crate::material::id::MaterialId;
use crate::material::properties::MaterialProperties;
use crate::material::terrain::TerrainType;

/// Schema version this loader was built against. MUST equal
/// `crate::MATERIAL_SCHEMA_VERSION`.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

const _: () = {
    assert!(CURRENT_SCHEMA_VERSION == crate::MATERIAL_SCHEMA_VERSION);
};

/// Top-level RON file shape: schema version + materials list.
#[derive(Debug, Deserialize)]
pub struct MaterialFile {
    /// File-declared schema version. Rejected if != `CURRENT_SCHEMA_VERSION`.
    pub schema_version: u32,
    /// Material definitions in source-string form.
    pub materials: Vec<MaterialDefRaw>,
}

/// On-disk material definition. `id` is a string here and gets hashed into
/// a `MaterialId` during conversion.
#[derive(Debug, Deserialize)]
pub struct MaterialDefRaw {
    /// Source string for the material id (djb2-hashed).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Classification.
    pub category: MaterialCategory,
    /// Physical / cultural properties.
    pub properties: MaterialProperties,
    /// Tier (0..=255).
    pub tier: u8,
    /// Terrains where the material naturally appears.
    #[serde(default)]
    pub natural_in: Vec<TerrainType>,
    /// Mod identifier; `None` for base game.
    #[serde(default)]
    pub mod_source: Option<String>,
}

impl MaterialDefRaw {
    fn into_def(self) -> Result<MaterialDef, MaterialError> {
        self.properties.validate()?;
        Ok(MaterialDef {
            id: MaterialId::from_str_hash(&self.id),
            name: self.name,
            category: self.category,
            properties: self.properties,
            tier: self.tier,
            natural_in: self.natural_in,
            mod_source: self.mod_source,
        })
    }
}

/// Load a single RON file.
pub fn load_ron(path: &Path) -> Result<Vec<MaterialDef>, MaterialError> {
    let text = std::fs::read_to_string(path)?;
    let file: MaterialFile = ron::from_str(&text).map_err(|e| MaterialError::ParseError(e.to_string()))?;
    if file.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(MaterialError::SchemaMismatch {
            file_version: file.schema_version,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }
    let mut out = Vec::with_capacity(file.materials.len());
    for raw in file.materials {
        out.push(raw.into_def()?);
    }
    Ok(out)
}

/// Load every `.ron` file directly inside `path` (non-recursive).
pub fn load_directory(path: &Path) -> Result<Vec<MaterialDef>, MaterialError> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("ron") {
            out.extend(load_ron(&p)?);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "worldsim_loader_{}_{}",
            name,
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{name}.ron"));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    fn valid_iron_ron() -> &'static str {
        r#"(
            schema_version: 1,
            materials: [
                (
                    id: "iron",
                    name: "Iron",
                    category: mineral,
                    properties: (
                        density: 7800.0,
                        hardness: 4.0,
                        shear_yield: 100000.0,
                        impact_yield: 200000.0,
                        fracture_toughness: 50000.0,
                        melting_point: 1538.0,
                        flammability: 0.0,
                        thermal_conductivity: 80.0,
                        cultural_value: 0.5,
                        rarity: 0.5,
                        work_difficulty: 0.5,
                        aesthetic_value: 0.5,
                        workability: 0.6,
                        preservation: 0.7,
                    ),
                    tier: 3,
                ),
            ],
        )"#
    }

    #[test]
    fn schema_mismatch_rejected() {
        let bad = valid_iron_ron().replace("schema_version: 1", "schema_version: 999");
        let path = write_temp("schema_bad", &bad);
        let err = load_ron(&path).expect_err("999 must reject");
        match err {
            MaterialError::SchemaMismatch { file_version, supported } => {
                assert_eq!(file_version, 999);
                assert_eq!(supported, 1);
            }
            other => panic!("expected SchemaMismatch, got {other:?}"),
        }
    }

    #[test]
    fn valid_ron_roundtrip() {
        let path = write_temp("iron_ok", valid_iron_ron());
        let defs = load_ron(&path).expect("load ok");
        assert_eq!(defs.len(), 1);
        let d = &defs[0];
        assert_eq!(d.id, MaterialId::from_str_hash("iron"));
        assert_eq!(d.name, "Iron");
        assert_eq!(d.category, MaterialCategory::Mineral);
        assert_eq!(d.properties.density, 7800.0);
    }

    #[test]
    fn property_out_of_range_rejected_by_loader() {
        let bad = valid_iron_ron().replace("hardness: 4.0,", "hardness: 11.0,");
        let path = write_temp("hardness_oor", &bad);
        let err = load_ron(&path).expect_err("oor must reject");
        match err {
            MaterialError::PropertyOutOfRange { property, value, expected } => {
                assert_eq!(property, "hardness");
                assert_eq!(value, 11.0);
                assert!(!expected.is_empty());
            }
            other => panic!("expected PropertyOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn load_directory_empty_dir_returns_empty_vec() {
        let dir = std::env::temp_dir().join(format!("worldsim_loader_empty_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Clear any leftovers.
        for e in std::fs::read_dir(&dir).unwrap() {
            let _ = std::fs::remove_file(e.unwrap().path());
        }
        let defs = load_directory(&dir).unwrap();
        assert!(defs.is_empty());
    }

    #[test]
    fn load_directory_nonexistent_returns_io_error() {
        let p = std::path::PathBuf::from("/nonexistent/worldsim/material/xyz/abc");
        let err = load_directory(&p).expect_err("missing dir");
        assert!(matches!(err, MaterialError::Io(_)));
    }

    #[test]
    fn schema_version_constants_match() {
        assert_eq!(CURRENT_SCHEMA_VERSION, crate::MATERIAL_SCHEMA_VERSION);
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
    }
}

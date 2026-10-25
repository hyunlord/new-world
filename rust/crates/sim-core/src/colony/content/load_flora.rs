//! Strict flora parsing, physical Wood resolution and explicit append/replacement merge.
use super::{sources, CatalogError, FloraCatalog, FloraDistribution, FloraSpecies, FloraWeight};
use crate::colony::materials::{embedded_definitions, MaterialStats};
use crate::material::{MaterialCategory, MaterialDef, MaterialId, TerrainType};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FloraFileV1 {
    schema_version: u32,
    default_material: String,
    species: Vec<SpeciesRaw>,
    distributions: Vec<DistributionRaw>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FloraExtensionV1 {
    schema_version: u32,
    species: Vec<SpeciesRaw>,
    #[serde(deserialize_with = "required_replacement")]
    replace_distributions: Option<Vec<DistributionRaw>>,
}
fn required_replacement<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<DistributionRaw>>, D::Error> {
    Option::deserialize(deserializer)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpeciesRaw {
    material: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DistributionRaw {
    terrain: TerrainType,
    elevation: (u8, u8),
    weights: Vec<WeightRaw>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeightRaw {
    material: String,
    weight: u32,
}
fn error(message: impl Into<String>) -> CatalogError {
    CatalogError::new("flora", message)
}
fn schema(version: u32) -> Result<(), CatalogError> {
    if version != 1 {
        return Err(error(format!("unsupported schema {version}")));
    }
    Ok(())
}
fn admit(
    rows: Vec<SpeciesRaw>,
    species: &mut Vec<FloraSpecies>,
    materials: &BTreeSet<String>,
    physical: &[MaterialDef],
) -> Result<(), CatalogError> {
    for row in rows {
        if !materials.contains(&row.material) {
            return Err(error(format!("undeclared material {}", row.material)));
        }
        let definition = physical
            .iter()
            .find(|def| def.name.to_lowercase() == row.material)
            .ok_or_else(|| error(format!("missing physical material {}", row.material)))?;
        if definition.category != MaterialCategory::Wood {
            return Err(error("flora requires physical Wood"));
        }
        // The validated embedded physical definition is the source of runtime integer stats.
        let _stats = MaterialStats::from_def(definition);
        if species
            .iter()
            .any(|s| s.name == row.material || s.material == definition.id)
        {
            return Err(error(format!("duplicate flora material {}", row.material)));
        }
        species.push(FloraSpecies {
            material: definition.id,
            name: row.material,
        });
    }
    species.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}
fn distributions(
    rows: Vec<DistributionRaw>,
    species: &[FloraSpecies],
) -> Result<Vec<FloraDistribution>, CatalogError> {
    let mut coverage = [false; 256];
    let mut result = Vec::new();
    for mut row in rows {
        if row.terrain != TerrainType::Forest {
            return Err(error("only Forest terrain is supported"));
        }
        let (lo, hi) = row.elevation;
        if lo > hi {
            return Err(error("reversed elevation interval"));
        }
        for elevation in lo..=hi {
            if std::mem::replace(&mut coverage[usize::from(elevation)], true) {
                return Err(error("overlapping elevation coverage"));
            }
        }
        if row.weights.is_empty() {
            return Err(error("weights must be nonempty and positive"));
        }
        row.weights.sort_by(|a, b| a.material.cmp(&b.material));
        let mut seen = BTreeSet::new();
        let mut weights = Vec::new();
        let mut cumulative = Vec::new();
        let mut total = 0u64;
        for weight in row.weights {
            if weight.weight == 0 {
                return Err(error("weights must be positive"));
            }
            let member = species
                .iter()
                .find(|s| s.name == weight.material)
                .ok_or_else(|| error(format!("unadmitted flora member {}", weight.material)))?;
            if !seen.insert(member.material) {
                return Err(error("duplicate distribution member"));
            }
            total = total
                .checked_add(u64::from(weight.weight))
                .ok_or_else(|| error("weight sum overflow"))?;
            cumulative.push(total);
            weights.push(FloraWeight {
                material: member.material,
                weight: weight.weight,
            });
        }
        result.push(FloraDistribution {
            terrain: row.terrain,
            elevation: row.elevation,
            weights,
            cumulative,
            total,
        });
    }
    if coverage.iter().any(|covered| !covered) {
        return Err(error("incomplete Forest elevation coverage"));
    }
    result.sort_by_key(|band| band.elevation);
    Ok(result)
}
pub(super) fn build(
    base: &BTreeMap<String, Vec<u8>>,
    extension: &BTreeMap<String, Vec<u8>>,
    base_materials: &BTreeSet<String>,
    materials: &BTreeSet<String>,
) -> Result<FloraCatalog, CatalogError> {
    let raw: FloraFileV1 =
        ron::from_str(sources::text(base, "flora.ron")?).map_err(|e| error(e.to_string()))?;
    schema(raw.schema_version)?;
    let physical = embedded_definitions().map_err(error)?;
    let mut species = Vec::new();
    admit(raw.species, &mut species, base_materials, &physical)?;
    let default: MaterialId = species
        .iter()
        .find(|s| s.name == raw.default_material)
        .map(|s| s.material)
        .ok_or_else(|| error("default material must be admitted"))?;
    // Validate the base before an extension can replace its distribution.
    let mut bands = distributions(raw.distributions, &species)?;
    if extension.contains_key("flora.ron") {
        let extra: FloraExtensionV1 = ron::from_str(sources::text(extension, "flora.ron")?)
            .map_err(|e| error(e.to_string()))?;
        schema(extra.schema_version)?;
        admit(extra.species, &mut species, materials, &physical)?;
        if let Some(rows) = extra.replace_distributions {
            bands = distributions(rows, &species)?;
        }
    }
    Ok(FloraCatalog {
        default,
        species,
        distributions: bands,
    })
}

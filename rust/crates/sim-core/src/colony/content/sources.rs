//! Frozen source bytes and deterministic selected-content revision.
use super::CatalogError;
use std::collections::BTreeMap;

/// One logically named immutable source, independent of its filesystem location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedSource {
    /// Logical family filename or presentation key.
    pub name: String,
    /// Exact source bytes, including schema and presentation data.
    pub bytes: Vec<u8>,
}
/// Owned input to the pure catalogue builder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawSources {
    /// Named bytes; duplicate names are errors.
    pub sources: Vec<NamedSource>,
}
impl RawSources {
    /// Embedded production definitions. No environment or filesystem access.
    pub fn bundled() -> Self {
        Self {
            sources: [
                (
                    "plants.ron",
                    include_str!("../../../data/colony/plants.ron"),
                ),
                ("flora.ron", include_str!("../../../data/colony/flora.ron")),
                (
                    "animals.ron",
                    include_str!("../../../data/colony/animals.ron"),
                ),
                (
                    "pressures.ron",
                    include_str!("../../../data/colony/pressures.ron"),
                ),
                ("items.ron", include_str!("../../../data/colony/items.ron")),
                (
                    "blueprints.ron",
                    include_str!("../../../data/colony/blueprints.ron"),
                ),
                (
                    "recipes.ron",
                    include_str!("../../../data/colony/recipes.ron"),
                ),
                (
                    "knowledge.ron",
                    include_str!("../../../data/colony/knowledge.ron"),
                ),
            ]
            .into_iter()
            .map(|(name, text)| NamedSource {
                name: name.into(),
                bytes: text.as_bytes().to_vec(),
            })
            .collect(),
        }
    }
    pub(super) fn into_map(self) -> Result<BTreeMap<String, Vec<u8>>, CatalogError> {
        let mut out = BTreeMap::new();
        for source in self.sources {
            let name = normalize(&source.name).to_owned();
            if !matches!(
                name.as_str(),
                "items.ron"
                    | "blueprints.ron"
                    | "recipes.ron"
                    | "knowledge.ron"
                    | "pressures.ron"
                    | "animals.ron"
                    | "flora.ron"
                    | "plants.ron"
                    | "pack.ron"
                    | "alerts"
                    | "lessons"
                    | "locale_ko"
                    | "locale_en"
                    | "bindings"
            ) {
                return Err(CatalogError::new(name, "unknown source family"));
            }
            if out.insert(name.clone(), source.bytes).is_some() {
                return Err(CatalogError::new(name, "duplicate source"));
            }
        }
        Ok(out)
    }
}
pub(super) fn normalize(name: &str) -> &str {
    match name {
        "items" => "items.ron",
        "blueprints" => "blueprints.ron",
        "recipes" => "recipes.ron",
        "knowledge" => "knowledge.ron",
        "pressures" => "pressures.ron",
        "animals" => "animals.ron",
        "plants" => "plants.ron",
        "flora" => "flora.ron",
        other => other,
    }
}
pub(super) fn text<'a>(
    map: &'a BTreeMap<String, Vec<u8>>,
    name: &str,
) -> Result<&'a str, CatalogError> {
    let bytes = map
        .get(normalize(name))
        .ok_or_else(|| CatalogError::new(name, "missing required source"))?;
    std::str::from_utf8(bytes).map_err(|e| CatalogError::new(name, e.to_string()))
}
pub(super) fn revision(
    base: &BTreeMap<String, Vec<u8>>,
    extension: &BTreeMap<String, Vec<u8>>,
) -> String {
    let mut hash = 14_695_981_039_346_656_037u64;
    let mut length = 0u64;
    let mut feed = |bytes: &[u8]| {
        for b in bytes {
            hash = (hash ^ u64::from(*b)).wrapping_mul(1_099_511_628_211);
        }
        length = length.saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
    };
    feed(b"worldsim-content-v1");
    for (label, map) in [
        (b"base".as_slice(), base),
        (b"extension".as_slice(), extension),
    ] {
        feed(label);
        for (name, bytes) in map {
            feed(&u64::try_from(name.len()).unwrap_or(u64::MAX).to_le_bytes());
            feed(name.as_bytes());
            feed(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
            feed(bytes);
        }
    }
    format!("catalog-v1-fnv1a64:{hash:016x}:{length}")
}

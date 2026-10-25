//! Explicit host-only descriptor acquisition; no environment reads or lazy world access.
use super::{BootSelection, CatalogError, NamedSource, RawSources};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Component, Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema_version: u32,
    id: String,
    revision: String,
    sources: BTreeMap<String, String>,
}

/// Freeze a strictly validated pack descriptor and its relative files.
/// Calling hosts own environment selection and sticky error handling before world creation.
pub fn read_selection(path: &Path) -> Result<BootSelection, CatalogError> {
    let path = path
        .canonicalize()
        .map_err(|e| CatalogError::new("pack path", e.to_string()))?;
    let root = path
        .parent()
        .ok_or_else(|| CatalogError::new("pack path", "missing parent"))?;
    let bytes =
        std::fs::read(&path).map_err(|e| CatalogError::new("pack descriptor", e.to_string()))?;
    let descriptor: Descriptor = ron::de::from_bytes(&bytes)
        .map_err(|e| CatalogError::new("pack descriptor", e.to_string()))?;
    if descriptor.schema_version != 1 || descriptor.id.is_empty() || descriptor.revision.is_empty()
    {
        return Err(CatalogError::new(
            "pack descriptor",
            "schema must be1 and id/revision nonempty",
        ));
    }
    let mut sources = vec![NamedSource {
        name: "pack.ron".into(),
        bytes,
    }];
    for (logical, relative) in descriptor.sources {
        if !matches!(
            logical.as_str(),
            "items"
                | "blueprints"
                | "recipes"
                | "knowledge"
                | "pressures"
                | "animals"
                | "flora"
                | "alerts"
                | "lessons"
                | "locale_ko"
                | "locale_en"
                | "bindings"
        ) {
            return Err(CatalogError::new(
                "pack descriptor",
                format!("unknown source key {logical:?}"),
            ));
        }
        let relative = Path::new(&relative);
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(CatalogError::new(
                logical,
                "source path must use relative normal components",
            ));
        }
        let file = root
            .join(relative)
            .canonicalize()
            .map_err(|e| CatalogError::new(&logical, e.to_string()))?;
        if !file.starts_with(root) {
            return Err(CatalogError::new(logical, "source escapes pack root"));
        }
        let bytes = std::fs::read(file).map_err(|e| CatalogError::new(&logical, e.to_string()))?;
        sources.push(NamedSource {
            name: super::sources::normalize(&logical).into(),
            bytes,
        });
    }
    Ok(BootSelection::Explicit {
        request_id: format!("{}:{}", descriptor.id, descriptor.revision),
        sources: RawSources { sources },
    })
}

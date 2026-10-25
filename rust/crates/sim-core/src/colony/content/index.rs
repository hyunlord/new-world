//! Local identity indices, with disjoint resolver and human-serde namespaces.
use super::{CatalogError, IdentityDef};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub(super) struct Index {
    pub codes: BTreeMap<u32, usize>,
    pub names: BTreeMap<String, u32>,
    pub serde_names: BTreeMap<String, u32>,
}
impl Index {
    pub fn build<'a>(
        family: &str,
        defs: impl Iterator<Item = &'a IdentityDef>,
    ) -> Result<Self, CatalogError> {
        let mut index = Self::default();
        for (position, def) in defs.enumerate() {
            if def.id.is_empty() || def.name_key.is_empty() {
                return Err(CatalogError::new(family, "empty id/name_key"));
            }
            if index.codes.insert(def.code, position).is_some() {
                return Err(CatalogError::new(
                    family,
                    format!("duplicate code {}", def.code),
                ));
            }
            for name in std::iter::once(&def.id).chain(&def.resolver_aliases) {
                if name.is_empty() || index.names.insert(name.clone(), def.code).is_some() {
                    return Err(CatalogError::new(
                        family,
                        format!("duplicate/empty resolver name {name:?}"),
                    ));
                }
            }
            for name in std::iter::once(&def.serde_name).chain(&def.serde_aliases) {
                let mut chars = name.chars();
                let valid = chars
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                    && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !matches!(name.as_str(), "true" | "false" | "Some" | "None");
                if !valid || index.serde_names.insert(name.clone(), def.code).is_some() {
                    return Err(CatalogError::new(
                        family,
                        format!("duplicate/invalid serde name {name:?}"),
                    ));
                }
            }
        }
        Ok(index)
    }
}

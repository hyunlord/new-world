//! One immutable, sticky process selection; ordinary accessors never inspect the environment.
use super::{BootError, Catalog, CatalogAccessError, CatalogError, RawSources};
use std::sync::OnceLock;

/// Explicit host selection before typed configuration, rules or world construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootSelection {
    /// Embedded versioned production data.
    Bundled,
    /// Frozen descriptor identity and extension bytes acquired by the host.
    Explicit {
        /// Stable descriptor id/revision, independent of staging directory.
        request_id: String,
        /// Frozen extension and presentation bytes.
        sources: RawSources,
    },
}
struct BootedCatalog {
    selection: BootSelection,
    catalog: Catalog,
}
static BOOT: OnceLock<Result<BootedCatalog, BootError>> = OnceLock::new();
static DIFFERENT_SELECTION: BootError = BootError::AlreadyInitialized;

/// Install once; a failed selection stays failed and cannot fall back to bundled data.
pub fn bootstrap(mut selection: BootSelection) -> Result<&'static Catalog, &'static BootError> {
    if let BootSelection::Explicit { sources, .. } = &mut selection {
        for source in &mut sources.sources {
            source.name = super::sources::normalize(&source.name).to_owned();
        }
        sources.sources.sort_by(|a, b| a.name.cmp(&b.name));
    }
    let result = BOOT.get_or_init(|| {
        let catalog = match &selection {
            BootSelection::Bundled => Catalog::from_sources(RawSources::bundled(), None)?,
            BootSelection::Explicit {
                request_id,
                sources,
            } => {
                if request_id.is_empty() || sources.sources.is_empty() {
                    return Err(CatalogError::new(
                        "selection",
                        "empty explicit descriptor/source bundle",
                    )
                    .into());
                }
                Catalog::from_sources(RawSources::bundled(), Some(sources.clone()))?
            }
        };
        Ok(BootedCatalog {
            selection: selection.clone(),
            catalog,
        })
    });
    match result {
        Ok(booted) if booted.selection == selection => Ok(&booted.catalog),
        Ok(_) => Err(&DIFFERENT_SELECTION),
        Err(error) => Err(error),
    }
}

/// Access only; this function and typed serde never initialize the catalogue.
pub fn installed() -> Result<&'static Catalog, CatalogAccessError> {
    match BOOT.get() {
        None => Err(CatalogAccessError::NotInstalled),
        Some(Ok(booted)) => Ok(&booted.catalog),
        Some(Err(error)) => Err(CatalogAccessError::BootFailed(error.to_string())),
    }
}

/// Legacy Rust constructor boundary; embedded corruption or a prior explicit failure is fatal.
pub fn installed_or_bundled() -> &'static Catalog {
    match BOOT.get() {
        Some(Ok(booted)) => &booted.catalog,
        Some(Err(error)) => panic!("content bootstrap invariant: {error}"),
        None => match bootstrap(BootSelection::Bundled) {
            Ok(catalog) => catalog,
            Err(error) => panic!("bundled content bootstrap invariant: {error}"),
        },
    }
}

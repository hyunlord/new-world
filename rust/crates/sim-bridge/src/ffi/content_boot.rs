//! Godot host preflight: freeze selection, validate presentation, then install before any engine.
use std::path::Path;
use std::sync::OnceLock;

use godot::classes::{RefCounted, ResourceLoader};
use godot::prelude::*;
use sim_core::colony::content::{self, BootSelection, Catalog, RawSources};

static BOOT: OnceLock<Result<(), String>> = OnceLock::new();

/// Sticky engine-free host preflight. Only plain Rust status is retained globally.
pub fn ensure_boot() -> Result<(), String> {
    BOOT.get_or_init(prepare).clone()
}

fn prepare() -> Result<(), String> {
    let selection = match std::env::var_os("WORLDSIM_CONTENT_PACK") {
        None => BootSelection::Bundled,
        Some(path) if path.is_empty() => return Err("WORLDSIM_CONTENT_PACK is empty".into()),
        Some(path) => content::host::read_selection(Path::new(&path)).map_err(|e| e.to_string())?,
    };
    let candidate = match &selection {
        BootSelection::Bundled => Catalog::from_sources(RawSources::bundled(), None),
        BootSelection::Explicit { sources, .. } => {
            Catalog::from_sources(RawSources::bundled(), Some(sources.clone()))
        }
    }
    .map_err(|e| e.to_string())?;
    let mut documents = VarDictionary::new();
    for name in ["alerts", "lessons", "locale_ko", "locale_en", "bindings"] {
        if let Some(bytes) = candidate.source_bytes(name) {
            let text = std::str::from_utf8(bytes).map_err(|e| format!("{name}: {e}"))?;
            documents.set(name, text);
        }
    }
    let resource = ResourceLoader::singleton()
        .load("res://scripts/core/presentation_bootstrap.gd")
        .ok_or_else(|| "Cannot load presentation bootstrap".to_string())?;
    let instance = resource.clone().call("new", &[]);
    let mut validator = instance
        .try_to::<Gd<RefCounted>>()
        .map_err(|e| format!("Cannot create presentation validator: {e}"))?;
    let result = validator
        .call(
            "prepare",
            &[
                super::content::catalog_to_dict(&candidate).to_variant(),
                documents.to_variant(),
            ],
        )
        .try_to::<VarDictionary>()
        .map_err(|e| format!("Invalid presentation receipt: {e}"))?;
    if !result
        .get("ok")
        .and_then(|v| v.try_to::<bool>().ok())
        .unwrap_or(false)
    {
        return Err(result
            .get("error")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "Presentation validation failed".into()));
    }
    content::bootstrap(selection).map_err(|e| e.to_string())?;
    Ok(())
}

/// Fresh dictionary receipt; failed preflight never constructs a default world.
pub fn receipt() -> VarDictionary {
    let mut out = VarDictionary::new();
    match ensure_boot() {
        Ok(()) => {
            out.set("ok", true);
            out.set("error", "");
            if let Ok(catalog) = content::installed() {
                out.set("registry_revision", catalog.revision());
            }
        }
        Err(error) => {
            out.set("ok", false);
            out.set("error", error.as_str());
        }
    }
    out
}

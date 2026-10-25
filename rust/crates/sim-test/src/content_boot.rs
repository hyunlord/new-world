//! Headless CLI content selection before typed arguments, rules or worker threads.
//!
//! `WORLDSIM_CONTENT_PACK` names a RON descriptor file. An absent variable selects
//! bundled content; an explicitly empty or unreadable path is an error. Source
//! bytes are acquired once by the host reader and installed immutably by core.
//! CLI presentation is NOT_VALIDATED: simulation catalog validation does not
//! prove Godot bindings, atlas readability, locale coverage or rendered behavior.
//! This helper adds no fields or messages to deterministic probe output.
use std::ffi::OsString;
use std::path::Path;

use sim_core::colony::content::{self, BootSelection};

/// Select and install the headless process's catalog before parsing typed input.
pub fn from_environment() -> Result<(), String> {
    let selection = selection(std::env::var_os("WORLDSIM_CONTENT_PACK"))?;
    content::bootstrap(selection)
        .map(|_| ())
        .map_err(|error| format!("content bootstrap: {error}"))
}

fn selection(path: Option<OsString>) -> Result<BootSelection, String> {
    match path {
        None => Ok(BootSelection::Bundled),
        Some(path) if path.is_empty() => {
            Err("WORLDSIM_CONTENT_PACK must name a descriptor file; empty is not bundled".into())
        }
        Some(path) => content::host::read_selection(Path::new(&path))
            .map_err(|error| format!("WORLDSIM_CONTENT_PACK: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_selection_is_bundled_without_installing_it() {
        assert_eq!(selection(None), Ok(BootSelection::Bundled));
    }

    #[test]
    fn explicit_empty_selection_is_not_bundled() {
        assert!(selection(Some(OsString::new())).is_err());
    }
}

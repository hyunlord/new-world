//! Save files on disk for `WorldSimNode::save_world` / `load_world` (M1.5 §7).
//!
//! Pure functions over a path: the `#[func]` bodies only forward here, and
//! sim-test calls them headless. The path is an absolute file-system path;
//! GDScript resolves `user://saves/<seed>-<tick>.wsave` with
//! `ProjectSettings.globalize_path` before calling. The bytes are
//! `sim_systems::save::save` / `load` (format and exactness documented
//! there).

use std::path::Path;

use sim_engine::SimEngine;
use sim_systems::save;

/// Save `engine` to `path`, creating missing parent folders. The file is
/// written beside the target and renamed over it, so a failed write never
/// leaves a truncated save. Returns the number of bytes written.
///
/// `engine` is `&mut` because saving rebuilds its world (see
/// `sim_systems::save::save`); its simulation state does not change.
pub fn save_engine_to_file(engine: &mut SimEngine, path: &Path) -> Result<usize, String> {
    let bytes = save::save(engine).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let mut partial = path.as_os_str().to_owned();
    partial.push(".partial");
    let partial = std::path::PathBuf::from(partial);
    std::fs::write(&partial, &bytes).map_err(|e| format!("{}: {e}", partial.display()))?;
    std::fs::rename(&partial, path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(bytes.len())
}

/// Load the engine saved at `path` (production system list).
pub fn load_engine_from_file(path: &Path) -> Result<SimEngine, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    save::load(&bytes).map_err(|e| e.to_string())
}

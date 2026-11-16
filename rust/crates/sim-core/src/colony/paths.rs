//! Worn paths: footfall-driven path stages (M-GROW 3.2).
//!
//! Data from `data/colony/paths.ron`; per-tile state lives in [`ColonyState`]
//! (`footfall`, `path_stage`, `footfall_touched`). The movement system
//! increments footfall; a day-boundary system in sim-systems applies decay,
//! promotes/demotes stages and refreshes nav costs.

use serde::Deserialize;
use std::sync::OnceLock;

const PATHS_RON: &str = include_str!("../../data/colony/paths.ron");

// ── RON schema ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PathsFile {
    schema_version: u32,
    stages: Vec<RawStage>,
    daily_decay_permille: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct RawStage {
    id: String,
    code: u8,
    footfall_threshold: u16,
    nav_cost: u8,
    step_interval: u64,
}

// ── Public API ─────────────────────────────────────────────────────────

/// One path stage (grass, trodden, dirt, paved_road).
#[derive(Debug, Clone, Copy)]
pub struct PathStage {
    /// Byte code stored in `ColonyState::path_stage` (0–3).
    pub code: u8,
    /// Minimum footfall to reach this stage.
    pub footfall_threshold: u16,
    /// A* cost byte for tiles at this stage.
    pub nav_cost: u8,
    /// Ticks between movement steps on this stage.
    pub step_interval: u64,
}

/// Parsed paths data, immutable after load.
#[derive(Debug)]
pub struct PathsData {
    /// All stages in code order.
    pub stages: Vec<PathStage>,
    /// Daily decay in permille of the current footfall value.
    pub daily_decay_permille: u16,
}

impl PathsData {
    /// The stage for a given footfall value (highest threshold that fits).
    /// Paved road (code 3) is never returned here; it is placed, not earned.
    pub fn stage_for_footfall(&self, footfall: u16) -> &PathStage {
        let mut best = &self.stages[0];
        for s in &self.stages {
            // paved_road (code 3) is placed-only; skip it.
            if s.code == 3 {
                continue;
            }
            if footfall >= s.footfall_threshold && s.footfall_threshold >= best.footfall_threshold {
                best = s;
            }
        }
        best
    }

    /// Look up a stage by its code byte.
    pub fn stage_by_code(&self, code: u8) -> Option<&PathStage> {
        self.stages.iter().find(|s| s.code == code)
    }
}

fn parse_paths(text: &str) -> Result<PathsData, String> {
    let file: PathsFile =
        ron::from_str(text).map_err(|e| format!("paths.ron parse error: {e}"))?;
    if file.schema_version != 1 {
        return Err(format!(
            "paths.ron: expected schema_version 1, got {}",
            file.schema_version
        ));
    }
    let stages: Vec<PathStage> = file
        .stages
        .into_iter()
        .map(|s| PathStage {
            code: s.code,
            footfall_threshold: s.footfall_threshold,
            nav_cost: s.nav_cost,
            step_interval: s.step_interval,
        })
        .collect();
    Ok(PathsData {
        stages,
        daily_decay_permille: file.daily_decay_permille,
    })
}

/// The bundled paths data (parsed once).
pub fn installed() -> &'static PathsData {
    static DATA: OnceLock<PathsData> = OnceLock::new();
    DATA.get_or_init(|| parse_paths(PATHS_RON).expect("paths.ron must parse"))
}

/// Grass (default, no path).
pub const STAGE_GRASS: u8 = 0;
/// Trodden grass (visible wear, no speed bonus).
pub const STAGE_TRODDEN: u8 = 1;
/// Dirt path (speed bonus).
pub const STAGE_DIRT: u8 = 2;
/// Paved road (placed only, speed bonus).
pub const STAGE_PAVED: u8 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_ron_parses() {
        let data = installed();
        assert_eq!(data.stages.len(), 4);
        assert_eq!(data.daily_decay_permille, 20);
    }

    #[test]
    fn stage_for_footfall_picks_highest_matching() {
        let data = installed();
        assert_eq!(data.stage_for_footfall(0).code, STAGE_GRASS);
        assert_eq!(data.stage_for_footfall(29).code, STAGE_GRASS);
        assert_eq!(data.stage_for_footfall(30).code, STAGE_TRODDEN);
        assert_eq!(data.stage_for_footfall(119).code, STAGE_TRODDEN);
        assert_eq!(data.stage_for_footfall(120).code, STAGE_DIRT);
        assert_eq!(data.stage_for_footfall(1000).code, STAGE_DIRT);
    }

    #[test]
    fn paved_road_stage_exists() {
        let data = installed();
        let paved = data.stage_by_code(STAGE_PAVED);
        assert!(paved.is_some());
        assert_eq!(paved.map(|s| s.nav_cost), Some(6));
        assert_eq!(paved.map(|s| s.step_interval), Some(2));
    }
}

//! Worn paths: footfall-driven path stages (M-GROW 3.2).
//!
//! Data from `data/colony/paths.ron`; per-tile state lives in [`super::ColonyState`]
//! (`footfall`, `path_stage`, `footfall_touched`). The movement system
//! increments footfall; a day-boundary system in sim-systems applies decay,
//! promotes/demotes stages and refreshes nav costs.

use serde::Deserialize;
use std::collections::BTreeSet;

#[cfg(test)]
const PATHS_RON: &str = include_str!("../../data/colony/paths.ron");

// ── RON schema ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PathsFile {
    schema_version: u32,
    stages: Vec<RawStage>,
    #[serde(default)]
    daily_decay_permille: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStage {
    id: String,
    code: u8,
    footfall_threshold: u16,
    nav_cost: u8,
    step_interval: u64,
    #[serde(default)]
    placed_only: bool,
}

// ── Public API ─────────────────────────────────────────────────────────

/// One authored natural or constructed walking surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathStage {
    /// Stable content name, independent of movement mechanics.
    pub id: String,
    /// Requires a completed floor; never reached through footfall alone.
    pub placed_only: bool,
    /// Stable byte code stored in `ColonyState::path_stage`.
    pub code: u8,
    /// Minimum footfall to reach this stage.
    pub footfall_threshold: u16,
    /// A* cost byte for tiles at this stage.
    pub nav_cost: u8,
    /// Ticks between movement steps on this stage.
    pub step_interval: u64,
}

/// Parsed paths data, immutable after load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathsData {
    /// All stages in code order.
    pub stages: Vec<PathStage>,
    /// Daily decay in permille of the current footfall value.
    pub daily_decay_permille: u16,
}

impl PathsData {
    /// The stage for a given footfall value (highest threshold that fits).
    /// Placed-only surfaces are never earned through footfall.
    pub fn stage_for_footfall(&self, footfall: u16) -> &PathStage {
        let mut best = &self.stages[0];
        for s in &self.stages {
            if s.placed_only {
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

impl PathsData {
    /// Validate base stages and append-only selected-pack stages.
    pub fn parse(base: &str, extension: Option<&str>) -> Result<Self, String> {
        let mut file = parse_file(base)?;
        if let Some(text) = extension {
            let extra = parse_file(text)?;
            if extra.daily_decay_permille.is_some() {
                return Err("paths: extensions cannot replace daily decay".into());
            }
            file.stages.extend(extra.stages);
        }
        let decay = file
            .daily_decay_permille
            .ok_or("paths: missing daily decay")?;
        if decay > 1000 {
            return Err("paths: daily decay must be in 0..=1000".into());
        }
        file.stages.sort_by_key(|stage| stage.code);
        let mut ids = BTreeSet::new();
        let mut codes = BTreeSet::new();
        let mut thresholds = BTreeSet::new();
        for stage in &file.stages {
            if stage.id.is_empty()
                || !ids.insert(stage.id.as_str())
                || !codes.insert(stage.code)
                || stage.nav_cost == 0
                || stage.step_interval == 0
                || (!stage.placed_only && !thresholds.insert(stage.footfall_threshold))
            {
                return Err(format!("paths: invalid or duplicate stage {}", stage.id));
            }
        }
        if file.stages.first().is_none_or(|stage| {
            stage.code != STAGE_GRASS || stage.footfall_threshold != 0 || stage.placed_only
        }) {
            return Err("paths: stage zero must be the natural zero-footfall fallback".into());
        }
        Ok(Self {
            stages: file
                .stages
                .into_iter()
                .map(|stage| PathStage {
                    id: stage.id,
                    placed_only: stage.placed_only,
                    code: stage.code,
                    footfall_threshold: stage.footfall_threshold,
                    nav_cost: stage.nav_cost,
                    step_interval: stage.step_interval,
                })
                .collect(),
            daily_decay_permille: decay,
        })
    }

    /// Preserve explicit movement-rule overrides while applying authored surface ratios.
    pub fn step_interval(&self, code: u8, base_interval: u64) -> u64 {
        let base = self.stages[0].step_interval;
        let interval = self
            .stage_by_code(code)
            .map_or(base, |stage| stage.step_interval);
        base_interval.saturating_mul(interval).div_ceil(base).max(1)
    }
}

fn parse_file(text: &str) -> Result<PathsFile, String> {
    let file: PathsFile = ron::from_str(text).map_err(|error| format!("paths: {error}"))?;
    if file.schema_version != 1 {
        return Err(format!("paths: unsupported schema {}", file.schema_version));
    }
    Ok(file)
}

/// Validated selected paths, shared with blueprint resolution and save revision checks.
pub fn installed() -> &'static PathsData {
    super::content::installed_or_bundled().paths()
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
    fn selected_stages_validate_and_drive_mechanics_without_code_changes() {
        let extension = r#"(schema_version:1, stages:[
            (id:"test_trail", code:4, footfall_threshold:200, nav_cost:5, step_interval:1),
            (id:"test_floor", code:7, footfall_threshold:0, nav_cost:4, step_interval:1, placed_only:true),
        ])"#;
        let selected = PathsData::parse(PATHS_RON, Some(extension)).expect("extension");
        assert_eq!(selected.stage_for_footfall(201).code, 4);
        assert_eq!(selected.stage_by_code(7).expect("floor").nav_cost, 4);
        assert_eq!(selected.step_interval(4, 3), 1);
        assert_eq!(selected.step_interval(STAGE_DIRT, 1), 1);
        assert_eq!(selected.step_interval(STAGE_DIRT, 6), 4);
        for invalid in [
            PATHS_RON.replace("nav_cost: 7", "nav_cost: 0"),
            PATHS_RON.replace("code: 2", "code: 1"),
            PATHS_RON.replace("step_interval: 3", "step_interval: 0"),
            PATHS_RON.replace("Some(20)", "Some(1001)"),
            PATHS_RON.replace("footfall_threshold: 30", "footfall_threshold: 0"),
        ] {
            assert!(PathsData::parse(&invalid, None).is_err());
        }
    }

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

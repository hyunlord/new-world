//! PathDaySystem: day-boundary decay and stage update for worn paths (M-GROW 3.2).
//!
//! Priority: 145 · Interval: `ticks_per_day` · Gate: colony only
//! Reads: `colony.footfall`, `colony.path_stage`, `colony.footfall_touched`
//! Writes: `colony.footfall` (decay), `colony.path_stage` (promote/demote), nav (refresh)

use hecs::World;
use sim_core::colony::paths;
use sim_engine::{RuntimeSystem, SimResources};

/// Synchronize the surface of a changed physical floor without changing footfall.
/// Called at completion, deconstruction and loaded-map reconstruction.
pub fn refresh_built_surface(resources: &mut SimResources, tile: (u32, u32)) {
    use sim_core::colony::content_state::CompletedLayer;
    if resources.rules.colony.is_none()
        || tile.0 >= resources.tile_grid.width
        || tile.1 >= resources.tile_grid.height
    {
        return;
    }
    let grid = &resources.tile_grid;
    let idx = grid.idx(tile.0, tile.1);
    let data = paths::installed();
    let constructed = resources
        .content_state
        .completed
        .get(&(tile, CompletedLayer::Floor))
        .filter(|_| grid.floor_material[idx].is_some() && !grid.is_dug(idx))
        .and_then(|identity| identity.blueprint.definition().completed_path_stage);
    let old = resources
        .colony
        .path_stage
        .get(idx)
        .copied()
        .unwrap_or(paths::STAGE_GRASS);
    let next = constructed.unwrap_or_else(|| {
        if data
            .stage_by_code(old)
            .is_some_and(|stage| stage.placed_only)
        {
            data.stage_for_footfall(resources.colony.footfall.get(idx).copied().unwrap_or(0))
                .code
        } else {
            old
        }
    });
    if next != old {
        let len = grid.width as usize * grid.height as usize;
        resources.colony.footfall.resize(len, 0);
        resources.colony.path_stage.resize(len, paths::STAGE_GRASS);
        resources.colony.path_stage[idx] = next;
        resources.colony.path_version = resources.colony.path_version.wrapping_add(1);
        resources.colony.touch(false);
    }
}

/// Day-boundary decay and stage transitions for worn paths (M-GROW 3.2).
pub struct PathDaySystem;

impl PathDaySystem {
    /// Create the day-boundary path system.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PathDaySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for PathDaySystem {
    fn name(&self) -> &str {
        "PathDaySystem"
    }

    fn priority(&self) -> u32 {
        145
    }

    fn tick_interval(&self) -> u64 {
        1 // checked inside tick(); the interval depends on ticks_per_day.
    }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        if resources.rules.colony.is_none() {
            return;
        }
        let tpd = resources.ticks_per_day;
        if tpd == 0 || !resources.current_tick.is_multiple_of(tpd) {
            return;
        }
        let data = paths::installed();
        let grid_len = (resources.tile_grid.width as usize) * (resources.tile_grid.height as usize);
        if resources.colony.footfall.len() != grid_len {
            resources.colony.footfall.resize(grid_len, 0);
        }
        if resources.colony.path_stage.len() != grid_len {
            resources.colony.path_stage.resize(grid_len, 0);
        }
        // Retain the active set until wear decays to zero; untouched grid cells are never scanned.
        let active = std::mem::take(&mut resources.colony.footfall_touched);
        let decay_permille = u32::from(data.daily_decay_permille);
        let mut nav_dirty = Vec::new();
        for tile in active {
            let idx = resources.tile_grid.idx(tile.0, tile.1);
            let old_footfall = resources.colony.footfall[idx];
            let loss = u16::try_from((u32::from(old_footfall) * decay_permille).div_ceil(1000))
                .unwrap_or(u16::MAX);
            let footfall = old_footfall.saturating_sub(loss);
            resources.colony.footfall[idx] = footfall;
            if footfall > 0 {
                resources.colony.footfall_touched.insert(tile);
            }
            let stage = resources.colony.path_stage[idx];
            if data
                .stage_by_code(stage)
                .is_some_and(|stage| stage.placed_only)
            {
                continue;
            }
            let new_stage = data.stage_for_footfall(footfall).code;
            if new_stage != stage {
                resources.colony.path_stage[idx] = new_stage;
                nav_dirty.push(tile);
            }
        }
        if !nav_dirty.is_empty() {
            resources.colony.path_version = resources.colony.path_version.wrapping_add(1);
            resources.colony.touch(false);
        }
        // Refresh nav for tiles whose stage (and thus cost) changed.
        for tile in nav_dirty {
            crate::runtime::nav::refresh(resources, tile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::paths::{STAGE_DIRT, STAGE_GRASS, STAGE_PAVED, STAGE_TRODDEN};
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;

    fn colony_engine(w: u32, h: u32) -> sim_engine::SimEngine {
        let mut engine = sim_engine::SimEngine::new(w, h, MaterialRegistry::new());
        engine.resources.rules = WorldRules::colony(0);
        engine.resources.ticks_per_day = 1440;
        let grid_len = (w as usize) * (h as usize);
        engine.resources.colony.footfall.resize(grid_len, 0);
        engine.resources.colony.path_stage.resize(grid_len, 0);
        engine
    }

    #[test]
    fn paths_footfall_decay_and_thresholds() {
        let mut engine = colony_engine(4, 4);
        let idx = engine.resources.tile_grid.idx(1, 1);
        // Set tile (1,1) footfall to 130 → dirt stage.
        engine.resources.colony.footfall[idx] = 130;
        engine.resources.colony.footfall_touched.insert((1, 1));
        // Run the day boundary at tick = ticks_per_day.
        let tpd = engine.resources.ticks_per_day;
        engine.resources.current_tick = tpd;
        let mut sys = PathDaySystem::new();
        sys.tick(&mut engine.world, &mut engine.resources);
        // Footfall decayed: 130 * 20/1000 = 2.6, ceil = 3 → 127.
        assert_eq!(engine.resources.colony.footfall[idx], 127);
        // 127 >= 120 → still dirt.
        assert_eq!(engine.resources.colony.path_stage[idx], STAGE_DIRT);
        // Decay over many days until it drops below dirt threshold (120).
        let tpd = engine.resources.ticks_per_day;
        for day in 2..=50 {
            engine.resources.current_tick = tpd * day;
            sys.tick(&mut engine.world, &mut engine.resources);
        }
        // After enough days footfall should drop below 120 → trodden or grass.
        assert!(engine.resources.colony.footfall[idx] < 120);
        assert_ne!(engine.resources.colony.path_stage[idx], STAGE_DIRT);
        // Eventually reaches 0.
        for day in 51..=500 {
            engine.resources.current_tick = tpd * day;
            sys.tick(&mut engine.world, &mut engine.resources);
        }
        assert_eq!(engine.resources.colony.footfall[idx], 0);
        assert_eq!(engine.resources.colony.path_stage[idx], STAGE_GRASS);
    }

    #[test]
    fn paved_road_not_demoted_by_decay() {
        let mut engine = colony_engine(4, 4);
        let idx = engine.resources.tile_grid.idx(2, 2);
        engine.resources.colony.path_stage[idx] = STAGE_PAVED;
        engine.resources.colony.footfall[idx] = 10;
        engine.resources.colony.footfall_touched.insert((2, 2));
        let tpd = engine.resources.ticks_per_day;
        engine.resources.current_tick = tpd;
        let mut sys = PathDaySystem::new();
        sys.tick(&mut engine.world, &mut engine.resources);
        // Paved stays paved regardless of footfall.
        assert_eq!(engine.resources.colony.path_stage[idx], STAGE_PAVED);
    }

    #[test]
    fn stage_promotion_on_touched_tile() {
        let mut engine = colony_engine(4, 4);
        crate::runtime::nav::ensure(&mut engine.resources);
        let nav_version = engine.resources.nav.version;
        let path_version = engine.resources.colony.path_version;
        let idx = engine.resources.tile_grid.idx(1, 0);
        engine.resources.colony.footfall[idx] = 35;
        engine.resources.colony.footfall_touched.insert((1, 0));
        let tpd = engine.resources.ticks_per_day;
        engine.resources.current_tick = tpd;
        let mut sys = PathDaySystem::new();
        sys.tick(&mut engine.world, &mut engine.resources);
        // 35 decays by ceil(35*20/1000) = 1 → 34, still >= 30 → trodden.
        assert_eq!(engine.resources.colony.path_stage[idx], STAGE_TRODDEN);
        assert_eq!(
            engine.resources.nav.version, nav_version,
            "trodden retains plain nav cost"
        );
        assert_eq!(
            engine.resources.colony.path_version,
            path_version + 1,
            "visual-only promotion still invalidates terrain"
        );
        engine.resources.current_tick += tpd;
        sys.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            engine.resources.colony.path_version,
            path_version + 1,
            "counter ignores unchanged stages"
        );
    }
}

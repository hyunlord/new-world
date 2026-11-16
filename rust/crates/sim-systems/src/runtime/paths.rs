//! PathDaySystem: day-boundary decay and stage update for worn paths (M-GROW 3.2).
//!
//! Priority: 145 · Interval: `ticks_per_day` · Gate: colony only
//! Reads: `colony.footfall`, `colony.path_stage`, `colony.footfall_touched`
//! Writes: `colony.footfall` (decay), `colony.path_stage` (promote/demote), nav (refresh)

use hecs::World;
use sim_core::colony::paths;
use sim_engine::{RuntimeSystem, SimResources};

pub struct PathDaySystem;

impl PathDaySystem {
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
        let grid_len =
            (resources.tile_grid.width as usize) * (resources.tile_grid.height as usize);
        if resources.colony.footfall.len() != grid_len {
            resources.colony.footfall.resize(grid_len, 0);
        }
        if resources.colony.path_stage.len() != grid_len {
            resources.colony.path_stage.resize(grid_len, 0);
        }
        // Only iterate tiles touched by footfall this day (O(touched), not O(grid)).
        let touched: Vec<(u32, u32)> = resources.colony.footfall_touched.iter().copied().collect();
        resources.colony.footfall_touched.clear();
        // Also check all tiles with nonzero footfall for decay (they may not
        // have been walked today). We collect indices to avoid borrowing issues.
        let decaying: Vec<usize> = resources
            .colony
            .footfall
            .iter()
            .enumerate()
            .filter(|(_, &f)| f > 0)
            .map(|(i, _)| i)
            .collect();
        let decay_permille = u32::from(data.daily_decay_permille);
        let mut nav_dirty: Vec<(u32, u32)> = Vec::new();
        for idx in decaying {
            let old_footfall = resources.colony.footfall[idx];
            // Decay: subtract decay_permille/1000 of the current value, minimum 1
            // when footfall > 0 so paths eventually fade.
            let loss = ((u32::from(old_footfall) * decay_permille + 999) / 1000).max(1) as u16;
            let new_footfall = old_footfall.saturating_sub(loss);
            resources.colony.footfall[idx] = new_footfall;
            let old_stage = resources.colony.path_stage[idx];
            // Paved road (code 3) is placed only; never demote it via footfall.
            if old_stage == paths::STAGE_PAVED {
                continue;
            }
            let new_stage = data.stage_for_footfall(new_footfall).code;
            if new_stage != old_stage {
                resources.colony.path_stage[idx] = new_stage;
                let w = resources.tile_grid.width;
                let x = (idx % w as usize) as u32;
                let y = (idx / w as usize) as u32;
                nav_dirty.push((x, y));
            }
        }
        // Also promote tiles that gained enough footfall this day.
        for tile in touched {
            let idx = resources.tile_grid.idx(tile.0, tile.1);
            let stage = resources.colony.path_stage[idx];
            if stage == paths::STAGE_PAVED {
                continue;
            }
            let footfall = resources.colony.footfall[idx];
            let new_stage = data.stage_for_footfall(footfall).code;
            if new_stage != stage {
                resources.colony.path_stage[idx] = new_stage;
                nav_dirty.push(tile);
            }
        }
        // Refresh nav for tiles whose stage (and thus cost) changed.
        for tile in nav_dirty {
            crate::runtime::nav::refresh(resources, tile);
        }
    }
}

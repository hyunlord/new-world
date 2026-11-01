//! M-LOOP decision-card reads (presentation only, never mutate): the harvest
//! rectangle a food card may offer, chosen with the same validity the order
//! uses (`DesignateHarvest`: a plant, not yet marked, reachable from camp), and
//! a summary of the colony command log so tools can find card answers in it.

use godot::prelude::*;
use sim_engine::SimResources;

/// Half-width of the offered harvest rectangle around its nearest plant.
pub const HARVEST_HALF: u32 = 3;

/// A rectangle in which `DesignateHarvest` would mark `count` plants (tile coordinates).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HarvestTarget {
    /// Whether any markable plant exists.
    pub found: bool,
    /// Corner x (inclusive).
    pub x0: u32,
    /// Corner y (inclusive).
    pub y0: u32,
    /// Corner x (inclusive).
    pub x1: u32,
    /// Corner y (inclusive).
    pub y1: u32,
    /// Plants the order would mark in the rectangle.
    pub count: u32,
}

fn markable(res: &SimResources, tile: (u32, u32)) -> bool {
    res.food.plants.contains_key(&tile)
        && !res.colony.designations.contains_key(&tile)
        && sim_systems::commands::camp_reaches(res, tile)
}

/// The rectangle around the markable plant nearest the hearth (ties by `(distance, y, x)`);
/// `found = false` without a colony hearth or a markable plant.
pub fn collect_harvest_target(res: &SimResources) -> HarvestTarget {
    let Some((hx, hy)) = sim_systems::worldgen::colony::hearth(res) else {
        return HarvestTarget::default();
    };
    let nearest = res
        .food
        .plants
        .keys()
        .copied()
        .filter(|t| markable(res, *t))
        .min_by_key(|&(x, y)| {
            let dx = i64::from(x) - i64::from(hx);
            let dy = i64::from(y) - i64::from(hy);
            (dx * dx + dy * dy, y, x)
        });
    let Some((px, py)) = nearest else {
        return HarvestTarget::default();
    };
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let x0 = px.saturating_sub(HARVEST_HALF);
    let y0 = py.saturating_sub(HARVEST_HALF);
    let x1 = (px + HARVEST_HALF).min(w.saturating_sub(1));
    let y1 = (py + HARVEST_HALF).min(h.saturating_sub(1));
    let count = res
        .food
        .plants
        .keys()
        .filter(|&&(x, y)| x >= x0 && x <= x1 && y >= y0 && y <= y1)
        .filter(|t| markable(res, **t))
        .count();
    HarvestTarget {
        found: true,
        x0,
        y0,
        x1,
        y1,
        count: u32::try_from(count).unwrap_or(u32::MAX),
    }
}

/// `{found, x0, y0, x1, y1, count}`.
pub(super) fn harvest_target_to_dict(t: HarvestTarget) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", t.found);
    d.set("x0", i64::from(t.x0));
    d.set("y0", i64::from(t.y0));
    d.set("x1", i64::from(t.x1));
    d.set("y1", i64::from(t.y1));
    d.set("count", i64::from(t.count));
    d
}

/// One command-log row: the tick it was logged at, the command's variant name and whether it was accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandLogRow {
    /// Tick recorded by the log (the engine's next tick at submission).
    pub tick: u64,
    /// The `Command` variant name (for example `DesignateHarvest`).
    pub kind: String,
    /// The command's result.
    pub accepted: bool,
}

/// Rows of `colony.command_log` from index `start` in log order. The log has no source field:
/// player and steward commands both enter it.
pub fn collect_command_log(res: &SimResources, start: usize) -> Vec<CommandLogRow> {
    res.colony
        .command_log
        .iter()
        .skip(start)
        .map(|r| {
            let debug = format!("{:?}", r.command);
            let kind = debug
                .split(|c: char| !c.is_ascii_alphanumeric())
                .next()
                .unwrap_or_default()
                .to_owned();
            CommandLogRow {
                tick: r.tick,
                kind,
                accepted: r.result.accepted,
            }
        })
        .collect()
}

/// `{total, ticks, kinds, accepted}`; `total` is the whole log length.
pub(super) fn command_log_to_dict(total: usize, rows: &[CommandLogRow]) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("total", i64::try_from(total).unwrap_or(i64::MAX));
    let ticks: Vec<i64> = rows
        .iter()
        .map(|r| i64::try_from(r.tick).unwrap_or(i64::MAX))
        .collect();
    d.set("ticks", PackedInt64Array::from(ticks.as_slice()));
    let mut kinds = PackedStringArray::new();
    for r in rows {
        kinds.push(&GString::from(&r.kind));
    }
    d.set("kinds", kinds);
    let accepted: Vec<u8> = rows.iter().map(|r| u8::from(r.accepted)).collect();
    d.set("accepted", PackedByteArray::from(accepted.as_slice()));
    d
}

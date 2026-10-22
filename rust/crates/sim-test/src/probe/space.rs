//! M3 measurement (goal §10, F1–F9): built space, rooms and pathing in a
//! colony probe run.
//!
//! Measurement only: reads the engine after each tick, never writes it. The
//! report is the `m3` block of the colony summary:
//! - `bedroom4_tick`: the first tick a roofed bedroom with ≥ 4 beds exists (F1);
//! - `storehouse_tick`: the first tick a roofed storehouse exists, and
//!   `stock_share_end`: the share of the items on stockpile cells that lie in a
//!   roofed storehouse at the end (F2);
//! - `dig_ordered` / `dig_tiles` / `dug_all_tick` / `dug_room_tick`: the dig
//!   order's tiles, when the last was dug and when a dug room first exists (F3);
//! - `bedroom_sleepers`: per night (the night that starts on day d), the distinct
//!   people seen asleep in a roofed bedroom at the hourly samples 22:00–06:00 (F4);
//! - `unreachable` / `unreachable_room_door`: Unreachable cancels in total and
//!   those whose target lay in a room with a door (F5), `door_steps`: agent-ticks
//!   spent on a door tile (≥ 1 = a route passed a door);
//! - `knap_top_level` (and `knap_top_level_by_day`, the top knapper's level at the end
//!   of each day), `blade_quality_early` (days 1–5) / `late` (days 15–20) (F7);
//! - `axe_chops_after_day3` (F8);
//! - `rooms_end`: every room at the end (id, role, tiles, beds, dug, doors).
use serde_json::{json, Value};
use sim_core::colony::{Command, RecipeKind, SkillKind, Skills};
use sim_core::components::{AgentState, Position, TargetKind};
use sim_core::rooms::RoomRole;
use sim_engine::SimEngine;
use std::collections::BTreeSet;

const DAY: u64 = 1_440;

/// An Unreachable cooldown entry: `((person, target tile), until tick)`.
type CooldownEntry = ((u64, (u32, u32)), u64);

/// Per-run observer (see the module header).
#[derive(Debug, Default)]
pub struct SpaceWatch {
    bedroom4_tick: Option<u64>,
    storehouse_tick: Option<u64>,
    dug_all_tick: Option<u64>,
    dug_room_tick: Option<u64>,
    bedroom_sleepers: Vec<u64>,
    night: BTreeSet<u64>,
    unreachable_seen: BTreeSet<CooldownEntry>,
    unreachable_room_door: u64,
    door_steps: u64,
    knap_by_day: Vec<u8>,
    quality_day5: (u64, u64),
    quality_day14: (u64, u64),
    axe_chops_day3: u64,
}

fn blade_quality(engine: &SimEngine) -> (u64, u64) {
    engine
        .resources
        .colony
        .tallies
        .craft_quality
        .iter()
        .filter(|((_, r), _)| *r == RecipeKind::KnapBlade.code())
        .fold((0, 0), |(n, s), (_, (c, q))| (n + c, s + q))
}

fn dig_tiles(engine: &SimEngine) -> Vec<(u32, u32)> {
    engine
        .resources
        .colony
        .command_log
        .iter()
        .filter(|r| r.result.accepted)
        .filter_map(|r| match r.command {
            Command::DesignateDig { x0, y0, x1, y1 } => Some((x0, y0, x1, y1)),
            _ => None,
        })
        .flat_map(|(x0, y0, x1, y1)| {
            (y0.min(y1)..=y0.max(y1))
                .flat_map(move |y| (x0.min(x1)..=x0.max(x1)).map(move |x| (x, y)))
        })
        .filter(|t| {
            let g = &engine.resources.tile_grid;
            g.terrain_type[g.idx(t.0, t.1)] == sim_core::TerrainType::Hill
        })
        .collect()
}

impl SpaceWatch {
    /// A fresh observer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe the state after one tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let res = &engine.resources;
        let c = &res.colony;
        let now = res.current_tick;
        let after = now + 1;
        if self.bedroom4_tick.is_none()
            && c.rooms.values().any(|r| {
                r.roofed
                    && r.role == RoomRole::Bedroom
                    && sim_systems::runtime::rooms::beds_in(res, r) >= 4
            })
        {
            self.bedroom4_tick = Some(now);
        }
        if self.storehouse_tick.is_none() && sim_systems::runtime::rooms::storehouse_exists(res) {
            self.storehouse_tick = Some(now);
        }
        if self.dug_room_tick.is_none() && c.rooms.values().any(|r| r.dug) {
            self.dug_room_tick = Some(now);
        }
        if self.dug_all_tick.is_none() && after.is_multiple_of(60) {
            let tiles = dig_tiles(engine);
            let g = &res.tile_grid;
            if !tiles.is_empty() && tiles.iter().all(|t| g.is_dug(g.idx(t.0, t.1))) {
                self.dug_all_tick = Some(now);
            }
        }
        // F5: new Unreachable cancels (cooldown entries) and where their target lay.
        for (key, until) in &c.unreachable {
            if self.unreachable_seen.insert((*key, *until)) {
                let in_door_room = sim_systems::runtime::rooms::room_at(res, key.1)
                    .is_some_and(|r| !r.doors.is_empty());
                if in_door_room {
                    self.unreachable_room_door += 1;
                }
            }
        }
        // Door traversals: agents on a door tile.
        let g = &res.tile_grid;
        for (_, p) in engine.world.query::<&Position>().iter() {
            if g.is_door(g.idx(p.x, p.y)) {
                self.door_steps += 1;
            }
        }
        // F4: sleepers in a roofed bedroom, sampled hourly 22:00–06:00 (with the
        // clock offset); a night's count is the distinct people seen, closed at 06:00.
        let minute = (now + res.rules.clock_offset_ticks) % DAY;
        if minute.is_multiple_of(60) && (minute >= 1320 || minute <= 360) {
            for (_, (a, s, p)) in engine
                .world
                .query::<(&sim_core::components::Agent, &AgentState, &Position)>()
                .iter()
            {
                let asleep = matches!(
                    s,
                    AgentState::Consuming {
                        target: TargetKind::Sleep
                    }
                );
                if asleep
                    && sim_systems::runtime::rooms::roofed_as(res, (p.x, p.y), RoomRole::Bedroom)
                {
                    self.night.insert(a.id);
                }
            }
            if minute == 360 {
                self.bedroom_sleepers.push(self.night.len() as u64);
                self.night.clear();
            }
        }
        if after.is_multiple_of(DAY) {
            // F7: the top knapper's level at the end of each day.
            let top = engine
                .world
                .query::<&Skills>()
                .iter()
                .map(|(_, s)| s.level(SkillKind::Knap))
                .max()
                .unwrap_or(0);
            self.knap_by_day.push(top);
        }
        if after == 5 * DAY {
            self.quality_day5 = blade_quality(engine);
        }
        if after == 14 * DAY {
            self.quality_day14 = blade_quality(engine);
        }
        if after == 3 * DAY {
            self.axe_chops_day3 = c.tallies.chops_with_axe;
        }
    }

    /// The `m3` summary block.
    pub fn report(&self, engine: &SimEngine) -> Value {
        let res = &engine.resources;
        let c = &res.colony;
        let stocked: Vec<(bool, u64)> = c
            .ground_items
            .iter()
            .filter(|(t, _)| c.zone_of_tile.contains_key(t))
            .map(|(t, s)| {
                (
                    sim_systems::runtime::rooms::covered(res, *t),
                    u64::from(s.count),
                )
            })
            .collect();
        let total: u64 = stocked.iter().map(|(_, n)| n).sum();
        let covered: u64 = stocked.iter().filter(|(c, _)| *c).map(|(_, n)| n).sum();
        let share = if total == 0 {
            0.0
        } else {
            covered as f64 / total as f64
        };
        let knap_top = engine
            .world
            .query::<&Skills>()
            .iter()
            .map(|(_, s)| s.level(SkillKind::Knap))
            .max()
            .unwrap_or(0);
        let end = blade_quality(engine);
        let mean = |(n, s): (u64, u64)| {
            if n == 0 {
                None
            } else {
                Some(s as f64 / n as f64)
            }
        };
        let late = (end.0 - self.quality_day14.0, end.1 - self.quality_day14.1);
        let rooms: Vec<Value> = c
            .rooms
            .values()
            .map(|r| {
                json!({"id": r.id, "role": r.role.name(), "tiles": r.tiles.len(), "beds": sim_systems::runtime::rooms::beds_in(res, r), "dug": r.dug, "doors": r.doors.len(), "roofed": r.roofed})
            })
            .collect();
        let dig = dig_tiles(engine);
        let knappers: Vec<(u64, u64)> = c
            .tallies
            .craft_quality
            .iter()
            .filter(|((_, r), _)| *r == RecipeKind::KnapBlade.code())
            .map(|((a, _), (n, _))| (*a, *n))
            .collect();
        json!({
            "bedroom4_tick": self.bedroom4_tick,
            "storehouse_tick": self.storehouse_tick,
            "stock_total_end": total,
            "stock_covered_end": covered,
            "stock_share_end": share,
            "dig_ordered": !dig.is_empty(),
            "dig_tiles": dig.len(),
            "dug_all_tick": self.dug_all_tick,
            "dug_room_tick": self.dug_room_tick,
            "bedroom_sleepers": self.bedroom_sleepers,
            "unreachable": c.tallies.cancels_of(sim_core::colony::CancelReason::Unreachable),
            "unreachable_room_door": self.unreachable_room_door,
            "door_steps": self.door_steps,
            "knap_top_level": knap_top,
            "knap_top_level_by_day": self.knap_by_day,
            "blade_quality_early": mean(self.quality_day5),
            "blade_quality_late": mean(late),
            "blades_early": self.quality_day5.0,
            "blades_late": late.0,
            "axe_chops_after_day3": c.tallies.chops_with_axe.saturating_sub(self.axe_chops_day3),
            "space_built": c.tallies.space_built,
            "space_removed": c.tallies.space_removed,
            "tiles_dug": c.tallies.tiles_dug,
            "rooms_end": rooms,
            "blades_by_agent": knappers,
        })
    }
}

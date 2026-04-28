//! WildlifeRuntimeSystem — spawn wildlife at tick 1 and drive idle wander.
//!
//! Cold-tier system (tick_interval = 1, priority = 115).
//! tick_interval = 1 ensures spawn fires at tick 1 (before any interval-60
//! logic). Wander is internally throttled to every 60 ticks.
//!
//! Phase A1 behaviour:
//!   - Spawn: once at first run (tick 1), places 3 wolves / 2 bears / 2 boars
//!     on passable tiles ≥ WILDLIFE_SPAWN_MIN_DIST_FROM_SETTLEMENT from any
//!     settlement centre.
//!   - Wander: every 60 ticks each wildlife entity attempts a ±1 tile step
//!     that stays within its wander_radius of home_tile.

use hecs::World;
use rand::Rng;
use sim_core::components::{Identity, InfluenceEmitter, Position, Wildlife, WildlifeKind};
use sim_core::config;
use sim_core::{ChannelId, FalloffType};
use sim_engine::{SimResources, SimSystem};

/// Runtime system: spawns wildlife at tick 1 then drives wander every 60 ticks.
pub struct WildlifeRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    spawned: bool,
}

impl WildlifeRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self { priority, tick_interval, spawned: false }
    }
}

impl SimSystem for WildlifeRuntimeSystem {
    fn name(&self) -> &'static str {
        "wildlife_runtime_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        // ── Spawn phase (once at tick 1) ───────────────────────────────────
        if !self.spawned {
            self.spawned = true;

            let settlement_centers: Vec<(i32, i32)> = resources
                .settlements
                .values()
                .map(|s| (s.x, s.y))
                .collect();

            let mut candidates: Vec<(i32, i32)> = Vec::new();
            for gy in 0..resources.map.height {
                for gx in 0..resources.map.width {
                    if !resources.map.get(gx, gy).passable {
                        continue;
                    }
                    let x = gx as i32;
                    let y = gy as i32;
                    let far_enough = settlement_centers.iter().all(|&(sx, sy)| {
                        (x - sx).abs().max((y - sy).abs())
                            >= config::WILDLIFE_SPAWN_MIN_DIST_FROM_SETTLEMENT
                    });
                    if far_enough {
                        candidates.push((x, y));
                    }
                }
            }

            let spawn_plan: &[(WildlifeKind, usize)] = &[
                (WildlifeKind::Wolf, config::WILDLIFE_WOLF_COUNT),
                (WildlifeKind::Bear, config::WILDLIFE_BEAR_COUNT),
                (WildlifeKind::Boar, config::WILDLIFE_BOAR_COUNT),
            ];

            for &(kind, count) in spawn_plan {
                for _ in 0..count {
                    if candidates.is_empty() {
                        break;
                    }
                    let idx = resources.rng.gen_range(0..candidates.len());
                    let (x, y) = candidates.swap_remove(idx);

                    let (species_id, name) = match kind {
                        WildlifeKind::Wolf => ("wolf", "Wolf"),
                        WildlifeKind::Bear => ("bear", "Bear"),
                        WildlifeKind::Boar => ("boar", "Boar"),
                    };

                    let wildlife = match kind {
                        WildlifeKind::Wolf => Wildlife::wolf((x, y)),
                        WildlifeKind::Bear => Wildlife::bear((x, y)),
                        WildlifeKind::Boar => Wildlife::boar((x, y)),
                    };

                    let danger_emitter = InfluenceEmitter {
                        channel: ChannelId::Danger,
                        radius: 0.0, // 0.0 → use channel default radius (5)
                        base_intensity: kind.danger_intensity(),
                        falloff: FalloffType::Exponential,
                        decay_rate: None,
                        tags: vec!["wildlife".to_string()],
                        enabled: true,
                    };

                    world.spawn((
                        Identity {
                            name: name.to_string(),
                            species_id: species_id.to_string(),
                            ..Default::default()
                        },
                        Position::new(x, y),
                        wildlife,
                        danger_emitter,
                    ));
                }
            }
        }

        // ── Danger emit liveness phase (every tick) ───────────────────────
        // Alive wildlife emit Danger via their `InfluenceEmitter` component;
        // dead wildlife (current_hp ≤ 0) must stop emitting. The emitter is
        // collected by `InfluenceRuntimeSystem::collect_component_emitters`
        // and stamped into the Danger channel during `tick_update`.
        for (_, (wildlife, emitter)) in
            world.query::<(&Wildlife, &mut InfluenceEmitter)>().iter()
        {
            if emitter.channel != ChannelId::Danger {
                continue;
            }
            let alive = wildlife.current_hp > 0.0;
            if emitter.enabled != alive {
                emitter.enabled = alive;
            }
        }

        // ── Wander phase (every 60 ticks, skip tick 0) ────────────────────
        // tick=0 satisfies 0%60==0 but spawn just fired; skip to avoid
        // displacing entities on the spawn tick before home_tile is captured.
        if tick == 0 || !tick.is_multiple_of(60) {
            return;
        }

        let mut moves: Vec<(hecs::Entity, i32, i32)> = Vec::new();
        for (entity, (wildlife, pos)) in world.query::<(&Wildlife, &Position)>().iter() {
            let px = pos.x as i32;
            let py = pos.y as i32;
            let dx: i32 = resources.rng.gen_range(-1..=1);
            let dy: i32 = resources.rng.gen_range(-1..=1);
            let cx = px + dx;
            let cy = py + dy;

            if !resources.map.in_bounds(cx, cy) {
                continue;
            }
            if !resources.map.get(cx as u32, cy as u32).passable {
                continue;
            }
            let hdx = (cx - wildlife.home_tile.0).abs();
            let hdy = (cy - wildlife.home_tile.1).abs();
            if hdx.max(hdy) > wildlife.wander_radius {
                continue;
            }
            moves.push((entity, cx, cy));
        }

        for (entity, cx, cy) in moves {
            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                pos.x = f64::from(cx);
                pos.y = f64::from(cy);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawned_flag_starts_false() {
        let sys = WildlifeRuntimeSystem::new(115, 1);
        assert!(!sys.spawned);
    }

    #[test]
    fn wolf_wander_radius_positive() {
        let w = Wildlife::wolf((50, 50));
        assert!(w.wander_radius > 0);
    }

    #[test]
    fn bear_wander_radius_positive() {
        let b = Wildlife::bear((50, 50));
        assert!(b.wander_radius > 0);
    }
}

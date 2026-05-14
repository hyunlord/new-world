//! V7 Phase 4-β — `AgentMovementSystem` (priority 120, every tick).
//!
//! Brownian-step motion for canonical agents. Each tick every agent receives
//! a `(dx, dy)` step in `{-1, 0, +1}^2` derived from its own [`MovementRng`]
//! state. Determinism comes from per-agent seeded `splitmix64` — replaying
//! the simulation with the same seeds reproduces every trajectory byte-for-byte.
//!
//! # Priority ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem  ← agents read current-side influence
//! 120  AgentMovementSystem         ← agents move AFTER reading influence
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Reading influence before moving keeps the `InfluenceSample` value
//! consistent with the agent's pre-move tile — a property β does not yet
//! exploit but γ/δ will need when motion becomes influence-guided.
//!
//! # Why inline splitmix64 (not the `rand` crate)?
//!
//! Phase 4-β scope is "agents move and tests prove determinism." Pulling in
//! `rand` along with `rand_chacha` would be net-new transitive dependency
//! surface for a 12-line PRNG; planning §2.2 explicitly authorises a
//! "deterministic seeded RNG" without dictating the crate. A per-agent `u64`
//! state advanced by `splitmix64` covers every β requirement, ships with
//! zero new deps, and keeps the migration to a proper `rand_chacha`-backed
//! source one type-swap away if a later phase needs it.

use hecs::World;
use sim_core::components::Position;
use sim_engine::{RuntimeSystem, SimResources};

/// Per-agent PRNG state used by [`AgentMovementSystem`] to compute the
/// next Brownian step. Each agent owns its own `MovementRng`, so trajectories
/// are independent and reproducible from the initial seed.
///
/// The state is advanced via `splitmix64`, a 64-bit mixing function whose
/// output stream passes the standard PractRand statistical battery for the
/// step counts a single simulation will ever realistically consume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementRng {
    state: u64,
}

impl MovementRng {
    /// Construct a new RNG seeded with `seed`.
    ///
    /// `seed == 0` is a valid seed — splitmix64 escapes the all-zero state
    /// on its first call because the additive constant is non-zero.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the internal state once and return the mixed output.
    ///
    /// Reference: Sebastiano Vigna, "Further scramblings of Marsaglia's
    /// xorshift generators." This is the canonical splitmix64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Return the next Brownian step axis value in `{-1, 0, +1}`.
    ///
    /// Uses a `u64 % 3` reduction on a fresh 64-bit sample; the bias from
    /// `2^64 mod 3 == 1` is on the order of 1 part in 2^63 and is well below
    /// the tolerance of any β assertion.
    fn next_step(&mut self) -> i32 {
        ((self.next_u64() % 3) as i32) - 1
    }
}

/// Phase 4-β movement system.
///
/// On every tick, queries `(&mut Position, &mut MovementRng)` and mutates each
/// agent's position by a Brownian `(dx, dy) ∈ {-1, 0, +1}^2` step, clamped to
/// the tile-grid bounds so coordinates never underflow `u32` or escape the
/// grid.
pub struct AgentMovementSystem;

impl AgentMovementSystem {
    /// Construct a new movement system.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentMovementSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for AgentMovementSystem {
    fn name(&self) -> &str {
        "AgentMovementSystem"
    }

    fn priority(&self) -> u32 {
        120
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let w = resources.tile_grid.width;
        let h = resources.tile_grid.height;
        if w == 0 || h == 0 {
            return;
        }
        let max_x = (w - 1) as i64;
        let max_y = (h - 1) as i64;
        for (_, (pos, rng)) in world.query::<(&mut Position, &mut MovementRng)>().iter() {
            let dx = rng.next_step() as i64;
            let dy = rng.next_step() as i64;
            let nx = (pos.x as i64 + dx).clamp(0, max_x);
            let ny = (pos.y as i64 + dy).clamp(0, max_y);
            pos.x = nx as u32;
            pos.y = ny as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn fresh_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = AgentMovementSystem::new();
        assert_eq!(s.name(), "AgentMovementSystem");
        assert_eq!(s.priority(), 120);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn splitmix_escapes_zero_seed() {
        let mut r = MovementRng::new(0);
        assert_ne!(r.next_u64(), 0);
    }

    #[test]
    fn same_seed_produces_same_stream() {
        let mut a = MovementRng::new(42);
        let mut b = MovementRng::new(42);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn agent_moves_after_one_tick() {
        let mut e = fresh_engine();
        let id = e.world.spawn((Position::new(10, 10), MovementRng::new(42)));
        let mut sys = AgentMovementSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let p = *e.world.get::<&Position>(id).unwrap();
        // (dx, dy) ∈ {-1,0,+1}^2 → distance ≤ 1 on each axis
        assert!(p.x.abs_diff(10) <= 1);
        assert!(p.y.abs_diff(10) <= 1);
    }

    #[test]
    fn boundary_clamp_low() {
        let mut e = fresh_engine();
        // Spawn at (0,0) and run many ticks; coords must stay in-bounds.
        let id = e.world.spawn((Position::new(0, 0), MovementRng::new(7)));
        let mut sys = AgentMovementSystem::new();
        for _ in 0..200 {
            sys.tick(&mut e.world, &mut e.resources);
            let p = *e.world.get::<&Position>(id).unwrap();
            assert!(p.x < 32);
            assert!(p.y < 32);
        }
    }

    #[test]
    fn distinct_seeds_diverge() {
        let mut e = fresh_engine();
        let a = e.world.spawn((Position::new(16, 16), MovementRng::new(1)));
        let b = e.world.spawn((Position::new(16, 16), MovementRng::new(2)));
        let mut sys = AgentMovementSystem::new();
        // After enough ticks two independent walks must visit different cells
        // somewhere along the way. Compare full trajectories, not endpoints.
        let mut diverged = false;
        for _ in 0..32 {
            sys.tick(&mut e.world, &mut e.resources);
            let pa = *e.world.get::<&Position>(a).unwrap();
            let pb = *e.world.get::<&Position>(b).unwrap();
            if pa != pb {
                diverged = true;
                break;
            }
        }
        assert!(diverged, "distinct seeds should produce distinct trajectories");
    }
}

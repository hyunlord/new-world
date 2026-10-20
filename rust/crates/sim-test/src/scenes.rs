//! The test crate's world fixture: the live game's baseline world.
//!
//! V8-0 replaced the per-pass scenes (`legacy_scene`, `game_scene`,
//! `game_v9_scene … game_v26_scene` and their `_nofission` / `_neutral` /
//! `_inert` / `_with` variants; available at tag `v7-final-art`) with one
//! baseline (ADR 0011).
use sim_engine::SimEngine;

/// The live game's world `seed`: `WorldGenConfig::baseline` under
/// `WorldRules::baseline` (= `game_v26` at `v7-final-art`), with the world's
/// hidden parameters stored in `resources.world_variety` for measurement.
pub fn baseline_scene(seed: u64) -> SimEngine {
    let variety = sim_core::variety::WorldVariety::draw_v18(seed);
    let mut engine = sim_systems::worldgen::build_world_with(
        &sim_systems::worldgen::WorldGenConfig::baseline(seed),
        sim_core::rules::WorldRules::baseline(seed),
    );
    engine.resources.world_variety = Some(variety);
    engine
}

#[cfg(test)]
mod tests {
    #[test]
    fn baseline_scene_starts_at_tick_zero_with_its_world_variety() {
        let engine = super::baseline_scene(7);
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(engine.resources.world_seed, 7);
        assert_eq!(
            engine.resources.world_variety,
            Some(sim_core::variety::WorldVariety::draw_v18(7))
        );
    }
}

//! `worldsim_systems`: print the live engine's registered systems.
//!
//! Builds the baseline world exactly as the game's bridge does
//! (`sim_bridge::ffi::build_game_engine`) and prints one JSON object per
//! registered system, in execution order: `name`, `priority`, `interval`.
//! `tools/v8/system_table.py` turns this into the system table of
//! `docs/ARCHITECTURE.md`, so the table always follows the registration code.
fn main() {
    let engine = sim_bridge::ffi::build_game_engine(sim_systems::worldgen::DEFAULT_GAME_SEED);
    for system in engine.new_tick_profile().systems {
        println!(
            "{}",
            serde_json::json!({"name": system.name, "priority": system.priority, "interval": system.interval})
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_live_engine_registers_systems() {
        let engine = sim_bridge::ffi::build_game_engine(1);
        assert!(engine.system_count() >= 28);
    }
}

use std::sync::{Arc, Mutex};
use sim_core::config::GameConfig;
use sim_core::{GameCalendar, WorldMap, Settlement};
use sim_core::ids::SettlementId;
use sim_engine::{SimEngine, SimResources};
use sim_systems as _systems; // Phase R-1 will populate this

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    println!("[sim-test] WorldSim Phase R-0 headless test");

    // ── Build config + calendar + map ─────────────────────────────────────────
    let config = GameConfig::default();
    let calendar = GameCalendar::new(&config);
    let map = WorldMap::new(256, 256, 0xDEAD_BEEF);

    // ── Build SimResources ────────────────────────────────────────────────────
    let mut resources = SimResources::new(calendar, map, 0xDEAD_BEEF);

    // ── Attempt data load ─────────────────────────────────────────────────────
    let data_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()  // crates/
        .parent().unwrap()  // rust/
        .parent().unwrap()  // lead/ project root
        .join("data");

    match sim_data::load_all(&data_dir) {
        Ok((emotions, tech, values)) => {
            log::info!(
                "[sim-test] data loaded: {} emotions, {} techs, {} value_events",
                emotions.len(),
                tech.len(),
                values.len(),
            );
        }
        Err(_) => {
            log::warn!("[sim-test] data not found at {:?}, skipping", data_dir);
        }
    }

    // ── Subscribe event counter ───────────────────────────────────────────────
    let event_count = Arc::new(Mutex::new(0u64));
    let ec = event_count.clone();
    resources.event_bus.subscribe(Box::new(move |_e| {
        *ec.lock().unwrap() += 1;
    }));

    // ── Create engine (no systems registered in Phase R-0) ───────────────────
    let mut engine = SimEngine::new(resources);

    // ── Add one settlement ────────────────────────────────────────────────────
    let s = Settlement::new(
        SettlementId(1),
        "Ember Hold".to_string(),
        128, 128,
        0, // founded_tick
    );
    engine.resources_mut().settlements.insert(SettlementId(1), s);

    // ── Run one in-game year (12 ticks/day × 365 days = 4380 ticks) ──────────
    engine.run_ticks(4380);

    // ── Capture snapshot ──────────────────────────────────────────────────────
    let snap = engine.snapshot();
    let dispatched = *event_count.lock().unwrap();

    // ── Print results ─────────────────────────────────────────────────────────
    println!("[sim-test] \u{2500}\u{2500} Results \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
    println!("[sim-test]   Tick:            {}", snap.tick);
    println!("[sim-test]   Date:            {}", snap.date_string());
    println!("[sim-test]   Entities:        {}", snap.entity_count);
    println!("[sim-test]   Settlements:     {}", snap.settlement_count);
    println!("[sim-test]   Events total:    {}", dispatched);
    println!("[sim-test]   Systems run:     {}", snap.system_count);
    println!("[sim-test] \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
    println!("[sim-test] PASS");

    // ── Assertions ────────────────────────────────────────────────────────────
    assert_eq!(snap.tick, 4380, "wrong tick count");
    assert_eq!(snap.year, 2, "wrong year (should be year 2 after 4380 ticks)");
    assert_eq!(snap.day_of_year, 1, "should be start of year 2");
    assert_eq!(snap.settlement_count, 1, "should have 1 settlement");
}

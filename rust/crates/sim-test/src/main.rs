use sim_core::config::GameConfig;
use sim_core::ids::SettlementId;
use sim_core::{GameCalendar, Settlement, WorldMap};
use sim_engine::{SimEngine, SimResources};
use sim_systems::stat_curve;
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--bench-stress-math") {
        run_stress_math_bench(&args);
        return;
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("[sim-test] WorldSim Phase R-0 headless test");

    // ── Build config + calendar + map ─────────────────────────────────────────
    let config = GameConfig::default();
    let calendar = GameCalendar::new(&config);
    let map = WorldMap::new(256, 256, 0xDEAD_BEEF);

    // ── Build SimResources ────────────────────────────────────────────────────
    let mut resources = SimResources::new(calendar, map, 0xDEAD_BEEF);

    // ── Attempt data load ─────────────────────────────────────────────────────
    let data_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // rust/
        .parent()
        .unwrap() // lead/ project root
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
        128,
        128,
        0, // founded_tick
    );
    engine
        .resources_mut()
        .settlements
        .insert(SettlementId(1), s);

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
    assert_eq!(
        snap.year, 2,
        "wrong year (should be year 2 after 4380 ticks)"
    );
    assert_eq!(snap.day_of_year, 1, "should be start of year 2");
    assert_eq!(snap.settlement_count, 1, "should have 1 settlement");
}

fn run_stress_math_bench(args: &[String]) {
    let mut iterations: u32 = 200_000;
    for i in 0..args.len() {
        if args[i] == "--iters" && i + 1 < args.len() {
            if let Ok(parsed) = args[i + 1].parse::<u32>() {
                iterations = parsed.max(1);
            }
        }
    }

    let trace_per_tick: [f32; 8] = [3.0, 2.5, 2.0, 1.5, 1.0, 0.8, 0.5, 0.2];
    let trace_decay: [f32; 8] = [0.05, 0.04, 0.03, 0.02, 0.05, 0.06, 0.07, 0.08];

    let started = Instant::now();
    let mut checksum = 0.0_f32;
    for i in 0..iterations {
        let t = (i % 100) as f32 / 100.0;
        let hunger = 0.2 + 0.6 * t;
        let energy = 0.1 + 0.7 * (1.0 - t);
        let social = 0.15 + 0.6 * t;
        let stress = 120.0 + 520.0 * t;
        let allostatic = 5.0 + 80.0 * t;

        let continuous =
            stat_curve::stress_continuous_inputs(hunger, energy, social, 1.0 + 0.5 * t);
        let appraisal = stat_curve::stress_appraisal_scale(
            hunger,
            energy,
            social,
            0.1 + 0.7 * t,
            0.2 * t,
            0.3 + 0.5 * (1.0 - t),
            0.4 + 0.4 * t,
            20.0 + 70.0 * t,
            70.0 - 40.0 * t,
            0.3 + 0.6 * (1.0 - t),
            0.3 + 0.6 * t,
            0.2 + 0.7 * (1.0 - t),
        );
        let emotion = stat_curve::stress_emotion_contribution(
            10.0 + 80.0 * t,
            5.0 + 70.0 * t,
            12.0 + 65.0 * t,
            8.0 + 50.0 * t,
            8.0 + 35.0 * t,
            30.0 - 20.0 * t,
            30.0 - 15.0 * t,
            25.0 - 10.0 * t,
            -50.0 + 20.0 * t,
            40.0 + 45.0 * t,
        );
        let recovery = stat_curve::stress_recovery_value(
            stress,
            0.2 + 0.6 * t,
            0.1 + 0.7 * (1.0 - t),
            30.0 + 60.0 * (1.0 - t),
            i % 3 == 0,
            i % 2 == 0,
        );
        let reserve_step = stat_curve::stress_reserve_step(
            30.0 + 60.0 * (1.0 - t),
            stress,
            0.1 + 0.7 * (1.0 - t),
            -10.0 + 80.0 * t,
            (i % 4) as i32,
            i % 3 == 0,
        );
        let allo_step = stat_curve::stress_allostatic_step(
            allostatic,
            stress,
            if i % 2 == 0 { 1.35 } else { 1.0 },
        );
        let state = stat_curve::stress_state_snapshot(stress, allostatic);
        let resilience = stat_curve::stress_resilience_value(
            0.2 + 0.6 * t,
            0.2 + 0.7 * (1.0 - t),
            0.2 + 0.6 * t,
            0.2 + 0.5 * t,
            0.2 + 0.5 * (1.0 - t),
            0.2 + 0.4 * t,
            0.3 + 0.6 * (1.0 - t),
            allostatic,
            hunger,
            energy,
            -0.03 * t,
        );
        let traces = stat_curve::stress_trace_batch_step(&trace_per_tick, &trace_decay, 0.01);
        let delta_step = stat_curve::stress_delta_step(
            continuous.total,
            traces.total_contribution,
            emotion.total,
            1.1,
            0.95,
            recovery,
            0.05,
            i % 2 == 0,
            0.6,
            50.0 + 400.0 * t,
            800.0,
        );

        checksum += black_box(continuous.total)
            + black_box(appraisal)
            + black_box(emotion.total)
            + black_box(recovery)
            + black_box(reserve_step.reserve)
            + black_box(allo_step)
            + black_box(state.stress_blunt_mult)
            + black_box(resilience)
            + black_box(traces.total_contribution)
            + black_box(delta_step.delta)
            + black_box(delta_step.hidden_threat_accumulator);
    }
    let elapsed = started.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / f64::from(iterations);
    println!(
        "[sim-test] stress-math bench: iterations={} elapsed_ms={:.3} ns_per_iter={:.1} checksum={:.5}",
        iterations,
        elapsed.as_secs_f64() * 1000.0,
        ns_per_iter,
        checksum
    );
}

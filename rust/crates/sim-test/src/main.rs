use sim_core::config::GameConfig;
use sim_core::ids::SettlementId;
use sim_core::{GameCalendar, Settlement, WorldMap};
use sim_engine::{SimEngine, SimResources};
use sim_bridge::{
    get_pathfind_backend_mode, pathfind_backend_dispatch_counts, pathfind_grid_batch_dispatch_bytes,
    pathfind_grid_batch_xy_dispatch_bytes, resolve_pathfind_backend_mode,
    reset_pathfind_backend_dispatch_counts, set_pathfind_backend_mode,
};
use sim_systems::{body, stat_curve};
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--bench-needs-math") {
        run_needs_math_bench(&args);
        return;
    }
    if args.iter().any(|arg| arg == "--bench-pathfind-bridge-split") {
        run_pathfind_bridge_split_bench(&args);
        return;
    }
    if args.iter().any(|arg| arg == "--bench-pathfind-backend-smoke") {
        run_pathfind_backend_smoke(&args);
        return;
    }
    if args.iter().any(|arg| arg == "--bench-pathfind-bridge") {
        run_pathfind_bridge_bench(&args);
        return;
    }
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

fn parse_bench_iterations(args: &[String], default_iterations: u32) -> u32 {
    let mut iterations: u32 = default_iterations.max(1);
    for i in 0..args.len() {
        if args[i] == "--iters" && i + 1 < args.len() {
            if let Ok(parsed) = args[i + 1].parse::<u32>() {
                iterations = parsed.max(1);
            }
        }
    }
    iterations
}

fn parse_pathfind_backend_arg(args: &[String]) -> String {
    let mut mode = String::from("auto");
    for i in 0..args.len() {
        if args[i] == "--backend" && i + 1 < args.len() {
            mode = args[i + 1].to_ascii_lowercase();
        }
    }
    mode
}

fn run_needs_math_bench(args: &[String]) {
    let iterations = parse_bench_iterations(args, 200_000);
    let potentials: [i32; 6] = [640, 700, 670, 620, 690, 710];
    let trainabilities: [i32; 5] = [560, 520, 610, 500, 580];
    let training_ceilings: [f32; 5] = [2.5, 0.3, 1.5, 0.2, 0.6];

    let started = Instant::now();
    let mut checksum = 0.0_f32;
    for i in 0..iterations {
        let t = (i % 100) as f32 / 100.0;
        let age_years = 5.0 + 85.0 * t;
        let end_norm = 0.1 + 0.8 * t;
        let rec_norm = 0.9 - 0.7 * t;
        let hunger = 0.1 + 0.8 * (1.0 - t);
        let tile_temp = -0.2 + 1.2 * t;
        let xps = [
            100.0 + 1800.0 * t,
            120.0 + 1400.0 * t,
            90.0 + 2100.0 * t,
            30.0 + 700.0 * t,
            140.0 + 1700.0 * t,
        ];

        let curves = body::compute_age_curves(age_years);
        let train_mods = body::age_trainability_modifiers(age_years);
        let gains = body::calc_training_gains(
            &potentials[..5],
            &trainabilities,
            &xps,
            &training_ceilings,
            5000.0,
        );
        let realized = body::calc_realized_values(
            &potentials,
            &trainabilities,
            &xps,
            &training_ceilings,
            age_years,
            5000.0,
        );
        let action_cost = body::action_energy_cost(0.006, end_norm, 0.35);
        let rest_recovery = body::rest_energy_recovery(0.012, rec_norm, 0.5);
        let thirst = body::thirst_decay(0.0035, tile_temp, 0.5);
        let warmth = body::warmth_decay(0.0030, tile_temp, i % 5 != 0, 0.5, 0.2, 0.35);
        let decay_step = body::needs_base_decay_step(
            hunger,
            0.0030,
            1.0,
            0.2,
            0.7,
            0.0020,
            0.0016,
            0.0012,
            0.0035,
            0.0030,
            tile_temp,
            i % 7 != 0,
            0.5,
            0.2,
            0.35,
            true,
        );
        let severity = body::needs_critical_severity_step(
            0.09 + 0.5 * t,
            0.12 + 0.45 * t,
            0.2 + 0.5 * t,
            0.2,
            0.25,
            0.35,
        );
        let best_skill_norm = body::upper_needs_best_skill_normalized(
            &[
                20 + (i % 30) as i32,
                15 + (i % 40) as i32,
                10 + (i % 45) as i32,
                12,
                18,
            ],
            100,
        );
        let job_code = if i % 2 == 0 { 1 } else { 2 };
        let alignment =
            body::upper_needs_job_alignment(job_code, 0.5 + 0.3 * t, 0.4 + 0.2 * t, 0.3, 0.6, 0.5);
        let upper_step = body::upper_needs_step(
            &[0.5, 0.6, 0.55, 0.52, 0.48, 0.51, 0.58, 0.47],
            &[0.002, 0.002, 0.002, 0.0015, 0.0012, 0.0018, 0.0017, 0.0016],
            0.01,
            0.01,
            0.01,
            0.01,
            0.02,
            0.02,
            0.005,
            0.008,
            0.006,
            0.007,
            best_skill_norm,
            alignment,
            -0.2 + 1.0 * t,
            i % 3 != 0,
            i % 4 != 0,
            i % 5 != 0,
        );
        let parent_transfer = body::child_parent_stress_transfer(
            0.1 + 0.8 * t,
            0.3 + 0.6 * (1.0 - t),
            (i % 4) as i32,
            i % 2 == 0,
            0.1 + 0.6 * t,
            0.2 + 0.4 * (1.0 - t),
        );
        let ace_step =
            body::child_simultaneous_ace_step(&[0.2 + 0.5 * t, 0.1 + 0.6 * t, 0.3], 0.4 * t);
        let shrp_step = body::child_shrp_step(0.2 + 0.9 * t, i % 2 == 0, 0.7, 1.2);
        let stress_type_code =
            body::child_stress_type_code(0.2 + 0.7 * t, i % 3 == 0, 0.4 + 0.5 * t);
        let child_apply = body::child_stress_apply_step(
            0.3 + 0.4 * t,
            50.0 + 40.0 * (1.0 - t),
            200.0 + 300.0 * t,
            5.0 + 25.0 * t,
            0.2 + 0.7 * t,
            0.9 + 0.5 * t,
            0.8 + 0.7 * (1.0 - t),
            0.8 + 0.6 * t,
            stress_type_code,
        );
        let child_parent_applied = body::child_parent_transfer_apply_step(
            100.0 + 800.0 * t,
            parent_transfer,
            0.05,
            20.0,
            2000.0,
        );
        let child_deprivation =
            body::child_deprivation_damage_step(0.2 + 1.0 * t, 0.01 + 0.03 * (1.0 - t));
        let child_stage_code = body::child_stage_code_from_age_ticks(
            8760 * ((i % 22) as i32),
            2.0,
            5.0,
            12.0,
            18.0,
        );

        checksum += black_box(curves[0])
            + black_box(curves[5])
            + black_box(train_mods[2])
            + black_box(train_mods[4])
            + black_box(*gains.first().unwrap_or(&0) as f32)
            + black_box(*gains.get(2).unwrap_or(&0) as f32)
            + black_box(*realized.first().unwrap_or(&0) as f32)
            + black_box(*realized.get(5).unwrap_or(&0) as f32)
            + black_box(action_cost)
            + black_box(rest_recovery)
            + black_box(thirst)
            + black_box(warmth)
            + black_box(decay_step[0])
            + black_box(decay_step[4])
            + black_box(decay_step[5])
            + black_box(severity[0])
            + black_box(severity[1])
            + black_box(severity[2])
            + black_box(best_skill_norm)
            + black_box(alignment)
            + black_box(upper_step[0])
            + black_box(upper_step[4])
            + black_box(upper_step[7])
            + black_box(parent_transfer)
            + black_box(ace_step[0])
            + black_box(ace_step[1])
            + black_box(ace_step[2])
            + black_box(shrp_step[0])
            + black_box(shrp_step[1])
            + black_box(stress_type_code as f32)
            + black_box(child_apply[0])
            + black_box(child_apply[2])
            + black_box(child_apply[4])
            + black_box(child_parent_applied)
            + black_box(child_deprivation)
            + black_box(child_stage_code as f32);
    }
    let elapsed = started.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / f64::from(iterations);
    println!(
        "[sim-test] needs-math bench: iterations={} elapsed_ms={:.3} ns_per_iter={:.1} checksum={:.5}",
        iterations,
        elapsed.as_secs_f64() * 1000.0,
        ns_per_iter,
        checksum
    );
}

fn pathfind_bench_inputs() -> (
    i32,
    i32,
    Vec<u8>,
    Vec<f32>,
    [(i32, i32); 8],
    [(i32, i32); 8],
    Vec<i32>,
    Vec<i32>,
    usize,
) {
    let width: i32 = 64;
    let height: i32 = 64;
    let cell_count = (width * height) as usize;
    let max_steps: usize = cell_count;

    let mut walkable: Vec<u8> = vec![1; cell_count];
    let mut move_cost: Vec<f32> = vec![1.0; cell_count];
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            walkable[idx] = 1;
            move_cost[idx] = 1.0 + ((x * 3 + y * 5) % 7) as f32 * 0.05;
        }
    }

    let from_points: [(i32, i32); 8] = [
        (1, 1),
        (2, 40),
        (8, 10),
        (12, 50),
        (20, 4),
        (32, 18),
        (40, 40),
        (5, 58),
    ];
    let to_points: [(i32, i32); 8] = [
        (62, 62),
        (55, 3),
        (45, 20),
        (50, 54),
        (60, 8),
        (10, 45),
        (3, 60),
        (58, 12),
    ];

    let mut from_xy: Vec<i32> = Vec::with_capacity(from_points.len() * 2);
    let mut to_xy: Vec<i32> = Vec::with_capacity(to_points.len() * 2);
    for idx in 0..from_points.len() {
        from_xy.push(from_points[idx].0);
        from_xy.push(from_points[idx].1);
        to_xy.push(to_points[idx].0);
        to_xy.push(to_points[idx].1);
    }

    (
        width, height, walkable, move_cost, from_points, to_points, from_xy, to_xy, max_steps,
    )
}

fn run_pathfind_bridge_bench(args: &[String]) {
    let iterations = parse_bench_iterations(args, 1_000);
    let backend_mode = parse_pathfind_backend_arg(args);
    if !set_pathfind_backend_mode(&backend_mode) {
        eprintln!(
            "[sim-test] invalid --backend value: {} (expected auto|cpu|gpu)",
            backend_mode
        );
        std::process::exit(2);
    }
    let (width, height, walkable, move_cost, from_points, to_points, from_xy, to_xy, max_steps) =
        pathfind_bench_inputs();

    reset_pathfind_backend_dispatch_counts();
    let started = Instant::now();
    let mut checksum = 0.0_f32;
    for _ in 0..iterations {
        let groups = pathfind_grid_batch_dispatch_bytes(
            width,
            height,
            &walkable,
            &move_cost,
            &from_points,
            &to_points,
            max_steps,
        )
        .expect("pathfind_grid_batch_dispatch_bytes");
        let groups_xy = pathfind_grid_batch_xy_dispatch_bytes(
            width, height, &walkable, &move_cost, &from_xy, &to_xy, max_steps,
        )
        .expect("pathfind_grid_batch_xy_dispatch_bytes");

        for i in 0..groups.len() {
            checksum += black_box(groups[i].len() as f32);
        }
        for i in 0..groups_xy.len() {
            checksum += black_box(groups_xy[i].len() as f32);
        }
    }

    let elapsed = started.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / f64::from(iterations);
    println!(
        "[sim-test] pathfind-bridge bench: iterations={} elapsed_ms={:.3} ns_per_iter={:.1} checksum={:.5}",
        iterations,
        elapsed.as_secs_f64() * 1000.0,
        ns_per_iter,
        checksum
    );
    let (cpu_dispatches, gpu_dispatches) = pathfind_backend_dispatch_counts();
    println!(
        "[sim-test] pathfind-bridge backend: configured={} resolved={} cpu={} gpu={} total={}",
        get_pathfind_backend_mode(),
        resolve_pathfind_backend_mode(),
        cpu_dispatches,
        gpu_dispatches,
        cpu_dispatches + gpu_dispatches
    );
}

fn run_pathfind_bridge_split_bench(args: &[String]) {
    let iterations = parse_bench_iterations(args, 1_000);
    let backend_mode = parse_pathfind_backend_arg(args);
    if !set_pathfind_backend_mode(&backend_mode) {
        eprintln!(
            "[sim-test] invalid --backend value: {} (expected auto|cpu|gpu)",
            backend_mode
        );
        std::process::exit(2);
    }
    let (width, height, walkable, move_cost, from_points, to_points, from_xy, to_xy, max_steps) =
        pathfind_bench_inputs();

    reset_pathfind_backend_dispatch_counts();
    let started_tuple = Instant::now();
    let mut checksum_tuple = 0.0_f32;
    for _ in 0..iterations {
        let groups = pathfind_grid_batch_dispatch_bytes(
            width,
            height,
            &walkable,
            &move_cost,
            &from_points,
            &to_points,
            max_steps,
        )
        .expect("pathfind_grid_batch_dispatch_bytes");
        for i in 0..groups.len() {
            checksum_tuple += black_box(groups[i].len() as f32);
        }
    }
    let elapsed_tuple = started_tuple.elapsed();
    let tuple_ns_per_iter = elapsed_tuple.as_nanos() as f64 / f64::from(iterations);
    println!(
        "[sim-test] pathfind-bridge-split tuple: iterations={} elapsed_ms={:.3} ns_per_iter={:.1} checksum={:.5}",
        iterations,
        elapsed_tuple.as_secs_f64() * 1000.0,
        tuple_ns_per_iter,
        checksum_tuple
    );
    let (cpu_tuple, gpu_tuple) = pathfind_backend_dispatch_counts();
    println!(
        "[sim-test] pathfind-bridge-split tuple-backend-dispatches: configured={} resolved={} cpu={} gpu={} total={}",
        get_pathfind_backend_mode(),
        resolve_pathfind_backend_mode(),
        cpu_tuple,
        gpu_tuple,
        cpu_tuple + gpu_tuple
    );

    reset_pathfind_backend_dispatch_counts();
    let started_xy = Instant::now();
    let mut checksum_xy = 0.0_f32;
    for _ in 0..iterations {
        let groups_xy = pathfind_grid_batch_xy_dispatch_bytes(
            width, height, &walkable, &move_cost, &from_xy, &to_xy, max_steps,
        )
        .expect("pathfind_grid_batch_xy_dispatch_bytes");
        for i in 0..groups_xy.len() {
            checksum_xy += black_box(groups_xy[i].len() as f32);
        }
    }
    let elapsed_xy = started_xy.elapsed();
    let xy_ns_per_iter = elapsed_xy.as_nanos() as f64 / f64::from(iterations);
    println!(
        "[sim-test] pathfind-bridge-split xy: iterations={} elapsed_ms={:.3} ns_per_iter={:.1} checksum={:.5}",
        iterations,
        elapsed_xy.as_secs_f64() * 1000.0,
        xy_ns_per_iter,
        checksum_xy
    );
    let (cpu_xy, gpu_xy) = pathfind_backend_dispatch_counts();
    println!(
        "[sim-test] pathfind-bridge-split xy-backend-dispatches: configured={} resolved={} cpu={} gpu={} total={}",
        get_pathfind_backend_mode(),
        resolve_pathfind_backend_mode(),
        cpu_xy,
        gpu_xy,
        cpu_xy + gpu_xy
    );
}

fn run_pathfind_backend_smoke(args: &[String]) {
    let iterations = parse_bench_iterations(args, 10);
    let (width, height, walkable, move_cost, from_points, to_points, from_xy, to_xy, max_steps) =
        pathfind_bench_inputs();

    for mode in ["auto", "cpu", "gpu"] {
        if !set_pathfind_backend_mode(mode) {
            eprintln!(
                "[sim-test] invalid backend mode in smoke run: {} (expected auto|cpu|gpu)",
                mode
            );
            std::process::exit(2);
        }
        reset_pathfind_backend_dispatch_counts();

        let mut checksum = 0.0_f32;
        for _ in 0..iterations {
            let groups = pathfind_grid_batch_dispatch_bytes(
                width,
                height,
                &walkable,
                &move_cost,
                &from_points,
                &to_points,
                max_steps,
            )
            .expect("pathfind_grid_batch_dispatch_bytes");
            let groups_xy = pathfind_grid_batch_xy_dispatch_bytes(
                width, height, &walkable, &move_cost, &from_xy, &to_xy, max_steps,
            )
            .expect("pathfind_grid_batch_xy_dispatch_bytes");
            for i in 0..groups.len() {
                checksum += black_box(groups[i].len() as f32);
            }
            for i in 0..groups_xy.len() {
                checksum += black_box(groups_xy[i].len() as f32);
            }
        }

        let (cpu_dispatches, gpu_dispatches) = pathfind_backend_dispatch_counts();
        println!(
            "[sim-test] pathfind-backend-smoke: mode={} configured={} resolved={} iterations={} checksum={:.5} cpu={} gpu={} total={}",
            mode,
            get_pathfind_backend_mode(),
            resolve_pathfind_backend_mode(),
            iterations,
            checksum,
            cpu_dispatches,
            gpu_dispatches,
            cpu_dispatches + gpu_dispatches
        );
    }
}

fn run_stress_math_bench(args: &[String]) {
    let iterations = parse_bench_iterations(args, 200_000);

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
        let support_score = body::stress_support_score(&[
            0.1 + 0.8 * t,
            0.2 + 0.6 * (1.0 - t),
            0.15 + 0.5 * t,
            0.05 + 0.4 * (1.0 - t),
        ]);
        let rebound_apply = body::stress_rebound_apply_step(
            stress,
            50.0 + 400.0 * t,
            2.0 + 10.0 * t,
            2000.0,
        );
        let injection_apply = body::stress_injection_apply_step(
            stress,
            12.0 + 18.0 * t,
            0.8 + 0.5 * t,
            0.01,
            2000.0,
        );
        let shaken_step = body::stress_shaken_countdown_step((i % 6) as i32);

        let primary = stat_curve::stress_primary_step(
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
        let post_step = stat_curve::stress_post_update_step(
            30.0 + 60.0 * (1.0 - t),
            stress,
            0.1 + 0.7 * (1.0 - t),
            -10.0 + 80.0 * t,
            (i % 4) as i32,
            i % 3 == 0,
            allostatic,
            if i % 2 == 0 { 1.35 } else { 1.0 },
        );
        let post_res_step = stat_curve::stress_post_update_resilience_step(
            30.0 + 60.0 * (1.0 - t),
            stress,
            0.1 + 0.7 * (1.0 - t),
            -10.0 + 80.0 * t,
            (i % 4) as i32,
            i % 3 == 0,
            allostatic,
            if i % 2 == 0 { 1.35 } else { 1.0 },
            0.2 + 0.6 * t,
            0.2 + 0.7 * (1.0 - t),
            0.2 + 0.6 * t,
            0.2 + 0.5 * t,
            0.2 + 0.5 * (1.0 - t),
            0.2 + 0.4 * t,
            0.3 + 0.6 * (1.0 - t),
            hunger,
            energy,
            -0.03 * t,
        );
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
        let work_eff = stat_curve::stress_work_efficiency(stress, -0.03 * t);
        let personality_scale = stat_curve::stress_personality_scale(
            &[0.2 + 0.7 * t, 0.1 + 0.8 * (1.0 - t), 0.4 + 0.5 * t],
            &[0.8, 0.6, 0.5],
            &[1_u8, 0_u8, 1_u8],
            &[if i % 2 == 0 { 1.15 } else { 0.9 }],
        );
        let relationship_scale = stat_curve::stress_relationship_scale(
            if i % 3 == 0 { "bond_strength" } else { "none" },
            0.2 + 0.6 * t,
            0.3,
            1.5,
        );
        let context_scale =
            stat_curve::stress_context_scale(&[1.1, if i % 2 == 0 { 0.9 } else { 1.3 }]);
        let emotion_inject = stat_curve::stress_emotion_inject_step(
            &[20.0, 50.0, 10.0, 5.0, 8.0, 35.0, 40.0, 25.0],
            &[-10.0, 20.0, 5.0, -5.0, 3.0, 15.0, 12.0, 6.0],
            &[2.0, -1.0, 3.5, 0.0, 1.5, -2.0, 1.0, 0.5],
            &[0.5, 1.2, -0.6, 0.8, 0.0, -1.1, 0.7, -0.3],
            0.8 + 0.6 * t,
        );
        let rebound_step = stat_curve::stress_rebound_queue_step(
            &[5.0 + 3.0 * t, 2.0 + 2.0 * t, 1.0 + t],
            &[1, 2, 4],
            0.0,
        );
        let event_scale_step = stat_curve::stress_event_scale_step(
            12.0 + 18.0 * t,
            0.8 + 0.5 * t,
            i % 2 == 0,
            personality_scale,
            1.0,
            if i % 3 == 0 { "bond_strength" } else { "none" },
            0.2 + 0.6 * t,
            0.3,
            1.5,
            &[1.1, if i % 2 == 0 { 0.9 } else { 1.3 }],
        );
        let event_scale_step_code = stat_curve::stress_event_scale_step_code(
            12.0 + 18.0 * t,
            0.8 + 0.5 * t,
            i % 2 == 0,
            personality_scale,
            1.0,
            if i % 3 == 0 { 1 } else { 0 },
            0.2 + 0.6 * t,
            0.3,
            1.5,
            &[1.1, if i % 2 == 0 { 0.9 } else { 1.3 }],
        );
        let event_inject_step = stat_curve::stress_event_inject_step(
            12.0 + 18.0 * t,
            0.8 + 0.5 * t,
            i % 2 == 0,
            personality_scale,
            1.0,
            if i % 3 == 0 { "bond_strength" } else { "none" },
            0.2 + 0.6 * t,
            0.3,
            1.5,
            &[1.1, if i % 2 == 0 { 0.9 } else { 1.3 }],
            &[20.0, 50.0, 10.0, 5.0, 8.0, 35.0, 40.0, 25.0],
            &[-10.0, 20.0, 5.0, -5.0, 3.0, 15.0, 12.0, 6.0],
            &[2.0, -1.0, 3.5, 0.0, 1.5, -2.0, 1.0, 0.5],
            &[0.5, 1.2, -0.6, 0.8, 0.0, -1.1, 0.7, -0.3],
        );
        let event_inject_step_code = stat_curve::stress_event_inject_step_code(
            12.0 + 18.0 * t,
            0.8 + 0.5 * t,
            i % 2 == 0,
            personality_scale,
            1.0,
            if i % 3 == 0 { 1 } else { 0 },
            0.2 + 0.6 * t,
            0.3,
            1.5,
            &[1.1, if i % 2 == 0 { 0.9 } else { 1.3 }],
            &[20.0, 50.0, 10.0, 5.0, 8.0, 35.0, 40.0, 25.0],
            &[-10.0, 20.0, 5.0, -5.0, 3.0, 15.0, 12.0, 6.0],
            &[2.0, -1.0, 3.5, 0.0, 1.5, -2.0, 1.0, 0.5],
            &[0.5, 1.2, -0.6, 0.8, 0.0, -1.1, 0.7, -0.3],
        );
        let event_scaled = stat_curve::stress_event_scaled(
            12.0 + 18.0 * t,
            0.8 + 0.5 * t,
            i % 2 == 0,
            0.8 + 0.4 * t,
            0.6 + 0.6 * (1.0 - t),
            0.7 + 0.5 * t,
            1.0,
        );
        let traces = stat_curve::stress_trace_batch_step(&trace_per_tick, &trace_decay, 0.01);
        let delta_step = stat_curve::stress_delta_step(
            primary.total,
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
        let emotion_delta_step = stat_curve::stress_emotion_recovery_delta_step(
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
            stress,
            0.2 + 0.6 * t,
            0.1 + 0.7 * (1.0 - t),
            30.0 + 60.0 * (1.0 - t),
            i % 3 == 0,
            i % 2 == 0,
            primary.total,
            traces.total_contribution,
            1.1,
            0.95,
            0.05,
            i % 2 == 0,
            0.6,
            50.0 + 400.0 * t,
            800.0,
        );
        let trace_emotion_delta_step = stat_curve::stress_trace_emotion_recovery_delta_step(
            &trace_per_tick,
            &trace_decay,
            0.01,
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
            stress,
            0.2 + 0.6 * t,
            0.1 + 0.7 * (1.0 - t),
            30.0 + 60.0 * (1.0 - t),
            i % 3 == 0,
            i % 2 == 0,
            primary.total,
            1.1,
            0.95,
            0.05,
            i % 2 == 0,
            0.6,
            50.0 + 400.0 * t,
            800.0,
        );
        let tick_step = stat_curve::stress_tick_step(
            &trace_per_tick,
            &trace_decay,
            0.01,
            hunger,
            energy,
            social,
            0.1 + 0.7 * t,
            0.2 * t,
            0.2 + 0.6 * t,
            0.4 + 0.4 * t,
            10.0 + 80.0 * t,
            30.0 - 15.0 * t,
            0.3 + 0.6 * (1.0 - t),
            0.3 + 0.6 * t,
            (30.0 + 60.0 * (1.0 - t)) / 100.0,
            5.0 + 70.0 * t,
            12.0 + 65.0 * t,
            8.0 + 50.0 * t,
            8.0 + 35.0 * t,
            30.0 - 20.0 * t,
            25.0 - 10.0 * t,
            -50.0 + 20.0 * t,
            40.0 + 45.0 * t,
            stress,
            0.1 + 0.7 * (1.0 - t),
            30.0 + 60.0 * (1.0 - t),
            -10.0 + 80.0 * t,
            (i % 4) as i32,
            i % 3 == 0,
            i % 2 == 0,
            allostatic,
            1.1,
            0.95,
            0.05,
            i % 2 == 0,
            0.6,
            50.0 + 400.0 * t,
            800.0,
            if i % 2 == 0 { 1.35 } else { 1.0 },
            0.2 + 0.6 * t,
            0.2 + 0.7 * (1.0 - t),
            0.2 + 0.6 * t,
            0.2 + 0.5 * t,
            0.2 + 0.5 * (1.0 - t),
            0.2 + 0.4 * t,
            -0.03 * t,
        );

        checksum += black_box(primary.total)
            + black_box(primary.appraisal_scale)
            + black_box(emotion.total)
            + black_box(recovery)
            + black_box(reserve_step.reserve)
            + black_box(allo_step)
            + black_box(state.stress_blunt_mult)
            + black_box(post_step.allostatic)
            + black_box(post_step.stress_blunt_mult)
            + black_box(post_res_step.resilience)
            + black_box(resilience)
            + black_box(work_eff)
            + black_box(support_score)
            + black_box(rebound_apply[0])
            + black_box(rebound_apply[1])
            + black_box(injection_apply[0])
            + black_box(injection_apply[1])
            + black_box(shaken_step[0])
            + black_box(shaken_step[1])
            + black_box(personality_scale)
            + black_box(relationship_scale)
            + black_box(context_scale)
            + black_box(emotion_inject.fast[0])
            + black_box(emotion_inject.slow[1])
            + black_box(rebound_step.total_rebound)
            + black_box(event_scale_step.final_instant)
            + black_box(event_scale_step_code.final_instant)
            + black_box(event_inject_step.final_instant)
            + black_box(event_inject_step_code.final_instant)
            + black_box(event_inject_step.fast[0])
            + black_box(event_scaled.final_instant)
            + black_box(traces.total_contribution)
            + black_box(delta_step.delta)
            + black_box(delta_step.hidden_threat_accumulator)
            + black_box(emotion_delta_step.delta)
            + black_box(emotion_delta_step.hidden_threat_accumulator)
            + black_box(trace_emotion_delta_step.delta)
            + black_box(trace_emotion_delta_step.hidden_threat_accumulator)
            + black_box(tick_step.delta)
            + black_box(tick_step.resilience);
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

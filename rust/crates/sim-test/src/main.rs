// Test binary exercises simulation kernels with many-parameter scientific functions.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::type_complexity)]

/// Number of RuntimeSystems registered by [`register_all_systems`].
/// Update this when adding or removing systems from that function.
const EXPECTED_SYSTEM_COUNT: usize = 58;

use sim_bridge::{
    get_pathfind_backend_mode, has_gpu_pathfind_backend, pathfind_backend_dispatch_counts,
    pathfind_grid_batch_dispatch_bytes, pathfind_grid_batch_xy_dispatch_bytes,
    reset_pathfind_backend_dispatch_counts, resolve_pathfind_backend_mode,
    set_pathfind_backend_mode,
};
use sim_core::components::{
    Behavior, Body, Coping, Economic, Emotion, Faith, Identity, Intelligence, LlmRequestType,
    Memory, Needs, Personality, Skills, Social, Stress, Traits, Values,
};
use sim_core::config::GameConfig;
use sim_core::components::LlmRole;
use sim_systems::entity_spawner;
use sim_core::ids::SettlementId;
use sim_core::{GameCalendar, Settlement, WorldMap};
use sim_engine::{
    generate_fallback_content, LlmPromptVariant, LlmRequest, LlmRuntime, SimEngine, SimResources,
};
use sim_systems::runtime::{
    // biology
    AceTrackerRuntimeSystem, AgeRuntimeSystem, AttachmentRuntimeSystem,
    ChildcareRuntimeSystem, IntergenerationalRuntimeSystem, MortalityRuntimeSystem,
    ParentingRuntimeSystem, PersonalityGeneratorRuntimeSystem, PopulationRuntimeSystem,
    // cognition
    BehaviorRuntimeSystem, IntelligenceRuntimeSystem, MemoryRuntimeSystem,
    // economy
    BuildingEffectRuntimeSystem, ConstructionRuntimeSystem, GatheringRuntimeSystem,
    JobAssignmentRuntimeSystem, JobSatisfactionRuntimeSystem, ResourceRegenSystem,
    // needs
    ChildStressProcessorRuntimeSystem, NeedsRuntimeSystem, UpperNeedsRuntimeSystem,
    // llm
    LlmRequestRuntimeSystem, LlmResponseRuntimeSystem, LlmTimeoutRuntimeSystem,
    // psychology
    ContagionRuntimeSystem, CopingRuntimeSystem, EmotionRuntimeSystem,
    MentalBreakRuntimeSystem, MoraleRuntimeSystem, PersonalityMaturationRuntimeSystem,
    StressRuntimeSystem, TraitRuntimeSystem, TraitViolationRuntimeSystem,
    TraumaScarRuntimeSystem,
    // record
    ChronicleRuntimeSystem, StatSyncRuntimeSystem, StatThresholdRuntimeSystem,
    StatsRecorderRuntimeSystem,
    // social
    EconomicTendencyRuntimeSystem, FamilyRuntimeSystem, LeaderRuntimeSystem,
    NetworkRuntimeSystem, OccupationRuntimeSystem, ReputationRuntimeSystem,
    SettlementCultureRuntimeSystem, SocialEventRuntimeSystem,
    StorySifterRuntimeSystem, StratificationMonitorRuntimeSystem, TitleRuntimeSystem,
    ValueRuntimeSystem,
    // world
    MigrationRuntimeSystem, MovementRuntimeSystem, SteeringRuntimeSystem,
    TechDiscoveryRuntimeSystem,
    TechMaintenanceRuntimeSystem, TechPropagationRuntimeSystem,
    TechUtilizationRuntimeSystem, TensionRuntimeSystem,
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
    if args
        .iter()
        .any(|arg| arg == "--bench-pathfind-bridge-split")
    {
        run_pathfind_bridge_split_bench(&args);
        return;
    }
    if args
        .iter()
        .any(|arg| arg == "--bench-pathfind-backend-smoke")
    {
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
    if args.iter().any(|arg| arg == "--llm-smoke") {
        run_llm_smoke();
        return;
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("[sim-test] WorldSim Phase R-1 headless test");

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
        Ok(data) => {
            log::info!(
                "[sim-test] data loaded: {} emotions, {} techs, {} value_events, {} stressors, {} coping_defs, {} mental_breaks, {} traits, {} species, {} mortality_profiles, {} developmental_stages, {} occupation_categories, {} job_profiles",
                data.emotions.len(),
                data.tech.len(),
                data.values.len(),
                data.stressors.len(),
                data.coping.len(),
                data.mental_breaks.len(),
                data.traits.len(),
                data.species.len(),
                data.mortality.len(),
                data.developmental_stages.len(),
                data.occupation.categories.len(),
                data.occupation.jobs.len(),
            );
            resources.personality_distribution = Some(data.personality_distribution.clone());
            resources.name_generator = Some(sim_data::NameGenerator::new(data.name_cultures.clone()));
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

    // ── Create engine ────────────────────────────────────────────────────────
    let mut engine = SimEngine::new(resources);

    // ── Register runtime systems (Phase R-1) ──────────────────────────────────
    register_all_systems(&mut engine);

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

    // ── Spawn 20 fully-initialized agents via entity_spawner ──────────────────
    {
        let (world, resources) = engine.world_and_resources_mut();
        entity_spawner::spawn_initial_population(world, resources, 20, SettlementId(1));
    }
    println!("[sim-test] Spawned 20 agents into hecs::World");

    // ── Run one in-game year (12 ticks/day × 365 days = 4380 ticks) ──────────
    engine.run_ticks(4380);

    // ── Phase A Entity Checks ─────────────────────────────────────────────────
    println!("[sim-test] === Phase A Entity Checks ===");
    {
        let world = engine.world();
        // Query both Identity and Values together; collect to (bits, name, nonzero_count).
        let entity_data: Vec<(u64, String, usize)> = world
            .query::<(&Identity, &Values)>()
            .iter()
            .map(|(e, (id, vals))| {
                let nonzero = vals.values.iter().filter(|v| v.abs() > 0.001).count();
                (e.to_bits().get(), id.name.clone(), nonzero)
            })
            .collect();

        let mut name_set = std::collections::HashSet::new();
        let mut values_nonzero_count = 0usize;

        for (bits, name, nonzero) in &entity_data {
            println!("[sim-test]   entity: id={} name={}", bits, name);
            assert!(!name.starts_with("Agent "), "Name should not be placeholder: {}", name);
            assert!(!name.is_empty(), "Name should not be empty");
            assert!(name_set.insert(name.clone()), "Duplicate name: {}", name);
            println!("[sim-test]   values_nonzero={}", nonzero);
            if *nonzero >= 10 { values_nonzero_count += 1; }
        }

        let entity_count = entity_data.len();
        assert!(entity_count >= 20, "Expected ≥20 entities, got {}", entity_count);
        assert!(
            values_nonzero_count >= 18,
            "Expected ≥18 entities with ≥10 non-zero values, got {}",
            values_nonzero_count
        );
    }
    println!("[sim-test] === Phase A Entity Checks PASS ===");

    // ── Phase A Comprehensive Validation (T10) ────────────────────────────────
    println!("[sim-test] === Phase A Comprehensive Validation ===");
    {
        let world = engine.world();

        // 4. All 15 component types present on every entity
        println!("[sim-test] === EntityDetail L2 Data Check ===");
        for (entity, _) in world.query::<&Identity>().iter() {
            assert!(world.get::<&Personality>(entity).is_ok(), "Missing Personality");
            assert!(world.get::<&Values>(entity).is_ok(), "Missing Values");
            assert!(world.get::<&Emotion>(entity).is_ok(), "Missing Emotion");
            assert!(world.get::<&Needs>(entity).is_ok(), "Missing Needs");
            assert!(world.get::<&Stress>(entity).is_ok(), "Missing Stress");
            assert!(world.get::<&Body>(entity).is_ok(), "Missing Body");
            assert!(world.get::<&Intelligence>(entity).is_ok(), "Missing Intelligence");
            assert!(world.get::<&Skills>(entity).is_ok(), "Missing Skills");
            assert!(world.get::<&Social>(entity).is_ok(), "Missing Social");
            assert!(world.get::<&Memory>(entity).is_ok(), "Missing Memory");
            assert!(world.get::<&Economic>(entity).is_ok(), "Missing Economic");
            assert!(world.get::<&Behavior>(entity).is_ok(), "Missing Behavior");
            assert!(world.get::<&Coping>(entity).is_ok(), "Missing Coping");
            assert!(world.get::<&Faith>(entity).is_ok(), "Missing Faith");
            assert!(world.get::<&Traits>(entity).is_ok(), "Missing Traits");
        }
        println!("[sim-test] === EntityDetail L2 Data Check PASS ===");

        // 5. Values are in [-1.0, 1.0] range
        for (_, vals) in world.query::<&Values>().iter() {
            for (i, v) in vals.values.iter().enumerate() {
                assert!(
                    *v >= -1.0 && *v <= 1.0,
                    "Value[{}] = {} is out of [-1.0, 1.0] range",
                    i, v
                );
            }
        }
        println!("[sim-test]   Values range [-1,1]: OK");

        // 7. ExplainLog exists in resources (stub, may be empty)
        let _explain = &engine.resources().explain_log;
        println!("[sim-test]   ExplainLog stub: OK");
    }
    println!("[sim-test] === Phase A Comprehensive Validation PASS ===");

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
    println!("[sim-test]   Systems run:     {} (full registration)", snap.system_count);
    println!("[sim-test] \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
    println!("[sim-test] PASS");

    // ── Assertions ────────────────────────────────────────────────────────────
    assert_eq!(snap.tick, 4380, "wrong tick count");
    assert_eq!(
        snap.year, 2,
        "wrong year (should be year 2 after 4380 ticks)"
    );
    assert_eq!(snap.day_of_year, 1, "should be start of year 2");
    assert!(
        snap.entity_count >= 1,
        "expected at least 1 entity after spawning 20, got {} (entity spawner not working)",
        snap.entity_count
    );
    assert_eq!(snap.settlement_count, 1, "should have 1 settlement");
    assert!(snap.system_count > 0, "should have systems registered");
    assert!(
        snap.system_count >= EXPECTED_SYSTEM_COUNT,
        "expected {} registered systems, got {}",
        EXPECTED_SYSTEM_COUNT,
        snap.system_count
    );
}

fn register_all_systems(engine: &mut SimEngine) {
    // Priority/interval values mirror scripts/scenes/main/main.gd registration.
    // SimEngine sorts by priority ascending — lower runs first.
    engine.register(StatSyncRuntimeSystem::new(1, 10));
    engine.register(ResourceRegenSystem::new(5, 1));
    engine.register(ChildcareRuntimeSystem::new(8, 2));
    engine.register(JobAssignmentRuntimeSystem::new(8, 1));
    engine.register(NeedsRuntimeSystem::new(10, 1));
    engine.register(StatThresholdRuntimeSystem::new(12, 5));
    engine.register(UpperNeedsRuntimeSystem::new(12, 1));
    engine.register(BuildingEffectRuntimeSystem::new(15, 1));
    engine.register(IntelligenceRuntimeSystem::new(18, 50));
    engine.register(MemoryRuntimeSystem::new(18, 1));
    engine.register(BehaviorRuntimeSystem::new(20, 1));
    engine.register(GatheringRuntimeSystem::new(25, 1));
    engine.register(ConstructionRuntimeSystem::new(28, 1));
    engine.register(SteeringRuntimeSystem::new(
        sim_core::config::STEERING_SYSTEM_PRIORITY,
        sim_core::config::STEERING_SYSTEM_INTERVAL,
    ));
    engine.register(MovementRuntimeSystem::new(30, 1));
    engine.register(EmotionRuntimeSystem::new(32, 12));
    engine.register(ChildStressProcessorRuntimeSystem::new(32, 2));
    engine.register(StressRuntimeSystem::new(34, 50));
    engine.register(MentalBreakRuntimeSystem::new(35, 1));
    engine.register(OccupationRuntimeSystem::new(36, 1));
    engine.register(TraumaScarRuntimeSystem::new(36, 10));
    engine.register(TitleRuntimeSystem::new(37, 1));
    engine.register(TraitViolationRuntimeSystem::new(37, 1));
    engine.register(SocialEventRuntimeSystem::new(37, 30));
    engine.register(ContagionRuntimeSystem::new(38, 3));
    engine.register(ReputationRuntimeSystem::new(38, 1));
    engine.register(EconomicTendencyRuntimeSystem::new(39, 1));
    engine.register(MoraleRuntimeSystem::new(40, 5));
    engine.register(JobSatisfactionRuntimeSystem::new(40, 1));
    engine.register(CopingRuntimeSystem::new(42, 30));
    engine.register(IntergenerationalRuntimeSystem::new(45, 240));
    engine.register(ParentingRuntimeSystem::new(46, 240));
    engine.register(AgeRuntimeSystem::new(48, 50));
    engine.register(MortalityRuntimeSystem::new(49, 1));
    engine.register(PopulationRuntimeSystem::new(50, 1));
    engine.register(FamilyRuntimeSystem::new(52, 365));
    engine.register(LeaderRuntimeSystem::new(52, 1));
    engine.register(ValueRuntimeSystem::new(55, 200));
    engine.register(LlmResponseRuntimeSystem::new(
        sim_core::config::LLM_RESPONSE_SYSTEM_PRIORITY,
        sim_core::config::LLM_RESPONSE_SYSTEM_INTERVAL,
    ));
    engine.register(LlmTimeoutRuntimeSystem::new(
        sim_core::config::LLM_TIMEOUT_SYSTEM_PRIORITY,
        sim_core::config::LLM_TIMEOUT_SYSTEM_INTERVAL,
    ));
    engine.register(NetworkRuntimeSystem::new(58, 1));
    engine.register(MigrationRuntimeSystem::new(60, 1));
    engine.register(TechDiscoveryRuntimeSystem::new(62, 1));
    engine.register(TechPropagationRuntimeSystem::new(62, 1));
    engine.register(TechMaintenanceRuntimeSystem::new(63, 1));
    engine.register(TensionRuntimeSystem::new(64, 1));
    engine.register(TechUtilizationRuntimeSystem::new(65, 1));
    engine.register(StratificationMonitorRuntimeSystem::new(90, 1));
    engine.register(StatsRecorderRuntimeSystem::new(90, 200));
    engine.register(StorySifterRuntimeSystem::new(
        sim_core::config::STORY_SIFTER_PRIORITY,
        sim_core::config::STORY_SIFTER_TICK_INTERVAL,
    ));
    engine.register(LlmRequestRuntimeSystem::new(
        sim_core::config::LLM_REQUEST_SYSTEM_PRIORITY,
        sim_core::config::LLM_REQUEST_SYSTEM_INTERVAL,
    ));
    engine.register(SettlementCultureRuntimeSystem::new(95, 100));
    engine.register(PersonalityMaturationRuntimeSystem::new(96, 100));
    engine.register(PersonalityGeneratorRuntimeSystem::new(97, 100));
    engine.register(AttachmentRuntimeSystem::new(98, 100));
    engine.register(AceTrackerRuntimeSystem::new(99, 100));
    engine.register(TraitRuntimeSystem::new(100, 10));
    engine.register(ChronicleRuntimeSystem::new(101, 1));
}

fn run_llm_smoke() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    println!("[sim-test] LLM smoke starting");

    let mut runtime = LlmRuntime::default();
    if let Err(error) = runtime.start() {
        eprintln!("[sim-test] failed to start llama-server runtime: {error}");
        std::process::exit(1);
    }

    let judgment_request = sample_llm_request(
        LlmRequestType::Layer3Judgment,
        LlmPromptVariant::Judgment,
    );
    let judgment_id = match runtime.submit_request(judgment_request) {
        Ok(request_id) => request_id,
        Err(error) => {
            eprintln!("[sim-test] failed to enqueue Layer 3 request: {error}");
            runtime.stop();
            std::process::exit(1);
        }
    };
    let judgment_response = wait_for_llm_response(&mut runtime, judgment_id);
    let Some(judgment_response) = judgment_response else {
        eprintln!("[sim-test] timed out waiting for Layer 3 response");
        runtime.stop();
        std::process::exit(1);
    };
    assert!(judgment_response.success, "Layer 3 request should succeed");
    assert!(
        matches!(judgment_response.content, sim_core::components::LlmContent::Judgment(_)),
        "Layer 3 response should parse as JudgmentData",
    );

    let narrative_request = sample_llm_request(
        LlmRequestType::Layer4Narrative,
        LlmPromptVariant::Narrative,
    );
    let narrative_id = match runtime.submit_request(narrative_request) {
        Ok(request_id) => request_id,
        Err(error) => {
            eprintln!("[sim-test] failed to enqueue Layer 4 request: {error}");
            runtime.stop();
            std::process::exit(1);
        }
    };
    let narrative_response = wait_for_llm_response(&mut runtime, narrative_id);
    let Some(narrative_response) = narrative_response else {
        eprintln!("[sim-test] timed out waiting for Layer 4 response");
        runtime.stop();
        std::process::exit(1);
    };
    assert!(narrative_response.success, "Layer 4 request should succeed");
    match narrative_response.content {
        sim_core::components::LlmContent::Narrative(ref text) => {
            assert!(
                !text.trim().is_empty(),
                "Layer 4 response should be non-empty Korean text",
            );
        }
        _ => panic!("Layer 4 response should be narrative text"),
    }

    let fallback = generate_fallback_content(LlmRequestType::Layer4Narrative, "카야");
    match fallback {
        sim_core::components::LlmContent::Narrative(text) => {
            assert!(!text.trim().is_empty(), "Fallback narrative should be non-empty");
        }
        _ => panic!("Fallback should be a narrative string"),
    }

    runtime.stop();
    println!("[sim-test] LLM smoke PASS");
}

fn sample_llm_request(
    request_type: LlmRequestType,
    variant: LlmPromptVariant,
) -> LlmRequest {
    LlmRequest {
        request_id: 0,
        entity_id: 1,
        request_type,
        variant,
        entity_name: "카야".to_string(),
        role: LlmRole::Agent,
        growth_stage: sim_core::enums::GrowthStage::Adult,
        sex: sim_core::enums::Sex::Female,
        occupation: "채집꾼".to_string(),
        action_id: 3,
        action_label: "Socialize".to_string(),
        personality_axes: [0.41, 0.62, 0.78, 0.54, 0.66, 0.58],
        emotions: [0.2, 0.35, 0.05, 0.1, 0.12, 0.04, 0.08, 0.44],
        needs: [0.73, 0.91, 0.64, 0.88, 0.76, 0.55, 0.46, 0.61, 0.72, 0.68, 0.59, 0.63, 0.71],
        values: [0.0; 33],
        stress_level: 0.34,
        stress_state: 1,
        recent_event_type: Some("social_conflict".to_string()),
        recent_event_cause: Some("hurtful_words".to_string()),
        recent_target_name: Some("하린".to_string()),
    }
}

fn wait_for_llm_response(runtime: &mut LlmRuntime, request_id: u64) -> Option<sim_engine::LlmResponse> {
    let started = Instant::now();
    while started.elapsed().as_secs_f64() < 30.0 {
        for response in runtime.drain_responses() {
            if response.request_id == request_id {
                return Some(response);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    None
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
        let child_stage_code =
            body::child_stage_code_from_age_ticks(8760 * ((i % 22) as i32), 2.0, 5.0, 12.0, 18.0);

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

#[cfg(test)]
mod tests {
    use super::{
        entity_spawner, register_all_systems, EXPECTED_SYSTEM_COUNT,
    };
    use sim_core::components::{Behavior, Identity, Personality, Position, SteeringParams};
    use sim_core::config::{GameConfig, TICKS_PER_YEAR};
    use sim_core::{ActionType, GameCalendar, Settlement, SettlementId, WorldMap};
    use sim_engine::{build_agent_snapshots, SimEngine, SimResources};
    use sim_systems::entity_spawner::SpawnConfig;
    use sim_systems::runtime::derive_steering_params;

    fn make_stage1_engine(seed: u64, agent_count: usize) -> SimEngine {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(256, 256, seed);
        let resources = SimResources::new(calendar, map, seed);
        let mut engine = SimEngine::new(resources);
        register_all_systems(&mut engine);
        engine.resources_mut().settlements.insert(
            SettlementId(1),
            Settlement::new(SettlementId(1), "Test Hold".to_string(), 128, 128, 0),
        );
        {
            let (world, resources) = engine.world_and_resources_mut();
            entity_spawner::spawn_initial_population(world, resources, agent_count, SettlementId(1));
        }
        engine
    }

    fn collect_positions(engine: &SimEngine) -> Vec<(u64, (f64, f64))> {
        let mut positions: Vec<(u64, (f64, f64))> = engine
            .world()
            .query::<&Position>()
            .iter()
            .map(|(entity, position)| (entity.to_bits().get(), (position.x, position.y)))
            .collect();
        positions.sort_by_key(|(entity_id, _)| *entity_id);
        positions
    }

    #[test]
    fn stage1_simulation_100_ticks_no_panic() {
        let mut engine = make_stage1_engine(42, 20);
        assert_eq!(engine.system_count(), EXPECTED_SYSTEM_COUNT);
        engine.run_ticks(100);
        assert!(
            engine.world().len() >= 1,
            "at least one agent should remain after 100 ticks"
        );
    }

    #[test]
    fn stage1_agents_move_after_100_ticks() {
        let mut engine = make_stage1_engine(42, 10);
        let initial_positions = collect_positions(&engine);
        engine.run_ticks(100);
        let final_positions = collect_positions(&engine);

        let moved_count = initial_positions
            .iter()
            .zip(final_positions.iter())
            .filter(|((initial_id, (initial_x, initial_y)), (final_id, (final_x, final_y)))| {
                initial_id == final_id
                    && ((initial_x - final_x).abs() > 0.1 || (initial_y - final_y).abs() > 0.1)
            })
            .count();

        assert!(
            moved_count >= 5,
            "at least half of agents should move after 100 ticks, only {moved_count} moved"
        );
    }

    #[test]
    fn stage1_personality_affects_speed() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(256, 256, 42);
        let resources = SimResources::new(calendar, map, 42);
        let mut engine = SimEngine::new(resources);
        register_all_systems(&mut engine);
        engine.resources_mut().settlements.insert(
            SettlementId(1),
            Settlement::new(SettlementId(1), "Test Hold".to_string(), 128, 128, 0),
        );

        let adult_ticks = 20_u64 * u64::from(TICKS_PER_YEAR);
        let extrovert = {
            let cfg = SpawnConfig {
                settlement_id: Some(SettlementId(1)),
                position: (120, 128),
                initial_age_ticks: adult_ticks,
                sex: None,
                parent_a: None,
                parent_b: None,
            };
            let (world, resources) = engine.world_and_resources_mut();
            entity_spawner::spawn_agent(world, resources, &cfg)
        };
        let introvert = {
            let cfg = SpawnConfig {
                settlement_id: Some(SettlementId(1)),
                position: (136, 128),
                initial_age_ticks: adult_ticks,
                sex: None,
                parent_a: None,
                parent_b: None,
            };
            let (world, resources) = engine.world_and_resources_mut();
            entity_spawner::spawn_agent(world, resources, &cfg)
        };

        let mut high_x = Personality::default();
        high_x.axes[2] = 0.95;
        let mut low_x = Personality::default();
        low_x.axes[2] = 0.05;
        let high_params = derive_steering_params(&high_x);
        let low_params = derive_steering_params(&low_x);

        {
            let mut personality = engine
                .world_mut()
                .get::<&mut Personality>(extrovert)
                .expect("extrovert personality missing");
            *personality = high_x;
        }
        {
            let mut steering = engine
                .world_mut()
                .get::<&mut SteeringParams>(extrovert)
                .expect("extrovert steering missing");
            *steering = high_params;
        }
        {
            let mut behavior = engine
                .world_mut()
                .get::<&mut Behavior>(extrovert)
                .expect("extrovert behavior missing");
            behavior.current_action = ActionType::Wander;
        }
        {
            let mut personality = engine
                .world_mut()
                .get::<&mut Personality>(introvert)
                .expect("introvert personality missing");
            *personality = low_x;
        }
        {
            let mut steering = engine
                .world_mut()
                .get::<&mut SteeringParams>(introvert)
                .expect("introvert steering missing");
            *steering = low_params;
        }
        {
            let mut behavior = engine
                .world_mut()
                .get::<&mut Behavior>(introvert)
                .expect("introvert behavior missing");
            behavior.current_action = ActionType::Wander;
        }

        let extrovert_base_speed = engine
            .world()
            .get::<&SteeringParams>(extrovert)
            .expect("extrovert steering missing")
            .base_speed;
        let introvert_base_speed = engine
            .world()
            .get::<&SteeringParams>(introvert)
            .expect("introvert steering missing")
            .base_speed;
        engine.run_ticks(10);

        assert!(
            (extrovert_base_speed - introvert_base_speed).abs() > 1.0,
            "different personalities should yield different base speeds ({extrovert_base_speed} vs {introvert_base_speed})"
        );
    }

    #[test]
    fn stage1_event_store_receives_events_after_simulation() {
        let mut engine = make_stage1_engine(42, 10);
        engine.run_ticks(200);
        assert!(
            engine.resources().event_store.len() > 0,
            "event store should contain events after 200 ticks"
        );
    }

    #[test]
    fn stage1_frame_snapshot_builds_correctly() {
        let mut engine = make_stage1_engine(42, 10);
        engine.tick();

        let snapshots = build_agent_snapshots(engine.world());
        assert!(
            snapshots.len() >= 10,
            "should build at least one snapshot per spawned agent"
        );
        for snapshot in &snapshots {
            assert!(snapshot.mood_color <= 4);
            assert!(snapshot.stress_phase <= 4);
        }
    }

    #[test]
    fn stage1_simulation_tick_under_budget() {
        let mut engine = make_stage1_engine(42, 20);
        let started = std::time::Instant::now();
        engine.run_ticks(60);
        let elapsed = started.elapsed();
        let millis_per_tick = elapsed.as_secs_f64() * 1000.0 / 60.0;
        assert!(
            millis_per_tick < 16.0,
            "tick should stay under 16ms budget, got {millis_per_tick:.3}ms"
        );
    }

    #[test]
    fn stage1_spawned_entities_include_identity_components() {
        let engine = make_stage1_engine(42, 5);
        let identities: Vec<u64> = engine
            .world()
            .query::<&Identity>()
            .iter()
            .map(|(entity, _)| entity.to_bits().get())
            .collect();
        assert_eq!(identities.len(), 5);
    }
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
        width,
        height,
        walkable,
        move_cost,
        from_points,
        to_points,
        from_xy,
        to_xy,
        max_steps,
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
            "[sim-test] pathfind-backend-smoke: mode={} has_gpu={} configured={} resolved={} iterations={} checksum={:.5} cpu={} gpu={} total={}",
            mode,
            has_gpu_pathfind_backend(),
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
        let rebound_apply =
            body::stress_rebound_apply_step(stress, 50.0 + 400.0 * t, 2.0 + 10.0 * t, 2000.0);
        let injection_apply =
            body::stress_injection_apply_step(stress, 12.0 + 18.0 * t, 0.8 + 0.5 * t, 0.01, 2000.0);
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

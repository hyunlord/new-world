// Test binary exercises simulation kernels with many-parameter scientific functions.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::type_complexity)]

/// Number of RuntimeSystems registered by [`register_all_systems`].
/// Update this when adding or removing systems from that function.
const EXPECTED_SYSTEM_COUNT: usize = 64;

use sim_bridge::{
    get_pathfind_backend_mode, has_gpu_pathfind_backend, pathfind_backend_dispatch_counts,
    pathfind_grid_batch_dispatch_bytes, pathfind_grid_batch_xy_dispatch_bytes,
    reset_pathfind_backend_dispatch_counts, resolve_pathfind_backend_mode,
    set_pathfind_backend_mode,
};
use sim_core::components::LlmRole;
use sim_core::components::{
    Behavior, Body, Coping, Economic, Emotion, Faith, Identity, Intelligence, LlmRequestType,
    Memory, Needs, Personality, Skills, Social, Stress, Traits, Values,
};
use sim_core::config::GameConfig;
use sim_core::ids::SettlementId;
use sim_core::{GameCalendar, Settlement, WorldMap};
use sim_engine::{
    generate_fallback_content, LlmPromptVariant, LlmRequest, LlmRuntime, SimEngine, SimResources,
};
use sim_systems::entity_spawner;
use sim_systems::runtime::{
    // biology
    AceTrackerRuntimeSystem,
    // influence / territory
    InfluenceRuntimeSystem,
    TerritoryRuntimeSystem,
    AgeRuntimeSystem,
    AttachmentRuntimeSystem,
    // cognition
    BandBehaviorSystem,
    BandFormationSystem,
    BehaviorRuntimeSystem,
    // economy
    BuildingEffectRuntimeSystem,
    CraftingRuntimeSystem,
    // needs
    ChildStressProcessorRuntimeSystem,
    ChildcareRuntimeSystem,
    // record
    ChronicleRuntimeSystem,
    ConstructionRuntimeSystem,
    // psychology
    ContagionRuntimeSystem,
    CopingRuntimeSystem,
    // social
    EconomicTendencyRuntimeSystem,
    EffectApplySystem,
    EmotionRuntimeSystem,
    FamilyRuntimeSystem,
    GatheringRuntimeSystem,
    IntelligenceRuntimeSystem,
    IntergenerationalRuntimeSystem,
    JobAssignmentRuntimeSystem,
    JobSatisfactionRuntimeSystem,
    LeaderRuntimeSystem,
    // llm
    LlmRequestRuntimeSystem,
    LlmResponseRuntimeSystem,
    LlmTimeoutRuntimeSystem,
    MemoryRuntimeSystem,
    MentalBreakRuntimeSystem,
    // world
    MigrationRuntimeSystem,
    MoraleRuntimeSystem,
    MortalityRuntimeSystem,
    MovementRuntimeSystem,
    NeedsRuntimeSystem,
    NetworkRuntimeSystem,
    OccupationRuntimeSystem,
    ParentingRuntimeSystem,
    PersonalityGeneratorRuntimeSystem,
    PersonalityMaturationRuntimeSystem,
    PopulationRuntimeSystem,
    ReputationRuntimeSystem,
    ResourceRegenSystem,
    SettlementCultureRuntimeSystem,
    SocialEventRuntimeSystem,
    StatSyncRuntimeSystem,
    StatThresholdRuntimeSystem,
    StatsRecorderRuntimeSystem,
    SteeringRuntimeSystem,
    StorySifterRuntimeSystem,
    StratificationMonitorRuntimeSystem,
    StressRuntimeSystem,
    TechDiscoveryRuntimeSystem,
    TechMaintenanceRuntimeSystem,
    TechPropagationRuntimeSystem,
    TechUtilizationRuntimeSystem,
    TensionRuntimeSystem,
    TitleRuntimeSystem,
    TraitRuntimeSystem,
    TraitViolationRuntimeSystem,
    TraumaScarRuntimeSystem,
    UpperNeedsRuntimeSystem,
    ValueRuntimeSystem,
};
use sim_systems::{body, stat_curve};
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn authoritative_ron_data_dir() -> Option<PathBuf> {
    let crates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .to_path_buf();
    Some(crates_dir.join("sim-data").join("data"))
}

fn legacy_json_data_dir() -> Option<PathBuf> {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .parent()?
        .to_path_buf();
    Some(project_root.join("data"))
}

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
    if let Some(registry_dir) = authoritative_ron_data_dir() {
        match sim_data::DataRegistry::load_from_directory(&registry_dir) {
            Ok(registry) => {
                log::info!(
                    "[sim-test] authoritative RON registry loaded: {} materials, {} recipes, {} furniture, {} structures, {} actions",
                    registry.materials.len(),
                    registry.recipes.len(),
                    registry.furniture.len(),
                    registry.structures.len(),
                    registry.actions.len(),
                );
            }
            Err(errors) => {
                log::warn!(
                    "[sim-test] authoritative RON registry not available at {:?}: {:?}",
                    registry_dir,
                    errors
                );
            }
        }
    } else {
        log::warn!("[sim-test] could not resolve authoritative RON registry path");
    }

    if let Some(data_dir) = legacy_json_data_dir() {
        match sim_data::load_personality_distribution(&data_dir) {
            Ok(distribution) => {
                resources.personality_distribution = Some(distribution);
                log::info!("[sim-test] legacy personality distribution compatibility data loaded");
            }
            Err(error) => {
                log::warn!(
                    "[sim-test] legacy personality distribution not available at {:?}: {:?}",
                    data_dir,
                    error
                );
            }
        }

        let name_cultures = sim_data::load_name_cultures(&data_dir);
        if !name_cultures.is_empty() {
            resources.name_generator = Some(sim_data::NameGenerator::new(name_cultures));
            log::info!("[sim-test] legacy naming cultures compatibility data loaded");
        } else {
            log::warn!(
                "[sim-test] legacy naming cultures not found at {:?}, skipping",
                data_dir
            );
        }
    } else {
        log::warn!("[sim-test] could not resolve legacy JSON data path");
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
            assert!(
                !name.starts_with("Agent "),
                "Name should not be placeholder: {}",
                name
            );
            assert!(!name.is_empty(), "Name should not be empty");
            assert!(name_set.insert(name.clone()), "Duplicate name: {}", name);
            println!("[sim-test]   values_nonzero={}", nonzero);
            if *nonzero >= 10 {
                values_nonzero_count += 1;
            }
        }

        let entity_count = entity_data.len();
        assert!(
            entity_count >= 20,
            "Expected ≥20 entities, got {}",
            entity_count
        );
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
            assert!(
                world.get::<&Personality>(entity).is_ok(),
                "Missing Personality"
            );
            assert!(world.get::<&Values>(entity).is_ok(), "Missing Values");
            assert!(world.get::<&Emotion>(entity).is_ok(), "Missing Emotion");
            assert!(world.get::<&Needs>(entity).is_ok(), "Missing Needs");
            assert!(world.get::<&Stress>(entity).is_ok(), "Missing Stress");
            assert!(world.get::<&Body>(entity).is_ok(), "Missing Body");
            assert!(
                world.get::<&Intelligence>(entity).is_ok(),
                "Missing Intelligence"
            );
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
                    i,
                    v
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
    println!(
        "[sim-test]   Systems run:     {} (full registration)",
        snap.system_count
    );
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
    engine.register(CraftingRuntimeSystem::new(29, 10));
    engine.register(BandBehaviorSystem::new(
        sim_core::config::BAND_BEHAVIOR_SYSTEM_PRIORITY,
        sim_core::config::BAND_BEHAVIOR_TICK_INTERVAL,
    ));
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
    engine.register(BandFormationSystem::new(
        sim_core::config::BAND_FORMATION_SYSTEM_PRIORITY,
        sim_core::config::BAND_FORMATION_TICK_INTERVAL,
    ));
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
    engine.register(InfluenceRuntimeSystem::new(
        sim_core::config::INFLUENCE_SYSTEM_PRIORITY,
        sim_core::config::INFLUENCE_SYSTEM_INTERVAL,
    ));
    engine.register(TerritoryRuntimeSystem::new(
        sim_core::config::TERRITORY_SYSTEM_PRIORITY,
        sim_core::config::TERRITORY_SYSTEM_INTERVAL,
    ));
    engine.register(EffectApplySystem::new(9999, 1));
}

fn run_llm_smoke() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    println!("[sim-test] LLM smoke starting");

    let mut runtime = LlmRuntime::default();
    if let Err(error) = runtime.start() {
        eprintln!("[sim-test] failed to start llama-server runtime: {error}");
        std::process::exit(1);
    }

    let judgment_request =
        sample_llm_request(LlmRequestType::Layer3Judgment, LlmPromptVariant::Judgment);
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
        matches!(
            judgment_response.content,
            sim_core::components::LlmContent::Judgment(_)
        ),
        "Layer 3 response should parse as JudgmentData",
    );

    let narrative_request =
        sample_llm_request(LlmRequestType::Layer4Narrative, LlmPromptVariant::Narrative);
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
            assert!(
                !text.trim().is_empty(),
                "Fallback narrative should be non-empty"
            );
        }
        _ => panic!("Fallback should be a narrative string"),
    }

    runtime.stop();
    println!("[sim-test] LLM smoke PASS");
}

fn sample_llm_request(request_type: LlmRequestType, variant: LlmPromptVariant) -> LlmRequest {
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
        needs: [
            0.73, 0.91, 0.64, 0.88, 0.76, 0.55, 0.46, 0.61, 0.72, 0.68, 0.59, 0.63, 0.71,
        ],
        values: [0.0; 33],
        stress_level: 0.34,
        stress_state: 1,
        recent_event_type: Some("social_conflict".to_string()),
        recent_event_cause: Some("hurtful_words".to_string()),
        recent_target_name: Some("하린".to_string()),
    }
}

fn wait_for_llm_response(
    runtime: &mut LlmRuntime,
    request_id: u64,
) -> Option<sim_engine::LlmResponse> {
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
    use super::{entity_spawner, register_all_systems, EXPECTED_SYSTEM_COUNT};
    use sim_core::components::{Age, Behavior, Identity, Personality, Position, SteeringParams};
    use sim_core::config::{GameConfig, TICKS_PER_YEAR};
    use sim_core::{ActionType, GameCalendar, Settlement, SettlementId, TerrainType, WorldMap};
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
            entity_spawner::spawn_initial_population(
                world,
                resources,
                agent_count,
                SettlementId(1),
            );
        }
        // Seed tile resources near settlement spawn point (128, 128).
        // In production, world generation populates tile resources; headless tests must do it explicitly.
        {
            let resources = engine.resources_mut();
            for dy in -30_i32..=30 {
                for dx in -30_i32..=30 {
                    let tx = 128_i32 + dx;
                    let ty = 128_i32 + dy;
                    if tx < 0 || ty < 0 || tx >= 256 || ty >= 256 {
                        continue;
                    }
                    let tile = resources.map.get_mut(tx as u32, ty as u32);
                    if !tile.passable {
                        continue;
                    }
                    let pattern = ((dx.abs() + dy.abs()) % 3) as u32;
                    let resource_type = match pattern {
                        0 => sim_core::ResourceType::Stone,
                        1 => sim_core::ResourceType::Wood,
                        _ => sim_core::ResourceType::Food,
                    };
                    tile.resources.push(sim_core::world::TileResource {
                        resource_type,
                        amount: 100.0,
                        max_amount: 100.0,
                        regen_rate: 0.1,
                    });
                }
            }
        }
        // Add Hill terrain tiles to give test maps terrain diversity.
        // Ensures the terrain-search path in GatherStone is exercised alongside
        // the TileResource-search path.
        {
            let resources = engine.resources_mut();
            for dy in 25_i32..=30 {
                for dx in 25_i32..=30 {
                    let tx = 128 + dx;
                    let ty = 128 + dy;
                    if tx < 256 && ty < 256 {
                        resources
                            .map
                            .get_mut(tx as u32, ty as u32)
                            .terrain = TerrainType::Hill;
                    }
                }
            }
        }
        engine
    }

    fn count_alive(engine: &SimEngine) -> usize {
        engine
            .world()
            .query::<&Age>()
            .iter()
            .filter(|(_, age)| age.alive)
            .count()
    }

    fn advance_ticks(engine: &mut SimEngine, ticks: u64) {
        engine.run_ticks(ticks);
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
            .filter(
                |((initial_id, (initial_x, initial_y)), (final_id, (final_x, final_y)))| {
                    initial_id == final_id
                        && ((initial_x - final_x).abs() > 0.1 || (initial_y - final_y).abs() > 0.1)
                },
            )
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

    /// After 2000 ticks (~3.3 minutes game time), job distribution should include miners.
    #[test]
    fn harness_job_distribution_includes_miner() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(2000);

        let world = engine.world();
        let mut job_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (_, behavior) in world.query::<&Behavior>().iter() {
            *job_counts.entry(behavior.job.clone()).or_insert(0) += 1;
        }

        let miner_count = *job_counts.get("miner").unwrap_or(&0);
        let lumberjack_count = *job_counts.get("lumberjack").unwrap_or(&0);
        let builder_count = *job_counts.get("builder").unwrap_or(&0);
        let gatherer_count = *job_counts.get("gatherer").unwrap_or(&0);

        println!(
            "[harness] jobs: gatherer={} lumberjack={} builder={} miner={}",
            gatherer_count, lumberjack_count, builder_count, miner_count
        );

        // Type A: Stone deficit requires miners. 0 miners = JobDistributionSystem bug.
        assert!(
            miner_count >= 1,
            "expected at least 1 miner, got {miner_count}. jobs={job_counts:?}"
        );
        // Type A: Wood deficit requires lumberjacks. 0 = assignment bug.
        assert!(
            lumberjack_count >= 1,
            "expected at least 1 lumberjack, got {lumberjack_count}"
        );
        // Type A: Incomplete buildings require builders. 0 = assignment bug.
        assert!(
            builder_count >= 1,
            "expected at least 1 builder, got {builder_count}"
        );
    }

    #[test]
    fn harness_agent_snapshot_stride_and_band_populated() {
        use sim_engine::frame_snapshot::{AgentSnapshot, build_agent_snapshots};

        assert_eq!(
            std::mem::size_of::<AgentSnapshot>(),
            36,
            "AgentSnapshot must remain 36 bytes — byte protocol with GDScript decoder"
        );

        let mut engine = make_stage1_engine(42, 20);
        advance_ticks(&mut engine, 2000);

        let world = engine.world();
        let snapshots = build_agent_snapshots(world);
        // Type C: empirical — 20 agents at start, 2000 ticks, observed 43 survivors (seed 42)
        assert!(
            snapshots.len() >= 10,
            "expected ≥10 living agents after 2000 ticks, got {} (seed 42, 20 agents)",
            snapshots.len()
        );

        let agents_with_band = snapshots
            .iter()
            .filter(|s| s.band_color_idx != 0xFF)
            .count();
        // Type C: empirical — observed 42/43 banded at seed 42 tick 2000; threshold 30 ≈ 70% of observed
        assert!(
            agents_with_band >= 30,
            "expected ≥30 agents with band membership at tick 2000, got {} (seed 42, 20 agents)",
            agents_with_band
        );

        // Type A: invariant — 3-byte sentinel contract must be symmetric in both directions
        for s in &snapshots {
            if s.band_color_idx != 0xFF {
                let band_id = (s.band_id_hi as u16) << 8 | s.band_id_lo as u16;
                assert_ne!(
                    band_id, 0xFFFF,
                    "agent with band_color should have valid band_id"
                );
                assert!(
                    s.band_color_idx < 8,
                    "band_color_idx must be 0..7, got {}",
                    s.band_color_idx
                );
            } else {
                // converse: no-band sentinel must be complete (all 3 bytes = 0xFF)
                assert_eq!(
                    s.band_id_lo, 0xFF,
                    "band_color_idx=0xFF but band_id_lo={} — sentinel must be all 0xFF",
                    s.band_id_lo
                );
                assert_eq!(
                    s.band_id_hi, 0xFF,
                    "band_color_idx=0xFF but band_id_hi={} — sentinel must be all 0xFF",
                    s.band_id_hi
                );
            }
        }

        // Type C: empirical — observed 17 agents with recognized jobs (seed 42 tick 2000); threshold 10
        let recognized_job_agents = snapshots.iter().filter(|s| s.atlas_var >> 4 > 0).count();
        assert!(
            recognized_job_agents >= 10,
            "expected ≥10 agents with recognized job in atlas_var upper nibble, got {}",
            recognized_job_agents
        );

        println!(
            "[harness] snapshot_stride_and_band: {} total agents, {} with band, {} with recognized job",
            snapshots.len(),
            agents_with_band,
            recognized_job_agents
        );
    }

    /// After 1 year (4380 ticks), settlement should have accumulated some stone.
    #[test]
    fn harness_stone_collected_after_one_year() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(4380);

        let resources = engine.resources();
        let total_stone: f64 = resources.settlements.values().map(|s| s.stockpile_stone).sum();
        let total_wood: f64 = resources.settlements.values().map(|s| s.stockpile_wood).sum();

        println!(
            "[harness] stockpile: stone={:.1} wood={:.1}",
            total_stone, total_wood
        );

        // Type C: Observed 368.0 at seed=42 (2026-04-01). Threshold > 0 is intentionally weak — predates v2 criteria. Consider tightening to > 50.
        assert!(
            total_stone > 0.0,
            "expected stone > 0 after 1 year, got {total_stone}"
        );
    }

    /// On an all-Grassland map with stone only at radius 60–80 (beyond normal wander range),
    /// agents must actively seek TileResource::Stone via progressive search.
    /// Validates Fix C: find_nearest_tile_with_resource in GatherStone chain.
    #[test]
    fn harness_resource_stone_accessible_from_flatland() {
        let mut engine = make_stage1_engine(42, 20);
        {
            let resources = engine.resources_mut();
            // Step 1: clear ALL tile resources within radius 50 of settlement (128,128).
            for dy in -55_i32..=55 {
                for dx in -55_i32..=55 {
                    let tx = 128 + dx;
                    let ty = 128 + dy;
                    if tx < 0 || ty < 0 || tx >= 256 || ty >= 256 {
                        continue;
                    }
                    resources.map.get_mut(tx as u32, ty as u32).resources.clear();
                }
            }
            // Step 2: place stone-only at radius 60–75 (beyond wander reach).
            for dy in -75_i32..=75 {
                for dx in -75_i32..=75 {
                    let manhattan = dx.abs() + dy.abs();
                    if manhattan < 60 || manhattan > 75 {
                        continue;
                    }
                    let tx = 128 + dx;
                    let ty = 128 + dy;
                    if tx < 0 || ty < 0 || tx >= 256 || ty >= 256 {
                        continue;
                    }
                    let tile = resources.map.get_mut(tx as u32, ty as u32);
                    tile.terrain = TerrainType::Grassland;
                    tile.resources.push(sim_core::world::TileResource {
                        resource_type: sim_core::ResourceType::Stone,
                        amount: 200.0,
                        max_amount: 200.0,
                        regen_rate: 0.0,
                    });
                }
            }
            // Step 3: override all terrain to Grassland (no Hill/Mountain).
            for y in 0..256u32 {
                for x in 0..256u32 {
                    resources.map.get_mut(x, y).terrain = TerrainType::Grassland;
                }
            }
        }
        engine.run_ticks(4380); // 1 year

        let resources = engine.resources();
        let total_stone: f64 = resources.settlements.values().map(|s| s.stockpile_stone).sum();
        println!("[harness] distant-only stone after 1 year: {total_stone:.1}");
        // Type D: Regression guard for GatherStone 6-stage progressive fallback (2026-04-01). Agents must reach radius-60+ stone tiles.
        assert!(
            total_stone > 20.0,
            "agents must gather >20 stone from radius-60+ tiles in 1 year (directed search), got {total_stone:.1}"
        );
    }

    /// On an all-Grassland map with TileResource::Stone within radius 30,
    /// agents must gather >50 stone in 1 year using find_nearest_tile_with_resource.
    /// Validates Fix A+C: stone nodes on flat terrain + progressive search chain.
    #[test]
    fn harness_stone_accessible_from_flatland() {
        let mut engine = make_stage1_engine(42, 20);
        // Override all terrain to Grassland — no Hill/Mountain for terrain-search
        {
            let resources = engine.resources_mut();
            for y in 0..256u32 {
                for x in 0..256u32 {
                    resources.map.get_mut(x, y).terrain = TerrainType::Grassland;
                }
            }
        }
        engine.run_ticks(4380); // 1 year
        let resources = engine.resources();
        let total_stone: f64 = resources
            .settlements
            .values()
            .map(|s| s.stockpile_stone)
            .sum();
        println!("[harness] flatland stone after 1 year: {total_stone:.1}");
        // Type D: Regression guard for Fix A+C — stone TileResource on flat terrain + progressive search chain (2026-04-01). Observed 94.0 at seed=42.
        assert!(
            total_stone > 50.0,
            "flatland settlement must gather >50 stone in 1 year via TileResource search, got {total_stone:.1}"
        );
    }

    /// Over 3 years, population must grow beyond 20 and approach the migration threshold (30).
    /// Validates that birth/death balance supports net positive growth.
    #[test]
    fn harness_population_growth_reaches_migration_threshold() {
        let mut engine = make_stage1_engine(42, 20);

        engine.run_ticks(4380);
        let alive_y1 = count_alive(&engine);
        let settlements_y1 = engine.resources().settlements.len();
        println!("[harness] Year 1: alive={alive_y1}, settlements={settlements_y1}");

        engine.run_ticks(4380);
        let alive_y2 = count_alive(&engine);
        let settlements_y2 = engine.resources().settlements.len();
        println!("[harness] Year 2: alive={alive_y2}, settlements={settlements_y2}");

        engine.run_ticks(4380);
        let alive_y3 = count_alive(&engine);
        let settlements_y3 = engine.resources().settlements.len();
        println!("[harness] Year 3: alive={alive_y3}, settlements={settlements_y3}");

        let peak = alive_y1.max(alive_y2).max(alive_y3);
        println!("[harness] Growth: Y1={alive_y1} Y2={alive_y2} Y3={alive_y3} peak={peak}");

        // Type C: Observed 49 at seed=42 (2026-04-01). 20 = initial count, must grow beyond it. Threshold = initial value (weak, consider tightening).
        assert!(
            alive_y3 > 20,
            "Population should grow beyond initial 20 in 3 years, got {alive_y3}"
        );
        // Type B: Ethnographic hunter-gatherer band sizes 25-30 before fission (Service 1962). Peak must approach migration threshold (config MIGRATION_MIN_POP=30).
        assert!(
            peak >= 28,
            "Peak population should approach migration threshold (30), got {peak}"
        );
    }

    /// Over 5 years, a second settlement must form via migration.
    /// Validates the full chain: population growth → migration trigger → new settlement.
    #[test]
    fn harness_multi_settlement_emerges() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(4380 * 5);

        let alive = count_alive(&engine);
        let settlements = engine.resources().settlements.len();

        println!("[harness] 5-year result: alive={alive}, settlements={settlements}");
        for (id, settlement) in engine.resources().settlements.iter() {
            println!(
                "[harness]   Settlement {:?}: members={}, food={:.1}, stone={:.1}",
                id,
                settlement.members.len(),
                settlement.stockpile_food,
                settlement.stockpile_stone
            );
        }

        // Type C: Observed 3 settlements at seed=42 (2026-04-01). Threshold 2 = minimum for migration validation. Margin 1.5×.
        assert!(
            settlements >= 2,
            "Expected ≥2 settlements after 5 years, got {settlements}. Population was {alive}."
        );
        for (id, settlement) in engine.resources().settlements.iter() {
            // Type A: Every settlement must have at least 1 member. Empty settlement = migration or cleanup bug.
            assert!(
                !settlement.members.is_empty(),
                "Settlement {:?} has no members",
                id
            );
        }
    }

    /// After 1 year, at least one shelter should be built (total completed buildings > 2).
    #[test]
    fn harness_shelter_built_after_one_year() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(4380);

        let resources = engine.resources();
        let building_count = resources.buildings.len();
        let complete_count = resources.buildings.values().filter(|b| b.is_complete).count();
        let shelter_count = resources
            .buildings
            .values()
            .filter(|b| b.is_complete && b.building_type == "shelter")
            .count();

        println!(
            "[harness] buildings: total={} complete={} shelters={}",
            building_count, complete_count, shelter_count
        );

        // Type C: Observed 10 buildings at seed=42 (2026-04-01). Threshold 3 = minimum viable (campfire+stockpile+shelter). Margin 3.3×.
        assert!(
            complete_count >= 3,
            "expected at least 3 completed buildings (campfire+stockpile+shelter), got {complete_count}"
        );
    }

    /// After 1 year with 20 agents, band count should be reasonable (not over-splitting).
    /// Also validates per-band settlement coherence and Dunbar L2 size cap.
    #[test]
    fn harness_band_count_reasonable() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(4380);

        let resources = engine.resources();
        let band_count = resources.band_store.all().count();

        println!("[harness] bands: {}", band_count);

        // Type B: 20 agents ÷ Dunbar L2 (15) ≈ 1-2 bands. Max 5 prevents over-fission. (Hill et al. 2011)
        assert!(
            band_count <= 5,
            "expected at most 5 bands for 20 agents, got {band_count} (over-splitting)"
        );
        // Type A: BandFormationSystem must produce at least 1 band from 20 agents with GFS threshold 0.5.
        assert!(band_count >= 1, "expected at least 1 band, got {band_count}");

        // Build entity → settlement_id lookup.
        let world = engine.world();
        let mut entity_to_sid: std::collections::HashMap<
            sim_core::ids::EntityId,
            Option<SettlementId>,
        > = std::collections::HashMap::new();
        for (entity, identity) in world.query::<&Identity>().iter() {
            entity_to_sid.insert(
                sim_core::ids::EntityId(entity.id() as u64),
                identity.settlement_id,
            );
        }

        let resources = engine.resources();
        for band in resources.band_store.all() {
            // Type A: Config-enforced cap. BAND_MAX_SIZE (15) = Dunbar Layer 2. Violation = fission not triggering.
            assert!(
                band.members.len() <= sim_core::config::BAND_MAX_SIZE,
                "Band '{}' has {} members, exceeds max {} (Dunbar L2)",
                band.name,
                band.members.len(),
                sim_core::config::BAND_MAX_SIZE
            );

            let mut sids: std::collections::HashSet<Option<SettlementId>> =
                std::collections::HashSet::new();
            for &member in &band.members {
                if let Some(&sid) = entity_to_sid.get(&member) {
                    sids.insert(sid);
                }
            }
            // Type A: Band = co-residential group. Cross-settlement membership = broken invariant (fixed 2026-04-01).
            assert!(
                sids.len() <= 1,
                "Band '{}' spans {} settlements — cross-settlement band detected",
                band.name,
                sids.len()
            );
        }
    }

    /// After 2 years, no band should span multiple settlements.
    #[test]
    fn harness_band_settlement_coherence() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(8760);

        let world = engine.world();
        let mut entity_to_sid: std::collections::HashMap<
            sim_core::ids::EntityId,
            Option<SettlementId>,
        > = std::collections::HashMap::new();
        for (entity, identity) in world.query::<&Identity>().iter() {
            entity_to_sid.insert(
                sim_core::ids::EntityId(entity.id() as u64),
                identity.settlement_id,
            );
        }

        let band_store = &engine.resources().band_store;
        let mut violations = 0;
        for band in band_store.all() {
            let mut sids: std::collections::HashSet<Option<SettlementId>> =
                std::collections::HashSet::new();
            for &member in &band.members {
                if let Some(&sid) = entity_to_sid.get(&member) {
                    sids.insert(sid);
                }
            }
            if sids.len() > 1 {
                violations += 1;
                eprintln!(
                    "[harness] VIOLATION: band '{}' (id={:?}) spans {} settlements",
                    band.name,
                    band.id,
                    sids.len()
                );
            }
        }

        // Type A: Band = co-residential invariant. Any cross-settlement membership is a structural bug (fixed 2026-04-01).
        assert_eq!(violations, 0, "No band should span multiple settlements");
    }

    /// After 2 years, no band should exceed BAND_MAX_SIZE (Dunbar Layer 2).
    #[test]
    fn harness_band_size_within_dunbar_l2() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(8760);

        let band_store = &engine.resources().band_store;
        for band in band_store.all() {
            // Type A: Config-enforced cap. BAND_MAX_SIZE (15) = Dunbar Layer 2 sympathy group (Hill et al. 2011). Violation = fission not triggering.
            assert!(
                band.members.len() <= sim_core::config::BAND_MAX_SIZE,
                "Band '{}' has {} members, max allowed is {} (Dunbar L2)",
                band.name,
                band.members.len(),
                sim_core::config::BAND_MAX_SIZE
            );
        }
    }

    /// After extended simulation, no band should span settlements, and not all agents are bandless.
    #[test]
    fn harness_migration_clears_band() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(4380 * 3);

        let world = engine.world();

        // Build entity → settlement_id lookup.
        let mut entity_to_sid: std::collections::HashMap<
            sim_core::ids::EntityId,
            Option<SettlementId>,
        > = std::collections::HashMap::new();
        for (entity, identity) in world.query::<&Identity>().iter() {
            entity_to_sid.insert(
                sim_core::ids::EntityId(entity.id() as u64),
                identity.settlement_id,
            );
        }

        // No band should span multiple settlements.
        let resources = engine.resources();
        for band in resources.band_store.all() {
            let mut sids: std::collections::HashSet<Option<SettlementId>> =
                std::collections::HashSet::new();
            for &member in &band.members {
                if let Some(&sid) = entity_to_sid.get(&member) {
                    sids.insert(sid);
                }
            }
            // Type D: Regression guard for migration-band fix (2026-04-01). MigrationRuntimeSystem must clear band_id on settlement change.
            assert!(
                sids.len() <= 1,
                "Post-migration: band '{}' spans {} settlements",
                band.name,
                sids.len()
            );
        }

        // Count bandless vs total agents.
        let mut bandless = 0usize;
        let mut total = 0usize;
        for (_entity, identity) in world.query::<&Identity>().iter() {
            total += 1;
            if identity.band_id.is_none() {
                bandless += 1;
            }
        }
        println!(
            "[harness] bandless agents: {}/{} ({:.1}%)",
            bandless,
            total,
            if total > 0 {
                bandless as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        );
        // Type A: If all agents are bandless, BandFormationSystem is broken. Some must form bands.
        assert!(bandless < total, "Not all agents should be bandless");
    }

    /// After buildings are placed, territory grid should have non-zero data.
    #[test]
    fn harness_territory_grid_has_data() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(2000);

        let resources = engine.resources();
        let factions = resources.territory_grid.active_factions();

        println!("[harness] territory factions: {}", factions.len());

        // Type A: At least 1 settlement exists with buildings → at least 1 territory faction must exist.
        assert!(
            !factions.is_empty(),
            "expected at least one territory faction after 2000 ticks"
        );

        let mut max_value: f32 = 0.0;
        for faction_id in &factions {
            if let Some(data) = resources.territory_grid.get(*faction_id) {
                for &val in data {
                    if val > max_value {
                        max_value = val;
                    }
                }
            }
        }

        println!("[harness] territory max_value: {:.4}", max_value);
        // Type A: Buildings stamp intensity ≥ 0.10 via Gaussian. Max territory value must be nonzero.
        assert!(
            max_value > 0.01,
            "expected non-trivial territory values, max={max_value}"
        );
    }

    /// Territory should not exist on impassable tiles (water).
    #[test]
    fn harness_territory_not_on_water() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(2000);

        let resources = engine.resources();
        let map_w = resources.map.width as usize;

        for faction_id in resources.territory_grid.active_factions() {
            if let Some(data) = resources.territory_grid.get(faction_id) {
                for y in 0..resources.map.height {
                    for x in 0..resources.map.width {
                        let tile = resources.map.get(x, y);
                        if !tile.passable && data[y as usize * map_w + x as usize] > 0.001 {
                            panic!(
                                "territory found on impassable tile ({},{}) terrain={:?} value={:.4}",
                                x,
                                y,
                                tile.terrain,
                                data[y as usize * map_w + x as usize]
                            );
                        }
                    }
                }
            }
        }
    }

    /// Observational test: verifies compute_disputes() and border_friction accumulate correctly
    /// when two or more settlements exist and their territories overlap.
    /// Uses soft assertions — dispute occurrence depends on settlement proximity and seed.
    #[test]
    fn harness_territory_dispute_detected() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(4380 * 5); // 5 years

        let resources = engine.resources();
        let settlements = resources.settlements.len();
        println!("[harness] settlements: {settlements}");

        // Log settlement positions for diagnostics.
        for (id, s) in &resources.settlements {
            println!(
                "[harness]   Settlement {:?} at ({}, {}), pop={}",
                id,
                s.x,
                s.y,
                s.members.len()
            );
        }

        if settlements < 2 {
            println!(
                "[harness] SKIP: only {settlements} settlement(s), need ≥2 for dispute test"
            );
            return;
        }

        let disputes = resources
            .territory_grid
            .compute_disputes(sim_core::config::TERRITORY_DISPUTE_MIN_STRENGTH);
        println!("[harness] territory disputes found: {}", disputes.len());
        for d in &disputes {
            println!(
                "[harness]   factions {} vs {}: overlap={} tiles, intensity={:.2}, epicenter=({},{})",
                d.faction_a,
                d.faction_b,
                d.overlap_tile_count,
                d.overlap_intensity,
                d.epicenter_x,
                d.epicenter_y
            );
        }

        let total_friction: f64 = resources.border_friction.values().sum();
        println!("[harness] total border friction: {total_friction:.2}");
        println!(
            "[harness] border friction pairs: {}",
            resources.border_friction.len()
        );

        // Settlement-only disputes (faction_id 1–999): count must not exceed pairs.
        let settlement_disputes: Vec<_> = disputes
            .iter()
            .filter(|d| d.faction_a < 1000 && d.faction_b < 1000)
            .collect();
        println!(
            "[harness] settlement-only disputes: {}",
            settlement_disputes.len()
        );
        // Type A: Settlement dispute count cannot exceed C(n,2) = n×(n-1)/2 pairs. Violation = double-counting bug.
        assert!(
            settlement_disputes.len() <= settlements * (settlements - 1) / 2,
            "settlement dispute count {} exceeds theoretical max for {settlements} settlements",
            settlement_disputes.len(),
        );

        // export_dispute_map() must return the correct buffer size.
        let dispute_map = resources
            .territory_grid
            .export_dispute_map(sim_core::config::TERRITORY_DISPUTE_MIN_STRENGTH);
        // Type A: export_dispute_map() must return width×height buffer. Size mismatch = export bug.
        assert_eq!(
            dispute_map.len(),
            (resources.map.width * resources.map.height) as usize,
            "dispute map size mismatch"
        );

        println!("[harness] harness_territory_dispute_detected: PASS");
    }

    /// After 2 years, territory_hardness should be populated.
    /// Settlement factions must be within [HARDNESS_MIN, HARDNESS_MAX].
    /// Band factions must not exceed TERRITORY_HARDNESS_BAND_CAP.
    #[test]
    fn harness_territory_hardness_scales_with_settlement() {
        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(8760);

        let resources = engine.resources();
        let hardness_map = &resources.territory_hardness;

        // Must have at least one settlement faction entry.
        let settlement_factions: Vec<(u16, f32)> = hardness_map
            .iter()
            .filter(|(&fid, _)| fid < 1000)
            .map(|(&fid, &h)| (fid, h))
            .collect();

        // Type A: At least 1 settlement exists → at least 1 settlement faction in territory_hardness.
        assert!(
            !settlement_factions.is_empty(),
            "Expected at least one settlement faction in territory_hardness, got 0"
        );

        for (fid, hardness) in &settlement_factions {
            eprintln!(
                "[harness] Settlement faction {}: hardness = {:.3}",
                fid, hardness
            );
            // Type A: Hardness is clamped by formula to [HARDNESS_MIN, HARDNESS_MAX]. Violation = arithmetic bug.
            assert!(
                *hardness >= sim_core::config::TERRITORY_HARDNESS_MIN,
                "Faction {} hardness {:.3} below minimum {:.3}",
                fid,
                hardness,
                sim_core::config::TERRITORY_HARDNESS_MIN
            );
            // Type A: Hardness is clamped by formula to [HARDNESS_MIN, HARDNESS_MAX]. Violation = arithmetic bug.
            assert!(
                *hardness <= sim_core::config::TERRITORY_HARDNESS_MAX,
                "Faction {} hardness {:.3} above maximum {:.3}",
                fid,
                hardness,
                sim_core::config::TERRITORY_HARDNESS_MAX
            );
        }

        // Band factions must be capped.
        let band_factions: Vec<(u16, f32)> = hardness_map
            .iter()
            .filter(|(&fid, _)| fid >= 1000)
            .map(|(&fid, &h)| (fid, h))
            .collect();

        for (fid, hardness) in &band_factions {
            eprintln!(
                "[harness] Band faction {}: hardness = {:.3}",
                fid, hardness
            );
            // Type A: Band factions are capped at TERRITORY_HARDNESS_BAND_CAP. Violation = cap not applied.
            assert!(
                *hardness <= sim_core::config::TERRITORY_HARDNESS_BAND_CAP + 0.01,
                "Band faction {} hardness {:.3} exceeds cap {:.3}",
                fid,
                hardness,
                sim_core::config::TERRITORY_HARDNESS_BAND_CAP
            );
        }

        // If population > 20 and buildings > 3, max hardness should be meaningful.
        let total_pop: usize = resources.settlements.values().map(|s| s.population()).sum();
        let total_buildings: usize = resources
            .buildings
            .values()
            .filter(|b| b.is_complete)
            .count();
        eprintln!(
            "[harness] total pop: {}, completed buildings: {}",
            total_pop, total_buildings
        );

        if total_pop > 20 && total_buildings > 3 {
            let max_hardness = settlement_factions
                .iter()
                .map(|(_, h)| *h)
                .fold(0.0_f32, f32::max);
            // Type E (soft): With 20+ pop and 3+ buildings, hardness formula gives ~0.32. Sanity check, not invariant.
            assert!(
                max_hardness > 0.25,
                "With pop={} buildings={}, max settlement hardness should be >0.25, got {:.3}",
                total_pop,
                total_buildings,
                max_hardness
            );
        }

        println!("[harness] harness_territory_hardness_scales_with_settlement: PASS");
    }

    #[test]
    fn harness_multimesh_buffer_valid() {
        use sim_engine::build_agent_multimesh_buffer;

        let mut engine = make_stage1_engine(42, 20);
        engine.run_ticks(500);

        let (buffer, count) = build_agent_multimesh_buffer(engine.world());

        // Type A: count must be positive
        assert!(count >= 10, "expected ≥10 alive agents, got {}", count);
        // Type A: buffer length must be count × 16
        assert_eq!(buffer.len(), count * 16, "buffer length must be count × 16 floats");

        for i in 0..count {
            let base = i * 16;
            let scale_x = buffer[base];     // col-a.x
            let scale_y = buffer[base + 3]; // col-b.y
            let ox = buffer[base + 4];      // origin.x
            let oy = buffer[base + 5];      // origin.y
            let cr = buffer[base + 8];      // color.r
            let cg = buffer[base + 9];      // color.g
            let cb = buffer[base + 10];     // color.b

            // Type B: scale must be positive and uniform
            assert!(scale_x > 0.0 && scale_x <= 2.0, "scale_x out of range: {}", scale_x);
            assert!((scale_x - scale_y).abs() < 1e-5, "scale must be uniform");
            // Type B: position must be within world pixel bounds (256*16=4096)
            assert!(ox >= 0.0 && ox <= 4096.0, "ox out of world range: {}", ox);
            assert!(oy >= 0.0 && oy <= 4096.0, "oy out of world range: {}", oy);
            // Type B: color channels in [0,1]
            assert!(cr >= 0.0 && cr <= 1.0, "color.r={}", cr);
            assert!(cg >= 0.0 && cg <= 1.0, "color.g={}", cg);
            assert!(cb >= 0.0 && cb <= 1.0, "color.b={}", cb);
        }

        println!("[harness] multimesh_buffer: {} agents, {} floats, all valid", count, buffer.len());
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

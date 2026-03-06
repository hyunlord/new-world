use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use sim_core::{config::GameConfig, GameCalendar, WorldMap};
use sim_engine::{GameEvent, SimEngine, SimResources};
use sim_systems::runtime::{
    AceTrackerRuntimeSystem, AgeRuntimeSystem, AttachmentRuntimeSystem,
    BehaviorRuntimeSystem, BuildingEffectRuntimeSystem,
    ChildStressProcessorRuntimeSystem, ChildcareRuntimeSystem, ChronicleRuntimeSystem,
    ContagionRuntimeSystem, ConstructionRuntimeSystem, CopingRuntimeSystem,
    EconomicTendencyRuntimeSystem, EmotionRuntimeSystem, FamilyRuntimeSystem,
    GatheringRuntimeSystem, IntelligenceRuntimeSystem, IntergenerationalRuntimeSystem,
    JobAssignmentRuntimeSystem, JobSatisfactionRuntimeSystem, LeaderRuntimeSystem,
    MemoryRuntimeSystem, MentalBreakRuntimeSystem, MigrationRuntimeSystem,
    MoraleRuntimeSystem, MortalityRuntimeSystem, MovementRuntimeSystem, SteeringRuntimeSystem,
    NeedsRuntimeSystem, NetworkRuntimeSystem, OccupationRuntimeSystem,
    ParentingRuntimeSystem, PersonalityGeneratorRuntimeSystem,
    PersonalityMaturationRuntimeSystem, PopulationRuntimeSystem, ReputationRuntimeSystem,
    ResourceRegenSystem, SettlementCultureRuntimeSystem, SocialEventRuntimeSystem,
    StatSyncRuntimeSystem, StatThresholdRuntimeSystem, StatsRecorderRuntimeSystem,
    StratificationMonitorRuntimeSystem, StressRuntimeSystem, TechDiscoveryRuntimeSystem,
    TechMaintenanceRuntimeSystem, TechPropagationRuntimeSystem,
    TechUtilizationRuntimeSystem, TensionRuntimeSystem, TitleRuntimeSystem,
    TraitRuntimeSystem, TraitViolationRuntimeSystem, TraumaScarRuntimeSystem,
    UpperNeedsRuntimeSystem, ValueRuntimeSystem,
};

use crate::runtime_bindings::runtime_default_compute_domain_modes;

pub(crate) const RUNTIME_SYSTEM_KEY_REPUTATION: &str = "reputation_system";
pub(crate) const RUNTIME_SYSTEM_KEY_SOCIAL_EVENT: &str = "social_event_system";
pub(crate) const RUNTIME_SYSTEM_KEY_MORALE: &str = "morale_system";
pub(crate) const RUNTIME_SYSTEM_KEY_VALUE: &str = "value_system";
pub(crate) const RUNTIME_SYSTEM_KEY_JOB_SATISFACTION: &str = "job_satisfaction_system";
pub(crate) const RUNTIME_SYSTEM_KEY_ECONOMIC_TENDENCY: &str = "economic_tendency_system";
pub(crate) const RUNTIME_SYSTEM_KEY_INTELLIGENCE: &str = "intelligence_system";
pub(crate) const RUNTIME_SYSTEM_KEY_MEMORY: &str = "memory_system";
pub(crate) const RUNTIME_SYSTEM_KEY_COPING: &str = "coping_system";
pub(crate) const RUNTIME_SYSTEM_KEY_NETWORK: &str = "network_system";
pub(crate) const RUNTIME_SYSTEM_KEY_OCCUPATION: &str = "occupation_system";
pub(crate) const RUNTIME_SYSTEM_KEY_CONTAGION: &str = "contagion_system";
pub(crate) const RUNTIME_SYSTEM_KEY_AGE: &str = "age_system";
pub(crate) const RUNTIME_SYSTEM_KEY_JOB_ASSIGNMENT: &str = "job_assignment_system";
pub(crate) const RUNTIME_SYSTEM_KEY_MORTALITY: &str = "mortality_system";
pub(crate) const RUNTIME_SYSTEM_KEY_MENTAL_BREAK: &str = "mental_break_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TRAUMA_SCAR: &str = "trauma_scar_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TRAIT_VIOLATION: &str = "trait_violation_system";
pub(crate) const RUNTIME_SYSTEM_KEY_EMOTION: &str = "emotion_system";
pub(crate) const RUNTIME_SYSTEM_KEY_STRESS: &str = "stress_system";
pub(crate) const RUNTIME_SYSTEM_KEY_NEEDS: &str = "needs_system";
pub(crate) const RUNTIME_SYSTEM_KEY_UPPER_NEEDS: &str = "upper_needs_system";
pub(crate) const RUNTIME_SYSTEM_KEY_RESOURCE_REGEN: &str = "resource_regen_system";
pub(crate) const RUNTIME_SYSTEM_KEY_CHILD_STRESS_PROCESSOR: &str = "child_stress_processor";
pub(crate) const RUNTIME_SYSTEM_KEY_STEERING: &str = "steering_system";
pub(crate) const RUNTIME_SYSTEM_KEY_MOVEMENT: &str = "movement_system";
pub(crate) const RUNTIME_SYSTEM_KEY_CHILDCARE: &str = "childcare_system";
pub(crate) const RUNTIME_SYSTEM_KEY_LEADER: &str = "leader_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TITLE: &str = "title_system";
pub(crate) const RUNTIME_SYSTEM_KEY_STRATIFICATION_MONITOR: &str = "stratification_monitor";
pub(crate) const RUNTIME_SYSTEM_KEY_TENSION: &str = "tension_system";
pub(crate) const RUNTIME_SYSTEM_KEY_BUILDING_EFFECT: &str = "building_effect_system";
pub(crate) const RUNTIME_SYSTEM_KEY_MIGRATION: &str = "migration_system";
pub(crate) const RUNTIME_SYSTEM_KEY_POPULATION: &str = "population_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TECH_UTILIZATION: &str = "tech_utilization_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TECH_MAINTENANCE: &str = "tech_maintenance_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TECH_DISCOVERY: &str = "tech_discovery_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TECH_PROPAGATION: &str = "tech_propagation_system";
pub(crate) const RUNTIME_SYSTEM_KEY_GATHERING: &str = "gathering_system";
pub(crate) const RUNTIME_SYSTEM_KEY_CONSTRUCTION: &str = "construction_system";
pub(crate) const RUNTIME_SYSTEM_KEY_FAMILY: &str = "family_system";
pub(crate) const RUNTIME_SYSTEM_KEY_INTERGENERATIONAL: &str = "intergenerational_system";
pub(crate) const RUNTIME_SYSTEM_KEY_PARENTING: &str = "parenting_system";
pub(crate) const RUNTIME_SYSTEM_KEY_STATS_RECORDER: &str = "stats_recorder";
pub(crate) const RUNTIME_SYSTEM_KEY_STAT_SYNC: &str = "stat_sync_system";
pub(crate) const RUNTIME_SYSTEM_KEY_STAT_THRESHOLD: &str = "stat_threshold_system";
pub(crate) const RUNTIME_SYSTEM_KEY_BEHAVIOR: &str = "behavior_system";
pub(crate) const RUNTIME_SYSTEM_KEY_SETTLEMENT_CULTURE: &str = "settlement_culture_system";
pub(crate) const RUNTIME_SYSTEM_KEY_CHRONICLE: &str = "chronicle_system";
pub(crate) const RUNTIME_SYSTEM_KEY_PERSONALITY_MATURATION: &str = "personality_maturation_system";
pub(crate) const RUNTIME_SYSTEM_KEY_PERSONALITY_GENERATOR: &str = "personality_generator_system";
pub(crate) const RUNTIME_SYSTEM_KEY_ATTACHMENT: &str = "attachment_system";
pub(crate) const RUNTIME_SYSTEM_KEY_ACE_TRACKER: &str = "ace_tracker_system";
pub(crate) const RUNTIME_SYSTEM_KEY_TRAIT: &str = "trait_system";
pub(crate) const RUNTIME_SPEED_OPTIONS: [u32; 5] = [1, 2, 3, 5, 10];
pub(crate) const RUNTIME_COMPUTE_DOMAINS: [&str; 1] = ["pathfinding"];

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RuntimeConfig {
    world_width: Option<u32>,
    world_height: Option<u32>,
    ticks_per_second: Option<u32>,
    max_ticks_per_frame: Option<u32>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            world_width: Some(256),
            world_height: Some(256),
            ticks_per_second: Some(10),
            max_ticks_per_frame: Some(5),
        }
    }
}

pub(crate) struct RuntimeState {
    pub(crate) engine: SimEngine,
    pub(crate) accumulator: f64,
    pub(crate) ticks_per_second: u32,
    pub(crate) max_ticks_per_frame: u32,
    pub(crate) speed_index: i32,
    pub(crate) paused: bool,
    pub(crate) captured_events: Arc<Mutex<Vec<GameEvent>>>,
    pub(crate) registered_systems: Vec<RuntimeSystemEntry>,
    pub(crate) rust_registered_systems: HashSet<String>,
    pub(crate) compute_domain_modes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSystemEntry {
    pub(crate) name: String,
    pub(crate) system_key: String,
    pub(crate) priority: i32,
    pub(crate) tick_interval: i32,
    pub(crate) active: bool,
    pub(crate) registration_index: i32,
    pub(crate) rust_implemented: bool,
    pub(crate) rust_registered: bool,
    pub(crate) exec_backend: String,
}

impl RuntimeState {
    pub(crate) fn from_seed(seed: u64, config: RuntimeConfig) -> Self {
        let game_config = GameConfig::default();
        let world_width = config.world_width.unwrap_or(256).max(1);
        let world_height = config.world_height.unwrap_or(256).max(1);
        let ticks_per_second = config.ticks_per_second.unwrap_or(10).max(1);
        let max_ticks_per_frame = config.max_ticks_per_frame.unwrap_or(5).max(1);
        let calendar = GameCalendar::new(&game_config);
        let map = WorldMap::new(world_width, world_height, seed);
        let captured_events = Arc::new(Mutex::new(Vec::<GameEvent>::with_capacity(256)));
        let mut resources = SimResources::new(calendar, map, seed);
        let event_sink = Arc::clone(&captured_events);
        resources
            .event_bus
            .subscribe(Box::new(move |event: &GameEvent| {
                if let Ok(mut buffer) = event_sink.lock() {
                    buffer.push(event.clone());
                }
            }));
        let engine = SimEngine::new(resources);
        Self {
            engine,
            accumulator: 0.0,
            ticks_per_second,
            max_ticks_per_frame,
            speed_index: 0,
            paused: false,
            captured_events,
            registered_systems: Vec::new(),
            rust_registered_systems: HashSet::new(),
            compute_domain_modes: runtime_default_compute_domain_modes(&RUNTIME_COMPUTE_DOMAINS),
        }
    }
}

pub(crate) fn parse_runtime_config(config_json: &str) -> RuntimeConfig {
    if config_json.trim().is_empty() {
        return RuntimeConfig::default();
    }
    serde_json::from_str::<RuntimeConfig>(config_json).unwrap_or_default()
}

pub(crate) fn clamp_speed_index(index: i32) -> i32 {
    index.clamp(0, (RUNTIME_SPEED_OPTIONS.len() - 1) as i32)
}

pub(crate) fn runtime_speed_multiplier(index: i32) -> f64 {
    let clamped = clamp_speed_index(index) as usize;
    f64::from(RUNTIME_SPEED_OPTIONS[clamped])
}

pub(crate) fn runtime_system_key_from_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let normalized = trimmed.replace('\\', "/").to_lowercase();
    let tail = normalized.rsplit('/').next().unwrap_or_default();
    let key = tail.strip_suffix(".gd").unwrap_or(tail);
    key.to_string()
}

pub(crate) fn runtime_supports_rust_system(system_key: &str) -> bool {
    matches!(
        system_key,
        RUNTIME_SYSTEM_KEY_RESOURCE_REGEN
            | RUNTIME_SYSTEM_KEY_UPPER_NEEDS
            | RUNTIME_SYSTEM_KEY_NEEDS
            | RUNTIME_SYSTEM_KEY_STRESS
            | RUNTIME_SYSTEM_KEY_EMOTION
            | RUNTIME_SYSTEM_KEY_REPUTATION
            | RUNTIME_SYSTEM_KEY_SOCIAL_EVENT
            | RUNTIME_SYSTEM_KEY_MORALE
            | RUNTIME_SYSTEM_KEY_VALUE
            | RUNTIME_SYSTEM_KEY_NETWORK
            | RUNTIME_SYSTEM_KEY_OCCUPATION
            | RUNTIME_SYSTEM_KEY_CONTAGION
            | RUNTIME_SYSTEM_KEY_AGE
            | RUNTIME_SYSTEM_KEY_JOB_ASSIGNMENT
            | RUNTIME_SYSTEM_KEY_MORTALITY
            | RUNTIME_SYSTEM_KEY_MENTAL_BREAK
            | RUNTIME_SYSTEM_KEY_TRAUMA_SCAR
            | RUNTIME_SYSTEM_KEY_TRAIT_VIOLATION
            | RUNTIME_SYSTEM_KEY_JOB_SATISFACTION
            | RUNTIME_SYSTEM_KEY_ECONOMIC_TENDENCY
            | RUNTIME_SYSTEM_KEY_INTELLIGENCE
            | RUNTIME_SYSTEM_KEY_MEMORY
            | RUNTIME_SYSTEM_KEY_COPING
            | RUNTIME_SYSTEM_KEY_CHILD_STRESS_PROCESSOR
            | RUNTIME_SYSTEM_KEY_STEERING
            | RUNTIME_SYSTEM_KEY_MOVEMENT
            | RUNTIME_SYSTEM_KEY_CHILDCARE
            | RUNTIME_SYSTEM_KEY_LEADER
            | RUNTIME_SYSTEM_KEY_TITLE
            | RUNTIME_SYSTEM_KEY_STRATIFICATION_MONITOR
            | RUNTIME_SYSTEM_KEY_TENSION
            | RUNTIME_SYSTEM_KEY_BUILDING_EFFECT
            | RUNTIME_SYSTEM_KEY_MIGRATION
            | RUNTIME_SYSTEM_KEY_POPULATION
            | RUNTIME_SYSTEM_KEY_TECH_UTILIZATION
            | RUNTIME_SYSTEM_KEY_TECH_MAINTENANCE
            | RUNTIME_SYSTEM_KEY_TECH_DISCOVERY
            | RUNTIME_SYSTEM_KEY_TECH_PROPAGATION
            | RUNTIME_SYSTEM_KEY_GATHERING
            | RUNTIME_SYSTEM_KEY_CONSTRUCTION
            | RUNTIME_SYSTEM_KEY_INTERGENERATIONAL
            | RUNTIME_SYSTEM_KEY_PARENTING
            | RUNTIME_SYSTEM_KEY_STATS_RECORDER
            | RUNTIME_SYSTEM_KEY_STAT_SYNC
            | RUNTIME_SYSTEM_KEY_STAT_THRESHOLD
            | RUNTIME_SYSTEM_KEY_BEHAVIOR
            | RUNTIME_SYSTEM_KEY_FAMILY
            | RUNTIME_SYSTEM_KEY_SETTLEMENT_CULTURE
            | RUNTIME_SYSTEM_KEY_CHRONICLE
            | RUNTIME_SYSTEM_KEY_PERSONALITY_MATURATION
            | RUNTIME_SYSTEM_KEY_PERSONALITY_GENERATOR
            | RUNTIME_SYSTEM_KEY_ATTACHMENT
            | RUNTIME_SYSTEM_KEY_ACE_TRACKER
            | RUNTIME_SYSTEM_KEY_TRAIT
    )
}

pub(crate) fn register_supported_rust_system(
    state: &mut RuntimeState,
    system_key: &str,
    priority: i32,
    tick_interval: i32,
) -> bool {
    if !runtime_supports_rust_system(system_key) {
        return false;
    }
    if state.rust_registered_systems.contains(system_key) {
        return true;
    }
    let priority_u32 = priority.max(0) as u32;
    let tick_interval_u64 = tick_interval.max(1) as u64;
    match system_key {
        RUNTIME_SYSTEM_KEY_AGE => {
            state
                .engine
                .register(AgeRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_MORTALITY => {
            state
                .engine
                .register(MortalityRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_MENTAL_BREAK => {
            state
                .engine
                .register(MentalBreakRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TRAUMA_SCAR => {
            state
                .engine
                .register(TraumaScarRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TRAIT_VIOLATION => {
            state
                .engine
                .register(TraitViolationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_JOB_ASSIGNMENT => {
            state
                .engine
                .register(JobAssignmentRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_CONTAGION => {
            state
                .engine
                .register(ContagionRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_OCCUPATION => {
            state
                .engine
                .register(OccupationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_NETWORK => {
            state
                .engine
                .register(NetworkRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_JOB_SATISFACTION => {
            state
                .engine
                .register(JobSatisfactionRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_ECONOMIC_TENDENCY => {
            state
                .engine
                .register(EconomicTendencyRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_INTELLIGENCE => {
            state
                .engine
                .register(IntelligenceRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_MEMORY => {
            state
                .engine
                .register(MemoryRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_COPING => {
            state
                .engine
                .register(CopingRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_CHILD_STRESS_PROCESSOR => {
            state.engine.register(ChildStressProcessorRuntimeSystem::new(
                priority_u32,
                tick_interval_u64,
            ));
        }
        RUNTIME_SYSTEM_KEY_STEERING => {
            state
                .engine
                .register(SteeringRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_MOVEMENT => {
            state
                .engine
                .register(MovementRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_CHILDCARE => {
            state
                .engine
                .register(ChildcareRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_LEADER => {
            state
                .engine
                .register(LeaderRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TITLE => {
            state
                .engine
                .register(TitleRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STRATIFICATION_MONITOR => {
            state
                .engine
                .register(StratificationMonitorRuntimeSystem::new(
                    priority_u32,
                    tick_interval_u64,
                ));
        }
        RUNTIME_SYSTEM_KEY_TENSION => {
            state
                .engine
                .register(TensionRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_BUILDING_EFFECT => {
            state
                .engine
                .register(BuildingEffectRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_MIGRATION => {
            state
                .engine
                .register(MigrationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_POPULATION => {
            state
                .engine
                .register(PopulationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TECH_UTILIZATION => {
            state
                .engine
                .register(TechUtilizationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TECH_MAINTENANCE => {
            state
                .engine
                .register(TechMaintenanceRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TECH_DISCOVERY => {
            state
                .engine
                .register(TechDiscoveryRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TECH_PROPAGATION => {
            state
                .engine
                .register(TechPropagationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_GATHERING => {
            state
                .engine
                .register(GatheringRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_CONSTRUCTION => {
            state
                .engine
                .register(ConstructionRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_FAMILY => {
            state
                .engine
                .register(FamilyRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_INTERGENERATIONAL => {
            state
                .engine
                .register(IntergenerationalRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_PARENTING => {
            state
                .engine
                .register(ParentingRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STATS_RECORDER => {
            state
                .engine
                .register(StatsRecorderRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STAT_SYNC => {
            state
                .engine
                .register(StatSyncRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STAT_THRESHOLD => {
            state
                .engine
                .register(StatThresholdRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_BEHAVIOR => {
            state
                .engine
                .register(BehaviorRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_VALUE => {
            state
                .engine
                .register(ValueRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_MORALE => {
            state
                .engine
                .register(MoraleRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_SOCIAL_EVENT => {
            state
                .engine
                .register(SocialEventRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_REPUTATION => {
            state
                .engine
                .register(ReputationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_EMOTION => {
            state
                .engine
                .register(EmotionRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_STRESS => {
            state
                .engine
                .register(StressRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_NEEDS => {
            state
                .engine
                .register(NeedsRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_UPPER_NEEDS => {
            state
                .engine
                .register(UpperNeedsRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_RESOURCE_REGEN => {
            state
                .engine
                .register(ResourceRegenSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_SETTLEMENT_CULTURE => {
            state
                .engine
                .register(SettlementCultureRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_CHRONICLE => {
            state
                .engine
                .register(ChronicleRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_PERSONALITY_MATURATION => {
            state
                .engine
                .register(PersonalityMaturationRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_PERSONALITY_GENERATOR => {
            state
                .engine
                .register(PersonalityGeneratorRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_ATTACHMENT => {
            state
                .engine
                .register(AttachmentRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_ACE_TRACKER => {
            state
                .engine
                .register(AceTrackerRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        RUNTIME_SYSTEM_KEY_TRAIT => {
            state
                .engine
                .register(TraitRuntimeSystem::new(priority_u32, tick_interval_u64));
        }
        _ => {
            return false;
        }
    }
    state
        .rust_registered_systems
        .insert(system_key.to_string());
    true
}

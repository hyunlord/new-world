use sim_core::config;
use sim_engine::SimEngine;
use sim_systems::runtime::{
    AceTrackerRuntimeSystem, AgeRuntimeSystem, AttachmentRuntimeSystem, BehaviorRuntimeSystem,
    BandBehaviorSystem, BandFormationSystem, BuildingEffectRuntimeSystem,
    ChildStressProcessorRuntimeSystem,
    ChildcareRuntimeSystem, ChronicleRuntimeSystem, ConstructionRuntimeSystem,
    ContagionRuntimeSystem, CopingRuntimeSystem, CraftingRuntimeSystem,
    EconomicTendencyRuntimeSystem, EffectApplySystem, EmotionRuntimeSystem,
    FamilyRuntimeSystem, GatheringRuntimeSystem, InfluenceRuntimeSystem,
    InfluenceSteeringSystem, IntelligenceRuntimeSystem, IntergenerationalRuntimeSystem,
    JobAssignmentRuntimeSystem, JobSatisfactionRuntimeSystem, LeaderRuntimeSystem,
    LlmRequestRuntimeSystem, LlmResponseRuntimeSystem, LlmTimeoutRuntimeSystem,
    MemoryRuntimeSystem, MentalBreakRuntimeSystem, MigrationRuntimeSystem, MoraleRuntimeSystem,
    MortalityRuntimeSystem, MovementRuntimeSystem, NeedsRuntimeSystem, NetworkRuntimeSystem,
    OccupationRuntimeSystem, PairwiseInteractionSystem, ParentingRuntimeSystem,
    PersonalityGeneratorRuntimeSystem, PersonalityMaturationRuntimeSystem,
    PopulationRuntimeSystem, ReputationRuntimeSystem, ResourceRegenSystem,
    SettlementCultureRuntimeSystem, SocialEventRuntimeSystem, StatSyncRuntimeSystem,
    StatThresholdRuntimeSystem, StatsRecorderRuntimeSystem, StorySifterRuntimeSystem,
    StratificationMonitorRuntimeSystem, StressRuntimeSystem, TechDiscoveryRuntimeSystem,
    TechMaintenanceRuntimeSystem, TechPropagationRuntimeSystem, TechUtilizationRuntimeSystem,
    TensionRuntimeSystem, TitleRuntimeSystem, TraitRuntimeSystem, TraitViolationRuntimeSystem,
    TraumaScarRuntimeSystem, UpperNeedsRuntimeSystem, ValueRuntimeSystem,
};

/// Stable typed runtime-system identifier used by the Rust registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub(crate) enum RuntimeSystemId {
    Reputation = 0,
    SocialEvent = 1,
    Morale = 2,
    Value = 3,
    JobSatisfaction = 4,
    EconomicTendency = 5,
    Intelligence = 6,
    Memory = 7,
    Coping = 8,
    Network = 9,
    Occupation = 10,
    Contagion = 11,
    Age = 12,
    JobAssignment = 13,
    Mortality = 14,
    MentalBreak = 15,
    TraumaScar = 16,
    TraitViolation = 17,
    Emotion = 18,
    Stress = 19,
    Needs = 20,
    UpperNeeds = 21,
    ResourceRegen = 22,
    ChildStressProcessor = 23,
    Steering = 24,
    Movement = 25,
    Childcare = 26,
    Leader = 27,
    Title = 28,
    StratificationMonitor = 29,
    Tension = 30,
    BuildingEffect = 31,
    Migration = 32,
    Population = 33,
    TechUtilization = 34,
    TechMaintenance = 35,
    TechDiscovery = 36,
    TechPropagation = 37,
    Gathering = 38,
    Construction = 39,
    Family = 40,
    Intergenerational = 41,
    Parenting = 42,
    StatsRecorder = 43,
    StatSync = 44,
    StatThreshold = 45,
    Behavior = 46,
    SettlementCulture = 47,
    Chronicle = 48,
    PersonalityMaturation = 49,
    PersonalityGenerator = 50,
    Attachment = 51,
    AceTracker = 52,
    Trait = 53,
    LlmRequest = 54,
    LlmResponse = 55,
    LlmTimeout = 56,
    StorySifter = 57,
    Influence = 58,
    EffectApply = 59,
    Crafting = 60,
    PairwiseInteraction = 61,
    BandFormation = 62,
    BandBehavior = 63,
}

impl RuntimeSystemId {
    /// Returns the stable registry name exposed to debug/UI callers.
    pub(crate) const fn registry_name(self) -> &'static str {
        match self {
            Self::Reputation => "reputation_system",
            Self::SocialEvent => "social_event_system",
            Self::Morale => "morale_system",
            Self::Value => "value_system",
            Self::JobSatisfaction => "job_satisfaction_system",
            Self::EconomicTendency => "economic_tendency_system",
            Self::Intelligence => "intelligence_system",
            Self::Memory => "memory_system",
            Self::Coping => "coping_system",
            Self::Network => "network_system",
            Self::Occupation => "occupation_system",
            Self::Contagion => "contagion_system",
            Self::Age => "age_system",
            Self::JobAssignment => "job_assignment_system",
            Self::Mortality => "mortality_system",
            Self::MentalBreak => "mental_break_system",
            Self::TraumaScar => "trauma_scar_system",
            Self::TraitViolation => "trait_violation_system",
            Self::Emotion => "emotion_system",
            Self::Stress => "stress_system",
            Self::Needs => "needs_system",
            Self::UpperNeeds => "upper_needs_system",
            Self::ResourceRegen => "resource_regen_system",
            Self::ChildStressProcessor => "child_stress_processor",
            Self::Steering => "steering_system",
            Self::Movement => "movement_system",
            Self::Childcare => "childcare_system",
            Self::Leader => "leader_system",
            Self::Title => "title_system",
            Self::StratificationMonitor => "stratification_monitor",
            Self::Tension => "tension_system",
            Self::BuildingEffect => "building_effect_system",
            Self::Migration => "migration_system",
            Self::Population => "population_system",
            Self::TechUtilization => "tech_utilization_system",
            Self::TechMaintenance => "tech_maintenance_system",
            Self::TechDiscovery => "tech_discovery_system",
            Self::TechPropagation => "tech_propagation_system",
            Self::Gathering => "gathering_system",
            Self::Construction => "construction_system",
            Self::Family => "family_system",
            Self::Intergenerational => "intergenerational_system",
            Self::Parenting => "parenting_system",
            Self::StatsRecorder => "stats_recorder",
            Self::StatSync => "stat_sync_system",
            Self::StatThreshold => "stat_threshold_system",
            Self::Behavior => "behavior_system",
            Self::SettlementCulture => "settlement_culture_system",
            Self::Chronicle => "chronicle_system",
            Self::PersonalityMaturation => "personality_maturation_system",
            Self::PersonalityGenerator => "personality_generator_system",
            Self::Attachment => "attachment_system",
            Self::AceTracker => "ace_tracker_system",
            Self::Trait => "trait_system",
            Self::LlmRequest => "llm_request_system",
            Self::LlmResponse => "llm_response_system",
            Self::LlmTimeout => "llm_timeout_system",
            Self::StorySifter => "story_sifter_system",
            Self::Influence => "influence_system",
            Self::EffectApply => "effect_apply_system",
            Self::Crafting => "crafting_system",
            Self::PairwiseInteraction => "pairwise_interaction_system",
            Self::BandFormation => "band_formation_system",
            Self::BandBehavior => "band_behavior_system",
        }
    }

    /// Returns the deterministic registry order for all supported Rust runtime systems.
    pub(crate) const fn all() -> &'static [Self] {
        &[
            Self::Reputation,
            Self::SocialEvent,
            Self::Morale,
            Self::Value,
            Self::JobSatisfaction,
            Self::EconomicTendency,
            Self::Intelligence,
            Self::Memory,
            Self::Coping,
            Self::Network,
            Self::Occupation,
            Self::Contagion,
            Self::Age,
            Self::JobAssignment,
            Self::Mortality,
            Self::MentalBreak,
            Self::TraumaScar,
            Self::TraitViolation,
            Self::Emotion,
            Self::Stress,
            Self::Needs,
            Self::UpperNeeds,
            Self::ResourceRegen,
            Self::ChildStressProcessor,
            Self::Steering,
            Self::Movement,
            Self::Childcare,
            Self::Leader,
            Self::Title,
            Self::StratificationMonitor,
            Self::Tension,
            Self::BuildingEffect,
            Self::Migration,
            Self::Population,
            Self::TechUtilization,
            Self::TechMaintenance,
            Self::TechDiscovery,
            Self::TechPropagation,
            Self::Gathering,
            Self::Construction,
            Self::Family,
            Self::Intergenerational,
            Self::Parenting,
            Self::StatsRecorder,
            Self::StatSync,
            Self::StatThreshold,
            Self::Behavior,
            Self::SettlementCulture,
            Self::Chronicle,
            Self::PersonalityMaturation,
            Self::PersonalityGenerator,
            Self::Attachment,
            Self::AceTracker,
            Self::Trait,
            Self::LlmRequest,
            Self::LlmResponse,
            Self::LlmTimeout,
            Self::StorySifter,
            Self::Influence,
            Self::Crafting,
            Self::PairwiseInteraction,
            Self::BandFormation,
            Self::BandBehavior,
            Self::EffectApply,
        ]
    }
}

/// Typed manifest entry used by the runtime scheduler bootstrap.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefaultRuntimeSystemSpec {
    pub(crate) system_id: RuntimeSystemId,
    pub(crate) priority: i32,
    pub(crate) tick_interval: i32,
}

/// Authoritative default runtime manifest in deterministic scheduler order.
pub(crate) const DEFAULT_RUNTIME_SYSTEMS: [DefaultRuntimeSystemSpec; 64] = [
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::StatSync,
        priority: 1,
        tick_interval: 10,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::ResourceRegen,
        priority: 5,
        tick_interval: config::RESOURCE_REGEN_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Childcare,
        priority: 8,
        tick_interval: 2,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::JobAssignment,
        priority: 8,
        tick_interval: config::JOB_ASSIGNMENT_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Needs,
        priority: 10,
        tick_interval: config::NEEDS_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::StatThreshold,
        priority: 12,
        tick_interval: 5,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::UpperNeeds,
        priority: 12,
        tick_interval: config::UPPER_NEEDS_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Influence,
        priority: config::INFLUENCE_SYSTEM_PRIORITY as i32,
        tick_interval: config::INFLUENCE_SYSTEM_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::BuildingEffect,
        priority: 15,
        tick_interval: config::BUILDING_EFFECT_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Intelligence,
        priority: 18,
        tick_interval: 50,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Memory,
        priority: 18,
        tick_interval: config::MEMORY_COMPRESS_INTERVAL_TICKS as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Behavior,
        priority: 20,
        tick_interval: config::BEHAVIOR_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Gathering,
        priority: 25,
        tick_interval: config::GATHERING_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Construction,
        priority: 28,
        tick_interval: config::CONSTRUCTION_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Crafting,
        priority: 29,
        tick_interval: 10,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::BandBehavior,
        priority: config::BAND_BEHAVIOR_SYSTEM_PRIORITY as i32,
        tick_interval: config::BAND_BEHAVIOR_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Steering,
        priority: config::STEERING_SYSTEM_PRIORITY as i32,
        tick_interval: config::STEERING_SYSTEM_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Movement,
        priority: 30,
        tick_interval: config::MOVEMENT_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Emotion,
        priority: 32,
        tick_interval: 12,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::ChildStressProcessor,
        priority: 32,
        tick_interval: 2,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Stress,
        priority: 34,
        tick_interval: config::STRESS_SYSTEM_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::MentalBreak,
        priority: 35,
        tick_interval: 1,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Occupation,
        priority: 36,
        tick_interval: config::OCCUPATION_EVAL_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::TraumaScar,
        priority: 36,
        tick_interval: 10,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::PairwiseInteraction,
        priority: 36,
        tick_interval: 10,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Title,
        priority: 37,
        tick_interval: config::TITLE_EVAL_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::TraitViolation,
        priority: 37,
        tick_interval: 1,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::SocialEvent,
        priority: 37,
        tick_interval: 30,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::BandFormation,
        priority: config::BAND_FORMATION_SYSTEM_PRIORITY as i32,
        tick_interval: config::BAND_FORMATION_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Contagion,
        priority: 38,
        tick_interval: 3,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Reputation,
        priority: 38,
        tick_interval: config::REPUTATION_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::EconomicTendency,
        priority: 39,
        tick_interval: config::ECON_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Morale,
        priority: 40,
        tick_interval: 5,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::JobSatisfaction,
        priority: 40,
        tick_interval: config::JOB_SAT_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Coping,
        priority: 42,
        tick_interval: 30,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Intergenerational,
        priority: 45,
        tick_interval: 240,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Parenting,
        priority: 46,
        tick_interval: 240,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Age,
        priority: 48,
        tick_interval: 50,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::PersonalityMaturation,
        priority: 49,
        tick_interval: config::TICKS_PER_YEAR as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Mortality,
        priority: 49,
        tick_interval: 1,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Population,
        priority: 50,
        tick_interval: config::POPULATION_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Family,
        priority: 52,
        tick_interval: 365,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Leader,
        priority: 52,
        tick_interval: config::LEADER_CHECK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Value,
        priority: 55,
        tick_interval: 200,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Network,
        priority: 58,
        tick_interval: config::REVOLUTION_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Migration,
        priority: 60,
        tick_interval: config::MIGRATION_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::TechDiscovery,
        priority: 62,
        tick_interval: config::TECH_DISCOVERY_INTERVAL_TICKS as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::TechPropagation,
        priority: 62,
        tick_interval: config::TEACHING_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::TechMaintenance,
        priority: 63,
        tick_interval: config::TECH_DISCOVERY_INTERVAL_TICKS as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Tension,
        priority: 64,
        tick_interval: config::TENSION_CHECK_INTERVAL_TICKS as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::TechUtilization,
        priority: 65,
        tick_interval: 1,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::StratificationMonitor,
        priority: 90,
        tick_interval: config::STRAT_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::StatsRecorder,
        priority: 90,
        tick_interval: 200,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::StorySifter,
        priority: config::STORY_SIFTER_PRIORITY as i32,
        tick_interval: config::STORY_SIFTER_TICK_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::SettlementCulture,
        priority: 95,
        tick_interval: 100,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::PersonalityGenerator,
        priority: 97,
        tick_interval: 100,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Attachment,
        priority: 98,
        tick_interval: 100,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::AceTracker,
        priority: 99,
        tick_interval: 100,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Trait,
        priority: 100,
        tick_interval: 10,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::Chronicle,
        priority: 101,
        tick_interval: 1,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::LlmResponse,
        priority: config::LLM_RESPONSE_SYSTEM_PRIORITY as i32,
        tick_interval: config::LLM_RESPONSE_SYSTEM_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::LlmTimeout,
        priority: config::LLM_TIMEOUT_SYSTEM_PRIORITY as i32,
        tick_interval: config::LLM_TIMEOUT_SYSTEM_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::LlmRequest,
        priority: config::LLM_REQUEST_SYSTEM_PRIORITY as i32,
        tick_interval: config::LLM_REQUEST_SYSTEM_INTERVAL as i32,
    },
    DefaultRuntimeSystemSpec {
        system_id: RuntimeSystemId::EffectApply,
        priority: 9999,
        tick_interval: 1,
    },
];

/// Registers one typed Rust runtime system into the scheduler.
pub(crate) fn register_runtime_system(
    engine: &mut SimEngine,
    system_id: RuntimeSystemId,
    priority: i32,
    tick_interval: i32,
) {
    debug_assert!(RuntimeSystemId::all().contains(&system_id));
    let priority_u32 = priority.max(0) as u32;
    let tick_interval_u64 = tick_interval.max(1) as u64;
    match system_id {
        RuntimeSystemId::Age => {
            engine.register(AgeRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Mortality => {
            engine.register(MortalityRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::MentalBreak => engine.register(MentalBreakRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::TraumaScar => engine.register(TraumaScarRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::TraitViolation => engine.register(TraitViolationRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::JobAssignment => engine.register(JobAssignmentRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Contagion => {
            engine.register(ContagionRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Occupation => engine.register(OccupationRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Network => {
            engine.register(NetworkRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::JobSatisfaction => engine.register(JobSatisfactionRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::EconomicTendency => engine.register(EconomicTendencyRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Intelligence => engine.register(IntelligenceRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Memory => {
            engine.register(MemoryRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Coping => {
            engine.register(CopingRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::ChildStressProcessor => engine.register(
            ChildStressProcessorRuntimeSystem::new(priority_u32, tick_interval_u64),
        ),
        RuntimeSystemId::Steering => engine.register(InfluenceSteeringSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Movement => {
            engine.register(MovementRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Childcare => {
            engine.register(ChildcareRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Leader => {
            engine.register(LeaderRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Title => {
            engine.register(TitleRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::StratificationMonitor => engine.register(
            StratificationMonitorRuntimeSystem::new(priority_u32, tick_interval_u64),
        ),
        RuntimeSystemId::Tension => {
            engine.register(TensionRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::BuildingEffect => engine.register(BuildingEffectRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Influence => {
            engine.register(InfluenceRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Migration => {
            engine.register(MigrationRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Population => engine.register(PopulationRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::TechUtilization => engine.register(TechUtilizationRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::TechMaintenance => engine.register(TechMaintenanceRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::TechDiscovery => engine.register(TechDiscoveryRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::TechPropagation => engine.register(TechPropagationRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Gathering => {
            engine.register(GatheringRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Construction => engine.register(ConstructionRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Family => {
            engine.register(FamilyRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Intergenerational => engine.register(IntergenerationalRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Parenting => {
            engine.register(ParentingRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::StatsRecorder => engine.register(StatsRecorderRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::StatSync => {
            engine.register(StatSyncRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::StatThreshold => engine.register(StatThresholdRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Behavior => {
            engine.register(BehaviorRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Value => {
            engine.register(ValueRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Morale => {
            engine.register(MoraleRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::SocialEvent => engine.register(SocialEventRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::PairwiseInteraction => engine.register(PairwiseInteractionSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::BandFormation => {
            engine.register(BandFormationSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::BandBehavior => {
            engine.register(BandBehaviorSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Reputation => engine.register(ReputationRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Emotion => {
            engine.register(EmotionRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Stress => {
            engine.register(StressRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::Needs => {
            engine.register(NeedsRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::UpperNeeds => engine.register(UpperNeedsRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::ResourceRegen => {
            engine.register(ResourceRegenSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::SettlementCulture => engine.register(SettlementCultureRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Chronicle => {
            engine.register(ChronicleRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::PersonalityMaturation => engine.register(
            PersonalityMaturationRuntimeSystem::new(priority_u32, tick_interval_u64),
        ),
        RuntimeSystemId::PersonalityGenerator => engine.register(
            PersonalityGeneratorRuntimeSystem::new(priority_u32, tick_interval_u64),
        ),
        RuntimeSystemId::Attachment => engine.register(AttachmentRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::AceTracker => engine.register(AceTrackerRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Trait => {
            engine.register(TraitRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::LlmRequest => engine.register(LlmRequestRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::LlmResponse => engine.register(LlmResponseRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::LlmTimeout => engine.register(LlmTimeoutRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::StorySifter => engine.register(StorySifterRuntimeSystem::new(
            priority_u32,
            tick_interval_u64,
        )),
        RuntimeSystemId::Crafting => {
            engine.register(CraftingRuntimeSystem::new(priority_u32, tick_interval_u64))
        }
        RuntimeSystemId::EffectApply => {
            engine.register(EffectApplySystem::new(priority_u32, tick_interval_u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn runtime_system_ids_have_unique_registry_names() {
        let mut names: HashSet<&'static str> = HashSet::new();
        for system_id in RuntimeSystemId::all() {
            assert!(names.insert(system_id.registry_name()));
        }
    }

    #[test]
    fn default_runtime_manifest_is_unique_and_deterministic() {
        let mut ids: HashSet<RuntimeSystemId> = HashSet::new();
        for spec in DEFAULT_RUNTIME_SYSTEMS {
            assert!(ids.insert(spec.system_id));
            assert!(spec.tick_interval > 0);
        }
        let all_ids: HashSet<RuntimeSystemId> = RuntimeSystemId::all().iter().copied().collect();
        assert_eq!(ids, all_ids);
        assert_eq!(
            DEFAULT_RUNTIME_SYSTEMS[0].system_id,
            RuntimeSystemId::StatSync
        );
        assert_eq!(
            DEFAULT_RUNTIME_SYSTEMS[DEFAULT_RUNTIME_SYSTEMS.len() - 1].system_id,
            RuntimeSystemId::EffectApply
        );
    }
}

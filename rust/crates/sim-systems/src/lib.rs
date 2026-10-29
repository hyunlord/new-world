//! WorldSim runtime systems.
//!
//! Phase 2 (T7.6 land): 4 influence systems via [`register_phase2_systems`].
//!
//! - [`runtime::influence::BuildingStampSystem`]       priority 90,   every tick
//! - [`runtime::influence::InfluenceUpdateSystem`]     priority 100,  every tick
//! - [`runtime::influence::AgentInfluenceSampleSystem`] priority 110, every tick
//! - [`runtime::influence::InfluenceVisualizationSystem`] priority 1000, every 6 ticks
//!
//! Phase 4-β land: 1 agent system via [`register_agent_systems`].
//!
//! - [`runtime::agent::AgentMovementSystem`]            priority 120, every tick
//!
//! Phase 5-α land: 1 needs system via [`register_needs_systems`].
//!
//! - [`runtime::needs::HungerDecaySystem`]              priority 130, every tick
//! - [`runtime::needs::ThirstDecaySystem`]              priority 131, every tick (Phase 5-β)
//! - [`runtime::needs::SleepDecaySystem`]               priority 132, every tick (Phase 5-γ)
//!
//! Phase 5-β land: 1 decision system via [`register_decision_systems`].
//!
//! - [`runtime::baseline::decision::AgentDecisionSystem`]         priority 125, every tick
//!
//! Phase 0 v0.1.3 patch Section 4.3 base.
//!
//! Dependencies:
//! - [`sim_core`] — ECS components, materials, influence, tile data
//! - [`sim_engine`] — `RuntimeSystem` trait + `SimResources` host

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use sim_engine::SimEngine;

/// Pass 18: time `$body` under `$name` when the `subprofile` feature is on.
/// Without the feature the macro is the bare block.
#[cfg(feature = "subprofile")]
macro_rules! scope {
    ($name:literal, $body:block) => {{
        let timer = $crate::subprofile::Timer::start();
        let value = $body;
        timer.stop($name);
        value
    }};
}

/// Pass 18: without the `subprofile` feature `scope!` is the bare block.
#[cfg(not(feature = "subprofile"))]
macro_rules! scope {
    ($name:literal, $body:block) => {
        $body
    };
}

/// Observe an expression without changing control flow or evaluating it twice.
macro_rules! arm_eval {
    ($name:literal, $expression:expr) => {{
        #[cfg(feature = "perf_arms")]
        let _arm_scope = $crate::perf_arms::Scope::start($name);
        $expression
    }};
}

/// Count the result of a boolean expression without evaluating it twice.
#[cfg(all(test, feature = "perf_arms"))]
macro_rules! arm_outcome {
    ($yes:literal, $no:literal, $expression:expr) => {{
        #[cfg(feature = "perf_arms")]
        {
            let value = $expression;
            $crate::perf_arms::count(if value { $yes } else { $no }, 1);
            value
        }
        #[cfg(not(feature = "perf_arms"))]
        {
            $expression
        }
    }};
}

#[cfg(feature = "perf_arms")]
pub mod perf_arms;

pub mod commands;
/// Read-only old-arm observations for the separately enabled shadow harness.
#[cfg(feature = "mind_shadow")]
pub mod mind_observer;

pub mod runtime;
pub mod save;
#[cfg(feature = "subprofile")]
pub mod subprofile;
pub mod worldgen;

/// Register the Phase 2 influence stack on `engine` in priority order.
///
/// Registers (in priority order after sorting):
/// - 90  : [`runtime::influence::BuildingStampSystem`]
/// - 100 : [`runtime::influence::InfluenceUpdateSystem`]
/// - 110 : [`runtime::influence::AgentInfluenceSampleSystem`]
/// - 1000: [`runtime::influence::InfluenceVisualizationSystem`] (every 6 ticks)
pub fn register_phase2_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::influence::BuildingStampSystem::new()));
    engine.register_system(Box::new(runtime::influence::InfluenceUpdateSystem::new()));
    engine.register_system(Box::new(
        runtime::influence::AgentInfluenceSampleSystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::influence::InfluenceVisualizationSystem::new(),
    ));
}

/// Register the Phase 4-β agent stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 120 : [`runtime::agent::AgentMovementSystem`]
///
/// Runs after `AgentInfluenceSampleSystem` (priority 110) so movement
/// operates on the post-sample tile.
pub fn register_agent_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::agent::AgentMovementSystem::new()));
}

/// Register the Phase 5-α/β/γ needs stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 130 : [`runtime::needs::HungerDecaySystem`]
/// - 131 : [`runtime::needs::ThirstDecaySystem`] (Phase 5-β)
/// - 132 : [`runtime::needs::SleepDecaySystem`] (Phase 5-γ)
/// - 135 : [`runtime::needs::SocialDecaySystem`] (Phase 7-β re-plan)
///
/// All four run after `AgentDecisionSystem` (priority 125) so the
/// decision system reads pre-decay need values, and before
/// `InfluenceVisualizationSystem` (1000) so the visualisation observes
/// the post-decay values. `SocialDecaySystem` sits at 135 (after
/// `SocialInteractionSystem` at 134) so the social handshake/completion
/// fires on pre-decay `loneliness`, and the loneliness advance applies
/// afterward — mirroring the Hunger/Thirst/Sleep pre-decision pattern.
pub fn register_needs_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::needs::HungerDecaySystem::new()));
    engine.register_system(Box::new(runtime::needs::ThirstDecaySystem::new()));
    engine.register_system(Box::new(runtime::needs::SleepDecaySystem::new()));
    engine.register_system(Box::new(runtime::needs::SocialDecaySystem::new()));
}

/// Register the Phase 5-β decision stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 125 : [`runtime::baseline::decision::AgentDecisionSystem`]
///
/// Slots between `AgentMovementSystem` (priority 120) and
/// `HungerDecaySystem` (priority 130) so it observes the post-move
/// position and the pre-decay need values for the current tick.
pub fn register_decision_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::baseline::decision::AgentDecisionSystem::new(),
    ));
}

/// Register the Phase 6-β construction stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 133 : [`runtime::construction::ConstructionSystem`]
///
/// Slots strictly after `SleepDecaySystem` (priority 132) and strictly
/// before `InfluenceVisualizationSystem` (priority 1000). Owns agent
/// `Consuming { ConstructionSite }` exit semantics + completion-edge
/// CausalEvent emission. See `runtime::construction::ConstructionSystem`
/// for full responsibilities.
pub fn register_construction_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::construction::ConstructionSystem::new()));
}

/// Register the Phase 7-β social interaction stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 134 : [`runtime::social::SocialInteractionSystem`]
///
/// Slots strictly after `ConstructionSystem` (priority 133) and strictly
/// before `InfluenceVisualizationSystem` (priority 1000). Owns agent
/// `Consuming { Agent(_) }` exit semantics + completion-edge
/// `SocialInteractionCompleted` emission. `AgentDecisionSystem`
/// (priority 125) owns the Idle→Seeking and Seeking→Consuming
/// transitions plus `SocialInteractionStarted` emission.
pub fn register_social_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::social::SocialInteractionSystem::new()));
}

/// Register the Phase 8-β memory stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 136 : [`runtime::memory::MemorySystem`]
///
/// Slots strictly after `SocialDecaySystem` (priority 135) so the encoding
/// pass observes every same-tick actor event emitted by upstream
/// production systems (Hunger/Thirst/Sleep/Construction/Social) and the
/// `AgentDecisionSystem`. The decay pass applies uniformly to all `Memory`
/// components, no eviction at the per-tick boundary.
pub fn register_memory_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::memory::MemorySystem::new()));
}

/// Register the Phase 9-β combat stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 137 : [`runtime::combat::CombatSystem`]
///
/// Slots strictly after `MemorySystem` (priority 136). Owns agent
/// `Consuming { Agent(_) }` exit semantics for combat pairs tracked in
/// `SimResources::combat_pairs`. `AgentDecisionSystem` (priority 125)
/// owns the Idle → Consuming transition and `CombatStarted` emission.
pub fn register_combat_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::combat::CombatSystem::new()));
}

/// Register the Phase 10-β settlement stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 138 : [`runtime::settlement::SettlementSystem`]
///
/// Slots strictly after `CombatSystem` (priority 137) so deaths from
/// the current tick are reflected in proximity-driven membership before
/// the formation/dissolution check runs. Owns settlement formation,
/// membership sync, community history, birth trigger, and dissolution.
pub fn register_settlement_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::settlement::SettlementSystem::new()));
}

/// Register the `add-starvation-death` survival stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 139 : [`runtime::survival::StarvationSystem`]
///
/// Slots strictly after `SettlementSystem` (priority 138) so a death's
/// `member_agents` removal is the last roster mutation that tick (the next
/// tick's proximity sync recomputes membership from survivors). Owns
/// needs-driven hp damage / recovery + death via the shared
/// [`runtime::survival::despawn_agent`] helper.
pub fn register_survival_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::survival::StarvationSystem::new()));
}

/// Register the Direction-2 slice 2-3a stockpile-deposit stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 141 : [`runtime::settlement::StockpileDepositSystem`] (interval 10)
///
/// Slots strictly after `SettlementSystem` (priority 138) so the
/// `member_agents` roster is current when depositing, and after
/// `StarvationSystem` (139) / `ResourceRegenSystem` (140) so a death's roster
/// mutation has already settled this tick. Passive Inventory→stockpile Food
/// transfer for members within their settlement's proximity radius; touches no
/// `AgentState`/`SeekTarget`/cascade state.
pub fn register_stockpile_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::settlement::StockpileDepositSystem::new()));
}

/// Register the Direction-2 slice 2-4 stockpile-consume (famine-fallback) stack.
///
/// Registers (in priority order after sorting):
/// - 142 : [`runtime::settlement::StockpileConsumeSystem`] (interval 5)
///
/// Slots strictly after `HungerDecaySystem` (130), `SettlementSystem` (138),
/// `StarvationSystem` (139), and `StockpileDepositSystem` (141) so the reserve
/// reflects this tick's deposits and saturation damage is computed from
/// PRE-relief hunger. A critically-hungry member at home that is NOT already on
/// the ground-eat path draws one meal of Food from its settlement `stockpile`
/// to relieve `Hunger`. Touches no `AgentState`/`SeekTarget`/cascade state.
pub fn register_stockpile_consume_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::settlement::StockpileConsumeSystem::new()));
}

/// Register the `add-resource-scarcity-regen` resource stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 126 : [`runtime::resource_regen::StaleSeekTargetSystem`] (interval 1)
/// - 140 : [`runtime::resource_regen::ResourceRegenSystem`]
///   (interval [`runtime::resource_regen::REGEN_INTERVAL`])
///
/// `ResourceRegenSystem` slots strictly after `StarvationSystem` (priority 139)
/// and strictly before `InfluenceVisualizationSystem` (priority 1000).
/// `StaleSeekTargetSystem` slots immediately after `AgentDecisionSystem`
/// (priority 125) so it reconciles a scarcity-stale resource `SeekTarget`
/// before the next tick's movement step.
///
/// Both are pure no-ops unless scarcity is active: `ResourceRegenSystem` is a
/// no-op while the `*_source_max` ceiling registries are empty, and
/// `StaleSeekTargetSystem` is a no-op while every resource seeker's target tile
/// is present (infinite `u8::MAX` sources are never removed). Only the
/// production `seed_finite_resource_scarcity` path makes sources finite, so the
/// 12 shared harnesses stay byte-for-byte unchanged.
pub fn register_resource_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::resource_regen::StaleSeekTargetSystem::new(),
    ));
    engine.register_system(Box::new(runtime::resource_regen::ResourceRegenSystem::new()));
}

/// V7 Phase 7-β / P7β-15 — canonical production system registration.
///
/// Single source of truth for "what systems run in a production engine":
/// callable from both `sim-bridge::ffi::world_node::WorldSimNode::init`
/// (the live Godot path) and harness tests that need a production-
/// equivalent system list without a Godot runtime.
///
/// Registration order (post-sort by priority):
/// - 90   BuildingStampSystem
/// - 100  InfluenceUpdateSystem
/// - 110  AgentInfluenceSampleSystem
/// - 120  AgentMovementSystem
/// - 125  AgentDecisionSystem
/// - 130  HungerDecaySystem
/// - 131  ThirstDecaySystem
/// - 132  SleepDecaySystem
/// - 133  ConstructionSystem
/// - 134  SocialInteractionSystem
/// - 135  SocialDecaySystem (Phase 7-β re-plan)
/// - 136  MemorySystem (Phase 8-β)
/// - 137  CombatSystem (Phase 9-β)
/// - 138  SettlementSystem (Phase 10-β)
/// - 126  StaleSeekTargetSystem (add-resource-scarcity-regen, interval 1)
/// - 139  StarvationSystem (add-starvation-death)
/// - 140  ResourceRegenSystem (add-resource-scarcity-regen, interval 120)
/// - 141  StockpileDepositSystem (Direction-2 slice 2-3a, interval 10)
/// - 142  StockpileConsumeSystem (Direction-2 slice 2-4, interval 5)
/// - 144  MaturationSystem (Revitalize Pass 3, interval 60)
/// - 145  BedAssignmentSystem (Revitalize Pass 3)
/// - 146  AgingSystem (Revitalize Pass 4, interval 60; no-op without the aging rule)
/// - 147  FireSystem (Revitalize Pass 5, interval 60; no-op without the firewood rule)
/// - 148  FissionSystem (Revitalize Pass 11; no-op without the fission rule)
/// - 149  RelationsSystem (Revitalize Pass 20; no-op without the relations rule)
/// - 150  FertilitySystem (M1.5, interval 1,440 at 12:00; no-op without `BirthRule::Fertility`)
/// - 1000 InfluenceVisualizationSystem
///
/// Harness A1b inspects this registry to verify
/// `SocialInteractionSystem` appears exactly once.
pub fn register_default_runtime_systems(engine: &mut SimEngine) {
    register_runtime_systems_with_decision(engine, register_decision_systems);
}

/// Full runtime stack with an initialized colony mind before initial commands.
/// Saved worlds must expose their complete decoded resources before calling this registrar.
pub fn register_colony_mind_runtime_systems(engine: &mut SimEngine) {
    register_runtime_systems_with_decision(engine, |engine| {
        let source = engine
            .resources
            .mind
            .policy_source
            .as_deref()
            .unwrap_or(include_str!("../../sim-core/data/colony/mind.ron"));
        let program = runtime::mind::MindProgram::load(source)
            .unwrap_or_else(|error| panic!("colony mind registration: invalid policy: {error}"));
        let mut mind = runtime::mind::ColonyMindSystem::new(program);
        mind.initialize(&mut engine.resources)
            .unwrap_or_else(|error| {
                panic!("colony mind registration: initialization failed: {error}")
            });
        engine.register_system(Box::new(mind));
    });
}

fn register_runtime_systems_with_decision(engine: &mut SimEngine, decision: fn(&mut SimEngine)) {
    register_phase2_systems(engine);
    register_agent_systems(engine);
    if engine.resources.rules.colony.is_some() {
        engine.register_system(Box::new(runtime::food::FoodDailySystem::new(
            &engine.resources,
        )));
    }
    decision(engine);
    register_needs_systems(engine);
    register_construction_systems(engine);
    register_social_systems(engine);
    register_memory_systems(engine);
    register_combat_systems(engine);
    register_settlement_systems(engine);
    register_survival_systems(engine);
    register_resource_systems(engine);
    register_stockpile_systems(engine);
    register_stockpile_consume_systems(engine);
    register_shelter_planning_systems(engine);
    register_rhythm_systems(engine);
    register_fire_systems(engine);
    register_job_systems(engine);
    register_story_systems(engine);
}

/// Register M5's pressure and story systems (ADR 0027–0030), all no-ops outside
/// the M5 colony: `DraftSystem` (121, every tick), `StorytellerSystem` (170,
/// daily), `AnimalSystem` (171, every 3 ticks), `MoodSystem` (172, every 60)
/// and `QuarrelSystem` (173, every 30); M5.5: `FleeSystem` (123, every 3 ticks), `NeighbourSystem` (174, daily)
/// and `VisitorSystem` (175, every 3 ticks).
pub fn register_story_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::wildlife::DraftSystem));
    engine.register_system(Box::new(runtime::wildlife::FleeSystem));
    engine.register_system(Box::new(runtime::story::StorytellerSystem));
    engine.register_system(Box::new(runtime::wildlife::AnimalSystem));
    engine.register_system(Box::new(runtime::mood::MoodSystem));
    engine.register_system(Box::new(runtime::mood::QuarrelSystem));
    // M5.5 (ADR 0031, 0032): the neighbours (174, daily) and their visitors (175, every 3 ticks).
    engine.register_system(Box::new(runtime::neighbours::NeighbourSystem));
    engine.register_system(Box::new(runtime::neighbours::VisitorSystem));
}

/// Register the M1 job layer (`runtime::jobs`): `BillSystem` (123, every 60
/// ticks, M2), `JobAssignSystem` (124) and `JobExecuteSystem` (127, both every
/// tick) and `HearthFeedSystem` (146, hourly, M2), all no-ops without
/// `rules.colony`; and M3's `RoomSystem` (152), a no-op without the space rule.
pub fn register_job_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::jobs::BillSystem::new()));
    engine.register_system(Box::new(runtime::jobs::JobAssignSystem::new()));
    engine.register_system(Box::new(runtime::steward::defence::StewardDefenceSystem));
    engine.register_system(Box::new(runtime::steward::StewardSystem));
    engine.register_system(Box::new(runtime::jobs::JobExecuteSystem::new()));
    engine.register_system(Box::new(runtime::jobs::HearthFeedSystem::new()));
    engine.register_system(Box::new(runtime::rooms::RoomSystem::new()));
    register_knowledge_systems(engine);
}

/// Register M4's knowledge systems (ADR 0024–0026), all no-ops without
/// `ColonyRule::knowledge`: `OralTransmissionSystem` (160, daily at the evening
/// gathering), `ObservationSystem` (161, every 30 ticks),
/// `KnowledgeDecaySystem` (162, every 60), `DiscoverySystem` (163, every 240)
/// and `TechAggregateSystem` (164, every 240).
pub fn register_knowledge_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::knowledge::OralTransmissionSystem));
    engine.register_system(Box::new(runtime::knowledge::ObservationSystem));
    engine.register_system(Box::new(runtime::knowledge::KnowledgeDecaySystem));
    engine.register_system(Box::new(runtime::knowledge::DiscoverySystem));
    engine.register_system(Box::new(runtime::knowledge::TechAggregateSystem));
}

/// Register the rules-gated Pass 5 hearth fire (priority 147, interval 60),
/// the Pass 11 fission system (priority 148, interval 1) and the Pass 20
/// relations system (priority 149, interval 1; no-op without the rule).
pub fn register_fire_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::settlement::FireSystem::new()));
    // Pass 11: rules-gated village fission (priority 148, after the fire).
    engine.register_system(Box::new(runtime::settlement::FissionSystem::new()));
    // Pass 20: rules-gated village relations and porters (priority 149).
    engine.register_system(Box::new(runtime::settlement::RelationsSystem::new()));
}

/// Register the rules-gated shelter planner.
pub fn register_shelter_planning_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::settlement::ShelterPlanningSystem::default(),
    ));
}

/// Register rules-gated child maturation, owned-shelter bed assignments, aging and fertility.
pub fn register_rhythm_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::agent::MaturationSystem));
    engine.register_system(Box::new(runtime::agent::PairingSystem));
    engine.register_system(Box::new(runtime::agent::BedAssignmentSystem::default()));
    engine.register_system(Box::new(runtime::agent::AgingSystem));
    // M1.5: per-woman fertility (priority 150, daily at 12:00; no-op unless
    // the birth rule is `Fertility`, the colony's).
    engine.register_system(Box::new(runtime::agent::FertilitySystem));
}

/// Select the world's decision system within the shared registration sequence.
/// Callers must pass decoded/requested world kind rather than fresh-engine
/// defaults; worldgen and save supply the actual requested or decoded rules.
pub fn register_world_runtime_systems(
    engine: &mut SimEngine,
    world: sim_engine::registration::WorldKind,
) {
    use sim_engine::registration::WorldKind;
    match world {
        WorldKind::Baseline => register_default_runtime_systems(engine),
        WorldKind::Colony => register_colony_mind_runtime_systems(engine),
    }
}

#[cfg(test)]
mod registration_tests {
    use super::*;
    use sim_engine::registration::{SystemWorlds, WorldKind, SYSTEM_REGISTRATIONS};

    const BASELINE_ORDER: &[&str] = &[
        "BuildingStampSystem",
        "InfluenceUpdateSystem",
        "AgentInfluenceSampleSystem",
        "AgentMovementSystem",
        "DraftSystem",
        "StewardDefenceSystem",
        "BillSystem",
        "FleeSystem",
        "JobAssignSystem",
        "AgentDecisionSystem",
        "StewardSystem",
        "StaleSeekTargetSystem",
        "JobExecuteSystem",
        "HungerDecaySystem",
        "ThirstDecaySystem",
        "SleepDecaySystem",
        "ConstructionSystem",
        "SocialInteractionSystem",
        "SocialDecaySystem",
        "MemorySystem",
        "CombatSystem",
        "SettlementSystem",
        "StarvationSystem",
        "ResourceRegenSystem",
        "StockpileDepositSystem",
        "StockpileConsumeSystem",
        "ShelterPlanningSystem",
        "MaturationSystem",
        "PairingSystem",
        "BedAssignmentSystem",
        "AgingSystem",
        "HearthFeedSystem",
        "FireSystem",
        "FissionSystem",
        "RelationsSystem",
        "FertilitySystem",
        "RoomSystem",
        "OralTransmissionSystem",
        "ObservationSystem",
        "KnowledgeDecaySystem",
        "DiscoverySystem",
        "TechAggregateSystem",
        "StorytellerSystem",
        "AnimalSystem",
        "MoodSystem",
        "QuarrelSystem",
        "NeighbourSystem",
        "VisitorSystem",
        "InfluenceVisualizationSystem",
    ];

    fn registered(world: WorldKind) -> SimEngine {
        let mut engine = SimEngine::new(1, 1, sim_core::MaterialRegistry::new());
        engine.resources.rules = match world {
            WorldKind::Baseline => sim_core::rules::WorldRules::baseline(0),
            WorldKind::Colony => sim_core::rules::WorldRules::colony(0),
        };
        register_world_runtime_systems(&mut engine, world);
        engine
    }

    #[test]
    fn world_registration_preserves_baseline_order_and_matches_membership() {
        // Given: a pinned pre-switch baseline and the declared membership table.
        let unique: std::collections::BTreeSet<_> =
            SYSTEM_REGISTRATIONS.iter().map(|row| row.name).collect();
        assert_eq!(unique.len(), SYSTEM_REGISTRATIONS.len());
        // When: constructing both real system lists without ticking.
        for world in [WorldKind::Baseline, WorldKind::Colony] {
            let engine = registered(world);
            let expected: Vec<_> = SYSTEM_REGISTRATIONS
                .iter()
                .filter(|row| row.worlds.includes(world))
                .map(|row| row.name)
                .collect();
            // Then: names and order match; baseline cannot lose its gated no-ops.
            assert_eq!(engine.system_names(), expected);
            if world == WorldKind::Baseline {
                assert_eq!(engine.system_names(), BASELINE_ORDER);
            }
        }
        let mut default = SimEngine::new(1, 1, sim_core::MaterialRegistry::new());
        register_default_runtime_systems(&mut default);
        assert_eq!(default.system_names(), BASELINE_ORDER);
    }

    #[test]
    fn world_membership_distinguishes_exclusive_systems() {
        // Given/When/Then: exclusive declarations must not leak into the other world.
        assert!(SystemWorlds::Baseline.includes(WorldKind::Baseline));
        assert!(!SystemWorlds::Baseline.includes(WorldKind::Colony));
        assert!(SystemWorlds::Colony.includes(WorldKind::Colony));
        assert!(!SystemWorlds::Colony.includes(WorldKind::Baseline));
        assert!(SystemWorlds::Both.includes(WorldKind::Baseline));
        assert!(SystemWorlds::Both.includes(WorldKind::Colony));
    }

    #[test]
    fn system_worlds_document_matches_real_registration() {
        // Given: metadata membership and actual runtime priority/cadence.
        let baseline = registered(WorldKind::Baseline).new_tick_profile();
        let colony = registered(WorldKind::Colony).new_tick_profile();
        let mut table = String::from(
            "| System | Priority | Interval ticks | Baseline | Colony |\n|---|---:|---:|---|---|\n",
        );
        // When: rendering in declared execution order from real system metadata.
        for row in SYSTEM_REGISTRATIONS {
            let b = baseline.systems.iter().find(|s| s.name == row.name);
            let c = colony.systems.iter().find(|s| s.name == row.name);
            assert_eq!(b.is_some(), row.worlds.includes(WorldKind::Baseline));
            assert_eq!(c.is_some(), row.worlds.includes(WorldKind::Colony));
            let actual = b.or(c).expect("registered in at least one world");
            if let (Some(b), Some(c)) = (b, c) {
                assert_eq!((b.priority, b.interval), (c.priority, c.interval));
            }
            table.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                row.name,
                actual.priority,
                actual.interval,
                if b.is_some() { "yes" } else { "no" },
                if c.is_some() { "yes" } else { "no" }
            ));
        }
        // Then: the checked-in table is an exact snapshot, and --nocapture prints it.
        let document = include_str!("../../../../docs/v8/core/SYSTEM_WORLDS.md");
        let (_, rest) = document
            .split_once("<!-- SYSTEM_WORLDS:START -->\n")
            .expect("start marker");
        let (expected, _) = rest
            .split_once("<!-- SYSTEM_WORLDS:END -->")
            .expect("end marker");
        assert_eq!(table, expected);
        println!("{table}");
    }

    #[test]
    fn switch_production_membership_has_exactly_one_world_specific_brain() {
        let baseline = registered(WorldKind::Baseline);
        let colony = registered(WorldKind::Colony);
        assert_eq!(baseline.system_names(), BASELINE_ORDER);
        assert_eq!(
            colony
                .system_names()
                .iter()
                .filter(|name| **name == "ColonyMindSystem")
                .count(),
            1
        );
        assert!(!colony.system_names().contains(&"AgentDecisionSystem"));
        assert!(!baseline.system_names().contains(&"ColonyMindSystem"));
        assert!(colony.resources.mind.history.is_some());
    }
}

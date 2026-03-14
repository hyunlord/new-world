//! Runtime simulation systems split into domain-specific submodules.
//!
//! All public types are re-exported here so that external consumers
//! (e.g. `sim-bridge`) can continue to use `crate::runtime::XxxSystem`.
// TODO(v3.1): REFACTOR - this module still centralizes v2-era scheduler wiring and many config:: references.
// TODO(v3.1): REFACTOR - move system cadence toward Hot/Warm/Cold metadata as A-5 lands.

mod biology;
mod cognition;
mod crafting;
mod economy;
mod effect_apply;
mod influence;
mod llm_request_system;
mod llm_response_system;
mod llm_timeout_system;
mod needs;
mod pairwise;
mod psychology;
mod record;
mod social;
mod steering;
mod steering_derive;
mod story_sifter;
mod world;

// ---- biology ----
pub use biology::{
    AceTrackerRuntimeSystem, AgeRuntimeSystem, AttachmentRuntimeSystem, ChildcareRuntimeSystem,
    IntergenerationalRuntimeSystem, MortalityRuntimeSystem, ParentingRuntimeSystem,
    PersonalityGeneratorRuntimeSystem, PopulationRuntimeSystem,
};

// ---- cognition ----
pub use cognition::{BehaviorRuntimeSystem, IntelligenceRuntimeSystem, MemoryRuntimeSystem};
pub use crafting::CraftingRuntimeSystem;

// ---- economy ----
pub use economy::{
    BuildingEffectRuntimeSystem, ConstructionRuntimeSystem, GatheringRuntimeSystem,
    JobAssignmentRuntimeSystem, JobSatisfactionRuntimeSystem, ResourceRegenSystem,
};
pub use effect_apply::EffectApplySystem;

// ---- influence ----
pub use influence::InfluenceRuntimeSystem;

// ---- llm ----
pub use llm_request_system::LlmRequestRuntimeSystem;
pub use llm_response_system::{drain_and_apply_llm_responses, LlmResponseRuntimeSystem};
pub use llm_timeout_system::LlmTimeoutRuntimeSystem;

// ---- needs ----
pub use needs::{ChildStressProcessorRuntimeSystem, NeedsRuntimeSystem, UpperNeedsRuntimeSystem};

// ---- psychology ----
pub use psychology::{
    ContagionRuntimeSystem, CopingRuntimeSystem, EmotionRuntimeSystem, MentalBreakRuntimeSystem,
    MoraleRuntimeSystem, PersonalityMaturationRuntimeSystem, StressRuntimeSystem, TraitCondition,
    TraitConditionSource, TraitDefinition, TraitDirection, TraitRuntimeSystem,
    TraitViolationRuntimeSystem, TraumaScarRuntimeSystem,
};

// ---- record ----
pub use record::{
    ChronicleRuntimeSystem, StatSyncRuntimeSystem, StatThresholdRuntimeSystem,
    StatsRecorderRuntimeSystem, STAT_THRESHOLD_FLAG_HUNGER_LOW,
};

// ---- social ----
pub use pairwise::PairwiseInteractionSystem;
pub use social::{
    EconomicTendencyRuntimeSystem, FamilyRuntimeSystem, LeaderRuntimeSystem, NetworkRuntimeSystem,
    OccupationRuntimeSystem, ReputationRuntimeSystem, SettlementCultureRuntimeSystem,
    SocialEventRuntimeSystem, StratificationMonitorRuntimeSystem, TitleRuntimeSystem,
    ValueRuntimeSystem,
};

// ---- steering ----
pub use steering::{InfluenceSteeringSystem, SteeringRuntimeSystem};
pub use steering_derive::derive_steering_params;
pub use story_sifter::StorySifterRuntimeSystem;

// ---- world ----
pub use world::{
    MigrationRuntimeSystem, MovementRuntimeSystem, TechDiscoveryRuntimeSystem,
    TechMaintenanceRuntimeSystem, TechPropagationRuntimeSystem, TechUtilizationRuntimeSystem,
    TensionRuntimeSystem,
};

#[cfg(test)]
mod tests {
    use super::{
        AceTrackerRuntimeSystem, AgeRuntimeSystem, AttachmentRuntimeSystem, BehaviorRuntimeSystem,
        BuildingEffectRuntimeSystem, ChildStressProcessorRuntimeSystem, ChildcareRuntimeSystem,
        ChronicleRuntimeSystem, ConstructionRuntimeSystem, ContagionRuntimeSystem,
        CopingRuntimeSystem, EconomicTendencyRuntimeSystem, EmotionRuntimeSystem,
        FamilyRuntimeSystem, GatheringRuntimeSystem, InfluenceRuntimeSystem,
        IntelligenceRuntimeSystem, IntergenerationalRuntimeSystem, JobAssignmentRuntimeSystem,
        JobSatisfactionRuntimeSystem, LeaderRuntimeSystem, MemoryRuntimeSystem,
        MentalBreakRuntimeSystem, MigrationRuntimeSystem, MoraleRuntimeSystem,
        MortalityRuntimeSystem, MovementRuntimeSystem, NeedsRuntimeSystem, NetworkRuntimeSystem,
        OccupationRuntimeSystem, ParentingRuntimeSystem, PersonalityGeneratorRuntimeSystem,
        PersonalityMaturationRuntimeSystem, PopulationRuntimeSystem, ReputationRuntimeSystem,
        ResourceRegenSystem, SettlementCultureRuntimeSystem, SocialEventRuntimeSystem,
        StatSyncRuntimeSystem, StatThresholdRuntimeSystem, StatsRecorderRuntimeSystem,
        SteeringRuntimeSystem, StratificationMonitorRuntimeSystem, StressRuntimeSystem,
        TechDiscoveryRuntimeSystem, TechMaintenanceRuntimeSystem, TechPropagationRuntimeSystem,
        TechUtilizationRuntimeSystem, TensionRuntimeSystem, TitleRuntimeSystem, TraitCondition,
        TraitConditionSource, TraitDefinition, TraitDirection, TraitRuntimeSystem,
        TraitViolationRuntimeSystem, TraumaScarRuntimeSystem, UpperNeedsRuntimeSystem,
        ValueRuntimeSystem, STAT_THRESHOLD_FLAG_HUNGER_LOW,
    };
    use crate::body;
    use hecs::World;
    use sim_core::components::{
        Age, Behavior, Body as BodyComponent, Coping, CopingRebound, Economic, Emotion, Identity,
        Intelligence, Inventory, Memory, MemoryEntry, Needs, Personality, Position, SkillEntry,
        Skills, Social, Stress, StressTrace, Traits, TraumaScar, Values,
    };
    use sim_core::ids::EntityId;
    use sim_core::world::TileResource;
    use sim_core::{
        config::GameConfig, ActionType, AttachmentType, Building, BuildingId, ChannelId,
        CopingStrategyId, EmotionType, GameCalendar, GrowthStage, HexacoAxis, HexacoFacet,
        IntelligenceType, ItemDerivedStats, ItemInstance, ItemOwner, MentalBreakType, NeedType,
        RelationType, ResourceType, SettlementId, Sex, SocialClass, TechState, ValueType, WorldMap,
    };
    use sim_engine::{SimEngine, SimResources, SimSystem};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(2, 1, 123);
        map.get_mut(0, 0).resources.push(TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 5.0,
            regen_rate: 0.75,
        });
        map.get_mut(1, 0).resources.push(TileResource {
            resource_type: ResourceType::Wood,
            amount: 4.8,
            max_amount: 5.0,
            regen_rate: 1.0,
        });
        SimResources::new(calendar, map, 999)
    }

    #[test]
    fn resource_regen_system_applies_regen_and_caps_at_max() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut system = ResourceRegenSystem::new(5, 10);
        system.run(&mut world, &mut resources, 10);

        let first = &resources.map.get(0, 0).resources[0];
        assert!((first.amount - 3.75).abs() < 1e-6);

        let second = &resources.map.get(1, 0).resources[0];
        assert!((second.amount - 5.0).abs() < 1e-6);
    }

    #[test]
    fn needs_runtime_system_applies_decay_and_action_energy_cost() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).temperature = 0.2;

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.6);
        needs.set(NeedType::Belonging, 0.7);
        needs.set(NeedType::Thirst, 0.8);
        needs.set(NeedType::Warmth, 0.9);
        needs.set(NeedType::Safety, 0.85);
        needs.energy = 0.75;
        needs.set(NeedType::Sleep, 0.75);

        let behavior = Behavior {
            current_action: ActionType::GatherWood,
            ..Behavior::default()
        };
        let body_component = BodyComponent {
            end_realized: 6000,
            rec_realized: 4000,
            ..BodyComponent::default()
        };
        let position = Position::new(0, 0);
        let entity = world.spawn((needs, behavior, body_component, position));

        let mut system = NeedsRuntimeSystem::new(10, 4);
        system.run(&mut world, &mut resources, 4);

        let decays = body::needs_base_decay_step(
            0.6,
            sim_core::config::HUNGER_DECAY_RATE as f32,
            1.0,
            sim_core::config::HUNGER_METABOLIC_MIN as f32,
            sim_core::config::HUNGER_METABOLIC_RANGE as f32,
            sim_core::config::ENERGY_DECAY_RATE as f32,
            sim_core::config::SOCIAL_DECAY_RATE as f32,
            sim_core::config::SAFETY_DECAY_RATE as f32,
            sim_core::config::THIRST_DECAY_RATE as f32,
            sim_core::config::WARMTH_DECAY_RATE as f32,
            0.2,
            true,
            sim_core::config::WARMTH_TEMP_NEUTRAL as f32,
            sim_core::config::WARMTH_TEMP_FREEZING as f32,
            sim_core::config::WARMTH_TEMP_COLD as f32,
            true,
        );
        let action_cost = body::action_energy_cost(
            sim_core::config::ENERGY_ACTION_COST as f32,
            (6000.0 / sim_core::config::BODY_REALIZED_MAX as f32).clamp(0.0, 1.0),
            sim_core::config::BODY_END_COST_REDUCTION as f32,
        );
        let expected_energy = (0.75 - decays[1] - action_cost).clamp(0.0, 1.0);

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!(
            (updated.get(NeedType::Hunger) as f32 - (0.6 - decays[0]).clamp(0.0, 1.0)).abs() < 1e-6
        );
        assert!(
            (updated.get(NeedType::Belonging) as f32 - (0.7 - decays[2]).clamp(0.0, 1.0)).abs()
                < 1e-6
        );
        assert!(
            (updated.get(NeedType::Thirst) as f32 - (0.8 - decays[3]).clamp(0.0, 1.0)).abs() < 1e-6
        );
        assert!(
            (updated.get(NeedType::Warmth) as f32 - (0.9 - decays[4]).clamp(0.0, 1.0)).abs() < 1e-6
        );
        assert!(
            (updated.get(NeedType::Safety) as f32 - (0.85 - decays[5]).clamp(0.0, 1.0)).abs()
                < 1e-6
        );
        assert!((updated.energy as f32 - expected_energy).abs() < 1e-6);
        assert!((updated.get(NeedType::Sleep) as f32 - expected_energy).abs() < 1e-6);
    }

    #[test]
    fn childcare_runtime_system_feeds_hungry_child_and_decrements_stockpile() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut settlement =
            sim_core::Settlement::new(SettlementId(1), "alpha".to_string(), 0, 0, 0);
        settlement.stockpile_food = 1.5;
        resources.settlements.insert(settlement.id, settlement);

        let age = Age {
            stage: GrowthStage::Child,
            ..Age::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.20);
        let identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        let entity = world.spawn((age, needs, identity));

        let mut system = ChildcareRuntimeSystem::new(8, sim_core::config::CHILDCARE_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::CHILDCARE_TICK_INTERVAL,
        );

        let updated_needs = world
            .get::<&Needs>(entity)
            .expect("child needs should be queryable");
        assert!(
            (updated_needs.get(NeedType::Hunger) as f32 - 0.35).abs() < 1e-6,
            "child hunger should increase by feed_amount * FOOD_HUNGER_RESTORE"
        );
        drop(updated_needs);

        let settlement_after = resources
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should remain present");
        assert!((settlement_after.stockpile_food as f32 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn childcare_runtime_system_prioritizes_more_hungry_child_when_food_limited() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut settlement =
            sim_core::Settlement::new(SettlementId(3), "beta".to_string(), 0, 0, 0);
        settlement.stockpile_food = 0.5;
        resources.settlements.insert(settlement.id, settlement);

        let low_hunger_age = Age {
            stage: GrowthStage::Child,
            ..Age::default()
        };
        let mut low_hunger_needs = Needs::default();
        low_hunger_needs.set(NeedType::Hunger, 0.20);
        let low_hunger_identity = Identity {
            settlement_id: Some(SettlementId(3)),
            ..Identity::default()
        };
        let low_hunger_entity =
            world.spawn((low_hunger_age, low_hunger_needs, low_hunger_identity));

        let high_hunger_age = Age {
            stage: GrowthStage::Toddler,
            ..Age::default()
        };
        let mut high_hunger_needs = Needs::default();
        high_hunger_needs.set(NeedType::Hunger, 0.60);
        let high_hunger_identity = Identity {
            settlement_id: Some(SettlementId(3)),
            ..Identity::default()
        };
        let high_hunger_entity =
            world.spawn((high_hunger_age, high_hunger_needs, high_hunger_identity));

        let mut system = ChildcareRuntimeSystem::new(8, sim_core::config::CHILDCARE_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::CHILDCARE_TICK_INTERVAL,
        );

        let low_hunger_after = world
            .get::<&Needs>(low_hunger_entity)
            .expect("first child needs should be queryable");
        let high_hunger_after = world
            .get::<&Needs>(high_hunger_entity)
            .expect("second child needs should be queryable");
        assert!(
            (low_hunger_after.get(NeedType::Hunger) as f32 - 0.35).abs() < 1e-6,
            "hungrier child should be fed first"
        );
        assert!(
            (high_hunger_after.get(NeedType::Hunger) as f32 - 0.60).abs() < 1e-6,
            "less hungry child should remain unchanged when stockpile is exhausted"
        );
    }

    #[test]
    fn stress_runtime_system_updates_stress_state_and_components() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.05);
        needs.set(NeedType::Thirst, 0.10);
        needs.set(NeedType::Warmth, 0.08);
        needs.set(NeedType::Safety, 0.10);
        needs.set(NeedType::Belonging, 0.20);
        needs.energy = 0.15;

        let mut emotion = Emotion::default();
        emotion.add(EmotionType::Fear, 0.7);
        emotion.add(EmotionType::Anger, 0.6);
        emotion.add(EmotionType::Sadness, 0.5);

        let stress = Stress::default();
        let entity = world.spawn((needs, stress, emotion));

        let mut system = StressRuntimeSystem::new(34, 4);
        system.run(&mut world, &mut resources, 4);

        let updated = world
            .get::<&Stress>(entity)
            .expect("updated stress component should be queryable");
        assert!(updated.level > 0.0);
        assert!(updated.reserve <= 1.0);
        assert!(updated.allostatic_load >= 0.0);
    }

    #[test]
    fn mental_break_runtime_system_triggers_break_and_sets_runtime_fields() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut stress = Stress {
            level: 0.95,
            reserve: 0.10,
            allostatic_load: 0.90,
            ..Stress::default()
        };
        stress.recalculate_state();
        let mut needs = Needs::default();
        needs.energy = 0.05;
        needs.set(NeedType::Hunger, 0.05);
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::C as usize] = 0.20;
        personality.axes[HexacoAxis::E as usize] = 0.85;
        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Anger) = 0.95;
        *emotion.get_mut(EmotionType::Fear) = 0.80;
        *emotion.get_mut(EmotionType::Sadness) = 0.70;
        *emotion.get_mut(EmotionType::Disgust) = 0.50;
        let entity = world.spawn((stress, needs, personality, emotion));

        let mut system = MentalBreakRuntimeSystem::new(35, 1);
        for _ in 0..200 {
            system.run(&mut world, &mut resources, 1);
            let updated = world
                .get::<&Stress>(entity)
                .expect("stress should be queryable");
            if updated.active_mental_break.is_some() {
                break;
            }
        }

        let updated = world
            .get::<&Stress>(entity)
            .expect("stress should be queryable");
        assert!(updated.active_mental_break.is_some());
        assert!(updated.mental_break_remaining > 0);
        assert!(updated.mental_break_count >= 1);
    }

    #[test]
    fn mental_break_runtime_system_clears_active_break_after_countdown() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut stress = Stress {
            level: 0.90,
            reserve: 0.20,
            allostatic_load: 0.60,
            active_mental_break: Some(MentalBreakType::Shutdown),
            mental_break_remaining: 1,
            mental_break_count: 1,
            ..Stress::default()
        };
        stress.recalculate_state();
        let entity = world.spawn((stress,));

        let mut system = MentalBreakRuntimeSystem::new(35, 1);
        system.run(&mut world, &mut resources, 1);

        let updated = world
            .get::<&Stress>(entity)
            .expect("stress should be queryable");
        assert!(updated.active_mental_break.is_none());
        assert_eq!(updated.mental_break_remaining, 0);
        assert!(updated.level <= 0.90);
    }

    #[test]
    fn trait_violation_runtime_system_increases_stress_on_violation() {
        let mut world = World::new();
        let mut resources = make_resources();

        let stress = Stress {
            level: 0.12,
            reserve: 0.82,
            allostatic_load: 0.18,
            ..Stress::default()
        };
        let behavior = Behavior {
            current_action: ActionType::TakeFromStockpile,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.80);
        let social = Social {
            social_capital: 0.80,
            ..Social::default()
        };
        let mut traits = Traits::default();
        traits.add_trait("f_fair_minded".to_string());
        let mut personality = Personality::default();
        personality.facets[HexacoFacet::Fairness as usize] = 0.95;

        let entity = world.spawn((stress, behavior, needs, social, traits, personality));
        let mut system = TraitViolationRuntimeSystem::new(36, 30);
        system.run(&mut world, &mut resources, 30);

        let updated = world
            .get::<&Stress>(entity)
            .expect("updated stress should be queryable");
        assert!(updated.level > 0.12);
        assert!(updated.reserve < 0.82);
        assert!(updated.allostatic_load > 0.18);
    }

    #[test]
    fn trait_violation_runtime_system_ptsd_path_amplifies_repeat_delta() {
        let mut world = World::new();
        let mut resources = make_resources();

        let stress = Stress {
            level: 0.05,
            reserve: 0.90,
            allostatic_load: 0.90,
            ..Stress::default()
        };
        let behavior = Behavior {
            current_action: ActionType::TakeFromStockpile,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.75);
        let social = Social {
            social_capital: 0.75,
            ..Social::default()
        };
        let mut traits = Traits::default();
        traits.add_trait("f_fair_minded".to_string());
        let mut personality = Personality::default();
        personality.facets[HexacoFacet::Fairness as usize] = 1.0;

        let entity = world.spawn((stress, behavior, needs, social, traits, personality));
        let mut system = TraitViolationRuntimeSystem::new(36, 30);

        system.run(&mut world, &mut resources, 30);
        let after_first = world
            .get::<&Stress>(entity)
            .expect("stress after first run should be queryable")
            .level;

        system.run(&mut world, &mut resources, 60);
        let after_second = world
            .get::<&Stress>(entity)
            .expect("stress after second run should be queryable")
            .level;

        let first_delta = after_first - 0.05;
        let second_delta = after_second - after_first;
        assert!(first_delta > 0.0);
        assert!(second_delta > first_delta);
    }

    #[test]
    fn trauma_scar_runtime_system_applies_baseline_drift_from_scars() {
        let mut world = World::new();
        let mut resources = make_resources();

        let memory = Memory {
            trauma_scars: vec![TraumaScar {
                scar_id: "chronic_paranoia".to_string(),
                acquired_tick: 100,
                severity: 0.90,
                reactivation_count: 2,
            }],
            ..Memory::default()
        };
        let mut emotion = Emotion::default();
        emotion.baseline[EmotionType::Joy as usize] = 0.60;
        emotion.baseline[EmotionType::Fear as usize] = 0.20;
        emotion.baseline[EmotionType::Sadness as usize] = 0.20;
        let entity = world.spawn((memory, emotion));

        let mut system = TraumaScarRuntimeSystem::new(36, 10);
        system.run(&mut world, &mut resources, 10);

        let updated = world
            .get::<&Emotion>(entity)
            .expect("emotion should be queryable");
        assert!(updated.baseline[EmotionType::Joy as usize] < 0.60);
        assert!(updated.baseline[EmotionType::Fear as usize] > 0.20);
        assert!(updated.baseline[EmotionType::Sadness as usize] > 0.20);
    }

    #[test]
    fn trauma_scar_runtime_system_clamps_baseline_with_repeated_updates() {
        let mut world = World::new();
        let mut resources = make_resources();

        let memory = Memory {
            trauma_scars: vec![TraumaScar {
                scar_id: "emotional_numbness".to_string(),
                acquired_tick: 200,
                severity: 1.0,
                reactivation_count: 4,
            }],
            ..Memory::default()
        };
        let mut emotion = Emotion::default();
        emotion.baseline[EmotionType::Joy as usize] = 0.01;
        emotion.baseline[EmotionType::Trust as usize] = 0.01;
        emotion.baseline[EmotionType::Sadness as usize] = 0.99;
        let entity = world.spawn((memory, emotion));

        let mut system = TraumaScarRuntimeSystem::new(36, 10);
        for _ in 0..5000 {
            system.run(&mut world, &mut resources, 10);
        }

        let updated = world
            .get::<&Emotion>(entity)
            .expect("emotion should be queryable");
        for idx in 0..updated.baseline.len() {
            assert!((0.0..=1.0).contains(&updated.baseline[idx]));
        }
    }

    #[test]
    fn coping_runtime_system_decrements_strategy_cooldowns() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut coping = Coping::default();
        coping
            .strategy_cooldowns
            .insert(CopingStrategyId::Denial, 40);
        let entity = world.spawn((coping,));

        let mut system = CopingRuntimeSystem::new(42, 30);
        system.run(&mut world, &mut resources, 30);

        let updated = world
            .get::<&Coping>(entity)
            .expect("coping should be queryable");
        let remaining = updated
            .strategy_cooldowns
            .get(&CopingStrategyId::Denial)
            .copied()
            .unwrap_or(0);
        assert_eq!(remaining, 10);
    }

    #[test]
    fn coping_runtime_system_selects_strategy_and_updates_usage() {
        let mut world = World::new();
        let mut resources = make_resources();

        let coping = Coping::default();
        let stress = Stress {
            level: 0.30,
            allostatic_load: 0.20,
            mental_break_count: 8,
            ..Stress::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::C as usize] = 0.85;
        personality.axes[HexacoAxis::O as usize] = 0.70;
        personality.axes[HexacoAxis::A as usize] = 0.65;
        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.55);
        needs.set(NeedType::Safety, 0.60);
        needs.energy = 0.70;
        let entity = world.spawn((coping, stress, personality, needs));

        let mut system = CopingRuntimeSystem::new(42, 30);
        for step in 0..50 {
            system.run(&mut world, &mut resources, step * 30);
        }

        let updated = world
            .get::<&Coping>(entity)
            .expect("coping should be queryable");
        assert!(updated.active_strategy.is_some());
        let total_uses: u32 = updated.usage_counts.values().copied().sum();
        assert!(total_uses >= 1);
        // Verify cooldown tracking mechanism is active (cooldown may have
        // expired after 50 iterations with tick_interval=30, so we check
        // the map is populated rather than requiring a positive value).
        assert!(
            !updated.strategy_cooldowns.is_empty(),
            "strategy_cooldowns should track at least one strategy"
        );
    }

    #[test]
    fn child_stress_processor_runtime_system_updates_child_stress_fields() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            stage: GrowthStage::Child,
            years: 10.0,
            ..Age::default()
        };
        let stress = Stress {
            level: 0.10,
            reserve: 0.90,
            allostatic_load: 0.10,
            ..Stress::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.05);
        needs.set(NeedType::Safety, 0.05);
        needs.set(NeedType::Hunger, 0.10);
        needs.energy = 0.05;
        let social = Social::default();
        let personality = Personality::default();
        let entity = world.spawn((age, stress, needs, social, personality));

        let mut system = ChildStressProcessorRuntimeSystem::new(32, 2);
        system.run(&mut world, &mut resources, 2);

        let updated = world
            .get::<&Stress>(entity)
            .expect("stress should be queryable");
        assert!(updated.level > 0.10);
        assert!(updated.reserve < 0.90);
        assert!(updated.allostatic_load >= 0.10);
    }

    #[test]
    fn child_stress_processor_runtime_system_skips_non_child_stages() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            stage: GrowthStage::Adult,
            years: 30.0,
            ..Age::default()
        };
        let stress = Stress {
            level: 0.25,
            reserve: 0.60,
            allostatic_load: 0.30,
            ..Stress::default()
        };
        let needs = Needs::default();
        let entity = world.spawn((age, stress, needs));

        let mut system = ChildStressProcessorRuntimeSystem::new(32, 2);
        system.run(&mut world, &mut resources, 2);

        let updated = world
            .get::<&Stress>(entity)
            .expect("stress should be queryable");
        assert!((updated.level as f32 - 0.25).abs() < 1e-6);
        assert!((updated.reserve as f32 - 0.60).abs() < 1e-6);
        assert!((updated.allostatic_load as f32 - 0.30).abs() < 1e-6);
    }

    #[test]
    fn emotion_runtime_system_updates_primary_and_baseline_values() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.1;
        *emotion.get_mut(EmotionType::Joy) = 0.2;

        let mut needs = Needs::default();
        needs.set(NeedType::Safety, 0.2);
        needs.set(NeedType::Belonging, 0.3);
        needs.set(NeedType::Hunger, 0.25);
        needs.energy = 0.3;

        let stress = Stress {
            level: 0.8,
            reserve: 0.5,
            allostatic_load: 0.2,
            ..Stress::default()
        };

        let personality = Personality::default();
        let entity = world.spawn((emotion, needs, stress, personality));
        let mut system = EmotionRuntimeSystem::new(32, 12);
        system.run(&mut world, &mut resources, 12);

        let updated = world
            .get::<&Emotion>(entity)
            .expect("updated emotion component should be queryable");
        assert!(updated.get(EmotionType::Fear) > 0.1);
        assert!(updated.get(EmotionType::Joy) < 0.2);
        assert!(updated.baseline(EmotionType::Trust) >= 0.0);
        assert!(updated.baseline(EmotionType::Trust) <= 1.0);
    }

    #[test]
    fn reputation_runtime_system_updates_reputation_state_and_tags() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut social = Social::default();
        social.reputation_local = 0.75;
        social.reputation_regional = 0.70;

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.90;
        *emotion.get_mut(EmotionType::Anger) = 0.80;
        *emotion.get_mut(EmotionType::Sadness) = 0.70;
        *emotion.get_mut(EmotionType::Disgust) = 0.60;
        *emotion.get_mut(EmotionType::Surprise) = 0.60;
        *emotion.get_mut(EmotionType::Joy) = 0.10;
        *emotion.get_mut(EmotionType::Trust) = 0.10;
        *emotion.get_mut(EmotionType::Anticipation) = 0.10;

        let stress = Stress {
            level: 0.90,
            reserve: 0.20,
            allostatic_load: 0.60,
            ..Stress::default()
        };

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.20);
        needs.set(NeedType::Safety, 0.10);
        needs.set(NeedType::Belonging, 0.20);

        let behavior = Behavior {
            current_action: ActionType::Idle,
            ..Behavior::default()
        };
        let mut values = Values::default();
        values.set(ValueType::Power, 0.60);

        let entity = world.spawn((social, emotion, stress, needs, behavior, values));
        let mut system = ReputationRuntimeSystem::new(38, 30);
        system.run(&mut world, &mut resources, 30);

        let updated = world
            .get::<&Social>(entity)
            .expect("updated social component should be queryable");
        assert!(updated.reputation_local < 0.75);
        assert!(updated.reputation_regional < 0.70);
        assert!((0.0..=1.0).contains(&updated.reputation_local));
        assert!((0.0..=1.0).contains(&updated.reputation_regional));
        assert!(
            updated
                .reputation_tags
                .iter()
                .any(|tag| tag == "suspect" || tag == "outcast"),
            "negative reputation tier tag should be assigned"
        );
    }

    #[test]
    fn social_event_runtime_system_updates_edges_and_social_capital() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut social = Social::default();
        social.reputation_local = 0.55;
        social.reputation_regional = 0.50;
        let mut edge = sim_core::components::RelationshipEdge::new(EntityId(2));
        edge.affinity = 35.0;
        edge.trust = 0.35;
        edge.familiarity = 0.20;
        edge.relation_type = RelationType::Friend;
        edge.is_bridge = true;
        social.edges.push(edge);

        let personality = Personality::default();
        let behavior = Behavior {
            current_action: ActionType::Socialize,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.80);
        let stress = Stress {
            level: 0.20,
            ..Stress::default()
        };

        let entity = world.spawn((social, personality, behavior, needs, stress));
        let mut system = SocialEventRuntimeSystem::new(37, 30);
        system.run(&mut world, &mut resources, 90);

        let updated = world
            .get::<&Social>(entity)
            .expect("updated social component should be queryable");
        assert_eq!(updated.edges.len(), 1);
        assert!(updated.edges[0].affinity > 35.0);
        assert!(updated.edges[0].trust >= 0.35);
        assert!(updated.edges[0].familiarity > 0.20);
        assert_eq!(updated.edges[0].last_interaction_tick, 90);
        assert!((0.0..=1.0).contains(&updated.social_capital));
    }

    #[test]
    fn morale_runtime_system_updates_behavior_and_upper_needs() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.30);
        needs.set(NeedType::Safety, 0.25);
        needs.set(NeedType::Belonging, 0.35);
        needs.set(NeedType::Autonomy, 0.40);
        needs.set(NeedType::Meaning, 0.55);
        needs.set(NeedType::Transcendence, 0.50);
        needs.energy = 0.30;

        let behavior = Behavior {
            job_satisfaction: 0.65,
            occupation_satisfaction: 0.62,
            ..Behavior::default()
        };

        let mut emotion = Emotion::default();
        *emotion.get_mut(EmotionType::Fear) = 0.70;
        *emotion.get_mut(EmotionType::Anger) = 0.60;
        *emotion.get_mut(EmotionType::Sadness) = 0.65;
        *emotion.get_mut(EmotionType::Joy) = 0.20;
        *emotion.get_mut(EmotionType::Trust) = 0.25;
        *emotion.get_mut(EmotionType::Anticipation) = 0.20;

        let stress = Stress {
            level: 0.75,
            ..Stress::default()
        };
        let personality = Personality::default();
        let social = Social::default();

        let entity = world.spawn((needs, behavior, emotion, stress, personality, social));
        let mut system = MoraleRuntimeSystem::new(40, 5);
        system.run(&mut world, &mut resources, 5);

        let updated_behavior = world
            .get::<&Behavior>(entity)
            .expect("updated behavior should be queryable");
        let updated_needs = world
            .get::<&Needs>(entity)
            .expect("updated needs should be queryable");
        assert!(updated_behavior.job_satisfaction < 0.65);
        assert!(updated_behavior.occupation_satisfaction < 0.62);
        assert!((0.0..=1.0).contains(&updated_needs.get(NeedType::Meaning)));
        assert!((0.0..=1.0).contains(&updated_needs.get(NeedType::Transcendence)));
        assert!(
            (updated_needs.get(NeedType::Meaning) - 0.55).abs() > f64::EPSILON
                || (updated_needs.get(NeedType::Transcendence) - 0.50).abs() > f64::EPSILON
        );
    }

    #[test]
    fn economic_tendency_runtime_system_updates_tendencies_and_applies_male_risk_bias() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut personality = Personality::default();
        personality.axes[HexacoAxis::H as usize] = 0.75;
        personality.axes[HexacoAxis::E as usize] = 0.35;
        personality.axes[HexacoAxis::X as usize] = 0.70;
        personality.axes[HexacoAxis::A as usize] = 0.65;
        personality.axes[HexacoAxis::C as usize] = 0.40;
        personality.axes[HexacoAxis::O as usize] = 0.60;

        let mut values = Values::default();
        values.set(ValueType::SelfControl, 0.30);
        values.set(ValueType::Law, 0.20);
        values.set(ValueType::Commerce, 0.45);
        values.set(ValueType::Competition, 0.35);
        values.set(ValueType::MartialProwess, 0.10);
        values.set(ValueType::Sacrifice, 0.25);
        values.set(ValueType::Cooperation, 0.30);
        values.set(ValueType::Family, 0.40);
        values.set(ValueType::Power, 0.22);
        values.set(ValueType::Fairness, 0.50);

        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.72);

        let age = Age {
            years: 28.0,
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let economic = Economic {
            wealth: 0.86,
            ..Economic::default()
        };

        let male_identity = Identity {
            sex: Sex::Male,
            ..Identity::default()
        };
        let male = world.spawn((
            economic.clone(),
            personality.clone(),
            values.clone(),
            needs.clone(),
            age,
            male_identity,
        ));

        let female_identity = Identity {
            sex: Sex::Female,
            ..Identity::default()
        };
        let female = world.spawn((
            economic,
            personality.clone(),
            values.clone(),
            needs.clone(),
            Age {
                years: 28.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            female_identity,
        ));

        let mut system =
            EconomicTendencyRuntimeSystem::new(39, sim_core::config::ECON_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::ECON_TICK_INTERVAL,
        );

        let male_updated = world
            .get::<&Economic>(male)
            .expect("male economic component should be queryable");
        let expected_male = body::economic_tendencies_step(
            personality.axis(HexacoAxis::H) as f32,
            personality.axis(HexacoAxis::E) as f32,
            personality.axis(HexacoAxis::X) as f32,
            personality.axis(HexacoAxis::A) as f32,
            personality.axis(HexacoAxis::C) as f32,
            personality.axis(HexacoAxis::O) as f32,
            28.0,
            values.get(ValueType::SelfControl) as f32,
            values.get(ValueType::Law) as f32,
            values.get(ValueType::Commerce) as f32,
            values.get(ValueType::Competition) as f32,
            values.get(ValueType::MartialProwess) as f32,
            values.get(ValueType::Sacrifice) as f32,
            values.get(ValueType::Cooperation) as f32,
            values.get(ValueType::Family) as f32,
            values.get(ValueType::Power) as f32,
            values.get(ValueType::Fairness) as f32,
            needs.get(NeedType::Belonging) as f32,
            0.86,
            0.0,
            0.0,
            true,
            sim_core::config::ECON_WEALTH_GENEROSITY_PENALTY as f32,
        );
        assert!((male_updated.saving_tendency as f32 - expected_male[0]).abs() < 1e-6);
        assert!((male_updated.risk_appetite as f32 - expected_male[1]).abs() < 1e-6);
        assert!((male_updated.generosity as f32 - expected_male[2]).abs() < 1e-6);
        assert!((male_updated.materialism as f32 - expected_male[3]).abs() < 1e-6);
        drop(male_updated);

        let female_updated = world
            .get::<&Economic>(female)
            .expect("female economic component should be queryable");
        assert!(female_updated.risk_appetite < expected_male[1] as f64);
    }

    #[test]
    fn economic_tendency_runtime_system_skips_child_stage() {
        let mut world = World::new();
        let mut resources = make_resources();

        let economic = Economic {
            saving_tendency: 0.5,
            risk_appetite: 0.5,
            generosity: 0.5,
            materialism: 0.3,
            ..Economic::default()
        };
        let personality = Personality::default();
        let values = Values::default();
        let needs = Needs::default();
        let age = Age {
            years: 8.0,
            stage: GrowthStage::Child,
            ..Age::default()
        };
        let entity = world.spawn((economic, personality, values, needs, age));

        let mut system =
            EconomicTendencyRuntimeSystem::new(39, sim_core::config::ECON_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::ECON_TICK_INTERVAL,
        );

        let updated = world
            .get::<&Economic>(entity)
            .expect("economic component should be queryable");
        assert!((updated.saving_tendency as f32 - 0.5).abs() < 1e-6);
        assert!((updated.risk_appetite as f32 - 0.5).abs() < 1e-6);
        assert!((updated.generosity as f32 - 0.5).abs() < 1e-6);
        assert!((updated.materialism as f32 - 0.3).abs() < 1e-6);
    }

    #[test]
    fn intelligence_runtime_system_applies_nutrition_penalty_in_critical_window() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.0);
        let age = Age {
            ticks: 100,
            years: 0.02,
            stage: GrowthStage::Infant,
            alive: true,
        };
        let intelligence = Intelligence {
            values: [0.80; 8],
            g_factor: 0.50,
            ace_penalty: 0.0,
            nutrition_penalty: 0.0,
        };
        let entity = world.spawn((intelligence, age, needs, Skills::default()));

        let mut system = IntelligenceRuntimeSystem::new(18, 50);
        system.run(&mut world, &mut resources, 50);

        let updated = world
            .get::<&Intelligence>(entity)
            .expect("intelligence should be queryable");
        assert!(updated.nutrition_penalty > 0.0);
        assert!(updated.values[IntelligenceType::Logical as usize] < 0.80);
    }

    #[test]
    fn intelligence_runtime_system_applies_ace_penalty_and_fluid_decline() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            ticks: (60.0 * sim_core::config::TICKS_PER_YEAR as f32) as u64,
            years: 60.0,
            stage: GrowthStage::Adult,
            alive: true,
        };
        let identity = Identity {
            birth_tick: 0,
            ..Identity::default()
        };
        let memory = Memory {
            trauma_scars: vec![
                TraumaScar {
                    scar_id: "scar_a".to_string(),
                    acquired_tick: 1000,
                    severity: 0.6,
                    reactivation_count: 0,
                },
                TraumaScar {
                    scar_id: "scar_b".to_string(),
                    acquired_tick: 2000,
                    severity: 0.7,
                    reactivation_count: 0,
                },
                TraumaScar {
                    scar_id: "scar_c".to_string(),
                    acquired_tick: 3000,
                    severity: 0.8,
                    reactivation_count: 0,
                },
            ],
            ..Memory::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::O as usize] = 0.80;
        let intelligence = Intelligence {
            values: [0.90; 8],
            g_factor: 0.50,
            ace_penalty: 0.0,
            nutrition_penalty: 0.0,
        };
        let entity = world.spawn((intelligence, age, identity, memory, personality));

        let mut system = IntelligenceRuntimeSystem::new(18, 50);
        system.run(&mut world, &mut resources, 50);

        let updated = world
            .get::<&Intelligence>(entity)
            .expect("intelligence should be queryable");
        assert!(
            (updated.ace_penalty as f32 - sim_core::config::INTEL_ACE_PENALTY_MAJOR as f32).abs()
                < 1e-6
        );
        assert!(
            updated.values[IntelligenceType::Logical as usize]
                < updated.values[IntelligenceType::Linguistic as usize]
        );
        assert!(updated.g_factor > 0.50);
    }

    #[test]
    fn intelligence_system_cleans_dead_entity_baselines() {
        let mut world = World::new();
        let mut resources = make_resources();

        let intelligence = Intelligence {
            values: [0.50; 8],
            g_factor: 0.50,
            ace_penalty: 0.0,
            nutrition_penalty: 0.0,
        };
        let e1 = world.spawn((intelligence.clone(),));
        let _e2 = world.spawn((intelligence,));

        let mut sys = IntelligenceRuntimeSystem::new(18, 1);
        sys.run(&mut world, &mut resources, 1);

        assert_eq!(sys.baseline_count(), 2);

        world.despawn(e1).unwrap();
        sys.run(&mut world, &mut resources, 2);

        assert_eq!(sys.baseline_count(), 1);
    }

    #[test]
    fn memory_runtime_system_decays_evicts_and_promotes_entries() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut memory = Memory::default();
        for idx in 0..101 {
            memory.short_term.push_back(MemoryEntry {
                event_type: format!("event_{idx}"),
                target_id: None,
                tick: 9_000 + idx as u64,
                intensity: 0.55,
                current_intensity: 0.55,
                is_permanent: false,
            });
        }
        memory.short_term.push_back(MemoryEntry {
            event_type: "proposal".to_string(),
            target_id: Some(7),
            tick: 9_999,
            intensity: 0.80,
            current_intensity: 0.80,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "forgettable".to_string(),
            target_id: None,
            tick: 10_000,
            intensity: 0.05,
            current_intensity: 0.02,
            is_permanent: false,
        });

        let entity = world.spawn((memory,));
        let mut system =
            MemoryRuntimeSystem::new(18, sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS * 3,
        );

        let updated = world
            .get::<&Memory>(entity)
            .expect("memory component should be queryable");
        assert_eq!(
            updated.short_term.len(),
            sim_core::config::MEMORY_WORKING_MAX
        );
        assert!(updated
            .short_term
            .iter()
            .all(|entry| entry.event_type != "forgettable"));
        assert!(updated
            .short_term
            .iter()
            .any(|entry| entry.event_type == "proposal" && entry.is_permanent));
        assert!(updated
            .permanent
            .iter()
            .any(|entry| entry.event_type == "proposal"
                && entry.tick == 9_999
                && entry.is_permanent));
    }

    #[test]
    fn memory_runtime_system_compresses_old_entries_into_summary() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut memory = Memory::default();
        memory.short_term.push_back(MemoryEntry {
            event_type: "deep_talk".to_string(),
            target_id: Some(42),
            tick: 10,
            intensity: 0.90,
            current_intensity: 0.90,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "deep_talk".to_string(),
            target_id: Some(42),
            tick: 20,
            intensity: 0.90,
            current_intensity: 0.90,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "deep_talk".to_string(),
            target_id: Some(42),
            tick: 30,
            intensity: 0.90,
            current_intensity: 0.90,
            is_permanent: false,
        });
        memory.short_term.push_back(MemoryEntry {
            event_type: "casual_talk".to_string(),
            target_id: Some(7),
            tick: 9_000,
            intensity: 0.10,
            current_intensity: 0.10,
            is_permanent: false,
        });

        let entity = world.spawn((memory,));
        let mut system =
            MemoryRuntimeSystem::new(18, sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS);
        let tick = sim_core::config::MEMORY_COMPRESS_INTERVAL_TICKS * 3;
        system.run(&mut world, &mut resources, tick);

        let updated = world
            .get::<&Memory>(entity)
            .expect("memory component should be queryable");
        assert_eq!(updated.short_term.len(), 2);
        assert_eq!(updated.last_compression_tick, tick);
        assert!(updated
            .short_term
            .iter()
            .any(|entry| entry.event_type == "casual_talk"));

        let summary = updated
            .short_term
            .iter()
            .find(|entry| entry.event_type == "deep_talk_summary")
            .expect("deep_talk summary entry should exist");
        assert_eq!(summary.target_id, Some(42));
        assert_eq!(summary.tick, 10);

        let decayed = body::memory_decay_intensity(
            0.90,
            sim_core::config::memory_decay_rate(0.90) as f32,
            1.0,
        );
        let expected_summary = body::memory_summary_intensity(decayed, 0.70);
        assert!((summary.current_intensity as f32 - expected_summary).abs() < 1e-5);
    }

    #[test]
    fn value_runtime_system_updates_value_axes_with_context() {
        let mut world = World::new();
        let mut resources = make_resources();

        let values = Values::default();
        let age = Age {
            years: 12.0,
            ..Age::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::H as usize] = 0.80;
        personality.axes[HexacoAxis::A as usize] = 0.75;
        personality.axes[HexacoAxis::X as usize] = 0.65;

        let mut needs = Needs::default();
        needs.set(NeedType::Safety, 0.85);
        needs.set(NeedType::Belonging, 0.90);
        let stress = Stress {
            level: 0.10,
            ..Stress::default()
        };
        let social = Social {
            social_capital: 0.70,
            ..Social::default()
        };

        let entity = world.spawn((values, age, personality, needs, stress, social));
        let mut system = ValueRuntimeSystem::new(55, 200);
        system.run(&mut world, &mut resources, 200);

        let updated = world
            .get::<&Values>(entity)
            .expect("updated values component should be queryable");
        assert!(updated.get(ValueType::Cooperation) > 0.0);
        assert!(updated.get(ValueType::Fairness) > 0.0);
        assert!(updated.get(ValueType::Family) > 0.0);
        assert!(updated.get(ValueType::Law) > 0.0);
    }

    #[test]
    fn job_satisfaction_runtime_system_updates_behavior_scores() {
        let mut world = World::new();
        let mut resources = make_resources();

        let behavior = Behavior {
            job: "builder".to_string(),
            job_satisfaction: 0.30,
            occupation_satisfaction: 0.35,
            ..Behavior::default()
        };
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::A as usize] = 0.75;
        personality.axes[HexacoAxis::C as usize] = 0.80;
        let mut values = Values::default();
        values.set(ValueType::HardWork, 0.70);
        values.set(ValueType::Cooperation, 0.60);
        values.set(ValueType::Fairness, 0.60);

        let mut needs = Needs::default();
        needs.set(NeedType::Autonomy, 0.65);
        needs.set(NeedType::Competence, 0.70);
        needs.set(NeedType::Meaning, 0.60);

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_CONSTRUCTION".to_string(),
            SkillEntry { level: 8, xp: 0.0 },
        );
        let age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };

        let entity = world.spawn((behavior, personality, values, needs, skills, age));
        let mut system = JobSatisfactionRuntimeSystem::new(40, 120);
        system.run(&mut world, &mut resources, 120);

        let updated = world
            .get::<&Behavior>(entity)
            .expect("updated behavior should be queryable");
        assert!(updated.job_satisfaction > 0.30);
        assert!(updated.occupation_satisfaction > 0.35);
        assert!((0.0..=1.0).contains(&updated.job_satisfaction));
    }

    #[test]
    fn job_assignment_runtime_system_assigns_jobs_with_age_rules() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).resources[0].amount = 300.0;

        let adult_age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let adult_behavior = Behavior {
            job: "none".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let adult = world.spawn((adult_age, adult_behavior));

        let teen_age = Age {
            stage: GrowthStage::Teen,
            ..Age::default()
        };
        let teen_behavior = Behavior {
            job: "none".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let teen = world.spawn((teen_age, teen_behavior));

        let child_age = Age {
            stage: GrowthStage::Child,
            ..Age::default()
        };
        let child_behavior = Behavior {
            job: "none".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let child = world.spawn((child_age, child_behavior));

        let infant_age = Age {
            stage: GrowthStage::Infant,
            ..Age::default()
        };
        let infant_behavior = Behavior {
            job: "builder".to_string(),
            occupation: "none".to_string(),
            ..Behavior::default()
        };
        let infant = world.spawn((infant_age, infant_behavior));

        let mut system =
            JobAssignmentRuntimeSystem::new(8, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL,
        );

        let adult_updated = world
            .get::<&Behavior>(adult)
            .expect("adult behavior should be queryable");
        assert_ne!(adult_updated.job, "none");
        assert!(matches!(
            adult_updated.job.as_str(),
            "gatherer" | "lumberjack" | "builder" | "miner"
        ));
        drop(adult_updated);

        let teen_updated = world
            .get::<&Behavior>(teen)
            .expect("teen behavior should be queryable");
        assert_eq!(teen_updated.job, "gatherer");
        drop(teen_updated);

        let child_updated = world
            .get::<&Behavior>(child)
            .expect("child behavior should be queryable");
        assert_eq!(child_updated.job, "gatherer");
        drop(child_updated);

        let infant_updated = world
            .get::<&Behavior>(infant)
            .expect("infant behavior should be queryable");
        assert_eq!(infant_updated.job, "none");
    }

    #[test]
    fn job_assignment_runtime_system_rebalances_one_idle_surplus_job() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.map.get_mut(0, 0).resources[0].amount = 300.0;

        for _ in 0..5 {
            let age = Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            };
            let behavior = Behavior {
                job: "builder".to_string(),
                occupation: "none".to_string(),
                current_action: ActionType::Idle,
                ..Behavior::default()
            };
            world.spawn((age, behavior));
        }
        for job in ["gatherer", "lumberjack", "miner"] {
            let age = Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            };
            let behavior = Behavior {
                job: job.to_string(),
                occupation: "none".to_string(),
                current_action: ActionType::Idle,
                ..Behavior::default()
            };
            world.spawn((age, behavior));
        }

        let count_jobs = |world_ref: &World, job_name: &str| -> usize {
            let mut total = 0_usize;
            let mut query = world_ref.query::<&Behavior>();
            for (_, behavior) in &mut query {
                if behavior.job == job_name {
                    total += 1;
                }
            }
            total
        };

        let before_builder = count_jobs(&world, "builder");
        let before_gatherer = count_jobs(&world, "gatherer");
        assert_eq!(before_builder, 5);
        assert_eq!(before_gatherer, 1);

        let mut system =
            JobAssignmentRuntimeSystem::new(8, sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL,
        );

        let after_builder = count_jobs(&world, "builder");
        let after_gatherer = count_jobs(&world, "gatherer");
        assert_eq!(after_builder, 4);
        assert_eq!(after_gatherer, 2);
    }

    #[test]
    fn network_runtime_system_recomputes_social_capital_from_edges() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut social = Social::default();
        social.reputation_local = 0.80;
        social.reputation_regional = 0.60;

        let mut strong_edge = sim_core::components::RelationshipEdge::new(EntityId(2));
        strong_edge.affinity = 72.0;
        strong_edge.relation_type = RelationType::CloseFriend;
        strong_edge.is_bridge = false;
        social.edges.push(strong_edge);

        let mut bridge_weak_edge = sim_core::components::RelationshipEdge::new(EntityId(3));
        bridge_weak_edge.affinity = 18.0;
        bridge_weak_edge.relation_type = RelationType::Friend;
        bridge_weak_edge.is_bridge = true;
        social.edges.push(bridge_weak_edge);

        let mut ignored_edge = sim_core::components::RelationshipEdge::new(EntityId(4));
        ignored_edge.affinity = 1.0;
        ignored_edge.relation_type = RelationType::Stranger;
        ignored_edge.is_bridge = false;
        social.edges.push(ignored_edge);

        let entity = world.spawn((social,));
        let mut system = NetworkRuntimeSystem::new(58, sim_core::config::REVOLUTION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::REVOLUTION_TICK_INTERVAL,
        );

        let updated = world
            .get::<&Social>(entity)
            .expect("updated social component should be queryable");
        let expected = body::network_social_capital_norm(
            1.0,
            0.0,
            0.5,
            0.70,
            sim_core::config::NETWORK_SOCIAL_CAP_STRONG_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_WEAK_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_BRIDGE_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_REP_W as f32,
            sim_core::config::NETWORK_SOCIAL_CAP_NORM_DIV as f32,
        );
        assert!((updated.social_capital as f32 - expected).abs() < 1e-6);
        assert!((0.0..=1.0).contains(&updated.social_capital));
    }

    #[test]
    fn occupation_runtime_system_assigns_and_switches_by_skill_margin() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut adult_skills = Skills::default();
        adult_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 20, xp: 0.0 },
        );
        adult_skills.entries.insert(
            "SKILL_MINING".to_string(),
            SkillEntry { level: 45, xp: 0.0 },
        );
        let adult_age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let adult_behavior = Behavior {
            occupation: "foraging".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let adult_entity = world.spawn((adult_age, adult_skills, adult_behavior));

        let mut teen_skills = Skills::default();
        teen_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 4, xp: 0.0 },
        );
        let teen_age = Age {
            stage: GrowthStage::Teen,
            ..Age::default()
        };
        let teen_behavior = Behavior {
            occupation: "laborer".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let teen_entity = world.spawn((teen_age, teen_skills, teen_behavior));

        let mut low_adult_skills = Skills::default();
        low_adult_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 3, xp: 0.0 },
        );
        let low_adult_age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let low_adult_behavior = Behavior {
            occupation: "foraging".to_string(),
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let low_adult_entity = world.spawn((low_adult_age, low_adult_skills, low_adult_behavior));

        let mut system =
            OccupationRuntimeSystem::new(36, sim_core::config::OCCUPATION_EVAL_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::OCCUPATION_EVAL_INTERVAL,
        );

        let adult_updated = world
            .get::<&Behavior>(adult_entity)
            .expect("adult behavior should be queryable");
        assert_eq!(adult_updated.occupation, "mining");
        assert_eq!(adult_updated.job, "miner");

        let teen_updated = world
            .get::<&Behavior>(teen_entity)
            .expect("teen behavior should be queryable");
        assert_eq!(teen_updated.occupation, "none");
        assert_eq!(teen_updated.job, "gatherer");

        let low_adult_updated = world
            .get::<&Behavior>(low_adult_entity)
            .expect("low-skill adult behavior should be queryable");
        assert_eq!(low_adult_updated.occupation, "laborer");
        assert_eq!(low_adult_updated.job, "gatherer");
    }

    #[test]
    fn building_effect_runtime_system_applies_campfire_social_with_influence_runtime() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.calendar.tick = 10; // hour=20 => night boost path
        resources.buildings.insert(
            BuildingId(1),
            Building {
                id: BuildingId(1),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 0,
                y: 0,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut needs = Needs::default();
        needs.set(NeedType::Belonging, 0.40);
        needs.set(NeedType::Warmth, 0.30);
        let entity = world.spawn((Position::new(1, 1), needs));

        let mut influence =
            InfluenceRuntimeSystem::new(sim_core::config::INFLUENCE_SYSTEM_PRIORITY, 1);
        let mut system =
            BuildingEffectRuntimeSystem::new(15, sim_core::config::BUILDING_EFFECT_TICK_INTERVAL);
        influence.run(&mut world, &mut resources, 1);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::BUILDING_EFFECT_TICK_INTERVAL,
        );

        let updated = world
            .get::<&Needs>(entity)
            .expect("needs should be queryable");
        assert!((updated.get(NeedType::Belonging) as f32 - 0.42).abs() < 1e-6);
        assert!((updated.get(NeedType::Warmth) as f32 - 0.30).abs() < 1e-6);
        drop(updated);

        resources.influence_grid.tick_update();
        assert!(resources.influence_grid.active_emitter_count() >= 1);
        assert!(resources.influence_grid.sample(0, 0, ChannelId::Warmth) > 0.0);
    }

    #[test]
    fn campfire_warmth_vertical_slice_uses_shelter_walls_for_inside_open_and_blocked_agents() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(12, 12, 123);
        let resources = SimResources::new(calendar, map, 999);
        let mut engine = SimEngine::new(resources);
        engine.register(NeedsRuntimeSystem::new(10, 1));
        engine.register(InfluenceRuntimeSystem::new(
            sim_core::config::INFLUENCE_SYSTEM_PRIORITY,
            1,
        ));
        engine.register(BuildingEffectRuntimeSystem::new(15, 1));
        engine.resources_mut().calendar.tick = 10; // night, so belonging boost still uses night path
        engine.resources_mut().buildings.insert(
            BuildingId(50),
            Building {
                id: BuildingId(50),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );
        engine.resources_mut().buildings.insert(
            BuildingId(51),
            Building {
                id: BuildingId(51),
                building_type: "shelter".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut inside_needs = Needs::default();
        inside_needs.set(NeedType::Warmth, 0.20);
        let inside = engine
            .world_mut()
            .spawn((Position::new(5, 5), inside_needs));

        let mut open_needs = Needs::default();
        open_needs.set(NeedType::Warmth, 0.20);
        let open = engine.world_mut().spawn((Position::new(5, 7), open_needs));

        let mut blocked_needs = Needs::default();
        blocked_needs.set(NeedType::Warmth, 0.20);
        let blocked = engine
            .world_mut()
            .spawn((Position::new(7, 5), blocked_needs));

        engine.tick();

        let inside_after_first = engine
            .world()
            .get::<&Needs>(inside)
            .expect("inside needs after first tick");
        let open_after_first = engine
            .world()
            .get::<&Needs>(open)
            .expect("open needs after first tick");
        let blocked_after_first = engine
            .world()
            .get::<&Needs>(blocked)
            .expect("blocked needs after first tick");
        let expected_after_first = 0.20_f64;
        assert!((inside_after_first.get(NeedType::Warmth) - expected_after_first).abs() < 1e-6);
        assert!((open_after_first.get(NeedType::Warmth) - expected_after_first).abs() < 1e-6);
        assert!((blocked_after_first.get(NeedType::Warmth) - expected_after_first).abs() < 1e-6);
        drop(inside_after_first);
        drop(open_after_first);
        drop(blocked_after_first);

        assert!(
            (engine.resources().influence_grid.wall_blocking_at(6, 5)
                - sim_core::config::BUILDING_SHELTER_WALL_BLOCK)
                .abs()
                < 1e-6
        );
        assert_eq!(
            engine.resources().influence_grid.wall_blocking_at(5, 6),
            0.0
        );

        let inside_signal = engine
            .resources()
            .influence_grid
            .sample(5, 5, ChannelId::Warmth);
        let open_signal = engine
            .resources()
            .influence_grid
            .sample(5, 7, ChannelId::Warmth);
        let blocked_signal = engine
            .resources()
            .influence_grid
            .sample(7, 5, ChannelId::Warmth);
        assert!(inside_signal > open_signal);
        assert!(open_signal > blocked_signal);

        engine.tick();

        let inside_after_second = engine
            .world()
            .get::<&Needs>(inside)
            .expect("inside needs after second tick");
        let open_after_second = engine
            .world()
            .get::<&Needs>(open)
            .expect("open needs after second tick");
        let blocked_after_second = engine
            .world()
            .get::<&Needs>(blocked)
            .expect("blocked needs after second tick");

        assert!(
            inside_after_second.get(NeedType::Warmth) > blocked_after_second.get(NeedType::Warmth)
        );
        assert!(
            open_after_second.get(NeedType::Warmth) > blocked_after_second.get(NeedType::Warmth)
        );
    }

    #[test]
    fn building_effect_runtime_system_applies_shelter_energy_and_safety() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources.buildings.insert(
            BuildingId(2),
            Building {
                id: BuildingId(2),
                building_type: "shelter".to_string(),
                settlement_id: SettlementId(1),
                x: 2,
                y: 0,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut needs = Needs::default();
        needs.energy = 0.25;
        needs.set(NeedType::Warmth, 0.20);
        needs.set(NeedType::Safety, 0.10);
        let entity = world.spawn((Position::new(2, 0), needs));

        let mut system =
            BuildingEffectRuntimeSystem::new(15, sim_core::config::BUILDING_EFFECT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::BUILDING_EFFECT_TICK_INTERVAL,
        );

        let updated = world
            .get::<&Needs>(entity)
            .expect("needs should be queryable");
        assert!(
            (updated.energy as f32
                - (0.25 + sim_core::config::BUILDING_SHELTER_ENERGY_RESTORE as f32))
                .abs()
                < 1e-6
        );
        assert!((updated.get(NeedType::Warmth) as f32 - 0.20).abs() < 1e-6);
        assert!(
            (updated.get(NeedType::Safety) as f32
                - (0.10 + sim_core::config::SAFETY_SHELTER_RESTORE as f32))
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn movement_runtime_system_moves_toward_target_on_passable_tile() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let behavior = Behavior {
            current_action: ActionType::Explore,
            action_target_x: Some(0),
            action_target_y: Some(0),
            action_timer: 4,
            ..Behavior::default()
        };
        let mut position = Position::new(1, 0);
        position.vel_x = -1.0;
        let entity = world.spawn((position, behavior, age));

        let mut system = MovementRuntimeSystem::new(30, sim_core::config::MOVEMENT_TICK_INTERVAL);
        system.run(&mut world, &mut resources, 10);

        let pos = world
            .get::<&Position>(entity)
            .expect("position should exist after movement");
        let behavior_after = world
            .get::<&Behavior>(entity)
            .expect("behavior should exist after movement");
        assert_eq!((pos.x, pos.y), (0.0, 0.0));
        assert_eq!(behavior_after.action_timer, 3);
    }

    #[test]
    fn movement_runtime_system_completes_action_and_applies_drink_restore() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let behavior = Behavior {
            current_action: ActionType::Drink,
            action_target_x: Some(4),
            action_target_y: Some(4),
            action_timer: 1,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Thirst, 0.10);
        let entity = world.spawn((Position::new(1, 0), behavior, needs, age));

        let mut system = MovementRuntimeSystem::new(30, sim_core::config::MOVEMENT_TICK_INTERVAL);
        system.run(&mut world, &mut resources, 20);

        let behavior_after = world
            .get::<&Behavior>(entity)
            .expect("behavior should exist after action completion");
        let needs_after = world
            .get::<&Needs>(entity)
            .expect("needs should exist after action completion");
        assert_eq!(behavior_after.current_action, ActionType::Idle);
        assert!(behavior_after.action_target_x.is_none());
        assert!(behavior_after.action_target_y.is_none());
        assert!(
            (needs_after.get(NeedType::Thirst) as f32 - 0.45).abs() < 1e-6,
            "thirst should increase by THIRST_DRINK_RESTORE"
        );
    }

    #[test]
    fn movement_runtime_system_eat_consumes_food_item_from_inventory() {
        let mut world = World::new();
        let mut resources = make_resources();

        let food_id = resources.item_store.allocate_id();
        resources.item_store.insert(ItemInstance {
            id: food_id,
            template_id: "berries".to_string(),
            material_id: "birch".to_string(),
            derived_stats: ItemDerivedStats::default(),
            current_durability: 100.0,
            quality: 0.5,
            owner: ItemOwner::Agent(EntityId(1)),
            stack_count: 1,
            created_tick: 0,
            creator_id: Some(EntityId(1)),
            equipped_slot: None,
        });

        let age = Age {
            stage: GrowthStage::Adult,
            ..Age::default()
        };
        let behavior = Behavior {
            current_action: ActionType::Eat,
            action_target_x: Some(1),
            action_target_y: Some(0),
            action_timer: 1,
            ..Behavior::default()
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.10);
        let mut inventory = Inventory::new();
        inventory.add(food_id);
        let entity = world.spawn((Position::new(1, 0), behavior, needs, age, inventory));

        let mut system = MovementRuntimeSystem::new(30, sim_core::config::MOVEMENT_TICK_INTERVAL);
        system.run(&mut world, &mut resources, 21);

        let behavior_after = world
            .get::<&Behavior>(entity)
            .expect("behavior should exist after eat completion");
        let needs_after = world
            .get::<&Needs>(entity)
            .expect("needs should exist after eat completion");
        let inventory_after = world
            .get::<&Inventory>(entity)
            .expect("inventory should exist after eat completion");

        assert_eq!(behavior_after.current_action, ActionType::Idle);
        assert!(!inventory_after.contains(food_id));
        assert!(resources.item_store.get(food_id).is_none());
        assert!(
            needs_after.get(NeedType::Hunger) > 0.10,
            "eat should still restore hunger after consuming an item"
        );
    }

    #[test]
    fn leader_runtime_system_elects_best_candidate_and_sets_countdown() {
        let mut world = World::new();
        let mut resources = make_resources();

        let settlement_id = SettlementId(17);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "delta".to_string(), 0, 0, 0),
        );

        let mut low_personality = Personality::default();
        low_personality.axes[HexacoAxis::X as usize] = 0.20;
        low_personality.axes[HexacoAxis::O as usize] = 0.20;
        let low_social = Social {
            reputation_local: 0.20,
            reputation_regional: 0.20,
            social_capital: 0.20,
            ..Social::default()
        };
        let low_identity = Identity {
            settlement_id: Some(settlement_id),
            ..Identity::default()
        };
        world.spawn((
            Age {
                alive: true,
                years: 24.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            low_identity,
            low_personality,
            low_social,
            Intelligence {
                g_factor: 0.20,
                ..Intelligence::default()
            },
            BodyComponent {
                str_realized: 900,
                ..BodyComponent::default()
            },
        ));

        let mut best_personality = Personality::default();
        best_personality.axes[HexacoAxis::X as usize] = 0.90;
        best_personality.axes[HexacoAxis::O as usize] = 0.90;
        let best_social = Social {
            reputation_local: 0.90,
            reputation_regional: 0.80,
            social_capital: 0.90,
            ..Social::default()
        };
        let best_identity = Identity {
            settlement_id: Some(settlement_id),
            ..Identity::default()
        };
        let best_entity = world.spawn((
            Age {
                alive: true,
                years: 42.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            best_identity,
            best_personality,
            best_social,
            Intelligence {
                g_factor: 0.95,
                ..Intelligence::default()
            },
            BodyComponent {
                str_realized: 1500,
                ..BodyComponent::default()
            },
        ));

        let mut mid_personality = Personality::default();
        mid_personality.axes[HexacoAxis::X as usize] = 0.55;
        mid_personality.axes[HexacoAxis::O as usize] = 0.45;
        let mid_social = Social {
            reputation_local: 0.40,
            reputation_regional: 0.40,
            social_capital: 0.45,
            ..Social::default()
        };
        let mid_identity = Identity {
            settlement_id: Some(settlement_id),
            ..Identity::default()
        };
        world.spawn((
            Age {
                alive: true,
                years: 30.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            mid_identity,
            mid_personality,
            mid_social,
            Intelligence {
                g_factor: 0.45,
                ..Intelligence::default()
            },
            BodyComponent {
                str_realized: 1100,
                ..BodyComponent::default()
            },
        ));

        let mut system = LeaderRuntimeSystem::new(52, sim_core::config::LEADER_CHECK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::LEADER_CHECK_INTERVAL,
        );

        let settlement = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should be queryable");
        assert_eq!(
            settlement.leader_id,
            Some(EntityId(best_entity.id() as u64)),
            "highest-score candidate should be elected leader"
        );
        assert_eq!(
            settlement.leader_reelection_countdown,
            sim_core::config::LEADER_REELECTION_INTERVAL as u32
        );
    }

    #[test]
    fn leader_runtime_system_decrements_countdown_without_re_election() {
        let mut world = World::new();
        let mut resources = make_resources();

        let settlement_id = SettlementId(22);
        let mut settlement = sim_core::Settlement::new(settlement_id, "omega".to_string(), 0, 0, 0);
        settlement.leader_reelection_countdown = 50;
        resources.settlements.insert(settlement_id, settlement);

        let leader_entity = world.spawn((
            Age {
                alive: true,
                years: 40.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Personality::default(),
            Social::default(),
        ));

        world.spawn((
            Age {
                alive: true,
                years: 37.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Personality::default(),
            Social::default(),
        ));
        world.spawn((
            Age {
                alive: true,
                years: 36.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Personality::default(),
            Social::default(),
        ));

        {
            let settlement_mut = resources
                .settlements
                .get_mut(&settlement_id)
                .expect("settlement should be mutable");
            settlement_mut.leader_id = Some(EntityId(leader_entity.id() as u64));
        }

        let mut system = LeaderRuntimeSystem::new(52, 12);
        system.run(&mut world, &mut resources, 12);

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should be queryable");
        assert_eq!(
            settlement_after.leader_id,
            Some(EntityId(leader_entity.id() as u64)),
            "leader should remain unchanged while countdown is still positive"
        );
        assert_eq!(settlement_after.leader_reelection_countdown, 38);
    }

    #[test]
    fn leader_runtime_system_replaces_invalid_leader() {
        let mut world = World::new();
        let mut resources = make_resources();

        let settlement_id = SettlementId(25);
        let mut settlement = sim_core::Settlement::new(settlement_id, "zeta".to_string(), 0, 0, 0);
        settlement.leader_id = Some(EntityId(9999));
        settlement.leader_reelection_countdown = 100;
        resources.settlements.insert(settlement_id, settlement);

        let candidate_entity = world.spawn((
            Age {
                alive: true,
                years: 45.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Personality::default(),
            Social {
                reputation_local: 0.70,
                reputation_regional: 0.60,
                social_capital: 0.80,
                ..Social::default()
            },
            Intelligence {
                g_factor: 0.80,
                ..Intelligence::default()
            },
        ));

        world.spawn((
            Age {
                alive: true,
                years: 39.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Personality::default(),
            Social::default(),
            Intelligence::default(),
        ));

        world.spawn((
            Age {
                alive: true,
                years: 35.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Personality::default(),
            Social::default(),
            Intelligence::default(),
        ));

        let mut system = LeaderRuntimeSystem::new(52, sim_core::config::LEADER_CHECK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::LEADER_CHECK_INTERVAL,
        );

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should be queryable");
        assert_eq!(
            settlement_after.leader_id,
            Some(EntityId(candidate_entity.id() as u64)),
            "invalid incumbent should be replaced by a valid candidate"
        );
        assert_eq!(
            settlement_after.leader_reelection_countdown,
            sim_core::config::LEADER_REELECTION_INTERVAL as u32
        );
    }

    #[test]
    fn title_runtime_system_grants_elder_master_and_chief_titles() {
        let mut world = World::new();
        let mut resources = make_resources();

        let settlement_id = SettlementId(31);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "title-a".to_string(), 0, 0, 0),
        );

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 85, xp: 0.0 },
        );
        let social = Social {
            titles: vec!["TITLE_EXPERT_FORAGING".to_string()],
            ..Social::default()
        };
        let entity = world.spawn((
            Age {
                alive: true,
                years: 61.0,
                stage: GrowthStage::Elder,
                ..Age::default()
            },
            skills,
            social,
        ));

        {
            let settlement = resources
                .settlements
                .get_mut(&settlement_id)
                .expect("settlement should be mutable");
            settlement.leader_id = Some(EntityId(entity.id() as u64));
        }

        let mut system = TitleRuntimeSystem::new(37, sim_core::config::TITLE_EVAL_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TITLE_EVAL_INTERVAL,
        );

        let updated = world
            .get::<&Social>(entity)
            .expect("social should be queryable");
        assert!(updated.has_title("TITLE_ELDER"));
        assert!(updated.has_title("TITLE_MASTER_FORAGING"));
        assert!(!updated.has_title("TITLE_EXPERT_FORAGING"));
        assert!(updated.has_title("TITLE_CHIEF"));
    }

    #[test]
    fn title_runtime_system_revokes_titles_and_marks_former_chief() {
        let mut world = World::new();
        let mut resources = make_resources();

        let settlement_id = SettlementId(32);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "title-b".to_string(), 0, 0, 0),
        );

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 20, xp: 0.0 },
        );
        let social = Social {
            titles: vec![
                "TITLE_ELDER".to_string(),
                "TITLE_CHIEF".to_string(),
                "TITLE_MASTER_FORAGING".to_string(),
                "TITLE_EXPERT_FORAGING".to_string(),
            ],
            ..Social::default()
        };
        let entity = world.spawn((
            Age {
                alive: true,
                years: 25.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            skills,
            social,
        ));

        let mut system = TitleRuntimeSystem::new(37, sim_core::config::TITLE_EVAL_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TITLE_EVAL_INTERVAL,
        );

        let updated = world
            .get::<&Social>(entity)
            .expect("social should be queryable");
        assert!(!updated.has_title("TITLE_ELDER"));
        assert!(!updated.has_title("TITLE_CHIEF"));
        assert!(updated.has_title("TITLE_FORMER_CHIEF"));
        assert!(!updated.has_title("TITLE_MASTER_FORAGING"));
        assert!(!updated.has_title("TITLE_EXPERT_FORAGING"));
    }

    #[test]
    fn stratification_monitor_runtime_system_updates_settlement_and_class_state() {
        let mut world = World::new();
        let mut resources = make_resources();

        let settlement_id = SettlementId(41);
        let mut settlement =
            sim_core::Settlement::new(settlement_id, "strat-a".to_string(), 0, 0, 0);
        settlement.stockpile_food = 120.0;
        resources.settlements.insert(settlement_id, settlement);

        let leader_entity = world.spawn((
            Age {
                alive: true,
                years: 45.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Social {
                reputation_local: 0.80,
                reputation_regional: 0.75,
                social_capital: 0.70,
                ..Social::default()
            },
            Economic {
                wealth: 0.90,
                ..Economic::default()
            },
        ));
        world.spawn((
            Age {
                alive: true,
                years: 31.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Social {
                reputation_local: 0.55,
                reputation_regional: 0.50,
                social_capital: 0.45,
                ..Social::default()
            },
            Economic {
                wealth: 0.45,
                ..Economic::default()
            },
        ));
        world.spawn((
            Age {
                alive: true,
                years: 24.0,
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
            Social {
                reputation_local: 0.20,
                reputation_regional: 0.25,
                social_capital: 0.20,
                ..Social::default()
            },
            Economic {
                wealth: 0.10,
                ..Economic::default()
            },
        ));

        {
            let settlement_mut = resources
                .settlements
                .get_mut(&settlement_id)
                .expect("settlement should be mutable");
            settlement_mut.leader_id = Some(EntityId(leader_entity.id() as u64));
        }

        let mut system =
            StratificationMonitorRuntimeSystem::new(90, sim_core::config::STRAT_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::STRAT_TICK_INTERVAL,
        );

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should be queryable");
        assert!((0.0..=1.0).contains(&settlement_after.gini_coefficient));
        assert!((0.0..=1.0).contains(&settlement_after.leveling_effectiveness));
        assert!(
            settlement_after.stratification_phase == "egalitarian"
                || settlement_after.stratification_phase == "transitional"
                || settlement_after.stratification_phase == "stratified"
        );

        let leader_social = world
            .get::<&Social>(leader_entity)
            .expect("leader social should be queryable");
        assert_eq!(leader_social.social_class, SocialClass::Ruler);
        drop(leader_social);

        let leader_economic = world
            .get::<&Economic>(leader_entity)
            .expect("leader economic should be queryable");
        assert!((0.0..=1.0).contains(&leader_economic.wealth_norm));
    }

    #[test]
    fn tension_runtime_system_updates_pair_tension_from_scarcity_and_decay() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut alpha = sim_core::Settlement::new(SettlementId(51), "alpha".to_string(), 0, 0, 0);
        alpha.stockpile_food = 0.0;
        alpha.members = vec![EntityId(1), EntityId(2), EntityId(3), EntityId(4)];
        resources.settlements.insert(alpha.id, alpha);

        let mut beta = sim_core::Settlement::new(SettlementId(52), "beta".to_string(), 3, 4, 0);
        beta.stockpile_food = 8.0;
        beta.members = vec![EntityId(11), EntityId(12), EntityId(13), EntityId(14)];
        resources.settlements.insert(beta.id, beta);

        resources.tension_pairs.insert("51:52".to_string(), 0.30);

        let mut system =
            TensionRuntimeSystem::new(64, sim_core::config::TENSION_CHECK_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TENSION_CHECK_INTERVAL_TICKS,
        );

        let pair_after = resources
            .tension_pairs
            .get("51:52")
            .copied()
            .expect("pair tension should be written");
        let scarcity = body::tension_scarcity_pressure(
            true,
            false,
            sim_core::config::TENSION_PER_SHARED_RESOURCE as f32,
        );
        let expected = body::tension_next_value(
            0.30,
            scarcity,
            sim_core::config::TENSION_DECAY_PER_YEAR as f32,
            sim_core::config::TENSION_CHECK_INTERVAL_TICKS as f32
                / sim_core::config::TICKS_PER_YEAR as f32,
        );
        assert!((pair_after as f32 - expected).abs() < 1e-6);
        assert!((0.0..=1.0).contains(&pair_after));
    }

    #[test]
    fn population_runtime_system_spawns_infant_and_consumes_food() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(64, 64, 17);
        let mut resources = SimResources::new(calendar, map, 9);
        let mut world = World::new();

        let settlement_id = SettlementId(61);
        let mut settlement =
            sim_core::Settlement::new(settlement_id, "core".to_string(), 12, 14, 0);
        settlement.stockpile_food = 120.0;
        resources.settlements.insert(settlement_id, settlement);
        resources.buildings.insert(
            BuildingId(901),
            Building {
                id: BuildingId(901),
                building_type: "shelter".to_string(),
                settlement_id,
                x: 12,
                y: 14,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut member_ids: Vec<EntityId> = Vec::new();
        for _ in 0..8 {
            let entity = world.spawn((
                Age {
                    alive: true,
                    stage: GrowthStage::Adult,
                    years: 26.0,
                    ..Age::default()
                },
                Identity {
                    settlement_id: Some(settlement_id),
                    growth_stage: GrowthStage::Adult,
                    ..Identity::default()
                },
                Behavior::default(),
                Needs::default(),
                Emotion::default(),
                Stress::default(),
            ));
            member_ids.push(EntityId(entity.id() as u64));
        }
        resources
            .settlements
            .get_mut(&settlement_id)
            .expect("settlement should exist")
            .members = member_ids;

        let world_before = world.len();
        let mut system =
            PopulationRuntimeSystem::new(50, sim_core::config::POPULATION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::POPULATION_TICK_INTERVAL,
        );

        assert_eq!(world.len(), world_before + 1);
        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist after population tick");
        assert_eq!(
            settlement_after.stockpile_food,
            120.0 - sim_core::config::BIRTH_FOOD_COST
        );
        assert_eq!(settlement_after.members.len(), 9);

        let mut infant_count: usize = 0;
        let mut query = world.query::<(&Age, &Identity)>();
        for (_, (age, identity)) in &mut query {
            if age.stage == GrowthStage::Infant && identity.settlement_id == Some(settlement_id) {
                infant_count += 1;
            }
        }
        assert_eq!(infant_count, 1);
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn tech_utilization_runtime_system_updates_era_by_known_tech_count() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(32, 32, 3);
        let mut resources = SimResources::new(calendar, map, 11);
        let mut world = World::new();

        let settlement_id = SettlementId(71);
        let mut settlement = sim_core::Settlement::new(settlement_id, "delta".to_string(), 4, 7, 0);
        settlement.current_era = "stone_age".to_string();
        for index in 0..sim_core::config::TECH_ERA_BRONZE_AGE_COUNT {
            settlement.tech_states.insert(
                format!("tech_{}", index),
                if index % 2 == 0 {
                    TechState::KnownStable
                } else {
                    TechState::KnownLow
                },
            );
        }
        resources.settlements.insert(settlement_id, settlement);

        let mut system = TechUtilizationRuntimeSystem::new(65, 1);
        system.run(&mut world, &mut resources, 1);

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert_eq!(settlement_after.current_era, "bronze_age");
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn tech_maintenance_runtime_system_transitions_states_by_population() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 19);
        let mut resources = SimResources::new(calendar, map, 21);
        let mut world = World::new();

        let settlement_id = SettlementId(81);
        let mut settlement = sim_core::Settlement::new(settlement_id, "theta".to_string(), 3, 3, 0);
        settlement.members = vec![EntityId(1)];
        settlement
            .tech_states
            .insert("pottery".to_string(), TechState::KnownStable);
        settlement
            .tech_states
            .insert("fishing".to_string(), TechState::KnownLow);
        resources.settlements.insert(settlement_id, settlement);

        let mut system =
            TechMaintenanceRuntimeSystem::new(63, sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS,
        );

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert_eq!(
            settlement_after.tech_states.get("pottery"),
            Some(&TechState::KnownLow)
        );
        assert_eq!(
            settlement_after.tech_states.get("fishing"),
            Some(&TechState::KnownLow)
        );
    }

    #[test]
    fn tech_maintenance_runtime_system_rediscovery_emits_event() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 23);
        let mut resources = SimResources::new(calendar, map, 31);
        let mut world = World::new();

        let settlement_id = SettlementId(82);
        let mut settlement =
            sim_core::Settlement::new(settlement_id, "lambda".to_string(), 8, 8, 0);
        settlement.members = (0..6_u64).map(EntityId).collect();
        settlement
            .tech_states
            .insert("weaving".to_string(), TechState::ForgottenRecent);
        resources.settlements.insert(settlement_id, settlement);

        let mut system =
            TechMaintenanceRuntimeSystem::new(63, sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS,
        );

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert_eq!(
            settlement_after.tech_states.get("weaving"),
            Some(&TechState::KnownLow)
        );
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn tech_discovery_runtime_system_discovers_unknown_tech_with_force_pop() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 25);
        let mut resources = SimResources::new(calendar, map, 41);
        let mut world = World::new();

        let settlement_id = SettlementId(83);
        let mut settlement = sim_core::Settlement::new(settlement_id, "sigma".to_string(), 9, 5, 0);
        settlement.members = (0..180_u64).map(EntityId).collect();
        settlement
            .tech_states
            .insert("bronze_working".to_string(), TechState::Unknown);
        resources.settlements.insert(settlement_id, settlement);

        let mut system =
            TechDiscoveryRuntimeSystem::new(62, sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS,
        );

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert_eq!(
            settlement_after.tech_states.get("bronze_working"),
            Some(&TechState::KnownLow)
        );
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn tech_discovery_runtime_system_skips_when_population_too_low() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 29);
        let mut resources = SimResources::new(calendar, map, 51);
        let mut world = World::new();

        let settlement_id = SettlementId(84);
        let mut settlement = sim_core::Settlement::new(settlement_id, "omega".to_string(), 2, 2, 0);
        settlement.members = vec![EntityId(1)];
        settlement
            .tech_states
            .insert("pottery".to_string(), TechState::Unknown);
        resources.settlements.insert(settlement_id, settlement);

        let mut system =
            TechDiscoveryRuntimeSystem::new(62, sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TECH_DISCOVERY_INTERVAL_TICKS,
        );

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert_eq!(
            settlement_after.tech_states.get("pottery"),
            Some(&TechState::Unknown)
        );
    }

    #[test]
    fn tech_propagation_runtime_system_imports_unknown_tech_from_stable_source() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 31);
        let mut resources = SimResources::new(calendar, map, 61);
        let mut world = World::new();

        let source_id = SettlementId(85);
        let mut source = sim_core::Settlement::new(source_id, "source".to_string(), 1, 1, 0);
        source
            .tech_states
            .insert("pottery".to_string(), TechState::KnownStable);

        let mut source_values = Values::default();
        source_values.set(ValueType::Knowledge, 0.4);
        source_values.set(ValueType::Tradition, -0.2);
        let mut source_skills = Skills::default();
        source_skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            sim_core::components::SkillEntry {
                level: 100,
                xp: 0.0,
            },
        );
        let source_entity = world.spawn((
            Identity {
                settlement_id: Some(source_id),
                ..Identity::default()
            },
            source_values,
            source_skills,
            Age::default(),
        ));
        source.members = vec![EntityId(source_entity.id() as u64)];
        resources.settlements.insert(source_id, source);

        let target_id = SettlementId(86);
        let mut target = sim_core::Settlement::new(target_id, "target".to_string(), 6, 6, 0);
        target
            .tech_states
            .insert("pottery".to_string(), TechState::Unknown);
        let mut target_values = Values::default();
        target_values.set(ValueType::Knowledge, 1.0);
        target_values.set(ValueType::Tradition, -1.0);
        let target_entity = world.spawn((
            Identity {
                settlement_id: Some(target_id),
                ..Identity::default()
            },
            target_values,
            Skills::default(),
            Age::default(),
        ));
        target.members = vec![EntityId(target_entity.id() as u64)];
        resources.settlements.insert(target_id, target);

        let mut system =
            TechPropagationRuntimeSystem::new(62, sim_core::config::TEACHING_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TEACHING_TICK_INTERVAL,
        );

        let target_after = resources
            .settlements
            .get(&target_id)
            .expect("target settlement should exist");
        assert_eq!(
            target_after.tech_states.get("pottery"),
            Some(&TechState::KnownLow)
        );
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn tech_propagation_runtime_system_skips_without_known_source() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 37);
        let mut resources = SimResources::new(calendar, map, 71);
        let mut world = World::new();

        let source_id = SettlementId(87);
        let mut source = sim_core::Settlement::new(source_id, "source".to_string(), 1, 1, 0);
        source
            .tech_states
            .insert("pottery".to_string(), TechState::Unknown);
        resources.settlements.insert(source_id, source);

        let target_id = SettlementId(88);
        let mut target = sim_core::Settlement::new(target_id, "target".to_string(), 8, 8, 0);
        target
            .tech_states
            .insert("pottery".to_string(), TechState::Unknown);
        resources.settlements.insert(target_id, target);

        let mut system =
            TechPropagationRuntimeSystem::new(62, sim_core::config::TEACHING_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::TEACHING_TICK_INTERVAL,
        );

        let target_after = resources
            .settlements
            .get(&target_id)
            .expect("target settlement should exist");
        assert_eq!(
            target_after.tech_states.get("pottery"),
            Some(&TechState::Unknown)
        );
        assert_eq!(resources.event_bus.pending_count(), 0);
    }

    #[test]
    fn gathering_runtime_system_harvests_tile_and_updates_stockpile() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(8, 8, 41);
        map.get_mut(2, 3).resources.push(TileResource {
            resource_type: ResourceType::Wood,
            amount: 5.0,
            max_amount: 5.0,
            regen_rate: 0.0,
        });
        let mut resources = SimResources::new(calendar, map, 77);
        let mut world = World::new();

        let settlement_id = SettlementId(91);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "wood".to_string(), 2, 3, 0),
        );

        let entity = world.spawn((
            Behavior {
                current_action: ActionType::GatherWood,
                ..Behavior::default()
            },
            Position::new(2, 3),
            Age::default(),
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
        ));
        resources
            .settlements
            .get_mut(&settlement_id)
            .expect("settlement should exist")
            .members = vec![EntityId(entity.id() as u64)];

        let mut system = GatheringRuntimeSystem::new(25, sim_core::config::GATHERING_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::GATHERING_TICK_INTERVAL,
        );

        let tile_after = resources.map.get(2, 3);
        let wood_after = tile_after
            .resources
            .iter()
            .find(|resource| resource.resource_type == ResourceType::Wood)
            .map(|resource| resource.amount)
            .unwrap_or(0.0);
        assert!((wood_after as f32 - (5.0 - sim_core::config::GATHER_AMOUNT) as f32).abs() < 1e-6);

        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert!(
            (settlement_after.stockpile_wood as f32 - sim_core::config::GATHER_AMOUNT as f32).abs()
                < 1e-6
        );
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn gathering_runtime_system_skips_infant_stage() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(8, 8, 43);
        map.get_mut(1, 1).resources.push(TileResource {
            resource_type: ResourceType::Food,
            amount: 4.0,
            max_amount: 4.0,
            regen_rate: 0.0,
        });
        let mut resources = SimResources::new(calendar, map, 79);
        let mut world = World::new();

        let settlement_id = SettlementId(92);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "food".to_string(), 1, 1, 0),
        );

        world.spawn((
            Behavior {
                current_action: ActionType::Forage,
                ..Behavior::default()
            },
            Position::new(1, 1),
            Age {
                stage: GrowthStage::Infant,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                ..Identity::default()
            },
        ));

        let mut system = GatheringRuntimeSystem::new(25, sim_core::config::GATHERING_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::GATHERING_TICK_INTERVAL,
        );

        let tile_after = resources.map.get(1, 1);
        let food_after = tile_after
            .resources
            .iter()
            .find(|resource| resource.resource_type == ResourceType::Food)
            .map(|resource| resource.amount)
            .unwrap_or(0.0);
        assert!((food_after - 4.0).abs() < 1e-6);
        let settlement_after = resources
            .settlements
            .get(&settlement_id)
            .expect("settlement should exist");
        assert_eq!(settlement_after.stockpile_food, 0.0);
        assert_eq!(resources.event_bus.pending_count(), 0);
    }

    #[test]
    fn construction_runtime_system_progresses_and_completes_building() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 47);
        let mut resources = SimResources::new(calendar, map, 83);
        let mut world = World::new();

        let settlement_id = SettlementId(93);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "build".to_string(), 4, 4, 0),
        );

        let building_id = BuildingId(901);
        resources.buildings.insert(
            building_id,
            Building {
                id: building_id,
                building_type: "campfire".to_string(),
                settlement_id,
                x: 4,
                y: 4,
                construction_progress: 0.0,
                is_complete: false,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_CONSTRUCTION".to_string(),
            SkillEntry {
                level: 100,
                xp: 0.0,
            },
        );
        world.spawn((
            Behavior {
                current_action: ActionType::Build,
                action_target_x: Some(4),
                action_target_y: Some(4),
                ..Behavior::default()
            },
            Position::new(4, 3),
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            skills,
        ));

        let mut system =
            ConstructionRuntimeSystem::new(28, sim_core::config::CONSTRUCTION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::CONSTRUCTION_TICK_INTERVAL,
        );
        let progress_after_first = resources
            .buildings
            .get(&building_id)
            .expect("building should exist")
            .construction_progress;
        assert!(progress_after_first > 0.0);

        for _ in 0..4 {
            system.run(
                &mut world,
                &mut resources,
                sim_core::config::CONSTRUCTION_TICK_INTERVAL,
            );
        }

        let building_after = resources
            .buildings
            .get(&building_id)
            .expect("building should exist");
        assert!(building_after.is_complete);
        assert!((building_after.construction_progress - 1.0).abs() < 1e-6);
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn construction_runtime_system_skips_non_adult_stage() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 53);
        let mut resources = SimResources::new(calendar, map, 89);
        let mut world = World::new();

        let settlement_id = SettlementId(94);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "teen".to_string(), 4, 4, 0),
        );

        let building_id = BuildingId(902);
        resources.buildings.insert(
            building_id,
            Building {
                id: building_id,
                building_type: "stockpile".to_string(),
                settlement_id,
                x: 4,
                y: 4,
                construction_progress: 0.0,
                is_complete: false,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        world.spawn((
            Behavior {
                current_action: ActionType::Build,
                action_target_x: Some(4),
                action_target_y: Some(4),
                ..Behavior::default()
            },
            Position::new(4, 4),
            Age {
                stage: GrowthStage::Teen,
                ..Age::default()
            },
        ));

        let mut system =
            ConstructionRuntimeSystem::new(28, sim_core::config::CONSTRUCTION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::CONSTRUCTION_TICK_INTERVAL,
        );

        let building_after = resources
            .buildings
            .get(&building_id)
            .expect("building should exist");
        assert!((building_after.construction_progress - 0.0).abs() < 1e-6);
        assert!(!building_after.is_complete);
        assert_eq!(resources.event_bus.pending_count(), 0);
    }

    #[test]
    fn early_construction_loop_places_assigns_and_completes_a_campfire() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(12, 12, 211);
        let resources = SimResources::new(calendar, map, 144);
        let mut engine = SimEngine::new(resources);
        engine.register(JobAssignmentRuntimeSystem::new(
            8,
            sim_core::config::JOB_ASSIGNMENT_TICK_INTERVAL,
        ));
        engine.register(BehaviorRuntimeSystem::new(
            20,
            sim_core::config::BEHAVIOR_TICK_INTERVAL as u64,
        ));
        engine.register(ConstructionRuntimeSystem::new(
            28,
            sim_core::config::CONSTRUCTION_TICK_INTERVAL,
        ));
        engine.register(SteeringRuntimeSystem::new(
            sim_core::config::STEERING_SYSTEM_PRIORITY,
            sim_core::config::STEERING_SYSTEM_INTERVAL,
        ));
        engine.register(MovementRuntimeSystem::new(
            sim_core::config::MOVEMENT_SYSTEM_PRIORITY,
            sim_core::config::MOVEMENT_TICK_INTERVAL,
        ));

        let settlement_id = SettlementId(1);
        let mut settlement = sim_core::Settlement::new(settlement_id, "alpha".to_string(), 4, 4, 0);
        settlement.stockpile_food = 20.0;
        settlement.stockpile_wood = 4.0;
        settlement.stockpile_stone = 2.0;
        settlement.buildings.push(BuildingId(1));
        engine
            .resources_mut()
            .settlements
            .insert(settlement_id, settlement);
        engine.resources_mut().buildings.insert(
            BuildingId(1),
            Building {
                id: BuildingId(1),
                building_type: "stockpile".to_string(),
                settlement_id,
                x: 4,
                y: 4,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        for (name, x, y) in [
            ("builder_a", 2, 1),
            ("builder_b", 3, 1),
            ("builder_c", 2, 2),
        ] {
            engine.world_mut().spawn((
                Age {
                    stage: GrowthStage::Adult,
                    ..Age::default()
                },
                Identity {
                    settlement_id: Some(settlement_id),
                    name: name.to_string(),
                    ..Identity::default()
                },
                Needs::default(),
                Position::new(x, y),
                Behavior::default(),
                Skills::default(),
            ));
        }

        engine.tick();

        let (campfire_id, campfire_x, campfire_y) = engine
            .resources()
            .buildings
            .iter()
            .find_map(|(building_id, building)| {
                (building.building_type == "campfire" && !building.is_complete).then_some((
                    *building_id,
                    building.x,
                    building.y,
                ))
            })
            .expect("job assignment should place an early campfire site on the first tick");

        let mut builder_count = 0_usize;
        let mut assigned_builders = 0_usize;
        let mut query = engine
            .world()
            .query::<(&Identity, &Behavior, Option<&Age>)>();
        for (_, (identity, behavior, age_opt)) in &mut query {
            let Some(age) = age_opt else {
                continue;
            };
            if !age.alive || age.stage != GrowthStage::Adult {
                continue;
            }
            if identity.settlement_id != Some(settlement_id) {
                continue;
            }
            if behavior.job == "builder" {
                builder_count += 1;
            }
            if behavior.current_action == ActionType::Build
                && behavior.action_target_x == Some(campfire_x)
                && behavior.action_target_y == Some(campfire_y)
            {
                assigned_builders += 1;
            }
        }
        assert_eq!(builder_count, 1);
        assert_eq!(assigned_builders, 1);
        drop(query);

        engine.run_ticks(40);

        let mid_progress = engine
            .resources()
            .buildings
            .get(&campfire_id)
            .expect("campfire should still exist")
            .construction_progress;
        assert!(mid_progress > 0.0);

        engine.run_ticks(80);

        let campfire_after = engine
            .resources()
            .buildings
            .get(&campfire_id)
            .expect("campfire should exist after completion");
        assert!(campfire_after.is_complete);
        assert!((campfire_after.construction_progress - 1.0).abs() < 1e-6);
    }

    #[test]
    fn family_runtime_system_pairs_single_adults_in_same_settlement() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 59);
        let mut resources = SimResources::new(calendar, map, 97);
        let mut world = World::new();

        let settlement_id = SettlementId(95);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "fam".to_string(), 2, 2, 0),
        );

        let male = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                sex: Sex::Male,
                ..Identity::default()
            },
            Social::default(),
        ));
        let female = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                sex: Sex::Female,
                ..Identity::default()
            },
            Social::default(),
        ));
        resources
            .settlements
            .get_mut(&settlement_id)
            .expect("settlement should exist")
            .members = vec![EntityId(male.id() as u64), EntityId(female.id() as u64)];

        let mut system = FamilyRuntimeSystem::new(52, 365);
        for _ in 0..64 {
            system.run(&mut world, &mut resources, 365);
            let male_spouse = world
                .get::<&Social>(male)
                .expect("male social should be queryable")
                .spouse;
            if male_spouse.is_some() {
                break;
            }
        }

        let male_social = world
            .get::<&Social>(male)
            .expect("male social should be queryable");
        let female_social = world
            .get::<&Social>(female)
            .expect("female social should be queryable");
        assert_eq!(male_social.spouse, Some(EntityId(female.id() as u64)));
        assert_eq!(female_social.spouse, Some(EntityId(male.id() as u64)));
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn family_runtime_system_skips_non_adult_entities() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 61);
        let mut resources = SimResources::new(calendar, map, 101);
        let mut world = World::new();

        let settlement_id = SettlementId(96);
        resources.settlements.insert(
            settlement_id,
            sim_core::Settlement::new(settlement_id, "fam2".to_string(), 3, 3, 0),
        );

        let teen_male = world.spawn((
            Age {
                stage: GrowthStage::Teen,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                sex: Sex::Male,
                ..Identity::default()
            },
            Social::default(),
        ));
        let adult_female = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                settlement_id: Some(settlement_id),
                sex: Sex::Female,
                ..Identity::default()
            },
            Social::default(),
        ));
        resources
            .settlements
            .get_mut(&settlement_id)
            .expect("settlement should exist")
            .members = vec![
            EntityId(teen_male.id() as u64),
            EntityId(adult_female.id() as u64),
        ];

        let mut system = FamilyRuntimeSystem::new(52, 365);
        for _ in 0..64 {
            system.run(&mut world, &mut resources, 365);
        }

        let teen_social = world
            .get::<&Social>(teen_male)
            .expect("teen social should be queryable");
        let female_social = world
            .get::<&Social>(adult_female)
            .expect("female social should be queryable");
        assert_eq!(teen_social.spouse, None);
        assert_eq!(female_social.spouse, None);
    }

    #[test]
    fn intergenerational_runtime_system_applies_parent_meaney_repair() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 63);
        let mut resources = SimResources::new(calendar, map, 103);
        let mut world = World::new();

        let child = world.spawn((
            Age {
                stage: GrowthStage::Child,
                ..Age::default()
            },
            Stress {
                level: 0.30,
                allostatic_load: 0.20,
                ..Stress::default()
            },
            Social::default(),
        ));
        let parent = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                sex: Sex::Female,
                ..Identity::default()
            },
            Needs::default(),
            Stress {
                level: 0.55,
                allostatic_load: 0.80,
                ..Stress::default()
            },
            Social {
                children: vec![EntityId(child.id() as u64)],
                ..Social::default()
            },
        ));

        if let Ok(mut child_social) = world.get::<&mut Social>(child) {
            child_social.parents = vec![EntityId(parent.id() as u64)];
        }

        let mut system = IntergenerationalRuntimeSystem::new(45, 240);
        system.run(&mut world, &mut resources, 240);

        let updated_parent = world
            .get::<&Stress>(parent)
            .expect("parent stress should be queryable");
        assert!(updated_parent.allostatic_load < 0.80);
        assert!(updated_parent.level <= 0.55);
    }

    #[test]
    fn intergenerational_runtime_system_applies_child_transmission() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 67);
        let mut resources = SimResources::new(calendar, map, 107);
        let mut world = World::new();

        let mother = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                sex: Sex::Female,
                ..Identity::default()
            },
            Stress {
                level: 0.70,
                allostatic_load: 0.92,
                ..Stress::default()
            },
            Memory {
                trauma_scars: vec![TraumaScar {
                    scar_id: "m".to_string(),
                    acquired_tick: 0,
                    severity: 0.7,
                    reactivation_count: 0,
                }],
                ..Memory::default()
            },
            Social::default(),
        ));
        let father = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Identity {
                sex: Sex::Male,
                ..Identity::default()
            },
            Stress {
                level: 0.65,
                allostatic_load: 0.86,
                ..Stress::default()
            },
            Memory {
                trauma_scars: vec![TraumaScar {
                    scar_id: "f".to_string(),
                    acquired_tick: 0,
                    severity: 0.6,
                    reactivation_count: 0,
                }],
                ..Memory::default()
            },
            Social::default(),
        ));
        let mut child_needs = Needs::default();
        child_needs.set(NeedType::Hunger, 0.30);
        let child = world.spawn((
            Age {
                stage: GrowthStage::Child,
                ..Age::default()
            },
            child_needs,
            Stress {
                level: 0.20,
                allostatic_load: 0.10,
                ..Stress::default()
            },
            Social {
                parents: vec![EntityId(mother.id() as u64), EntityId(father.id() as u64)],
                ..Social::default()
            },
        ));

        if let Ok(mut mother_social) = world.get::<&mut Social>(mother) {
            mother_social.children = vec![EntityId(child.id() as u64)];
        }
        if let Ok(mut father_social) = world.get::<&mut Social>(father) {
            father_social.children = vec![EntityId(child.id() as u64)];
        }

        let mut system = IntergenerationalRuntimeSystem::new(45, 240);
        system.run(&mut world, &mut resources, 240);

        let updated_child = world
            .get::<&Stress>(child)
            .expect("child stress should be queryable");
        assert!(updated_child.allostatic_load > 0.10);
        assert!(updated_child.level >= 0.20);
    }

    #[test]
    fn parenting_runtime_system_updates_parent_regulation_state() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 71);
        let mut resources = SimResources::new(calendar, map, 109);
        let mut world = World::new();

        let child = world.spawn((
            Age {
                stage: GrowthStage::Child,
                ..Age::default()
            },
            Stress {
                level: 0.20,
                allostatic_load: 0.10,
                ..Stress::default()
            },
            Coping::default(),
            Social::default(),
        ));
        let parent = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Stress {
                level: 0.82,
                allostatic_load: 0.55,
                ..Stress::default()
            },
            Social {
                children: vec![EntityId(child.id() as u64)],
                ..Social::default()
            },
        ));

        if let Ok(mut child_social) = world.get::<&mut Social>(child) {
            child_social.parents = vec![EntityId(parent.id() as u64)];
        }

        let mut system = ParentingRuntimeSystem::new(46, 240);
        system.run(&mut world, &mut resources, 240);

        let updated_parent = world
            .get::<&Stress>(parent)
            .expect("parent stress should be queryable");
        assert!(updated_parent.level < 0.82);
    }

    #[test]
    fn parenting_runtime_system_assigns_child_coping_strategy() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 73);
        let mut resources = SimResources::new(calendar, map, 113);
        let mut world = World::new();

        let child = world.spawn((
            Age {
                stage: GrowthStage::Child,
                ..Age::default()
            },
            Stress {
                level: 0.50,
                allostatic_load: 0.20,
                ..Stress::default()
            },
            Coping::default(),
            Social::default(),
        ));
        let parent = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            Stress {
                level: 0.18,
                allostatic_load: 0.10,
                ..Stress::default()
            },
            Social {
                children: vec![EntityId(child.id() as u64)],
                ..Social::default()
            },
        ));

        if let Ok(mut child_social) = world.get::<&mut Social>(child) {
            child_social.parents = vec![EntityId(parent.id() as u64)];
        }

        let mut system = ParentingRuntimeSystem::new(46, 240);
        system.run(&mut world, &mut resources, 240);

        let child_coping = world
            .get::<&Coping>(child)
            .expect("child coping should be queryable");
        let child_stress = world
            .get::<&Stress>(child)
            .expect("child stress should be queryable");
        assert_eq!(
            child_coping.active_strategy,
            Some(CopingStrategyId::ProblemSolving)
        );
        assert!(
            child_coping
                .usage_counts
                .get(&CopingStrategyId::ProblemSolving)
                .copied()
                .unwrap_or(0)
                >= 1
        );
        assert!(child_stress.level <= 0.50);
    }

    #[test]
    fn stat_sync_runtime_system_populates_derived_cache() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 75);
        let mut resources = SimResources::new(calendar, map, 117);
        let mut world = World::new();

        let mut values = Values::default();
        values.set(ValueType::Romance, 0.8);
        values.set(ValueType::Truth, 0.9);
        let entity = world.spawn((
            Age {
                years: 30.0,
                ..Age::default()
            },
            Personality::default(),
            Emotion::default(),
            BodyComponent::default(),
            values,
            Needs::default(),
            Intelligence::default(),
        ));

        let mut system = StatSyncRuntimeSystem::new(1, 10);
        system.run(&mut world, &mut resources, 10);

        let derived = resources
            .stat_sync_derived
            .get(&EntityId(entity.id() as u64))
            .expect("derived cache should contain alive entity");
        assert!(derived.iter().any(|value| *value > 0.0));
        assert_eq!(resources.stat_sync_derived.len(), 1);
    }

    #[test]
    fn stat_sync_runtime_system_skips_dead_entities() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 77);
        let mut resources = SimResources::new(calendar, map, 119);
        let mut world = World::new();

        let alive = world.spawn((Age::default(), Personality::default()));
        world.spawn((
            Age {
                alive: false,
                ..Age::default()
            },
            Personality::default(),
        ));

        let mut system = StatSyncRuntimeSystem::new(1, 10);
        system.run(&mut world, &mut resources, 10);

        assert_eq!(resources.stat_sync_derived.len(), 1);
        assert!(resources
            .stat_sync_derived
            .contains_key(&EntityId(alive.id() as u64)));
    }

    #[test]
    fn stat_threshold_runtime_system_applies_hunger_effect_action() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 81);
        let mut resources = SimResources::new(calendar, map, 137);
        let mut world = World::new();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.10);
        let entity = world.spawn((
            Age::default(),
            needs,
            Stress::default(),
            Behavior::default(),
        ));

        let mut system = StatThresholdRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(behavior.current_action, ActionType::Forage);
        let flags = resources
            .stat_threshold_flags
            .get(&EntityId(entity.id() as u64))
            .copied()
            .unwrap_or(0);
        assert!((flags & STAT_THRESHOLD_FLAG_HUNGER_LOW) != 0);
    }

    #[test]
    fn stat_threshold_runtime_system_hysteresis_clears_effect_after_recovery() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 85);
        let mut resources = SimResources::new(calendar, map, 139);
        let mut world = World::new();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.12);
        let entity = world.spawn((
            Age::default(),
            needs,
            Stress::default(),
            Behavior::default(),
        ));

        let mut system = StatThresholdRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        if let Ok(mut entity_needs) = world.get::<&mut Needs>(entity) {
            entity_needs.set(NeedType::Hunger, 0.24);
        }
        system.run(&mut world, &mut resources, 10);
        let flags_after_hysteresis = resources
            .stat_threshold_flags
            .get(&EntityId(entity.id() as u64))
            .copied()
            .unwrap_or(0);
        assert!((flags_after_hysteresis & STAT_THRESHOLD_FLAG_HUNGER_LOW) != 0);

        if let Ok(mut entity_needs) = world.get::<&mut Needs>(entity) {
            entity_needs.set(NeedType::Hunger, 0.80);
        }
        system.run(&mut world, &mut resources, 15);
        let flags_after_recovery = resources
            .stat_threshold_flags
            .get(&EntityId(entity.id() as u64))
            .copied()
            .unwrap_or(0);
        assert_eq!(flags_after_recovery & STAT_THRESHOLD_FLAG_HUNGER_LOW, 0);
    }

    #[test]
    fn behavior_runtime_system_assigns_forage_and_emits_event_for_hungry_adult() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 91);
        let mut resources = SimResources::new(calendar, map, 141);
        let mut world = World::new();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.10);
        needs.set(NeedType::Thirst, 0.80);
        needs.set(NeedType::Warmth, 0.80);
        needs.set(NeedType::Safety, 0.80);
        needs.set(NeedType::Belonging, 0.70);
        needs.energy = 0.75;

        let entity = world.spawn((
            Age {
                stage: GrowthStage::Adult,
                ..Age::default()
            },
            needs,
            Stress::default(),
            Emotion::default(),
            Position::new(4, 4),
            Behavior::default(),
        ));

        let mut system =
            BehaviorRuntimeSystem::new(20, sim_core::config::BEHAVIOR_TICK_INTERVAL as u64);
        system.run(&mut world, &mut resources, 50);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(behavior.current_action, ActionType::Forage);
        assert!(behavior.action_timer > 0);
        assert_eq!(behavior.action_duration, behavior.action_timer);
        assert!(behavior.action_target_x.is_some());
        assert!(behavior.action_target_y.is_some());
        assert!(resources.event_bus.pending_count() >= 1);
    }

    #[test]
    fn behavior_runtime_system_skips_migrate_and_active_timer_entities() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 93);
        let mut resources = SimResources::new(calendar, map, 143);
        let mut world = World::new();

        let migrating = world.spawn((
            Age::default(),
            Needs::default(),
            Position::new(2, 2),
            Behavior {
                current_action: ActionType::Migrate,
                action_timer: 0,
                action_duration: 0,
                ..Behavior::default()
            },
        ));
        let active = world.spawn((
            Age::default(),
            Needs::default(),
            Position::new(3, 3),
            Behavior {
                current_action: ActionType::Forage,
                action_timer: 6,
                action_duration: 6,
                action_target_x: Some(3),
                action_target_y: Some(3),
                ..Behavior::default()
            },
        ));

        let mut system =
            BehaviorRuntimeSystem::new(20, sim_core::config::BEHAVIOR_TICK_INTERVAL as u64);
        system.run(&mut world, &mut resources, 70);

        let migrate_behavior = world
            .get::<&Behavior>(migrating)
            .expect("migrating behavior should be queryable");
        assert_eq!(migrate_behavior.current_action, ActionType::Migrate);
        assert_eq!(migrate_behavior.action_timer, 0);

        let active_behavior = world
            .get::<&Behavior>(active)
            .expect("active behavior should be queryable");
        assert_eq!(active_behavior.current_action, ActionType::Forage);
        assert_eq!(active_behavior.action_timer, 6);
        assert_eq!(resources.event_bus.pending_count(), 0);
    }

    #[test]
    fn stats_recorder_runtime_system_records_snapshot_fields() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 79);
        let mut resources = SimResources::new(calendar, map, 127);
        let mut world = World::new();

        let settlement_id = SettlementId(97);
        let mut settlement = sim_core::Settlement::new(settlement_id, "stats".to_string(), 2, 2, 0);
        settlement.stockpile_food = 40.0;
        settlement.stockpile_wood = 12.0;
        settlement.stockpile_stone = 7.0;
        resources.settlements.insert(settlement_id, settlement);

        world.spawn((
            Age::default(),
            Behavior {
                job: "gatherer".to_string(),
                ..Behavior::default()
            },
        ));
        world.spawn((
            Age::default(),
            Behavior {
                job: "builder".to_string(),
                ..Behavior::default()
            },
        ));
        world.spawn((
            Age {
                alive: false,
                ..Age::default()
            },
            Behavior {
                job: "miner".to_string(),
                ..Behavior::default()
            },
        ));

        let mut system = StatsRecorderRuntimeSystem::new(90, 200);
        system.run(&mut world, &mut resources, 200);

        assert_eq!(resources.stats_history.len(), 1);
        let snapshot = resources
            .stats_history
            .last()
            .expect("stats snapshot should exist");
        assert_eq!(snapshot.tick, 200);
        assert_eq!(snapshot.pop, 2);
        assert_eq!(snapshot.gatherers, 1);
        assert_eq!(snapshot.builders, 1);
        assert_eq!(snapshot.miners, 0);
        assert_eq!(snapshot.none_job, 0);
        assert!((snapshot.food - 40.0).abs() < 1e-6);
        assert!((snapshot.wood - 12.0).abs() < 1e-6);
        assert!((snapshot.stone - 7.0).abs() < 1e-6);
        assert_eq!(resources.stats_peak_population, 2);
    }

    #[test]
    fn stats_recorder_runtime_system_caps_history_window() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 83);
        let mut resources = SimResources::new(calendar, map, 131);
        let mut world = World::new();

        world.spawn((
            Age::default(),
            Behavior {
                job: "none".to_string(),
                ..Behavior::default()
            },
        ));

        let mut system = StatsRecorderRuntimeSystem::new(90, 200);
        for idx in 0..220_u64 {
            system.run(&mut world, &mut resources, idx);
        }

        assert_eq!(resources.stats_history.len(), 200);
        let first_tick = resources
            .stats_history
            .first()
            .expect("history should contain entries")
            .tick;
        let last_tick = resources
            .stats_history
            .last()
            .expect("history should contain entries")
            .tick;
        assert_eq!(first_tick, 20);
        assert_eq!(last_tick, 219);
    }

    #[test]
    fn migration_runtime_system_founds_new_settlement_and_moves_members() {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(256, 256, 77);
        for x in 0..map.width {
            for y in 0..map.height {
                map.get_mut(x, y).resources.push(TileResource {
                    resource_type: ResourceType::Food,
                    amount: 8.0,
                    max_amount: 8.0,
                    regen_rate: 0.0,
                });
            }
        }
        let mut resources = SimResources::new(calendar, map, 91);
        let mut world = World::new();

        let source_id = SettlementId(61);
        let mut source = sim_core::Settlement::new(source_id, "origin".to_string(), 128, 128, 0);
        source.stockpile_food = 200.0;
        resources.settlements.insert(source_id, source);
        resources.buildings.insert(
            BuildingId(701),
            Building {
                id: BuildingId(701),
                building_type: "shelter".to_string(),
                settlement_id: source_id,
                x: 128,
                y: 128,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                condition: 1.0,
            },
        );

        let mut members: Vec<EntityId> = Vec::new();
        for _ in 0..50 {
            let entity = world.spawn((
                Age {
                    alive: true,
                    stage: GrowthStage::Adult,
                    years: 24.0,
                    ..Age::default()
                },
                Identity {
                    settlement_id: Some(source_id),
                    ..Identity::default()
                },
                Behavior::default(),
            ));
            members.push(EntityId(entity.id() as u64));
        }
        resources
            .settlements
            .get_mut(&source_id)
            .expect("source settlement should be mutable")
            .members = members;

        let mut system = MigrationRuntimeSystem::new(60, sim_core::config::MIGRATION_TICK_INTERVAL);
        system.run(
            &mut world,
            &mut resources,
            sim_core::config::MIGRATION_TICK_INTERVAL,
        );

        assert_eq!(resources.settlements.len(), 2);
        let new_id = resources
            .settlements
            .keys()
            .copied()
            .find(|settlement_id| *settlement_id != source_id)
            .expect("new settlement should be created");

        let source_after = resources
            .settlements
            .get(&source_id)
            .expect("source settlement should exist");
        let new_settlement = resources
            .settlements
            .get(&new_id)
            .expect("new settlement should exist");
        assert!(source_after.migration_cooldown > 0);
        assert!(source_after.stockpile_food < 200.0);
        assert_eq!(
            new_settlement.stockpile_food,
            sim_core::config::MIGRATION_STARTUP_FOOD
        );
        assert!(new_settlement.members.len() as u32 >= sim_core::config::MIGRATION_GROUP_SIZE_MIN);
        assert!(new_settlement.members.len() as u32 <= sim_core::config::MIGRATION_GROUP_SIZE_MAX);

        let mut moved_count: usize = 0;
        let mut query = world.query::<(&Identity, &Behavior)>();
        for (_, (identity, behavior)) in &mut query {
            if identity.settlement_id != Some(new_id) {
                continue;
            }
            moved_count += 1;
            assert_eq!(behavior.current_action, ActionType::Migrate);
            assert_eq!(behavior.action_timer, 100);
            assert_eq!(behavior.action_target_x, Some(new_settlement.x));
            assert_eq!(behavior.action_target_y, Some(new_settlement.y));
        }
        assert_eq!(moved_count, new_settlement.members.len());
        assert!(resources.event_bus.pending_count() >= moved_count + 1);
    }

    #[test]
    fn contagion_runtime_system_propagates_emotion_and_stress() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut recipient_emotion = Emotion::default();
        *recipient_emotion.get_mut(EmotionType::Joy) = 0.10;
        *recipient_emotion.get_mut(EmotionType::Trust) = 0.10;
        *recipient_emotion.get_mut(EmotionType::Sadness) = 0.60;
        *recipient_emotion.get_mut(EmotionType::Fear) = 0.40;
        let recipient_stress = Stress {
            level: 0.20,
            allostatic_load: 0.05,
            ..Stress::default()
        };
        let mut recipient_personality = Personality::default();
        recipient_personality.axes[HexacoAxis::X as usize] = 0.80;
        recipient_personality.axes[HexacoAxis::E as usize] = 0.70;
        recipient_personality.axes[HexacoAxis::A as usize] = 0.75;
        let recipient_identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        let recipient = world.spawn((
            Position::new(0, 0),
            recipient_emotion,
            recipient_stress,
            recipient_personality,
            recipient_identity,
        ));

        let mut donor1_emotion = Emotion::default();
        *donor1_emotion.get_mut(EmotionType::Joy) = 0.90;
        *donor1_emotion.get_mut(EmotionType::Trust) = 0.80;
        *donor1_emotion.get_mut(EmotionType::Sadness) = 0.10;
        *donor1_emotion.get_mut(EmotionType::Fear) = 0.10;
        let donor1_stress = Stress {
            level: 0.80,
            allostatic_load: 0.30,
            ..Stress::default()
        };
        let donor1_identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        world.spawn((
            Position::new(1, 0),
            donor1_emotion,
            donor1_stress,
            Personality::default(),
            donor1_identity,
        ));

        let mut donor2_emotion = Emotion::default();
        *donor2_emotion.get_mut(EmotionType::Joy) = 0.85;
        *donor2_emotion.get_mut(EmotionType::Trust) = 0.75;
        *donor2_emotion.get_mut(EmotionType::Sadness) = 0.05;
        *donor2_emotion.get_mut(EmotionType::Fear) = 0.05;
        let donor2_stress = Stress {
            level: 0.70,
            allostatic_load: 0.20,
            ..Stress::default()
        };
        let donor2_identity = Identity {
            settlement_id: Some(SettlementId(1)),
            ..Identity::default()
        };
        world.spawn((
            Position::new(2, 1),
            donor2_emotion,
            donor2_stress,
            Personality::default(),
            donor2_identity,
        ));

        let mut system = ContagionRuntimeSystem::new(38, 3);
        system.run(&mut world, &mut resources, 3);

        let mut query_one = world
            .query_one::<(&Emotion, &Stress)>(recipient)
            .expect("recipient components should be queryable");
        let (updated_emotion, updated_stress) = query_one
            .get()
            .expect("recipient emotion/stress tuple should be readable");
        let joy = updated_emotion.get(EmotionType::Joy);
        let trust = updated_emotion.get(EmotionType::Trust);
        let sadness = updated_emotion.get(EmotionType::Sadness);
        let stress_level = updated_stress.level;
        let allostatic = updated_stress.allostatic_load;
        drop(query_one);

        assert!(joy > 0.10);
        assert!(trust > 0.10);
        assert!(sadness < 0.60);
        assert!(stress_level > 0.20);
        assert!(allostatic >= 0.05);
    }

    #[test]
    fn age_runtime_system_updates_stage_identity_and_elder_builder_job() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            ticks: sim_core::config::AGE_ADULT_END - 10,
            years: 0.0,
            stage: GrowthStage::Adult,
            alive: true,
        };
        let identity = Identity {
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let behavior = Behavior {
            job: "builder".to_string(),
            ..Behavior::default()
        };
        let entity = world.spawn((age, identity, behavior));

        let mut system = AgeRuntimeSystem::new(48, 20);
        system.run(&mut world, &mut resources, 20);

        let updated_age = world.get::<&Age>(entity).expect("age should be queryable");
        assert_eq!(updated_age.stage, GrowthStage::Elder);
        assert!(updated_age.years > 0.0);
        drop(updated_age);

        let updated_identity = world
            .get::<&Identity>(entity)
            .expect("identity should be queryable");
        assert_eq!(updated_identity.growth_stage, GrowthStage::Elder);
        drop(updated_identity);

        let updated_behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should be queryable");
        assert_eq!(updated_behavior.job, "none");
    }

    #[test]
    fn mortality_runtime_system_marks_high_risk_entity_dead() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            ticks: sim_core::config::TICKS_PER_YEAR as u64,
            years: 1.0,
            stage: GrowthStage::Adult,
            alive: true,
        };
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.0);
        let body = BodyComponent {
            dr_realized: 0,
            ..BodyComponent::default()
        };
        let stress = Stress {
            allostatic_load: 1.0,
            ..Stress::default()
        };
        let entity = world.spawn((age, needs, body, stress));

        let mut system = MortalityRuntimeSystem::new(49, 1);
        for _ in 0..100 {
            system.run(&mut world, &mut resources, 1);
            let current_age = world.get::<&Age>(entity).expect("age should be queryable");
            if !current_age.alive {
                break;
            }
        }

        let updated_age = world.get::<&Age>(entity).expect("age should be queryable");
        assert!(!updated_age.alive);
    }

    #[test]
    fn upper_needs_runtime_system_updates_upper_need_buckets() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut needs = Needs::default();
        needs.set(NeedType::Competence, 0.45);
        needs.set(NeedType::Autonomy, 0.52);
        needs.set(NeedType::SelfActualization, 0.33);
        needs.set(NeedType::Meaning, 0.41);
        needs.set(NeedType::Transcendence, 0.29);
        needs.set(NeedType::Recognition, 0.35);
        needs.set(NeedType::Belonging, 0.38);
        needs.set(NeedType::Intimacy, 0.32);

        let mut skills = Skills::default();
        skills.entries.insert(
            "SKILL_FORAGING".to_string(),
            SkillEntry { level: 78, xp: 0.0 },
        );
        skills.entries.insert(
            "SKILL_MINING".to_string(),
            SkillEntry { level: 40, xp: 0.0 },
        );

        let mut values = Values::default();
        values.set(ValueType::Nature, 0.7);
        values.set(ValueType::Independence, 0.4);
        values.set(ValueType::HardWork, 0.5);
        values.set(ValueType::Sacrifice, 0.2);

        let behavior = Behavior {
            job: "gatherer".to_string(),
            ..Behavior::default()
        };
        let identity = Identity {
            settlement_id: Some(SettlementId(1)),
            growth_stage: GrowthStage::Adult,
            ..Identity::default()
        };
        let social = Social {
            spouse: Some(EntityId(2)),
            ..Social::default()
        };

        let entity = world.spawn((needs, skills, values, behavior, identity, social));
        let mut system = UpperNeedsRuntimeSystem::new(12, 5);
        system.run(&mut world, &mut resources, 5);

        let best_skill_norm = body::upper_needs_best_skill_normalized(&[78, 0, 40, 0, 0], 100);
        let alignment = body::upper_needs_job_alignment(2, 0.0, 0.0, 0.5, 0.7, 0.4);
        let expected = body::upper_needs_step(
            &[0.45, 0.52, 0.33, 0.41, 0.29, 0.35, 0.38, 0.32],
            &[
                sim_core::config::UPPER_NEEDS_COMPETENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_AUTONOMY_DECAY as f32,
                sim_core::config::UPPER_NEEDS_SELF_ACTUATION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_MEANING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_TRANSCENDENCE_DECAY as f32,
                sim_core::config::UPPER_NEEDS_RECOGNITION_DECAY as f32,
                sim_core::config::UPPER_NEEDS_BELONGING_DECAY as f32,
                sim_core::config::UPPER_NEEDS_INTIMACY_DECAY as f32,
            ],
            sim_core::config::UPPER_NEEDS_COMPETENCE_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_AUTONOMY_JOB_GAIN as f32,
            sim_core::config::UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_INTIMACY_PARTNER_GAIN as f32,
            sim_core::config::UPPER_NEEDS_RECOGNITION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF as f32,
            sim_core::config::UPPER_NEEDS_MEANING_BASE_GAIN as f32,
            sim_core::config::UPPER_NEEDS_MEANING_ALIGNED_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN as f32,
            sim_core::config::UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF as f32,
            best_skill_norm,
            alignment,
            0.2,
            true,
            true,
            true,
        );

        let updated = world
            .get::<&Needs>(entity)
            .expect("updated needs component should be queryable");
        assert!((updated.get(NeedType::Competence) as f32 - expected[0]).abs() < 1e-6);
        assert!((updated.get(NeedType::Recognition) as f32 - expected[5]).abs() < 1e-6);
        assert!((updated.get(NeedType::Belonging) as f32 - expected[6]).abs() < 1e-6);
        assert!((updated.get(NeedType::Intimacy) as f32 - expected[7]).abs() < 1e-6);
    }

    // === Enhanced Stress/Coping fidelity tests ===

    #[test]
    fn stress_runtime_system_uses_personality_scaling() {
        let mut world = World::new();
        let mut resources = make_resources();

        // Severe need deficits to produce a large base stress delta
        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.02);
        needs.set(NeedType::Thirst, 0.02);
        needs.set(NeedType::Safety, 0.05);
        needs.set(NeedType::Belonging, 0.10);
        needs.energy = 0.05;

        let mut emotion = Emotion::default();
        emotion.add(EmotionType::Fear, 0.80);
        emotion.add(EmotionType::Anger, 0.70);

        // High neuroticism (low E = high emotionality in HEXACO)
        let mut personality_reactive = Personality::default();
        personality_reactive.axes[HexacoAxis::E as usize] = 0.10;
        personality_reactive.axes[HexacoAxis::C as usize] = 0.20;

        // Low neuroticism (high E, high C)
        let mut personality_calm = Personality::default();
        personality_calm.axes[HexacoAxis::E as usize] = 0.95;
        personality_calm.axes[HexacoAxis::C as usize] = 0.90;

        let stress1 = Stress::default();
        let stress2 = Stress::default();

        let e1 = world.spawn((
            needs.clone(),
            stress1,
            emotion.clone(),
            personality_reactive,
        ));
        let e2 = world.spawn((needs.clone(), stress2, emotion.clone(), personality_calm));

        let mut system = StressRuntimeSystem::new(34, 4);
        // Run multiple ticks so personality scaling accumulates
        for i in 0..5 {
            system.run(&mut world, &mut resources, i * 4);
        }

        let s1 = world.get::<&Stress>(e1).unwrap();
        let s2 = world.get::<&Stress>(e2).unwrap();
        // Both should have stress > 0 under these conditions
        assert!(
            s1.level > 0.0,
            "reactive entity should have positive stress"
        );
        assert!(s2.level > 0.0, "calm entity should have positive stress");
        // Different personality profiles should produce different stress outcomes
        // (direction depends on complex multi-axis formula interactions)
        assert!(
            (s1.level - s2.level).abs() > 1e-10,
            "different personalities should produce different stress levels ({} vs {})",
            s1.level,
            s2.level
        );
    }

    #[test]
    fn stress_runtime_system_updates_gas_stage_and_reserve() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.02);
        needs.set(NeedType::Thirst, 0.02);
        needs.set(NeedType::Safety, 0.05);
        needs.energy = 0.05;

        let mut emotion = Emotion::default();
        emotion.add(EmotionType::Fear, 0.9);
        emotion.add(EmotionType::Anger, 0.8);

        let stress = Stress {
            level: 0.60,
            reserve: 0.80,
            ..Stress::default()
        };

        let entity = world.spawn((needs, stress, emotion));

        let mut system = StressRuntimeSystem::new(34, 4);
        for i in 0..10 {
            system.run(&mut world, &mut resources, i * 4);
        }

        let updated = world.get::<&Stress>(entity).unwrap();
        // Under sustained stress, reserve should decrease and GAS stage should advance
        assert!(
            updated.reserve < 0.80,
            "reserve ({}) should decrease under sustained stress",
            updated.reserve
        );
    }

    #[test]
    fn stress_runtime_system_manages_stress_traces() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, 0.03);
        needs.energy = 0.20;

        let stress = Stress {
            level: 0.30,
            stress_traces: vec![
                StressTrace {
                    source_id: "stressor_a".to_string(),
                    per_tick: 5.0,
                    decay_rate: 0.10,
                },
                StressTrace {
                    source_id: "stressor_b".to_string(),
                    per_tick: 0.001, // will decay below epsilon
                    decay_rate: 0.99,
                },
            ],
            ..Stress::default()
        };

        let entity = world.spawn((needs, stress));

        let mut system = StressRuntimeSystem::new(34, 4);
        system.run(&mut world, &mut resources, 4);

        let updated = world.get::<&Stress>(entity).unwrap();
        // stressor_b should be removed (per_tick decayed below threshold)
        // stressor_a should remain with updated per_tick
        assert!(
            !updated.stress_traces.is_empty(),
            "at least one trace should remain"
        );
        let has_a = updated
            .stress_traces
            .iter()
            .any(|t| t.source_id == "stressor_a");
        assert!(has_a, "stressor_a should survive with non-trivial per_tick");
        let has_b = updated
            .stress_traces
            .iter()
            .any(|t| t.source_id == "stressor_b");
        assert!(!has_b, "stressor_b should be removed after decay");
    }

    #[test]
    fn coping_runtime_system_denial_timer_explodes_debt() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut coping = Coping::default();
        coping.denial_accumulator = 400.0; // raw GDScript scale
        coping.denial_timer = 2; // will expire with dec=1

        let stress = Stress {
            level: 0.30,
            ..Stress::default()
        };

        let entity = world.spawn((coping, stress));

        let mut system = CopingRuntimeSystem::new(42, 1);
        // Run 3 ticks: timer 2→1→0 (explosion)
        for tick in 0..3 {
            system.run(&mut world, &mut resources, tick);
        }

        let updated_stress = world.get::<&Stress>(entity).unwrap();
        let updated_coping = world.get::<&Coping>(entity).unwrap();
        // Debt explodes at 1.5x: 400 * 1.5 / 2000 = 0.30 added
        assert!(
            updated_stress.level > 0.50,
            "stress ({}) should spike after denial explosion",
            updated_stress.level
        );
        assert!(
            updated_coping.denial_accumulator < 0.01,
            "denial accumulator ({}) should be cleared after explosion",
            updated_coping.denial_accumulator
        );
    }

    #[test]
    fn coping_runtime_system_rebound_queue_fires_after_delay() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut coping = Coping::default();
        coping.rebound_queue.push(CopingRebound {
            delay: 3,
            stress_rebound: 0.20,
            allostatic_add: 0.05,
        });

        let stress = Stress {
            level: 0.20,
            allostatic_load: 0.10,
            ..Stress::default()
        };

        let entity = world.spawn((coping, stress));

        let mut system = CopingRuntimeSystem::new(42, 1);

        // Run 2 ticks — rebound should NOT fire yet (delay 3→2→1)
        system.run(&mut world, &mut resources, 0);
        system.run(&mut world, &mut resources, 1);

        {
            let mid = world.get::<&Coping>(entity).unwrap();
            assert_eq!(mid.rebound_queue.len(), 1, "rebound should still be queued");
        }

        // Run 2 more ticks — rebound fires (delay 1→0)
        system.run(&mut world, &mut resources, 2);
        system.run(&mut world, &mut resources, 3);

        let updated_coping = world.get::<&Coping>(entity).unwrap();
        assert!(
            updated_coping.rebound_queue.is_empty(),
            "rebound queue should be empty after firing"
        );

        let updated_stress = world.get::<&Stress>(entity).unwrap();
        // Rebound: 0.20 * 300 / 2000 = 0.03 stress added
        assert!(
            updated_stress.level > 0.20,
            "stress ({}) should increase after rebound",
            updated_stress.level
        );
        assert!(
            updated_stress.allostatic_load > 0.10,
            "allostatic ({}) should increase after rebound",
            updated_stress.allostatic_load
        );
    }

    #[test]
    fn coping_runtime_system_substance_withdrawal_injects_stress() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut coping = Coping::default();
        coping.dependency_score = 0.80; // above 0.6 threshold
        coping.substance_recent_timer = 1; // will expire on first tick

        let stress = Stress {
            level: 0.20,
            ..Stress::default()
        };

        let entity = world.spawn((coping, stress));

        let mut system = CopingRuntimeSystem::new(42, 1);
        // Tick 0: timer 1→0, withdrawal fires
        system.run(&mut world, &mut resources, 0);
        // Tick 1: timer was reset to 24, decrements to 23
        system.run(&mut world, &mut resources, 1);

        let updated_stress = world.get::<&Stress>(entity).unwrap();
        // Withdrawal: 90/2000 = 0.045 stress added
        assert!(
            updated_stress.level > 0.20,
            "stress ({}) should increase from withdrawal",
            updated_stress.level
        );

        let updated_coping = world.get::<&Coping>(entity).unwrap();
        assert!(
            updated_coping.substance_recent_timer > 0,
            "substance_recent_timer ({}) should be reset after withdrawal injection",
            updated_coping.substance_recent_timer
        );
    }

    #[test]
    fn coping_runtime_system_clears_strategy_at_low_stress() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut coping = Coping::default();
        coping.active_strategy = Some(CopingStrategyId::Acceptance);

        let stress = Stress {
            level: 0.10,           // below COPING_CLEAR_STRESS_MAX (0.20)
            allostatic_load: 0.10, // below COPING_CLEAR_ALLOSTATIC_MAX (0.20)
            ..Stress::default()
        };

        let entity = world.spawn((coping, stress));

        let mut system = CopingRuntimeSystem::new(42, 1);
        system.run(&mut world, &mut resources, 0);

        let updated = world.get::<&Coping>(entity).unwrap();
        assert!(
            updated.active_strategy.is_none(),
            "active strategy should be cleared at low stress"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Settlement Culture Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn settlement_culture_runtime_system_drifts_member_toward_culture() {
        let mut world = World::new();
        let mut resources = make_resources();
        let sid = SettlementId(1);
        let mut settlement = sim_core::Settlement::new(sid, "village".to_string(), 0, 0, 0);
        settlement.leader_id = None;

        // Member 1: high values (0.95)
        let mut v1 = Values::default();
        for v in v1.values.iter_mut() {
            *v = 0.95;
        }
        let id1 = Identity {
            settlement_id: Some(sid),
            ..Identity::default()
        };
        let a1 = Age {
            years: 25.0,
            alive: true,
            ..Age::default()
        };
        let e1 = world.spawn((v1, id1, a1.clone()));
        let eid1 = sim_core::ids::EntityId(e1.id() as u64);

        // Member 2: low values (0.05)
        let mut v2 = Values::default();
        for v in v2.values.iter_mut() {
            *v = 0.05;
        }
        let id2 = Identity {
            settlement_id: Some(sid),
            ..Identity::default()
        };
        let e2 = world.spawn((v2, id2, a1.clone()));
        let eid2 = sim_core::ids::EntityId(e2.id() as u64);

        // Register both as settlement members
        settlement.members.push(eid1);
        settlement.members.push(eid2);
        resources.settlements.insert(sid, settlement);

        let mut system = SettlementCultureRuntimeSystem::new(350, 10);
        system.run(&mut world, &mut resources, 10);

        // Entity 1 had values=0.95, culture avg ~0.5, deviation=0.45>0.40 threshold
        let updated = world.get::<&Values>(e1).unwrap();
        assert!(
            updated.values[0] < 0.95,
            "high-value member should drift toward settlement culture average"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Chronicle Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn chronicle_runtime_system_prunes_old_low_importance_events() {
        use sim_engine::{
            ChronicleEvent, ChronicleEventCause, ChronicleEventMagnitude, ChronicleEventType,
        };

        let mut world = World::new();
        let mut resources = make_resources();

        let make_event =
            |tick: u64, entity_id: u64, significance: f64, summary_key: &str| ChronicleEvent {
                tick,
                entity_id: EntityId(entity_id),
                event_type: ChronicleEventType::MovementDecision,
                cause: ChronicleEventCause::Food,
                magnitude: ChronicleEventMagnitude {
                    influence: significance,
                    steering: significance,
                    significance,
                },
                tile_x: 0,
                tile_y: 0,
                summary_key: summary_key.to_string(),
                effect_key: "steering_velocity".to_string(),
            };
        resources
            .chronicle_log
            .append_event(make_event(100, 10, 0.10, "old_low_1"));
        resources
            .chronicle_log
            .append_event(make_event(1000, 11, 0.20, "old_low_2"));
        resources
            .chronicle_log
            .append_event(make_event(30000, 20, 0.20, "recent_low"));
        resources
            .chronicle_log
            .append_event(make_event(500, 21, 0.40, "old_med"));
        resources
            .chronicle_log
            .append_event(make_event(50, 22, 0.90, "high"));

        let tick = 25 * 4380; // year 25
        let mut system = ChronicleRuntimeSystem::new(310, 1);
        system.run(&mut world, &mut resources, tick as u64);

        assert_eq!(
            resources.chronicle_log.world_len(),
            3,
            "expected 3 world events after pruning old low-importance"
        );
        let types: Vec<&str> = resources
            .chronicle_log
            .recent_events(8)
            .iter()
            .map(|event| event.summary_key.as_str())
            .collect();
        assert!(types.contains(&"recent_low"), "recent low-importance kept");
        assert!(types.contains(&"old_med"), "old medium-importance kept");
        assert!(types.contains(&"high"), "high-importance always kept");
        assert!(!types.contains(&"old_low_1"), "old low-importance pruned");
        assert!(!types.contains(&"old_low_2"), "old low-importance 2 pruned");

        assert!(
            resources
                .chronicle_log
                .latest_for_entity(EntityId(10))
                .is_none(),
            "personal events referencing pruned low-significance world ticks should be GC'd"
        );
        assert_eq!(
            resources
                .chronicle_log
                .query_by_entity(EntityId(20), 4)
                .len(),
            1,
            "personal events referencing retained world ticks should be kept"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Personality Maturation Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn personality_maturation_runtime_system_drifts_facets() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut personality = Personality::default();
        personality.facets = [0.5; 24];
        personality.recalculate_axes();
        let age = Age {
            years: 20.0,
            alive: true,
            ..Age::default()
        };
        let entity = world.spawn((age, personality));

        let mut system = PersonalityMaturationRuntimeSystem::new(230, 50);
        // Run multiple ticks so OU process has time to drift
        for t in (50..=500).step_by(50) {
            system.run(&mut world, &mut resources, t);
        }

        let updated = world.get::<&Personality>(entity).unwrap();
        // At least some facets should have drifted from 0.5
        let any_changed = updated.facets.iter().any(|&f| (f - 0.5).abs() > 1e-6);
        assert!(
            any_changed,
            "personality facets should drift via OU process over maturation ticks"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Personality Generator Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn personality_generator_runtime_system_initializes_newborn() {
        let mut world = World::new();
        let mut resources = make_resources();

        let personality = Personality::default(); // all zeros/0.5
        let age = Age {
            years: 0.0,
            alive: true,
            stage: GrowthStage::Infant,
            ..Age::default()
        };
        let body = BodyComponent::default();
        let identity = Identity::default();
        let social = Social::default();
        let entity = world.spawn((personality, age, body, identity, social));

        let mut system = PersonalityGeneratorRuntimeSystem::new(99, 1);
        system.run(&mut world, &mut resources, 1);

        let updated = world.get::<&Personality>(entity).unwrap();
        // Personality generator should have set facets to non-default values
        let any_changed = updated.facets.iter().any(|&f| (f - 0.5).abs() > 0.01);
        assert!(
            any_changed,
            "newborn should have personality facets generated"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Attachment Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn attachment_runtime_system_determines_type_for_infant() {
        let mut world = World::new();
        let mut resources = make_resources();

        // Create a parent with high Agreeableness and low stress → should produce Secure attachment
        let mut parent_personality = Personality::default();
        parent_personality.axes[HexacoAxis::A as usize] = 0.80; // high A-axis → high sensitivity
        parent_personality.facets = [0.5; 24];
        let parent_stress = Stress {
            level: 0.10,
            ..Stress::default()
        }; // low stress → high consistency
        let parent_age = Age {
            years: 30.0,
            alive: true,
            ..Age::default()
        };
        let parent_social = Social::default();
        let parent_entity =
            world.spawn((parent_personality, parent_stress, parent_age, parent_social));
        let parent_eid = sim_core::ids::EntityId(parent_entity.id() as u64);

        // Create a 1-year-old infant with the parent
        let child_age = Age {
            years: 1.0,
            alive: true,
            stage: GrowthStage::Infant,
            ..Age::default()
        };
        let child_stress = Stress::default();
        let child_social = Social {
            parents: vec![parent_eid],
            ..Social::default()
        };
        let child_entity = world.spawn((child_age, child_stress, child_social));

        let mut system = AttachmentRuntimeSystem::new(125, 10);
        system.run(&mut world, &mut resources, 10);

        let updated_social = world.get::<&Social>(child_entity).unwrap();
        assert!(
            updated_social.attachment_type.is_some(),
            "1-year-old with parent should have attachment type determined"
        );
        assert_eq!(
            updated_social.attachment_type.unwrap(),
            AttachmentType::Secure,
            "high-A low-stress parent should produce Secure attachment"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // ACE Tracker Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn ace_tracker_runtime_system_backfills_adult_ace_score() {
        let mut world = World::new();
        let mut resources = make_resources();

        let age = Age {
            years: 25.0,
            alive: true,
            ..Age::default()
        };
        let stress = Stress {
            allostatic_load: 0.40, // moderate allostatic → nonzero ACE
            ace_score: 0.0,        // no ACE yet
            ..Stress::default()
        };
        let social = Social {
            attachment_type: Some(AttachmentType::Anxious), // insecure → adds to ACE
            ..Social::default()
        };
        let memory = Memory::default(); // 0 trauma scars
        let entity = world.spawn((age, stress, social, memory));

        let mut system = AceTrackerRuntimeSystem::new(126, 50);
        system.run(&mut world, &mut resources, 50);

        let updated_stress = world.get::<&Stress>(entity).unwrap();
        assert!(
            updated_stress.ace_score > 0.0,
            "adult with allostatic load and insecure attachment should get nonzero ACE"
        );
        assert!(
            updated_stress.ace_score <= 1.0,
            "ACE score should be normalized to 0..1"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Trait Runtime System
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn trait_runtime_system_activates_trait_from_personality() {
        let mut world = World::new();
        let mut resources = make_resources();

        // Entity with high Agreeableness
        let mut personality = Personality::default();
        personality.axes[HexacoAxis::A as usize] = 0.85;
        personality.facets = [0.5; 24];
        let values = Values::default();
        let traits = Traits::default();
        let entity = world.spawn((personality, values, traits));

        // Define a trait that activates when A >= 0.75
        let defs = vec![
            TraitDefinition {
                id: "compassionate".to_string(),
                conditions: vec![TraitCondition {
                    source: TraitConditionSource::HexacoAxis(HexacoAxis::A),
                    direction: TraitDirection::High,
                    threshold: 0.75,
                }],
                incompatibles: vec![],
            },
            TraitDefinition {
                id: "ruthless".to_string(),
                conditions: vec![TraitCondition {
                    source: TraitConditionSource::HexacoAxis(HexacoAxis::A),
                    direction: TraitDirection::Low,
                    threshold: 0.25,
                }],
                incompatibles: vec!["compassionate".to_string()],
            },
        ];

        let mut system = TraitRuntimeSystem::new(232, 50).with_definitions(defs);
        system.run(&mut world, &mut resources, 50);

        let updated = world.get::<&Traits>(entity).unwrap();
        assert!(
            updated.has_trait("compassionate"),
            "entity with high A should gain 'compassionate' trait"
        );
        assert!(
            !updated.has_trait("ruthless"),
            "entity with high A should not gain 'ruthless' trait"
        );
    }
}

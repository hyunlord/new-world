# A-13: Knowledge Learning System v1

## Feature Description

Implements the execution path for Knowledge Learning in the WorldSim simulation.
The AgentKnowledge component, KnowledgeEntry/LearningState/TransmissionSource types,
and 8 starter techs at spawn were already present. This feature adds:

1. Learn and Teach actions to BEHAVIOR_ACTION_ORDER (cognition.rs)
2. Action completion handlers in world.rs (sets LearningState / teaching_target)
3. KnowledgeLearningRuntimeSystem: Warm-tier (priority 105, interval 10) that advances
   LearningState.progress each tick using TCI g_factor + openness rate modulation,
   teacher proximity boost (KNOWLEDGE_LEARN_TEACHER_BOOST = 0.5), and promotes
   completed learning (progress >= 1.0) to KnowledgeEntry(proficiency: 0.5)
4. Registration in sim-bridge runtime_system.rs (KnowledgeLearning = 66)
5. 6 harness tests + localization keys (ACTION_LEARN/TEACH en+ko)

## Crate: sim-systems, sim-bridge, sim-core, sim-test

## Harness Tests (6)

1. harness_a13_knowledge_learning_system_registered — registry check, 63 entries
2. harness_a13_config_constants_are_valid — 7 constants positive/valid
3. harness_a13_learning_progress_advances — progress > 0 after 10 ticks
4. harness_a13_learning_completes_to_known_entry — progress 0.999 → known[]
5. harness_a13_teacher_nearby_boosts_rate — boosted > solo progress
6. harness_a13_all_agents_have_knowledge_component — ≥50% agents have starter knowledge

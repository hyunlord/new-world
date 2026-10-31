# System world registration

Baseline and colony each retain 49 registrations. The priority125 slot is `AgentDecisionSystem` for baseline and `ColonyMindSystem` for colony. Other registrations retain their prior membership, including baseline no-ops. Membership means registered, not that every arm fires.

Validation status: **NOT_RUN** for the production switch; parent-owned focused verification is pending. The table below was prepared from current source; the test must confirm it against real constructor metadata before acceptance.

`sim-engine::registration::SYSTEM_REGISTRATIONS` owns world membership. Concrete constructors stay in `sim-systems`. The `registration_tests::system_worlds_document_matches_real_registration` test renders this table from actual registered instances, checks this snapshot, and prints the table with `--nocapture`. Tests do not overwrite documentation.

<!-- SYSTEM_WORLDS:START -->
| System | Priority | Interval ticks | Baseline | Colony |
|---|---:|---:|---|---|
| BuildingStampSystem | 90 | 1 | yes | yes |
| InfluenceUpdateSystem | 100 | 1 | yes | yes |
| AgentInfluenceSampleSystem | 110 | 1 | yes | yes |
| FoodDailySystem | 119 | 1440 | no | yes |
| AgentMovementSystem | 120 | 1 | yes | yes |
| DraftSystem | 121 | 1 | yes | yes |
| StewardDefenceSystem | 122 | 1 | yes | yes |
| BillSystem | 123 | 60 | yes | yes |
| FleeSystem | 123 | 3 | yes | yes |
| JobAssignSystem | 124 | 1 | yes | yes |
| AgentDecisionSystem | 125 | 1 | yes | no |
| ColonyMindSystem | 125 | 1 | no | yes |
| StewardSystem | 125 | 360 | yes | yes |
| StaleSeekTargetSystem | 126 | 1 | yes | yes |
| JobExecuteSystem | 127 | 1 | yes | yes |
| HungerDecaySystem | 130 | 1 | yes | yes |
| ThirstDecaySystem | 131 | 1 | yes | yes |
| SleepDecaySystem | 132 | 1 | yes | yes |
| ConstructionSystem | 133 | 1 | yes | yes |
| SocialInteractionSystem | 134 | 1 | yes | yes |
| SocialDecaySystem | 135 | 1 | yes | yes |
| MemorySystem | 136 | 1 | yes | yes |
| CombatSystem | 137 | 1 | yes | yes |
| SettlementSystem | 138 | 1 | yes | yes |
| StarvationSystem | 139 | 1 | yes | yes |
| ResourceRegenSystem | 140 | 120 | yes | yes |
| StockpileDepositSystem | 141 | 10 | yes | yes |
| StockpileConsumeSystem | 142 | 5 | yes | yes |
| ShelterPlanningSystem | 143 | 1 | yes | yes |
| MaturationSystem | 144 | 60 | yes | yes |
| PairingSystem | 144 | 1 | yes | yes |
| BedAssignmentSystem | 145 | 1 | yes | yes |
| AgingSystem | 146 | 60 | yes | yes |
| HearthFeedSystem | 146 | 60 | yes | yes |
| FireSystem | 147 | 60 | yes | yes |
| FissionSystem | 148 | 1 | yes | yes |
| RelationsSystem | 149 | 1 | yes | yes |
| FertilitySystem | 150 | 1440 | yes | yes |
| RoomSystem | 152 | 1 | yes | yes |
| OralTransmissionSystem | 160 | 1440 | yes | yes |
| ObservationSystem | 161 | 30 | yes | yes |
| KnowledgeDecaySystem | 162 | 60 | yes | yes |
| DiscoverySystem | 163 | 240 | yes | yes |
| TechAggregateSystem | 164 | 240 | yes | yes |
| StorytellerSystem | 170 | 1440 | yes | yes |
| AnimalSystem | 171 | 3 | yes | yes |
| MoodSystem | 172 | 60 | yes | yes |
| QuarrelSystem | 173 | 6 | yes | yes |
| NeighbourSystem | 174 | 1440 | yes | yes |
| VisitorSystem | 175 | 3 | yes | yes |
| DiagnosisSystem | 250 | 60 | yes | yes |
| RequestsSystem | 251 | 60 | yes | yes |
| InfluenceVisualizationSystem | 1000 | 6 | yes | yes |
<!-- SYSTEM_WORLDS:END -->

## Integration seam for the switch

`sim_systems::register_world_runtime_systems(engine, WorldKind)` is an explicit entry point; it currently delegates both worlds to unchanged default registration. Existing production callers are unchanged in this preparatory change. Add the colony mind row and constructor selection together at the switch, retaining priority125 and interval1 and the pinned baseline sequence.

`worldgen::build_world_with` currently registers before assigning requested rules. `save::load_with` currently registers on a fresh default engine before installing saved resources. Both must pass the requested/decoded world kind (or install validated rules before dispatch) before world-specific registration can be activated. Do not infer colony from the fresh engine's defaults. Saved system names must match by position; migration/rejection for old colony saves is a separate required change. Preserve equal-priority ordering, including Bill/Flee at123, Decision/Steward at125, Maturation/Pairing at144, and Aging/HearthFeed at146.

After Cargo permission: `cd rust && cargo test -p sim-systems --lib registration_tests -- --nocapture`. Full baseline identity remains a separate required gate; an unrun metadata test proves no trajectory preservation.

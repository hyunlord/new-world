# M-CORE initial per-system measurement

Status: Part1 BASE design share measured; arm remeasurement and final paired comparison pending.

Admission follows [DECISIONS D10](CORE_DECISIONS.md): each repeat starts at one-minute load <8, with 1/5/15-minute loads recorded. Both external admission and benchmark JSON start samples pass for all nine repeats. Post-run loads are diagnostic only. Three scene rounds ran in one session; this is BASE-only and cannot substitute for the later interleaved BASE/M-CORE comparison.

BASE: `9855bd082ad88a4c49296410f206f148d98c4df9`. Frozen binary and raw evidence: `/Users/rexxa/work/core-data/base-bin/` and `/Users/rexxa/work/core-data/base-perf-ledger-1/`.

| Population | Tick mean us | Decision mean us | Decision share |
|---|---:|---:|---:|
| n200 | 424.806019 | 280.263230 | 0.659744020 |
| n1000 | 2092.741204 | 1462.832342 | 0.699002982 |
| n2000 | 4807.145139 | 3576.836370 | 0.744066648 |

At N1000, **s = 0.6990029820337981** (ratio of the three decision means to the three total-tick means). The final total-tick ratio bar is `1 - s/2 + 0.05 = 0.7004985089831011`. The final mind/old-decision ratio bar is `0.5`. These are thresholds, not achieved improvements.

All per-system rows remain in each raw JSON. `audit.json` SHA-256: `928d1fde99b32158ab8c667d444a9d7855e20ed402c662a06f1c03d835af0392`. Initial arm elapsed values remain INVALID; functional counts are unaffected. No final performance or milestone acceptance is claimed.

## Per-system means (us per simulation tick)

Costs use measured world ticks as denominator, including systems with cadence >1.

| System | N200 | N1000 | N2000 |
|---|---:|---:|---:|
| AgentDecisionSystem | 280.263230 | 1462.832342 | 3576.836370 |
| JobAssignSystem | 48.885864 | 181.595299 | 335.720870 |
| AgentMovementSystem | 14.642743 | 123.515789 | 210.042270 |
| SettlementSystem | 21.913347 | 103.155719 | 210.443710 |
| MemorySystem | 13.005904 | 86.774357 | 224.346858 |
| QuarrelSystem | 6.916101 | 31.182215 | 57.514732 |
| StockpileConsumeSystem | 3.453088 | 15.961735 | 35.203585 |
| CombatSystem | 3.031453 | 11.524837 | 25.398468 |
| AnimalSystem | 2.244951 | 9.129682 | 18.406999 |
| StaleSeekTargetSystem | 2.399023 | 7.850628 | 15.830628 |
| StarvationSystem | 3.671540 | 7.530583 | 11.900627 |
| StockpileDepositSystem | 1.714497 | 7.503338 | 15.717072 |
| InfluenceUpdateSystem | 6.606775 | 7.501447 | 8.502460 |
| SocialInteractionSystem | 1.965134 | 4.749542 | 7.921709 |
| SleepDecaySystem | 1.651964 | 3.690006 | 5.917119 |
| ConstructionSystem | 1.678251 | 3.422200 | 5.171181 |
| HungerDecaySystem | 1.219374 | 3.059064 | 5.167773 |
| MoodSystem | 0.639896 | 3.000541 | 5.969700 |
| ThirstDecaySystem | 0.901553 | 2.718210 | 4.972685 |
| SocialDecaySystem | 0.911623 | 2.673511 | 5.218302 |
| JobExecuteSystem | 1.020006 | 1.622419 | 1.995466 |
| AgentInfluenceSampleSystem | 1.042256 | 1.476440 | 1.599312 |
| StewardDefenceSystem | 0.858347 | 1.341347 | 1.885434 |
| KnowledgeDecaySystem | 0.293422 | 1.335537 | 2.669961 |
| DraftSystem | 0.789975 | 1.165355 | 1.379991 |
| ResourceRegenSystem | 0.625217 | 0.985986 | 1.001069 |
| OralTransmissionSystem | 0.115480 | 0.934144 | 2.831274 |
| TechAggregateSystem | 0.175752 | 0.640683 | 1.257913 |
| FleeSystem | 0.295303 | 0.458280 | 0.551619 |
| InfluenceVisualizationSystem | 0.386587 | 0.440144 | 0.500701 |
| AgingSystem | 0.116117 | 0.412930 | 0.835508 |
| DiscoverySystem | 0.121927 | 0.408840 | 0.785841 |
| FireSystem | 0.097569 | 0.362022 | 0.692072 |
| PairingSystem | 0.068160 | 0.198746 | 0.369241 |
| BedAssignmentSystem | 0.046772 | 0.131466 | 0.295815 |
| ObservationSystem | 0.061539 | 0.091676 | 0.100820 |
| RelationsSystem | 0.033480 | 0.074872 | 0.172518 |
| FertilitySystem | 0.021417 | 0.060638 | 0.114178 |
| ShelterPlanningSystem | 0.018735 | 0.028769 | 0.066952 |
| BuildingStampSystem | 0.018497 | 0.026950 | 0.058880 |
| MaturationSystem | 0.018163 | 0.025439 | 0.028608 |
| BillSystem | 0.015890 | 0.024315 | 0.026934 |
| RoomSystem | 0.016661 | 0.024196 | 0.046160 |
| VisitorSystem | 0.008688 | 0.024185 | 0.053640 |
| FissionSystem | 0.018275 | 0.023398 | 0.052884 |
| HearthFeedSystem | 0.012307 | 0.022661 | 0.023274 |
| StorytellerSystem | 0.002754 | 0.003260 | 0.003414 |
| StewardSystem | 0.000593 | 0.000574 | 0.000680 |
| NeighbourSystem | 0.000468 | 0.000448 | 0.000521 |

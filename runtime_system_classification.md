# Runtime System Classification

## Modern
The following runtime systems are registered through typed `RuntimeSystemId` manifest entries and `register_runtime_system(...)`:

- `Reputation`
- `SocialEvent`
- `Morale`
- `Value`
- `JobSatisfaction`
- `EconomicTendency`
- `Intelligence`
- `Memory`
- `Coping`
- `Network`
- `Occupation`
- `Contagion`
- `Age`
- `JobAssignment`
- `Mortality`
- `MentalBreak`
- `TraumaScar`
- `TraitViolation`
- `Emotion`
- `Stress`
- `Needs`
- `UpperNeeds`
- `ResourceRegen`
- `ChildStressProcessor`
- `Steering`
- `Movement`
- `Childcare`
- `Leader`
- `Title`
- `StratificationMonitor`
- `Tension`
- `BuildingEffect`
- `Migration`
- `Population`
- `TechUtilization`
- `TechMaintenance`
- `TechDiscovery`
- `TechPropagation`
- `Gathering`
- `Construction`
- `Family`
- `Intergenerational`
- `Parenting`
- `StatsRecorder`
- `StatSync`
- `StatThreshold`
- `Behavior`
- `SettlementCulture`
- `Chronicle`
- `PersonalityMaturation`
- `PersonalityGenerator`
- `Attachment`
- `AceTracker`
- `Trait`
- `LlmRequest`
- `LlmResponse`
- `LlmTimeout`
- `StorySifter`

## Bridge
- `display_label()`
  - debug/UI readability only
- `perf_label()`
  - compatibility lookup for existing engine perf names only
- `SimSystem::name()`
  - engine log/perf string only
  - not used for registry identity, registration, ordering, or boot dispatch
- `runtime_get_registry_snapshot()`
  - exposes typed IDs with display labels to Godot

## Legacy
- None in the active runtime registry authority path.

## Obsolete
- `register_system(...)` command path
- `runtime_clear_registry`
- string key normalizers / alias layers

## Classification Summary
- Active runtime systems: modern
- Bridge/debug labels: bridge
- String-key registration and alias dispatch: obsolete
- Active legacy authority systems: 0

# Legacy Deletion Report

## Deleted Earlier in the WS-REF-004C Branch

| File | Why Deleted | Replacement | Reference Check |
|---|---|---|---|
| `scripts/core/combat/combat_resolver.gd` | Dead legacy simulation residue with no active boot/runtime owner | Rust combat/danger runtime path | No active boot/runtime references remained |
| `scripts/core/simulation/runtime_shadow_reporter.gd` | Dead runtime-shadow helper that implied parallel simulation authority | Rust registry/runtime validation plus bridge debug surfaces | No active boot/runtime references remained |

## Deleted in This Pass
- none

## Why No Additional Script Deletion Was Safe Yet
- `EntityManager`, `BuildingManager`, `SettlementManager`, `RelationshipManager`, `ReputationManager`, and `ResourceMap` still have active fallback references from UI/setup/save paths.
- `ChronicleSystem` and `MemorySystem` remain active observer/debug infrastructure.
- `personality_generator.gd`, `intelligence_generator.gd`, and `value_system.gd` are now boot-idle, but they are still reachable through the deprecated local spawn helper in `EntityManager`.

## Safe Reductions Applied Instead of Deletion
- stopped boot-time initialization of `NameGenerator`
- moved entity legacy spawn helper init to lazy path only
- removed active camera/building renderer dependency on shadow managers where runtime-backed reads already exist

# T-490: Integration Wiring

## Action: DIRECT
## Files: scenes/main/main.gd, scripts/ai/behavior_system.gd

### main.gd:
1. Preload SettlementManager, MigrationSystem
2. var settlement_manager, migration_system
3. Init settlement_manager, create initial settlement at world center
4. Init migration_system with all refs
5. Register migration_system in sim_engine (priority 60)
6. Assign all initial entities to settlement 1
7. Pass settlement_manager to HUD, save_manager
8. Tab key: toggle resource overlay visibility
9. Update _save_game/_load_game calls with settlement_manager

### behavior_system.gd:
- In execute_tick(): skip entities with current_action == "migrate"
- Add init parameter for settlement_manager (optional, backward compat)

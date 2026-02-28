# Phase C-1a: TechNode V2 Schema + Knowledge Persistence System

## Classification Table

| Ticket | Description | 🟢/🔴 | Tool | Status |
|--------|-------------|--------|------|--------|
| T1 | `knowledge_type.gd` — create | 🔴 DIRECT | Task agent | Done |
| T2 | `tech_state.gd` — create | 🔴 DIRECT | Task agent | Done |
| T3 | `civ_tech_state.gd` — create | 🔴 DIRECT | Task agent | Done |
| T4 | `tech_schema_validator.gd` — create | 🔴 DIRECT | Task agent | Done |
| T5 | `localization/{en,ko}/tech.json` — create | 🔴 DIRECT | Task agent | Done |
| T6 | `game_config.gd` — V2 constants | 🔴 DIRECT | — | Done |
| T7 | `simulation_bus.gd` — 4 signals | 🔴 DIRECT | — | Done |
| T8 | `settlement_data.gd` — tech_states | 🔴 DIRECT | — | Done |
| T9 | `tech_tree_manager.gd` — V2 overhaul | 🔴 DIRECT | — | Done |
| T10 | `tech_discovery_system.gd` — V2 adapt | 🟢 DISPATCH | Task agent | Done |
| T11 | Migrate 5 V1 JSONs to V2 | 🟢 DISPATCH | Task agent | Done |
| T12 | Fix discovered_techs UI refs | 🟢 DISPATCH | Task agent | Done |

**Dispatch ratio: 3/12 = 25%** (ask_codex unavailable, fell back to Task agents)

## Verification
- Godot headless: `Loaded 5 tech definitions`, no new errors
- `discovered_techs` grep: only in V1 migration (settlement_data.gd:130-131)
- `class_name` in tech/: 0 (Godot 4.6 headless compat)
- Hardcoded UI strings: 0
- Locale keys: 21 en = 21 ko
- All 5 V2 JSONs valid

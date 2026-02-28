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

---

# C-1b-patch: Tier 0 Tech Discovery Speed Balancing

## Analysis

Prompt assumed per-tick discovery checks with `discovery_chance_per_tick: 0.001`.
Actual system: **annual checks** (`TECH_DISCOVERY_INTERVAL_TICKS = 4380`), field is `discovery.base_chance_per_year`.
`tier` field already exists in all 169 JSONs. `discovery_tier` does NOT need to be added.

V2 JSONs (C-1b) already differentiated values (T0: 0.20-0.40, T1: 0.05-0.20, T2: 0.02-0.06, T3: 0.005-0.02).
But with annual checks, even 0.99 base means minimum 4380 ticks (~2.4 min at 3x) before first discovery.

**Fix**: Reduce check interval to monthly (365 ticks) + convert probability math + boost Tier 0 values.

## Classification Table

| Ticket | Description | 🟢/🔴 | Tool | Status |
|--------|-------------|--------|------|--------|
| C-1b-P1 | GameConfig: TECH_DISCOVERY_INTERVAL_TICKS 4380→365 | 🔴 DIRECT | — | ✅ Done |
| C-1b-P2 | TechDiscoverySystem: annual→per-check conversion | 🔴 DIRECT | — | ✅ Done |
| C-1b-P3 | Boost 25 stone_age/*.json base_chance_per_year | 🔴 DIRECT | python script | ✅ Done |
| C-1b-P4 | Validate JSON + verify discovery math | 🔴 DIRECT | python script | ✅ Done |

## Verification

- 169 JSON files: all valid ✅
- Tier distribution: T0=25, T1=57, T2=63, T3=22, T4=2
- Tier 0 discovery @3x speed: 12-100 sec (fire guaranteed first monthly check)
- Tier 1 discovery @3x speed: 5-9 min
- Tier 2-4: gated by required_population + required_skills (real differentiation > probability alone)
- No localization changes needed (tier is internal metadata)
- TechDiscoverySystem: annual→per-check conversion preserves same expected annual rates for Tier 1+

**Dispatch ratio: 0/4 = 0%**

Justification: Prompt spec was based on incorrect assumptions about the discovery system
(per-tick vs per-year, field names, schema). All changes are shared interface modifications
or batch corrections requiring probability conversion math. Each <50 lines, tightly coupled.

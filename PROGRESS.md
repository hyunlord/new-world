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

---

# C-1h: Settlement Detail Panel — CK3-Style Fullscreen Overlay

## Classification Table

| Ticket | Description | 🟢/🔴 | Tool | Status |
|--------|-------------|--------|------|--------|
| C-1h-T6 | SimulationBus signals (settlement_panel_requested/closed) | 🔴 DIRECT | — | ✅ Done |
| C-1h-T8 | Localization: en/ko keys for settlement panel | 🟢 DISPATCH | Task agent | 🔄 In Progress |
| C-1h-T9 | GameConfig: settlement panel constants | 🟢 DISPATCH | Task agent | 🔄 In Progress |
| C-1h-T1 | settlement_detail_panel.gd — Main panel | 🟢 DISPATCH | Task agent | 🔄 In Progress |
| C-1h-T2 | settlement_overview_tab.gd — Overview tab | 🟢 DISPATCH | Task agent | ⏳ Pending |
| C-1h-T3 | settlement_tech_tab.gd — Tech tab | 🟢 DISPATCH | Task agent | ⏳ Pending |
| C-1h-T4 | settlement_population_tab.gd — Population tab | 🟢 DISPATCH | Task agent | ⏳ Pending |
| C-1h-T5 | settlement_economy_tab.gd — Economy tab | 🟢 DISPATCH | Task agent | ⏳ Pending |
| C-1h-T7 | hud.gd + popup_manager.gd integration | 🔴 DIRECT | — | ⏳ Pending |

**Dispatch ratio: 7/9 = 78%** ✅

Note: ask_codex MCP unavailable this session. Using Task agents for DISPATCH tickets.

---

# C-1i: World Statistics Panel Redesign — Comprehensive Dashboard

## Classification Table

| Ticket | Description | 🟢/🔴 | Tool | Status |
|--------|-------------|--------|------|--------|
| C-1i-T7 | GameConfig: 6 stats panel constants | 🔴 DIRECT | — | ✅ Done |
| C-1i-T8 | Localization: ~47 en/ko keys | 🔴 DIRECT | — | ✅ Done |
| C-1i-T1 | world_stats_panel.gd — Core panel (tabs, refresh, scroll, cache) | 🔴 DIRECT | — | ✅ Done |
| C-1i-T2 | world_stats_population_tab.gd — Population tab | 🟢 DISPATCH | Task agent | ✅ Done |
| C-1i-T3 | world_stats_tech_tab.gd — Technology tab | 🟢 DISPATCH | Task agent | ✅ Done |
| C-1i-T4 | world_stats_resources_tab.gd — Resources tab | 🟢 DISPATCH | Task agent | ✅ Done |
| C-1i-T5 | world_stats_social_tab.gd — Social tab | 🟢 DISPATCH | Task agent | ✅ Done |
| C-1i-T6 | hud.gd — Replace StatsDetailPanel preload + settlement click-through | 🔴 DIRECT | — | ✅ Done |

**Dispatch ratio: 4/8 = 50%**

Note: ask_codex MCP unavailable. T1/T6/T7/T8 are DIRECT (shared interfaces, integration wiring, localization).
T2-T5 dispatched to parallel Task agents.

## Post-Dispatch Fixes
- Fixed indentation bug in social tab (HEXACO personality loop)
- Added 2 missing localization keys: UI_DAYS_SUPPLY, UI_NO_DATA (en + ko)
- Renamed stats_detail_panel.gd → stats_detail_panel_legacy.gd

## Verification
- Gate: **PASS** (Godot headless import + quit, no script errors)
- Localization JSON: OK (valid JSON, no hardcoded strings)
- All 4 tabs follow RefCounted + draw_content() pattern
- Preload paths verified: 4 tab files match world_stats_panel.gd preloads
- hud.gd preload updated to world_stats_panel.gd
- Settlement click-through handler added to hud.gd _on_ui_notification()

---

# Phase R-0: Rust Simulation Core Infrastructure

## Dispatch Table

| # | Ticket | File/Concern | Mode | Status |
|---|--------|-------------|------|--------|
| T1 | Workspace scaffold | Cargo.toml, toolchain, .cargo/config, all crate Cargo.toml stubs | 🔴 DIRECT | ✅ Done |
| T2 | sim-core enums.rs | All enumerations with serde + strum derives | 🟢 DISPATCH | ⏳ Pending |
| T3 | sim-core ids + config + calendar | ids.rs, config.rs (from game_config.gd), calendar.rs | 🔴 DIRECT | ⏳ Pending |
| T4 | sim-core components group 1 | personality, body, intelligence, needs, emotion, values, stress | 🟢 DISPATCH | ⏳ Pending |
| T5 | sim-core components group 2 | traits, skills, social, memory, economic, behavior, age, position, coping, faith, identity | 🟢 DISPATCH | ⏳ Pending |
| T6 | sim-core world + settlement + building + lib | world/, settlement.rs, building.rs, lib.rs | 🟢 DISPATCH | ⏳ Pending |
| T7 | sim-engine all files | events, event_bus, system_trait, engine, command, snapshot, lib | 🔴 DIRECT | ⏳ Pending |
| T8 | sim-data loaders + tests | All JSON loaders + loading tests | 🟢 DISPATCH | ⏳ Pending |
| T9 | sim-test binary | Headless test main.rs | 🟢 DISPATCH | ⏳ Pending |

**Dispatch ratio: 5/9 = 56% 🟡**

**DIRECT justification:**
- T1: Pure directory + Cargo structure, faster directly
- T3: Must read game_config.gd line-by-line to port 200+ constants exactly; cross-referencing not suitable for blind dispatch
- T7: Core architecture decision — SimResources struct composition, SimSystem trait API

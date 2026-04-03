# WorldSim — CLAUDE.md

> Domain-specific rules live in subdirectory CLAUDE.md files.
> This file covers project-wide context only.

---

## Agent Identity

You are a **senior Rust + Godot 4 simulation architect**.

Core expertise: Rust (hecs ECS, gdext GDExtension, serde, rayon), GDScript (UI/rendering only), tick-based simulation, event sourcing, Utility AI, data-driven design.

When working on this project:
- Think like an engine programmer. Cache-friendly data, minimal allocations per tick, deterministic simulation.
- **ALL simulation logic is Rust.** GDScript exists only for UI, rendering, input, and localization.
- Simulation correctness > rendering polish. Always.
- **Your primary job is to implement correctly, test thoroughly, and commit cleanly.**

---

## Behavioral Guidelines

Derived from [Andrej Karpathy's observations](https://x.com/karpathy/status/2015883857489522876). **Bias toward caution over speed.**

### 1. Think Before Coding
- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a change affects tick ordering, ECS component layout, or FFI bridge — call it out before touching code.

### 2. Simplicity First
- Minimum code that solves the problem. Nothing speculative.
- No features beyond what was asked. No abstractions for single-use code.
- No premature optimization. Current target ~500 entities, architecture for 10,000+.

### 3. Surgical Changes
- Don't touch EventBus event definitions, sim-core component structs, or config constants unless the ticket explicitly requires it.
- Don't "improve" adjacent code. Don't refactor things that aren't broken.
- If you see a problem outside your scope, note it — don't fix it.

### 4. Goal-Driven Execution
- Every action traces back to the original request.
- When complete: list what changed and why, what wasn't changed, and any risks.

---

## Project Vision

AI-driven god simulation (WorldBox + Dwarf Fortress + CK3).
Player observes/intervenes as god; AI agents autonomously develop civilization.
All mechanics grounded in academic research (psychology, sociology, demographics).

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Simulation Core** | Rust (hecs ECS, rayon, serde) | All tick logic, state, AI, systems |
| **Bridge** | Rust GDExtension (gdext crate) | FFI boundary: snapshot out, commands in |
| **Renderer/UI** | Godot 4.6 GDScript | Panels, renderers, HUD, camera, input |
| **Data** | RON + validation (legacy JSON migration during A-1) | Materials, recipes, structures, actions, content defs |
| **Localization** | Custom Autoload `Locale` (GDScript) | UI text only — NOT Godot tr() |
| **Events** | Rust EventBus | All state changes recorded as events |
| **AI** | Rust Utility AI → GOAP → ONNX → LLM | Phased evolution, all in Rust |

## Repository

- GitHub: https://github.com/hyunlord/new-world
- Working branch: **lead/main** (always)
- Never use other branches for lead work

---

## Architecture Decisions (v3.1 — 2026-03-08, final)

### 14 Day-1 Decisions (immutable)
1. LOD 4-Tier (LodTier component)
2. System frequency tiering (Hot/Warm/Cold)
3. Influence Grid (8-12 typed channels, no direct entity refs)
4. Sparse social relations (cap 100, BTreeMap)
5. serde on all components
6. Data-Driven ("Build like a mod" -- all content in RON)
7. Causal tracking (per-entity 32-event ring buffer)
8. Double-buffer + damping + Sigmoid saturation
9. Sim 20-30 TPS + render 60 FPS (Gaffer accumulator)
10. Reactive ECS (ChangeTracker)
11. Building 2-layer model (structural grid + furniture ECS)
12. Tag+threshold recipes (no ID references)
13. Cloninger TCI temperament 4-axis (gene -> temperament -> HEXACO)
14. World Rules 5-Slot (resource/space/agent/society/global)

### Material System
- Material-level (not element, not abstract resource)
- Material = content generator: properties auto-derive item stats
- Formula: damage = hardness * 1.2, speed = 5.0 / density, durability = hardness * density * 10
- All materials in RON: sim-data/materials/
- Tag+threshold recipes: [tag: "metal", min_hardness: 50] -- never material IDs
- New material = new `.ron` file only, zero `.rs` changes

### Building System (2-layer)
- Structural Grid: tile[x][y] = { wall_material, wall_hp, floor_material, roof, room_id }
  - Walls/floors/roofs = terrain-like data, NEVER ECS entities, zero per-tick updates
- Interactive Entities: furniture/equipment = hecs Entity, event-driven only
- Room detection: dirty-flagged BFS flood fill (~0.01ms incremental)
  - Room role = auto-assigned by furniture (StructureDef matching)
- Influence shielding: walls block propagation (stone 90%, wood 50%)
  - Room = Influence container (cached aggregate)
- Autonomous construction: GOAP + Blueprint templates
  - need detect -> structure lookup -> site choose -> blueprint stamp -> gather -> build walls -> roof -> furnish -> role recognize
- Render: 6 layers (floor -> wall_lower -> furniture -> agent -> wall_upper -> roof_alpha)

### Temperament Pipeline (Cloninger TCI temperament)
- 38D polygenic core -> T = sigma(W_PRS * G + epsilon) -> TCI 4-axis:
  - NS (Novelty Seeking) = dopamine -> exploration/impulsion
  - HA (Harm Avoidance) = serotonin -> avoidance/caution
  - RD (Reward Dependence) = norepinephrine -> sociality/empathy
  - P (Persistence) = corticostriatal -> perseverance/stubbornness
- TCI -> HEXACO bias -> 187 traits -> 33 values -> needs -> emotions -> behavior -> skills -> roles
- Mutable: dramatic events shift axes +/-0.1~0.3, 0~3 times per lifetime, cascading
- Awakening: latent != expressed, dramatic event releases latent temperament
- UI: internal f64, display as 4-humor label (choleric/sanguine/phlegmatic/melancholic/mixed)
- 9th semi-convergence: founder temperament distribution -> civilization direction

### World Rules (5 Slots)
- Resource Economy: yield modifiers, special resources, conversion prereqs, currency
- Space & Environment: special zone spawners, terrain mods, movement rules
- Agent: lifespan, essence/blessing/curse, special abilities
- Society: tech prereqs, hierarchy, trade/war rules
- Global Constants: day length, seasons, regen rates, disaster frequency
- Lifecycle: Settings -> Compile -> Runtime (Factorio-style)
- Composition: base + world + scenario + oracle, priority/merge/override
- Dynamic change: event-based only (on_action), no polling
- Performance: init ~50ms, tick cost 0
- RuleHistory: "why is this value X?" traceback

### LLM Integration (5 Layers)
- Layer 0 (pre-game): Natural language -> World Rules JSON IR -> RON (2-step LLM + 1-step compiler)
- Layer 1-2: Pure code (simulation logic)
- Layer 3: Leader/priest decisions + oracle text interpretation
- Layer 5 (oracle): Player oracle text -> agent personality-filtered interpretation
- Model: Qwen3.5 0.8B (Q4_K_M), llama-cpp-2, GBNF/JSON Schema
- Fine-tuning: QLoRA, 3 tasks (narrative 60% + oracle interpretation 25% + worldbuilding->IR 15%)

### Oracle System (3 Layers)
- Layer 1: Observation (always free) -- zoom, inspector, chronicle, "why?" UI
- Layer 2: Oracle intervention (costs faith 15~35) -- text input -> LLM -> agent interprets via personality
  - Key: agent can misinterpret oracle based on temperament (dramatic irony)
- Layer 3: World law intervention (costs faith 80+, irreversible) -- runtime World Rules patch
- Faith Economy: 0~100, accumulates from prayer, depletes from intervention, natural decay
- Intervention history UI: timeline of oracle actions + consequences + side effects

### Content Scale
| Phase | Materials | Objects | Actions | Tech | Agents | Buildings | World Rules |
|:-----:|:---------:|:-------:|:-------:|:----:|:------:|:---------:|:-----------:|
| 1     | ~20       | ~50     | ~35     | ~10  | ~1K    | furniture | resource+const |
| 2     | ~50       | ~100    | ~65     | ~20  | ~5K    | wall+room | +space+agent |
| 3     | ~75       | ~150    | ~95     | ~40  | ~10K   | +underground | +society+dynamic |
| 4+    | ~120      | ~200    | ~150    | ~55  | ~100K  | z-level   | all 5 slots |

### Effect Primitive Standard (6 types)
AddStat / MulStat / SetFlag / EmitStimulus / SpawnEvent / Schedule

### Performance Budget (10K agents)
- Agent sim: ~0.53ms (parallel)
- Building: <3ms (room + influence + render)
- Influence Grid: <0.6ms
- World Rules: init ~50ms, tick 0
- Total: ~4ms / 9ms = 44%, headroom 56%

## Current Phase: Pre-requisite Architecture (A-1 through A-10)

### A-1: Data-Driven RON (sim-data/) — MaterialDef/FurnitureDef/ActionDef/RecipeDef/StructureDef
### A-2: Influence Grid (sim-core/) — 8-12 channels, stamp/sample, wall blocking
### A-3: Effect Primitive (sim-core/) — 6 types, double-buffer, damping, sigmoid
### A-4: Causal tracking (sim-core/) — ring buffer 32 events, world log
### A-5: System frequency tiering — Hot/Warm/Cold tags
### A-6: Building tile grid (sim-core/) — tile[x][y], BFS room detection
### A-7: Tag+threshold recipe schema (sim-data/) — no ID refs
### A-8: Temperament pipeline (sim-core/) — TCI 4-axis, PRS 4x38 weights, bias functions
### A-9: World Rules slot (sim-data/) — WorldRuleset RON schema, loader, composition, base_rules.ron
### A-10: Misc — serde, BTreeMap, NetworkId, LodTier, Sparse relations (cap 100)

After A-1~A-10: Phase 1 (Survival + Material + Temperament, ~4 weeks)
Then: Phase 2 (Social + Band + Building Structure, ~4 weeks)

---

## Runtime Topology

```
┌─────────────────────────────────────────┐
│           Godot 4.6 (GDScript)          │
│  UI Panels │ Renderers │ Camera │ HUD   │
│  Locale    │ MapEditor │ Input  │       │
└──────────────────┬──────────────────────┘
                   │ GDExtension FFI (sim-bridge)
                   │ - get_frame_snapshot()
                   │ - get_entity_detail(id)
                   │ - push_command(cmd)
                   │ - tick() / set_speed()
┌──────────────────┴──────────────────────┐
│          Rust Simulation Core            │
│                                          │
│  sim-bridge  → GDExtension ↔ Rust types │
│  sim-engine  → Tick loop, scheduling     │
│  sim-systems → All 30+ simulation systems│
│  sim-core    → ECS world, components     │
│  sim-data    → RON loaders, validation   │
│  sim-test    → Headless test binary      │
└──────────────────────────────────────────┘
```

### What lives where

| Component | Location | Language | Reason |
|-----------|----------|----------|--------|
| SimulationEngine (tick loop) | `rust/crates/sim-engine/` | Rust | Performance core |
| All simulation systems (30+) | `rust/crates/sim-systems/` | Rust | Performance core |
| ECS components (entity data) | `rust/crates/sim-core/` | Rust | State ownership |
| Config constants | `rust/crates/sim-core/config.rs` | Rust | Needed by sim logic |
| EventBus (inter-system comms) | `rust/crates/sim-engine/event_bus.rs` | Rust | Replaces SimulationBus for sim |
| Data loading (RON→structs) | `rust/crates/sim-data/` | Rust | Type-safe parsing |
| GDExtension bridge | `rust/crates/sim-bridge/` | Rust | FFI boundary |
| UI Panels | `scripts/ui/panels/` | GDScript | UI is Godot's strength |
| Renderers | `scripts/ui/renderers/` | GDScript | 2D rendering |
| Camera, HUD, Input | `scripts/ui/` | GDScript | Engine-native input |
| Locale (i18n) | `scripts/core/locale.gd` | GDScript | UI text only |
| SimulationBus | `scripts/core/simulation/` | GDScript | UI event relay only |

### The FFI Boundary Rule

```
Rust → Godot: Frame snapshots (PackedArrays), event notifications
Godot → Rust: Player commands (spawn, speed change, etc.)

GDScript NEVER reads entity state directly.
GDScript ALWAYS goes through SimBridge.get_*() methods.
```

---

## Directory Structure

```
rust/                          ← see rust/CLAUDE.md
  crates/
    sim-core/                  ← see rust/crates/sim-core/CLAUDE.md
    sim-data/                  ← see rust/crates/sim-data/CLAUDE.md
    sim-systems/               ← see rust/crates/sim-systems/CLAUDE.md
    sim-engine/                ← see rust/crates/sim-engine/CLAUDE.md
    sim-bridge/                ← see rust/crates/sim-bridge/CLAUDE.md
    sim-test/                  ← Headless test binary
  tests/
  Cargo.toml                   ← Workspace manifest
scripts/
  core/                        ← see scripts/core/CLAUDE.md
    simulation/                — SimulationBus (UI relay), GameConfig (GDScript mirror)
    locale.gd                  — Locale Autoload (i18n)
    save_manager.gd
  systems/                     ← LEGACY — all tick systems migrated to Rust
  ai/                          ← LEGACY — behavior logic migrated to Rust
  ui/                          ← see scripts/ui/CLAUDE.md
    panels/                    — EntityDetailPanel, BuildingDetailPanel, etc.
    renderers/                 — WorldRenderer, EntityRenderer, BuildingRenderer
    hud.gd, camera_controller.gd
data/                          ← legacy JSON content during A-1 migration
  species/, traits/, stressors/, emotions/, buildings/, skills/, tech/
localization/
  en/, ko/, compiled/
tools/
  gate.sh
```

---

## Autoloads (Registration Order Matters)

| Order | Name | File | Purpose |
|-------|------|------|---------|
| 1 | GameConfig | scripts/core/simulation/game_config.gd | UI-side constants mirror (Rust owns authoritative values) |
| 2 | SimulationBus | scripts/core/simulation/simulation_bus.gd | UI event relay — receives events from SimBridge |
| 3 | SimulationBusV2 | scripts/core/simulation/simulation_bus_v2.gd | Extended event relay |
| 4 | Locale | scripts/core/locale.gd | i18n — `Locale.ltr()` for all UI text |
| 5 | SimBridge | rust/worldsim.gdextension | GDExtension entry — Rust simulation interface |
| 6 | ComputeBackend | scripts/core/compute_backend.gd | Runtime mode routing (Rust/GDScript) |

---

## Shared Interface Contracts

### EventBus (Rust side — sim-engine)
All system-to-system communication in the simulation core goes through the Rust EventBus.
- Event names: past tense (`EntityDied`, `TraumaRecorded`, `TechDiscovered`)
- Adding an event: update `sim-engine/src/events.rs` + document in `rust/crates/sim-engine/CLAUDE.md`
- Events are relayed to GDScript via SimBridge → SimulationBus signals

### SimulationBus (GDScript side — UI relay only)
SimulationBus is now a **read-only relay** for the UI layer.
- It receives events from SimBridge and re-emits them as Godot signals
- **GDScript systems NEVER emit simulation events** — all simulation logic is Rust
- Only UI-specific events (camera, selection, UI state) originate in GDScript

### Config Constants
- **Authoritative values** live in Rust: `sim-core/src/config.rs`
- GDScript `GameConfig` mirrors a subset for UI display purposes
- When changing a constant: update Rust first, then sync GDScript mirror if UI needs it

### ECS Components (Entity Data)
- All entity state lives in Rust ECS (hecs World)
- Adding a component: update `sim-core/src/components/` + document in `rust/crates/sim-core/CLAUDE.md`
- GDScript accesses entity data ONLY through SimBridge getter methods

---

## Coding Conventions

### Rust
- Module names: `snake_case`
- Types: `PascalCase`, methods: `snake_case`
- ECS components: plain structs, `#[derive(Clone, Debug)]` minimum
- Systems: `fn system_name(world: &mut hecs::World, resources: &Resources, events: &mut EventBus)`
- All f64 for simulation math (determinism)
- `#[cfg(test)]` unit tests in every module
- `/// doc comments` on all pub items
- No `unwrap()` in production code — `Result` or `.unwrap_or_default()`

### GDScript (UI only)
- `class_name` at top of every `.gd` file
- PascalCase classes, snake_case variables/functions
- Type hints required: `var speed: float = 1.0`
- **No hardcoded UI text** — all strings through `Locale.ltr("KEY")`
- Read entity data ONLY from SimBridge: `SimBridge.get_entity_detail(id)`
- **NEVER modify simulation state from GDScript**

---

## Localization Rules (Non-Negotiable)

- Use `Locale.ltr("KEY")` for ALL user-visible text
- Never use Godot's built-in `tr()` — we use a custom Locale Autoload
- Both `en/` and `ko/` JSON files must be updated for every new key
- Debug/log strings are exempt
- See the checked-in Claude `worldsim-code` skill mirror for full localization protocol

---

## Execution Workflow

Claude Code receives implementation prompts from claude.ai web (architecture/design layer).

### Workflow
```
claude.ai web              → Architecture design, prompt authoring (.md files)
  ↓ copy-paste .md prompt
Claude Code (lead/main)    → Direct implementation, testing, commit
```

### Pre-flight (every task)
```bash
git fetch origin
git checkout lead/main
git pull origin lead/main
```

### Post-work (every task)
```bash
# Gate MUST pass before commit
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings

# Commit
git add -A
git commit -m "[t-000] <description>"
git push origin lead/main
```

### Rules
- Always work directly on `lead/main` branch
- Always pull before starting
- Never leave uncommitted changes
- Gate (cargo test + clippy) must pass before every commit
- When a prompt has `<promise>TAG</promise>`, output the tag ONLY when ALL stories pass

---

### Harness-Driven Development (HDD)

Every feature implementation MUST follow RED → GREEN → GATE:

**RED (test first):**
1. Decompose feature into testable assertions
2. Add `harness_<feature>_<assertion>` test to `rust/crates/sim-test/src/main.rs`
3. Run: `cargo test -p sim-test harness_<name> -- --nocapture` → MUST FAIL

**GREEN (implement):**
1. Write minimum code to pass the test
2. Run: `cargo test -p sim-test harness_<name> -- --nocapture` → MUST PASS

**GATE:**
```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

**RETRY on failure:**
1. Read assertion error message
2. Add diagnostic `println!` INSIDE the test (never in game code)
3. Fix root cause
4. Max 3 retry attempts. If still failing: STOP and report to user.

**Harness test patterns:**
```rust
// Standard setup
let mut engine = make_stage1_engine(42, 20);
engine.run_ticks(N);

// ECS queries
let world = engine.world();
for (_, (behavior, identity)) in world.query::<(&Behavior, &Identity)>().iter() { ... }

// SimResources checks
let resources = engine.resources();
assert!(resources.settlements.values().any(|s| s.stockpile_stone > 0.0));
```

**Naming convention:** `harness_<category>_<assertion>`
Categories: `job`, `resource`, `building`, `band`, `territory`, `population`, `economy`

**When NOT to use sim-test (use Godot headless harness instead):**
- SimBridge FFI boundary issues
- Shader/rendering verification
- UI panel data display
- Save/load cycle

---

### Harness Pipeline (Mandatory for Simulation Code)

Any change to `rust/crates/sim-core/`, `sim-systems/`, `sim-engine/`, or `sim-bridge/`
MUST go through the harness pipeline:

```bash
bash tools/harness/harness_pipeline.sh <feature> <prompt.md> [--quick]
```

The pre-commit hook blocks sim-* commits without an APPROVED verdict.
See `tools/harness/README.md` for details.
See `.claude/skills/worldsim-harness/SKILL.md` for when to use --quick vs full.

---

### Codex MCP Dispatch

Claude Code can dispatch tasks to the Codex MCP server for parallel execution.

**Codex MCP 도구 사용법:**
- `codex` MCP tool — 모든 티켓의 기본 디스패치 (Rust, GDScript, 모든 언어)
- `codex-reply` MCP tool — 기존 세션 이어가기 (follow-up, 에러 수정)

**Usage pattern:**
1. Start a session via `codex` tool — returns `threadId`
2. Continue via `codex-reply` using that `threadId`
3. If harness test fails: `codex-reply` with error output → auto-fix → re-test

**HDD + Codex MCP loop:**
```
Claude Code:
  1. harness test 작성 (RED)
  2. codex MCP로 구현 디스패치
  3. codex-reply: "cargo test -p sim-test harness_X -- --nocapture"
  4. 실패 시 codex-reply에 에러 전달 → 수정 → 재시도 (max 3회)
  5. 통과 시 완료
```

**Dispatch rules:**
- 모든 티켓 → `codex` MCP tool (`approval-policy: "never"`, `sandbox: "workspace-write"`)
- follow-up 필요 시 → `codex-reply` with `threadId`
- Dispatch ratio ≥ 60%

---

## Common Mistakes [READ BEFORE EVERY TASK]

1. Writing simulation logic in GDScript instead of Rust
2. Hardcoded UI strings instead of `Locale.ltr("KEY")`
3. Using `tr()` instead of `Locale.ltr()`
4. GDScript directly modifying entity state instead of going through SimBridge commands
5. Magic numbers in Rust instead of `config::` constants
6. Missing `#[cfg(test)]` unit tests in Rust modules
7. Using `unwrap()` in production Rust code
8. Modifying ECS component structs without updating SimBridge snapshot serialization
9. "Improving" code outside ticket scope
10. Not pulling latest before starting work (stale base → merge conflicts)
11. Committing without running gate (cargo test + clippy)
12. Modifying files outside the prompt's explicit scope
13. Forgetting `ko/` translations when adding localization keys
14. Adding Godot-specific types in hot-path Rust code
15. Using `String` matching in Rust hot paths instead of enums
16. Using manual pixel offsets (`offset_left`, `offset_bottom`) instead of Container layout for UI positioning
17. Calling `panel.visible = true` before `panel.set_data()` -- empty panel gets size (0,0)
18. Calling `queue_free()` on children before cache check -- causes flicker every refresh cycle
19. Connecting anonymous lambdas to signals in `_refresh()` without disconnecting old ones -- accumulates N connections
20. Fixing the same UI problem 3+ times with parameter tweaks instead of architectural fix
21. Not checking `is_inside_tree()` before operating on dynamically added panels

---

## Subdirectory CLAUDE.md Files

| Path | Covers |
|------|--------|
| `rust/CLAUDE.md` | Workspace structure, build commands, crate dependency graph |
| `rust/crates/sim-core/CLAUDE.md` | ECS components, World data, config constants |
| `rust/crates/sim-systems/CLAUDE.md` | All simulation systems, priorities, formulas |
| `rust/crates/sim-engine/CLAUDE.md` | Tick loop, EventBus, system scheduling |
| `rust/crates/sim-bridge/CLAUDE.md` | GDExtension FFI, snapshot format, command handling |
| `rust/crates/sim-data/CLAUDE.md` | RON data loading, validation, schema structs |
| `scripts/core/CLAUDE.md` | Locale, SimulationBus (UI relay), GDScript-side shared interfaces |
| `scripts/ui/CLAUDE.md` | UI layer rules, panel conventions, no simulation logic |
| `data/CLAUDE.md` | Legacy data rules and migration boundaries |

---

## Skills

Before any work touching these areas, read the corresponding SKILL.md:

| Skill | Path | When |
|-------|------|------|
| worldsim-code | `.claude/skills/worldsim-code/SKILL.md` | Any GDScript UI work (localization, patterns) |
| godot | `.claude/skills/godot/SKILL.md` | Godot scene/resource file work |
| systematic-debugging | `.claude/skills/systematic-debugging/SKILL.md` | Any bug or test failure |
| verification-before-completion | `.claude/skills/verification-before-completion/SKILL.md` | Before claiming any task complete |
---

## Current State (2026-03-17)

### UI Architecture — Migration In Progress
- **Agent inspector**: `entity_detail_panel_v4.gd` (1015 lines) — BBCode RichTextLabel based
  - Known crash: `float()` on non-numeric Variant from Rust FFI → use `_safe_float()` helper
  - Known issue: [table=3] BBCode alignment imperfect for CJK labels
  - **Planned: Phase 1 redesign → Godot UI nodes (ProgressBar + Container) to replace BBCode**
- **Settlement/Building detail**: moved from center popup to sidebar, `_draw()` based
- **Sidebar**: 6 tabs (상세정보/연대기/세력/통계/역사/외교) + band/civ/settlement/building panels
- **Bottom bar**: Z1-Z5 zoom | overlays (food/danger/warmth/social/knowledge/resource) | 7 layers | Oracle | TPS/FPS
- **Minimap**: 140×140, bottom-left
- **Performance**: building_renderer conditional redraw, entity panel 0.5s throttle, snapshot cache

### Key Files (most frequently edited)
| File | Lines | What |
|------|:-----:|------|
| `scripts/ui/hud.gd` | ~4200 | Master HUD — bottom bar, sidebar, popups, all signal handling |
| `scripts/ui/panels/entity_detail_panel_v4.gd` | ~1015 | Agent inspector (extends v3) |
| `scripts/ui/panels/entity_detail_panel_v3.gd` | ~1330 | Base inspector — data loading, tab infrastructure |
| `scripts/ui/renderers/entity_renderer.gd` | ~1590 | Agent rendering + click handling + settlement boundaries |
| `scripts/ui/renderers/building_renderer.gd` | ~330 | Building shapes + interiors + conditional redraw |
| `scripts/ui/camera_controller.gd` | ~735 | 5-stage zoom + camera limits |
| `rust/crates/sim-bridge/src/lib.rs` | ~8200 | ALL Rust↔Godot FFI |

### Known Crash Pattern
When accessing Rust FFI data in GDScript, field types can vary by entity:
- Entity A: `hex_c = 0.65` (float) → `float(0.65)` OK
- Entity B: `hex_c = {facets: [...]}` (Dictionary) → `float(Dictionary)` → CRASH

**Rule: NEVER use raw `float(dict.get("key"))`. Always use `_safe_float(dict, "key", default)`.**

### GDScript Safety Pattern
```gdscript
# BAD — crashes on non-numeric Variant:
var value: float = float(_detail.get("hex_c", 0.0))

# GOOD — safe type conversion:
func _safe_float(dict: Dictionary, key: String, default_value: float) -> float:
    var raw: Variant = dict.get(key, default_value)
    if raw is float or raw is int:
        return clampf(float(raw), 0.0, 1.0)
    return clampf(default_value, 0.0, 1.0)
```

---

## Harness MCP — Runtime Verification

Installed at `addons/harness/`. Provides MCP tools to run simulation and check invariants from Claude Code.

### Files
- `addons/harness/harness_server.gd` — WebSocket JSON-RPC server (activates on `--headless`)
- `addons/harness/harness_router.gd` — Command dispatcher
- `addons/harness/harness_invariants.gd` — Invariant checks
- `addons/harness/worldsim_adapter.gd` — **WorldSim-specific API bridge** (edit this when WorldSim API changes)
- `addons/harness/worldsim_mapping.md` — Interface mapping reference

### MCP Server
Register in `.mcp.json`:
```json
{
  "mcpServers": {
    "godot-rust-harness": {
      "command": "python",
      "args": ["-m", "godot_rust_harness"],
      "env": {
        "PROJECT_ROOT": "/Users/rexxa/github/Godot-Rust-MCP",
        "GODOT_BIN": "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot"
      }
    }
  }
}
```

### Usage
After simulation code changes:
- `godot_start` → starts WorldSim headless (HarnessServer auto-activates)
- `godot_reset(seed=42)` → resets tick counter + RNG
- `godot_tick(n=100)` → advance 100 ticks via `advance_ticks()`
- `godot_snapshot` → entity count + positions
- `godot_invariant` → run all 7 invariant checks
- `godot_stop` → terminate process
- `verify()` → full pipeline in one call

### Adapter architecture
WorldSim's `SimulationEngine` and `EntityManager` are `RefCounted` objects, not autoloads.
The adapter finds them via `get_tree().root.get_node_or_null("Main")`.
Edit **only** `worldsim_adapter.gd` when WorldSim's API changes; harness core stays generic.

### Reset limitation
`godot_reset` calls `sim_engine.init_with_seed(seed)` which resets tick+RNG but NOT entity population.
For full population reset, use `godot_stop` + `godot_start`.

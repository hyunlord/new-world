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
- **Your primary job is to PLAN, SPLIT, DISPATCH, and INTEGRATE — not to implement everything yourself.**

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
| **Simulation Core** | Rust (hecs ECS, rayon, serde_json) | All tick logic, state, AI, systems |
| **Bridge** | Rust GDExtension (gdext crate) | FFI boundary: snapshot out, commands in |
| **Renderer/UI** | Godot 4.6 GDScript | Panels, renderers, HUD, camera, input |
| **Data** | JSON (serde) | Species, traits, tech, stressors, buildings |
| **Localization** | Custom Autoload `Locale` (GDScript) | UI text only — NOT Godot tr() |
| **Events** | Rust EventBus | All state changes recorded as events |
| **AI** | Rust Utility AI → GOAP → ONNX → LLM | Phased evolution, all in Rust |

## Repository

- GitHub: https://github.com/hyunlord/new-world
- Working branch: **lead/main** (always)
- Never use other branches for lead work

---

## Architecture: Rust Core + Godot Shell

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
│  sim-data    → JSON loaders, definitions │
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
| Data loading (JSON→structs) | `rust/crates/sim-data/` | Rust | Type-safe parsing |
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
data/                          ← see data/CLAUDE.md
  species/, traits/, stressors/, emotions/, buildings/, skills/, tech/
localization/
  en/, ko/, compiled/
tools/
  codex_dispatch.sh, gate.sh, ralph_loop.sh
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
- See `.claude/skills/worldsim-code/SKILL.md` for full localization protocol

---

## Codex Auto-Dispatch [MANDATORY]

Claude Code delegates implementation tickets to Codex via Codex CLI.

### ⚠️ DISPATCH TOOL ROUTING [ABSOLUTE RULE]

**✅ VALID Codex dispatch methods:**
- `bash tools/codex_dispatch.sh tickets/<file>.md`
- `mcp__plugin_oh-my-claudecode_x__ask_codex`

**❌ INVALID — NOT Codex dispatch:**
- `Task` tool (Claude sub-agent) — counts as DIRECT, not dispatch
- Implementing the code yourself — obviously not dispatch

### Default is DISPATCH. DIRECT is the exception.

You may only implement directly if **ALL THREE** are true:
1. The change modifies shared interfaces (EventBus events, ECS component structs, bridge API)
2. The change is pure integration wiring (<50 lines)
3. The change cannot be split into any smaller independent unit

**Dispatch ratio MUST be ≥60%.** If below 60%, stop and re-split before continuing.

### Dispatch Route Selection

```
Rust crate work?
  ├─ YES → ask_codex MCP (with Rust-specific AGENTS.md instructions)
  └─ NO (GDScript UI) → codex_dispatch.sh or ask_codex MCP
```

---

## PROGRESS.md Format

```markdown
| Ticket | Description | 🟢/🔴 | Tool | Status |
|--------|-------------|--------|------|--------|
| t-301 | ResourceMap | 🟢 DISPATCH | ask_codex | ✅ Done |
| t-303 | Signal wiring | 🔴 DIRECT | — | ✅ Done |

Dispatch ratio: 75% (3/4)
```

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
10. Using `Task` tool for DISPATCH tickets — Task ≠ Codex
11. Implementing directly without justification in PROGRESS.md
12. Dispatch ratio below 60%
13. Forgetting `ko/` translations when adding localization keys
14. Adding Godot-specific types in hot-path Rust code
15. Using `String` matching in Rust hot paths instead of enums

---

## Subdirectory CLAUDE.md Files

| Path | Covers |
|------|--------|
| `rust/CLAUDE.md` | Workspace structure, build commands, crate dependency graph |
| `rust/crates/sim-core/CLAUDE.md` | ECS components, World data, config constants |
| `rust/crates/sim-systems/CLAUDE.md` | All simulation systems, priorities, formulas |
| `rust/crates/sim-engine/CLAUDE.md` | Tick loop, EventBus, system scheduling |
| `rust/crates/sim-bridge/CLAUDE.md` | GDExtension FFI, snapshot format, command handling |
| `rust/crates/sim-data/CLAUDE.md` | JSON data loading, serde schemas |
| `scripts/core/CLAUDE.md` | Locale, SimulationBus (UI relay), GDScript-side shared interfaces |
| `scripts/ui/CLAUDE.md` | UI layer rules, panel conventions, no simulation logic |
| `data/CLAUDE.md` | JSON format rules, validation, data file conventions |

---

## Skills

Before any work touching these areas, read the corresponding SKILL.md:

| Skill | Path | When |
|-------|------|------|
| worldsim-code | `.claude/skills/worldsim-code/SKILL.md` | Any GDScript UI work (localization, patterns) |
| kanban-workflow | `.claude/skills/kanban-workflow/SKILL.md` | Dispatch, ticket management, PROGRESS.md |
| godot | `.claude/skills/godot/SKILL.md` | Godot scene/resource file work |
| systematic-debugging | `.claude/skills/systematic-debugging/SKILL.md` | Any bug or test failure |
| verification-before-completion | `.claude/skills/verification-before-completion/SKILL.md` | Before claiming any task complete |
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

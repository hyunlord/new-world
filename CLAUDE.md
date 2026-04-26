# WorldSim — CLAUDE.md

> Domain-specific rules live in subdirectory CLAUDE.md files.
> This file covers project-wide context only.

## Session Startup (run FIRST before any work)

```bash
# Auto-install harness hooks (pre-commit + post-commit) if missing
bash tools/harness/install_hooks.sh 2>/dev/null || true
```

**Post-commit hook**: Every commit outputs a verification summary (bypass method + pipeline score).
Show this output to the user as-is — do not summarize or hide it.

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

### 5. Memory Axiom < Code Reality (added 2026-04-26)

Memory and roadmap claims about feature state are HINTS, not truth. Before
implementing any feature labeled "missing" or "미구현":

1. **Mandatory grep sweep** — search for the feature's likely symbols:
   ```bash
   grep -rn "<FeatureName>\|<FeatureType>" rust/crates/ | head
   find rust/crates/sim-core/src -name "<feature>*"
   grep -rn "use <feature>" rust/crates/ | head
   ```

2. **Verify before claiming**: 10 of 11 features in 2026-04-23~26 sessions
   were 85~100% already implemented. Memory said "missing"; reality showed
   only 1-2 entries or harness coverage missing.

3. **Production wiring check**: even if the type/struct exists, verify:
   - Is it registered in `DEFAULT_RUNTIME_SYSTEMS` (sim-bridge)?
   - Is it included in `BEHAVIOR_ACTION_ORDER` (cognition.rs)?
   - Does world.rs have a completion handler?
   - Is it exposed through SimBridge?

The "real gap" is usually: handler missing, harness missing, 1 enum
variant missing, 1 wiring entry missing — not the whole system.

### 6. Push Verification Mandatory (added 2026-04-26)

After every `git push`, verify origin reflection:

```bash
git push origin lead/main
git ls-remote origin refs/heads/lead/main | head    # MUST match local HEAD
git log --oneline origin/lead/main -3
```

Two push omissions occurred 2026-04-23 (a4-causal, hook-threshold).
This is now a hard requirement, not optional.

### 7. Hook Policy vs Code Quality (added 2026-04-26)

Pre-commit hook score threshold = 90 (lowered from 95 on 2026-04-23).

Reason: VLM analysis legitimately produces VISUAL_WARNING for stone-age
sims (conservative analyst behavior). Score -8 is environmental cost,
not code quality issue.

Quality gates (UNCHANGED):
- Evaluator verdict: must be APPROVE
- Tests: cargo test --workspace must pass
- Clippy: clean
- Regression: CLEAN

VLM WARNING alone never blocks merge. This is policy, not bug.

### 8. Overzealous Defense Awareness (added 2026-04-26)

When implementing isolation/security mechanisms, check what's actually
needed vs what was assumed. The vlm-login-env-fix discovery: `env -i`
isolation was preserving only 5 vars when 8-12 are needed for Claude
CLI auth.

Pattern: defense mechanisms tend to be too restrictive on first pass.
Validate by running an actual end-to-end test, not just by code review.

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

## Workspace Topology (Plan C+, established 2026-04-21)

WorldSim 개발은 **2 머신 / 2 repo**로 분리:

| Session | Machine | Repo | Workspace | Use |
|---------|---------|------|-----------|-----|
| Sprite session | DGX Spark (remote) | `worldsim-training` | `~/github/worldsim-training` | ComfyUI, Aseprite, asset generation |
| Game session | Local (Mac) | `new-world` | `~/github/new-world-wt/lead` | Game code, simulation, harness |

**규약**:
- Asset 전달은 PNG 직접 attachment (not branch sharing)
- 각 session은 자기 repo에만 push (cross-push 금지)
- Game session에서 `git remote -v` 검증 후 작업 시작
- 메모리의 repo 정보는 hint, 실제 작업 전 검증 필수

**위반 사례 (예방)**:
- 2026-04-21: 스프라이트 세션이 worldsim-training/main에 잘못 push → cherry-pick 복구
- 2026-04-23: A-4 push 누락 (push 후 origin 미확인)
- 2026-04-23: A-6 commit 후 hook threshold 차단 (정책 부재)

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

## Current Phase: Pre-requisite Architecture (A-1 through A-13)

**Progress (as of 2026-04-26)**: 9/13 prereqs complete.
- ✅ A-1, A-2, A-3, A-4, A-5, A-6, A-7, A-8, A-12, A-13
- 🟡 A-10 (partial: serde + coping BTreeMap + sparse rel cap done; remaining: NetworkId, LodTier, broader HashMap conversion)
- ❌ A-9 (World Rules slot system), A-11 (BodyHealth)

### A-1: Data-Driven RON (sim-data/) — MaterialDef/FurnitureDef/ActionDef/RecipeDef/StructureDef
### A-2: Influence Grid (sim-core/) — 8-12 channels, stamp/sample, wall blocking
### A-3: Effect Primitive (sim-core/) — 6 types, double-buffer, damping, sigmoid
### A-4: Causal tracking (sim-core/) — ring buffer 32 events, world log
### A-5: System frequency tiering — Hot/Warm/Cold tags
### A-6: Building tile grid (sim-core/) — tile[x][y], BFS room detection
### A-7: Tag+threshold recipe schema (sim-data/) — no ID refs
### A-8: Temperament pipeline (sim-core/) — TCI 4-axis, PRS 4x38 weights, bias functions
### A-9: World Rules slot (sim-data/) — WorldRuleset RON schema, loader, composition, base_rules.ron ❌
### A-10: Misc — serde, BTreeMap, NetworkId, LodTier, Sparse relations (cap 100) 🟡
### A-11: BodyHealth system (sim-core/) — HP, injury, recovery ❌
### A-12: Family/genealogy component (sim-core/) — parent/child links, lineage tracking ✅
### A-13: Knowledge learning system (sim-core/) — dual-axis skill/knowledge, learning events ✅

**Phase status**:
- Phase 1 'agents alive': 60% (temperament active, body/health pending)
- Phase 2 'shelter awareness → warmth/safety': ✅ COMPLETE (2026-04-26)
- Phase 3 'genealogy + knowledge dual-axis': ACTIVE (a12 + a13 working)

After A-1~A-13: Phase 1 (Survival + Material + Temperament, ~4 weeks)
Then: Phase 2 (Social + Band + Building Structure, ~4 weeks)

---

## Runtime Topology

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
# 1. Workspace verification (added 2026-04-26 — Plan C+ regulation)
git remote -v                    # MUST be hyunlord/new-world (not worldsim-training)
git branch --show-current        # MUST be lead/main
pwd                              # MUST be inside new-world (not worldsim-training)

# 2. Sync
git fetch origin
git checkout lead/main
git pull origin lead/main

# 3. Memory axiom check (added 2026-04-26)
# Before claiming any feature is "missing", run grep sweep first.
# See "Behavioral Guidelines" rule 5.
```

### Post-work (every task)
```bash
# 1. Gate MUST pass before commit
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings

# 2. Commit
git add -A
git commit -m "[t-000] <description>"

# 3. Push + verify (added 2026-04-26 — push omission protection)
git push origin lead/main
git ls-remote origin refs/heads/lead/main | head    # MUST match local HEAD
git log --oneline origin/lead/main -3
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

### Harness Pipeline (Mandatory for All Code Changes)

**ALL changes to code, shaders, assets, data, or scenes** MUST go through the harness pipeline.

## Harness Pipeline Modes

| Mode | Command | LLM Calls | Use When |
|------|---------|:---------:|----------|
| `--full` | `harness_pipeline.sh feat prompt.md --full` | 7-16 | New simulation features (sim-core/systems/engine) |
| `--quick` | `harness_pipeline.sh feat prompt.md --quick` | 4-10 | GDScript, shaders, sim-data/test/bridge, refactoring |
| `--light` | `harness_pipeline.sh feat prompt.md --light` | 2 | Assets, RON data files |

### Hook tiers (auto-detected by pre-commit)
- `sim-core/sim-systems/sim-engine` `.rs` → `--full` (Planning debate + Visual Verify + Evaluator)
- `.gd`, `.gdshader`, `sim-data/test/bridge` `.rs` → `--quick` (Visual Verify + Evaluator, no debate)
- `.png`, `.ron`, `.wav`, `.tscn`, `.tres` assets → `--light` (Visual Verify + VLM only)
- Docs, tools, config → free commit (no harness required)

### Bypass
```bash
HARNESS_SKIP=1 git commit -m "..."
```

The pre-commit hook blocks commits containing code/asset files without an APPROVED verdict.

**Exempt from pipeline** (commit normally with `HARNESS_SKIP=1`):
- Documentation only (.md, .txt)
- Harness infrastructure itself (tools/harness/*, .claude/skills/worldsim-harness/*)

**Requires pipeline** (even though non-code):
- Localization source files (localization/fluent/*, localization/ko/*, localization/en/*)
- Localization compiled files (localization/compiled/*)
- Localization registry (localization/key_registry.json)
- All game data (.ron, .json in data/, sim-data/)
- Any file that affects runtime behavior

**Rule: harness exemption = `tools/harness/` and `.claude/` files ONLY.
Everything under `localization/`, `rust/`, `scripts/`, `data/` requires pipeline.**

See `tools/harness/README.md` for details.
See `.claude/skills/worldsim-harness/SKILL.md` for mode selection guide.

**NEVER use `git commit --no-verify` to bypass the pipeline for code changes.**
The only acceptable use of `--no-verify` is for documentation-only commits or emergency hotfixes
(which must get a regression harness test added in the next commit).

---

### Codex MCP Dispatch

See `.claude/skills/worldsim-code/SKILL.md` for Codex MCP dispatch protocol.

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
16-21. UI-specific mistakes → see `scripts/ui/CLAUDE.md` "GDScript UI Patterns" section

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
| worldsim-code | `.claude/skills/worldsim-code/SKILL.md` | Any Rust/GDScript work (localization, coding standards, Codex dispatch) |
| worldsim-harness | `.claude/skills/worldsim-harness/SKILL.md` | Harness pipeline execution |
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

**FFI Safety**: Never use raw `float(dict.get("key"))` — use `_safe_float()`. See `scripts/ui/CLAUDE.md`.

---

## Harness MCP — Runtime Verification

See `tools/harness/README.md` and `addons/harness/worldsim_mapping.md` for details.

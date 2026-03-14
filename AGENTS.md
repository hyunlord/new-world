# WorldSim Codex Agent Instructions (v3.1)

## Pre-flight (MANDATORY — every task)

모든 작업 시작 전 반드시 실행. 예외 없음.
```bash
# 1. Sync with remote
git fetch origin
git checkout lead/main
git reset --hard origin/lead/main

# 2. Verify clean state
git status
# EXPECTED: "nothing to commit, working tree clean"
# If dirty: git reset --hard origin/lead/main

# 3. Gate check
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
# If fails: STOP and report. Do not start work on broken baseline.
```

### Post-work (MANDATORY — after every task)
```bash
# 1. Gate must pass
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings

# 2. Commit to lead/main directly
git add -A
git commit -m "[t-000] <description>"
git push origin lead/main

# 3. Create PR: lead/main → main, merge immediately
# Do NOT create separate worktrees or branches unless explicitly asked.
```

### Rules
- Always work directly on lead/main branch
- Always reset to origin/lead/main before starting
- Never leave uncommitted changes
- Never create worktrees unless the prompt explicitly requests it
- PR target is always lead/main → main

## Project Overview
WorldSim: god simulation where autonomous agents form civilizations from the Stone Age.
Rust (hecs ECS, rayon) + Godot 4 (render only). 1-person dev. Data-driven via RON.

## Architecture Rules (MUST follow)

### 14 Day-1 Decisions
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
13. TCI temperament 4-axis (gene -> temperament -> HEXACO pipeline)
14. World Rules 5-Slot (resource/space/agent/society/global)

### Critical Rules for Code Generation
- ALL simulation logic in Rust. GDScript is UI rendering ONLY.
- NEVER hardcode simulation parameters. Use RON data files.
- NEVER reference material IDs in recipes. Use tags + thresholds.
- NEVER make walls into ECS entities. Walls = tile grid data.
- NEVER update structural building data per tick. Building state changes are event-driven.
- NEVER poll for World Rules changes. Use Settings -> Compile -> Runtime and on_action events.
- NEVER put oracle/LLM interpretation logic in GDScript. UI sends text and commands only.
- ALL new content = new `.ron` file. Zero `.rs` changes when the schema already supports it.
- ALL f64 for simulation math (determinism).
- ALL shared components derive `Serialize` + `Deserialize`.
- Influence Grid is the sole ambient interaction medium. Do not wire direct entity references where typed channels suffice.
- Temperament is gene -> TCI -> HEXACO bias. Do not hardcode role/personality outcomes.

### Crate Structure
- sim-core: components, Influence Grid, Effect Primitives, CausalLog, TileGrid, Room, Temperament
- sim-data: RON loaders (`MaterialDef`, `RecipeDef`, `StructureDef`, `WorldRuleset`, `TemperamentRules`)
- sim-systems: RuntimeSystems (Hot/Warm/Cold), GOAP, recipe resolution, temperament
- sim-engine: tick loop, scheduling, double-buffer, World Rules lifecycle
- sim-bridge: Rust<->Godot FFI, snapshots, MultiMesh buffers, Oracle pipeline boundary
- sim-test: headless tests

### Key Formulas
- weapon_damage = material.hardness * 1.2
- weapon_speed = 5.0 / material.density
- weapon_durability = material.hardness * material.density * 10
- temperament = sigma(W_PRS_4x38 * genes_38D + epsilon)
- hexaco_init = base_distribution + temperament_bias_matrix_4x24 * temperament
- wall_insulation = material.density * 0.15 (stone ~0.9, wood ~0.5)

### Current Phase: A-1 through A-10 (Prerequisites)
A-1: RON loader | A-2: Influence Grid | A-3: Effect Primitives | A-4: CausalLog
A-5: Frequency tiering | A-6: Tile grid + BFS room | A-7: Tag recipes
A-8: Temperament (TCI) | A-9: World Rules slot | A-10: serde/BTreeMap/misc

### Gate Command (mandatory before commit)
```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

### Unwrap Audit
```bash
grep -rn '\.unwrap()' <new_files> | grep -v test  # expect 0 hits
```

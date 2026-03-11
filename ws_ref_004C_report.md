# WS-REF-004C Report

## Implementation Intent
This pass tightened the final simulation authority cutover at the active boot/runtime boundary. The main remaining ambiguity was that Godot still booted several legacy shadow helpers before proving Rust runtime authority, and it still initialized deprecated local spawn helpers that are not used by the active runtime. The goal here was to make the active gameplay path more explicitly Rust-owned without breaking the existing UI/setup shell.

## How It Was Implemented
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/scenes/main/main.gd)
  - moved Rust registry validation to immediately after `SimulationEngine.init_with_seed()`
  - aborts boot before shadow/bootstrap managers initialize if Rust registry authority is invalid
  - removed unused `Pathfinder` boot residue
  - removed active boot/save/load calls to `NameGenerator`
  - switched active boot wiring so:
    - `building_renderer` uses runtime-backed data only
    - camera no longer receives legacy `entity_manager` fallback on boot
- [scripts/core/entity/entity_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/scripts/core/entity/entity_manager.gd)
  - converted personality/intelligence/name legacy spawn helpers to lazy init
  - deprecated local spawn logic is no longer initialized during normal boot
- Added/refreshed reports:
  - [gdscript_full_authority_scan.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/gdscript_full_authority_scan.md)
  - [gdscript_authority_map.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/gdscript_authority_map.md)
  - [simulation_ownership_cutover_map.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/simulation_ownership_cutover_map.md)
  - [simulation_cutover_migration_plan.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/simulation_cutover_migration_plan.md)
  - [bridge_boundary_enforcement.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/bridge_boundary_enforcement.md)
  - [legacy_deletion_report.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/legacy_deletion_report.md)
  - [post_cutover_boot_validation.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/post_cutover_boot_validation.md)
  - [verify_simulation_authority_cutover.md](/Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover/verify_simulation_authority_cutover.md)

## What Feature It Adds
The active runtime path is now harder to misread as hybrid-authoritative:
- Rust registry authority is proven before Godot builds its legacy shadow shell
- boot no longer initializes deprecated local spawn-generation helpers that are unused by the active runtime
- active camera/render boot wiring leans further toward runtime-backed state and away from shadow-manager fallback

In practice, this makes the repository’s real authority boundary easier to trust:
- Rust owns runtime registration
- Rust owns the tick
- Rust owns gameplay mutation
- Godot remains shell/setup/UI/render/bridge, with legacy shadow helpers explicitly demoted

## Verification After Implementation
- Static evidence:
  - no active `register_system(...)` or `runtime_clear_registry` path on boot
  - no `NameGenerator.init/save_registry/load_registry` remains on the active gameplay path
  - `EntityManager.spawn_entity()` still exists, but has no repository callers
- Rust verification:
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Godot verification:
  - headless boot exits successfully
  - direct headless runtime harness passes

## Deleted Legacy Scripts
- No new script files were deleted in this pass.
- Previously deleted on the 004C branch and preserved:
  - `scripts/core/combat/combat_resolver.gd`
  - `scripts/core/simulation/runtime_shadow_reporter.gd`

## Remaining Technical Debt
- Godot still instantiates shadow managers (`EntityManager`, `BuildingManager`, `SettlementManager`, `RelationshipManager`, `ReputationManager`, `ResourceMap`) for UI/setup/save fallback.
- `ChronicleSystem` and `MemorySystem` are still Godot-side observer/archive logic.
- Pre-runtime setup world generation and resource painting still happen in Godot before Rust bootstrap.
- Several legacy scripts remain on disk and are only demoted, not deleted, because dependency chains still exist through fallback UI or deprecated local spawn paths.

## Follow-Up Refactors
- Replace remaining UI fallbacks on `entity_manager` / `building_manager` / `relationship_manager` with runtime-backed bridge getters.
- Move setup/editor world/resource authority into a Rust-backed bootstrap model.
- Delete `personality_generator.gd`, `intelligence_generator.gd`, `value_system.gd`, and `settlement_culture.gd` once the deprecated local spawn path is removed.
- Move chronicle/archive ownership behind Rust or a pure read-only bridge feed.

## In-Game Checks (한국어)
- 게임이 정상적으로 부팅되는지 확인한다.
  - Rust registry 검증이 실패하면 초반에 바로 부팅이 중단되어야 하고, 실패 원인이 경고로 남아야 한다.
- 에이전트가 정상적으로 생성되는지 확인한다.
  - setup 이후 bootstrap이 끝나면 Probe/Sandbox 모두 정상 인구가 보여야 한다.
- 시뮬레이션 tick이 Rust에서만 실행되는지 확인한다.
  - GDScript 쪽 시스템 등록/재등록 경고 없이 `runtime_tick_frame` 기반으로 상태가 변해야 한다.
- Godot 스크립트가 simulation state를 직접 변경하지 않는지 확인한다.
  - 카메라/렌더/UI를 움직여도 시뮬레이션이 그 때문에 재초기화되거나 registry가 바뀌면 안 된다.
- UI가 정상적으로 월드 상태를 렌더링하는지 확인한다.
  - building/entity render, HUD, detail panel, debug overlay가 그대로 떠야 한다.
- bridge를 통해 상태 스냅샷이 정상 전달되는지 확인한다.
  - headless harness와 runtime detail/summary 표시가 계속 동작해야 한다.
- 이상하면 이런 증상이 보인다:
  - 부팅 직후 registry mismatch로 멈춤
  - 에이전트가 spawn되지만 상태가 갱신되지 않음
  - 카메라/렌더러가 runtime-backed 데이터를 못 읽어 빈 화면처럼 보임
  - setup 이후에는 괜찮지만 legacy panel만 비어 있으면 shadow fallback 의존이 남아 있는 것이다

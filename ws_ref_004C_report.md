# WS-REF-004C Report

## Implementation Intent
- Finalize the active simulation authority cutover so that boot and tick ownership live in Rust ECS only.
- Remove dead legacy GDScript runtime systems that were no longer referenced after the WS-REF-004A/004B boot and registry refactors.
- Document the remaining non-authoritative shadow/bootstrap helpers so the repository truth is explicit instead of ambiguous.

## How It Was Implemented
- Reused the WS-REF-004A boot refactor:
  - `SimulationEngine` initializes Rust runtime and registers default systems through `sim-bridge`
  - `main.gd` no longer instantiates or registers legacy GDScript runtime systems
- Reused the WS-REF-004B typed runtime registry:
  - runtime registration is now backed by typed Rust system ids, not legacy string keys
- Removed runtime-unused legacy authority scripts under:
  - `scripts/ai`
  - `scripts/systems/biology`
  - `scripts/systems/cognition`
  - `scripts/systems/development`
  - `scripts/systems/psychology`
  - `scripts/systems/record`
  - `scripts/systems/social`
  - `scripts/systems/work`
  - `scripts/systems/world`
- Added:
  - `gdscript_authority_map.md`
  - `verify_sim_authority_cutover.md`
  - this final report

## What Feature It Adds
- The repository now has a clear active authority boundary:
  - Godot boots the shell, relays commands/events, and renders state
  - Rust sim-bridge and ECS own runtime system registration and simulation ticking
- Dead legacy runtime system files that could falsely imply GDScript tick authority are removed.
- Remaining shadow/bootstrap helpers are explicitly documented as non-authoritative residue instead of implicit sources of confusion.

## Verification After Implementation
- Static verification:
  - no `register_system(...)` calls remain in `main.gd` or `simulation_engine.gd`
  - no `runtime_clear_registry` use remains in the active boot path
  - Rust default runtime registration is referenced by both GDScript bridge and Rust sim-bridge
- Rust verification:
  - `cargo build -p sim-bridge`
  - `cargo test -p sim-bridge`
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Godot verification:
  - headless startup exits successfully

Residual note:
- `scripts/core/entity/entity_manager.gd` and a small set of helper generators/chronicle/value files still exist as shadow/bootstrap/observer residue.
- They are not part of the active runtime scheduler and do not own the simulation tick.

## In-Game Checks (한국어)
- 게임 시작 후 시뮬레이션이 정상 진행되더라도, GDScript 쪽 시스템 등록 에러나 누락 경고가 새로 뜨지 않는지 확인한다.
- Probe/Sandbox 시작 후 에이전트 이동, needs 변화, 건설 진행이 계속 보이는지 확인한다.
- 일시정지/속도 변경/선택/디버그 오버레이가 기존처럼 동작하는지 확인한다.
- 런타임이 시작될 때 GDScript 시스템을 수동 등록했다는 흔적이 없어야 한다.
- 문제가 있으면 이런 증상이 보인다:
  - 시작 직후 registry mismatch 경고가 반복된다
  - 에이전트가 전혀 갱신되지 않는다
  - UI는 뜨지만 tick이 진행되지 않는다
  - 특정 GDScript 시스템 preload 실패로 시작이 멈춘다

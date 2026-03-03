# 0021 - Full Rust tracking baseline (autopilot)

## Summary
`$autopilot` 요청에 따라 Rust 전환 상태를 전수 추적 기준으로 재정의했다.  
기존 가중치 지표(인프라 중심)와 별도로, 실제 시뮬레이션 로직 실행 소유권(GD/Rust)을 분리해 추적한다.

## Generated Data
- `reports/rust-migration/data/gd-inventory.csv`
  - 전체 `*.gd` 인벤토리
  - 정적 참조 여부(`referenced`)
  - 런타임 등록 시스템 포함 여부(`runtime_registered`)
  - 카테고리(`category`)
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `main.gd` 등록 46개 시스템 전수 매핑
  - `registration_order`는 등록 순서(실행 순서 아님)
  - 실행 소유자(`exec_owner`) / SimBridge 오프로딩(`simbridge_offload`) 포함
  - 오프로딩 판정 규칙(`simbridge_offload_rule`) 명시
- `reports/rust-migration/data/tracking-metadata.json`
  - 생성 시각/기준 커밋/생성 규칙 버전

## Current Tracking Snapshot
1. Repository GDScript inventory
- Total GD: `144`
- Referenced: `138`
- Unreferenced: `6`
- Runtime-registered subset: `46`

2. GD category coverage (`referenced/total`)
- `simulation_system`: `57/57`
- `runtime_infra`: `12/12`
- `core_domain`: `37/37`
- `ui_or_scene`: `32/34`
- `test`: `0/3`
- `tool`: `0/1`
- `simulation_system` 중 runtime-registered 포함: `46`
- `simulation_system` 중 runtime-registered 제외: `11`

3. Runtime-registered system coverage
- Registered systems: `46`
- `exec_owner=gdscript`: `46`
- `exec_owner=rust`: `0`
- `simbridge_offload=yes`: `43`
- No direct SimBridge offload (3):
  - `behavior_system` (`res://scripts/ai/behavior_system.gd`)
  - `gathering_system` (`res://scripts/systems/work/gathering_system.gd`)
  - `construction_system` (`res://scripts/systems/work/construction_system.gd`)

4. Rust system implementation reality
- `impl SimSystem for` (all): `1`
- Production Rust systems (`system_trait.rs` test impl 제외): `0`
- `runtime_*` 함수 총계(내부 helper 포함): `14`
- `#[func]`로 노출된 Runtime API (`runtime_*`): `12`

## Migration Metrics (Dual Track)
1. Infra Migration Index (기존 가중치 지표)
- Current: `100% complete / 0% remaining`
- 의미: runtime/bus/save/shadow/localization/gpu-path 연결 완료 상태

2. Runtime Logic Port Index (신규)
- 정의: `rust_exec_systems / registered_systems`
- Current: `0 / 46 = 0.0%`
- Remaining: `100.0%`

3. Rust Assist Coverage (신규)
- 정의: `simbridge_offload_systems / registered_systems`
- Current: `43 / 46 = 93.5%`

## Methodology / Limits
1. Static reference scope
- `referenced`는 `.gd/.tscn/.tres/.godot` 내 정적 문자열 참조 기준이다.
- 동적 `load(...)`/런타임 문자열 조합 경로는 과소/과대 집계 가능성이 있다.

2. `registration_order` semantics
- CSV의 `registration_order`는 `main.gd`에서 `register_system()` 호출 순서다.
- 실제 tick 실행 순서는 priority 정렬 결과가 기준이다.

3. Offload rule
- `simbridge_offload=yes` 판정은 아래 패턴 정규식 매칭 기반:
  - `SimBridge`, `_get_sim_bridge`, `Engine.get_singleton(\"SimBridge\")`, `bridge.call(...)`

4. Disclosure caution
- 본 추적 파일은 내부 스크립트 구조/실행 순서를 포함하므로 외부 공유 시 비식별화가 필요하다.

## Code Evidence (Key)
- GD 시스템 실행 경로:
  - `scripts/core/simulation/simulation_engine.gd:226` (`system.execute_tick(current_tick)`)
- Rust tick 루프:
  - `rust/crates/sim-bridge/src/lib.rs:1075` (`state.engine.tick()`)
- Rust registry는 현재 메타데이터 등록:
  - `rust/crates/sim-bridge/src/lib.rs:1236`
- Rust SimSystem trait 구현 상태:
  - `rust/crates/sim-engine/src/system_trait.rs:68` (test-only impl)

## Conclusion
- "Rust 전환 100%"은 인프라 완료 기준으로는 맞다.
- 하지만 실게임플레이 시스템 실행 소유권 기준으로는 아직 `0%`다.
- 앞으로는 두 지표를 동시에 보고해야 상태 왜곡이 없다.

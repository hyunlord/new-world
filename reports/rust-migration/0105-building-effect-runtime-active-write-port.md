# 0105 - building-effect runtime active-write port

## Commit
- `[rust-r0-205] Port building-effect runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-core/src/config.rs`
  - 건물 효과 계산에 필요한 상수 추가:
    - `BUILDING_STOCKPILE_RADIUS`
    - `BUILDING_SHELTER_RADIUS`
    - `BUILDING_CAMPFIRE_RADIUS`
    - `BUILDING_SHELTER_ENERGY_RESTORE`
- `rust/crates/sim-engine/src/engine.rs`
  - `SimResources`에 `buildings` 저장소(`HashMap<BuildingId, Building>`) 추가.
- `rust/crates/sim-systems/src/runtime.rs`
  - `BuildingEffectRuntimeSystem` 추가.
  - 완료된 `campfire/shelter` 건물 기반으로 `Needs`(belonging/warmth/safety/energy) active-write 처리.
  - 런타임 테스트 2건 추가:
    - `building_effect_runtime_system_applies_campfire_social_and_warmth`
    - `building_effect_runtime_system_applies_shelter_energy_warmth_and_safety`
- `rust/crates/sim-bridge/src/lib.rs`
  - `building_effect_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `building_effect_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `building_effect_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `building_effect_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0105 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `building_effect_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime data schema 변경:
  - `SimResources.buildings` 추가 (`BuildingId -> Building`).
- 시그널/이벤트 공개 계약 변경 없음.

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `31 / 46 = 67.39%`
- Owner transfer 완료율 (`exec_owner=rust`): `30 / 46 = 65.22%`
- State-write 잔여율: `32.61%`
- Owner transfer 잔여율: `34.78%`

## 메모
- 이번 단계는 `building_effect_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

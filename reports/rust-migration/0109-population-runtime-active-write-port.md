# 0109 - population runtime active-write port

## Commit
- `[rust-r0-209] Port population runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `PopulationRuntimeSystem` 추가.
  - 출생 게이트(`population_birth_block_code`) 평가, 식량 차감, 신생 엔티티 스폰, 정착지 멤버 갱신 구현.
  - `population_runtime_system_spawns_infant_and_consumes_food` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `population_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `population_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `population_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `population_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0109 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `population_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `SimResources.settlements[*].stockpile_food`
  - `SimResources.settlements[*].members`
  - ECS 월드 신생 엔티티 스폰
- 이벤트 큐 write:
  - `EntitySpawned`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `33 / 46 = 71.74%`
- Owner transfer 완료율 (`exec_owner=rust`): `32 / 46 = 69.57%`
- State-write 잔여율: `28.26%`
- Owner transfer 잔여율: `30.43%`

## 메모
- 이번 단계는 `population_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 parity 확인 이후 단계에서 진행한다.

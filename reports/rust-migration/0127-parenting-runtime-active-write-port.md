# 0127 - parenting runtime active-write port

## Commit
- `[rust-r0-227] Port parenting runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ParentingRuntimeSystem` 추가.
  - 부모(성인/노년) 조절 경로 구현 (`Stress.level` write).
  - 아동 관찰 학습 경로 구현 (`Coping.active_strategy`, `Coping.usage_counts`, `Stress.level`, `Stress.allostatic_load` write).
  - `parenting_runtime_system_updates_parent_regulation_state` 테스트 추가.
  - `parenting_runtime_system_assigns_child_coping_strategy` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `parenting_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `parenting_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `parenting_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `parenting_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0127 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `parenting_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `Coping.active_strategy`
  - `Coping.usage_counts`
  - `Stress.level`
  - `Stress.allostatic_load`
- 이벤트 큐 write:
  - `StressChanged`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `42 / 46 = 91.30%`
- Owner transfer 완료율 (`exec_owner=rust`): `41 / 46 = 89.13%`
- State-write 잔여율: `8.70%`
- Owner transfer 잔여율: `10.87%`

## 메모
- 이번 단계는 `parenting_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

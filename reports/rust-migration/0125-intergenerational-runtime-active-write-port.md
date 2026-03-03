# 0125 - intergenerational runtime active-write port

## Commit
- `[rust-r0-225] Port intergenerational runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `IntergenerationalRuntimeSystem` 추가.
  - 부모(성인/노년) 대상 Meaney repair 경로 구현 (`Stress.allostatic_load` write).
  - 자녀(유아/아동/청소년) 대상 세대 전이 경로 구현 (`Stress.allostatic_load`, `Stress.level` write).
  - `intergenerational_runtime_system_applies_parent_meaney_repair` 테스트 추가.
  - `intergenerational_runtime_system_applies_child_transmission` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `intergenerational_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `intergenerational_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `intergenerational_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `intergenerational_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0125 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `intergenerational_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `Stress.allostatic_load`
  - `Stress.level`
- 이벤트 큐 write:
  - `StressChanged`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `41 / 46 = 89.13%`
- Owner transfer 완료율 (`exec_owner=rust`): `40 / 46 = 86.96%`
- State-write 잔여율: `10.87%`
- Owner transfer 잔여율: `13.04%`

## 메모
- 이번 단계는 `intergenerational_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

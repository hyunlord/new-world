# 0121 - construction runtime active-write port

## Commit
- `[rust-r0-221] Port construction runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ConstructionRuntimeSystem` 추가.
  - 성인 빌더(`ActionType::Build`)의 건설 진행 로직 구현.
  - 빌딩 타입별 `build_ticks`(stockpile/shelter/campfire/default) 기반 진행률 계산 구현.
  - `Building.construction_progress` / `Building.is_complete` write 구현.
  - 완공 시 `GameEvent::BuildingConstructed` 이벤트 emission 구현.
  - `construction_runtime_system_progresses_and_completes_building` 테스트 추가.
  - `construction_runtime_system_skips_non_adult_stage` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `construction_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `construction_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `construction_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `construction_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0121 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `construction_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `Building.construction_progress`
  - `Building.is_complete`
- 이벤트 큐 write:
  - `BuildingConstructed`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `39 / 46 = 84.78%`
- Owner transfer 완료율 (`exec_owner=rust`): `38 / 46 = 82.61%`
- State-write 잔여율: `15.22%`
- Owner transfer 잔여율: `17.39%`

## 메모
- 이번 단계는 `construction_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

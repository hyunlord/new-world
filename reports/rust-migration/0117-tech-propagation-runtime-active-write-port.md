# 0117 - tech-propagation runtime active-write port

## Commit
- `[rust-r0-217] Port tech-propagation runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `TechPropagationRuntimeSystem` 추가.
  - 정착지별 문화/기술 전달 프로파일(지식/전통 평균, 최대 스킬) 집계 로직 추가.
  - `Unknown/Forgotten` 기술을 타 정착지 `KnownLow/KnownStable` 원천에서 `KnownLow`로 전이하는 cross-settlement propagation 구현.
  - `TechDiscovered` 이벤트 emission 구현.
  - `tech_propagation_runtime_system_imports_unknown_tech_from_stable_source` 테스트 추가.
  - `tech_propagation_runtime_system_skips_without_known_source` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `tech_propagation_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `tech_propagation_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `tech_propagation_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `tech_propagation_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0117 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `tech_propagation_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `Settlement.tech_states`
- 이벤트 큐 write:
  - `TechDiscovered`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `37 / 46 = 80.43%`
- Owner transfer 완료율 (`exec_owner=rust`): `36 / 46 = 78.26%`
- State-write 잔여율: `19.57%`
- Owner transfer 잔여율: `21.74%`

## 메모
- 이번 단계는 `tech_propagation_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

# 0115 - tech-discovery runtime active-write port

## Commit
- `[rust-r0-215] Port tech-discovery runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `TechDiscoveryRuntimeSystem` 추가.
  - `Unknown/Forgotten` 상태 기술의 발견/재발견 전이(`KnownLow`) 구현.
  - 고인구 구간 강제 발견 경로와 확률 기반 발견 경로 구현.
  - `TechDiscovered` 이벤트 emission 구현.
  - `tech_discovery_runtime_system_discovers_unknown_tech_with_force_pop` 테스트 추가.
  - `tech_discovery_runtime_system_skips_when_population_too_low` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `tech_discovery_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `tech_discovery_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `tech_discovery_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `tech_discovery_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0115 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `tech_discovery_system`
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
- State-write 기준 완료율: `36 / 46 = 78.26%`
- Owner transfer 완료율 (`exec_owner=rust`): `35 / 46 = 76.09%`
- State-write 잔여율: `21.74%`
- Owner transfer 잔여율: `23.91%`

## 메모
- 이번 단계는 `tech_discovery_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

# 0129 - stats recorder runtime active-write port

## Commit
- `[rust-r0-229] Port stats recorder runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-engine/src/engine.rs`
  - `RuntimeStatsSnapshot` 구조체 추가.
  - `SimResources`에 `stats_history`, `stats_peak_population`, `stats_total_births`, `stats_total_deaths` 필드 추가.
- `rust/crates/sim-engine/src/lib.rs`
  - `RuntimeStatsSnapshot` re-export 추가.
- `rust/crates/sim-systems/src/runtime.rs`
  - `StatsRecorderRuntimeSystem` 추가.
  - 집계 스냅샷 기록/최대 인구 갱신/히스토리 윈도우(200) 유지 구현.
  - `stats_recorder_runtime_system_records_snapshot_fields` 테스트 추가.
  - `stats_recorder_runtime_system_caps_history_window` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `stats_recorder`를 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트를 `stats_recorder=true`로 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stats_recorder`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `stats_recorder` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0129 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `stats_recorder`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `SimResources.stats_history`
  - `SimResources.stats_peak_population`

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `43 / 46 = 93.48%`
- Owner transfer 완료율 (`exec_owner=rust`): `42 / 46 = 91.30%`
- State-write 잔여율: `6.52%`
- Owner transfer 잔여율: `8.70%`

## 메모
- 이번 단계는 `stats_recorder`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

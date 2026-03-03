# 0101 - stratification-monitor runtime active-write port

## Commit
- `[rust-r0-201] Port stratification-monitor runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-core/src/components/economic.rs`
  - `Economic.wealth_norm` 필드 추가 (`serde(default)`).
- `rust/crates/sim-core/src/settlement.rs`
  - 정착지 계층화 상태 필드 추가:
    - `gini_coefficient`
    - `leveling_effectiveness`
    - `stratification_phase`
- `rust/crates/sim-systems/src/runtime.rs`
  - `StratificationMonitorRuntimeSystem` 추가.
  - 실제 write 경로 구현:
    - `Settlement.gini_coefficient` 갱신
    - `Settlement.leveling_effectiveness` 갱신
    - `Settlement.stratification_phase` 갱신
    - `Social.social_class` 갱신
    - `Economic.wealth_norm` 갱신
  - 단위 테스트 1건 추가:
    - `stratification_monitor_runtime_system_updates_settlement_and_class_state`
- `rust/crates/sim-bridge/src/lib.rs`
  - `stratification_monitor` 지원 키 추가.
  - 런타임 등록 경로에 `StratificationMonitorRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신(`runtime_supports_expected_ported_systems`).
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stratification_monitor`의 `rust_runtime_impl=no -> yes` 반영 (`exec_owner`는 `gdscript` 유지).
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `stratification_monitor` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0101 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `stratification_monitor`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `stratification_monitor`가 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (251 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `29 / 46 = 63.04%`
- Owner transfer 완료율 (`exec_owner=rust`): `28 / 46 = 60.87%`
- State-write 잔여율: `36.96%`
- Owner transfer 잔여율: `39.13%`

## 메모
- 이번 단계로 `stratification_monitor`는 no-op이 아닌 실제 정착지/사회/경제 상태 변경(write) 경로로 Rust 전환됐다.
- 다음 단계는 owner-ready allowlist에 `stratification_monitor`를 추가해 실행 소유권을 Rust로 승격하는 것이다.

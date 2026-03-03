# 0081 - economic-tendency runtime active-write port

## Commit
- `[rust-r0-181] Port economic-tendency runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `EconomicTendencyRuntimeSystem` 추가.
  - `Economic.saving_tendency`, `Economic.risk_appetite`, `Economic.generosity`, `Economic.materialism`에 대한 실제 write 경로 구현.
  - `body::economic_tendencies_step` 기반으로 성별 리스크 편향/wealth generosity penalty 반영.
  - 단위 테스트 2건 추가:
    - `economic_tendency_runtime_system_updates_tendencies_and_applies_male_risk_bias`
    - `economic_tendency_runtime_system_skips_child_stage`
- `rust/crates/sim-bridge/src/lib.rs`
  - `economic_tendency_system` 지원 키 추가.
  - 런타임 등록 경로에 `EconomicTendencyRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `economic_tendency_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `economic_tendency_system` 추가.
- `reports/rust-migration/README.md`
  - 0081 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `economic_tendency_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `economic_tendency_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (231 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `19 / 46 = 41.30%`
- Owner transfer 완료율 (`exec_owner=rust`): `18 / 46 = 39.13%`
- State-write 잔여율: `58.70%`
- Owner transfer 잔여율: `60.87%`

## 메모
- 이번 단계는 경제 성향 계산 결과를 Rust ECS `Economic` 컴포넌트에 직접 기록하도록 전환한 실포팅이다.
- 다음 단계는 owner-ready allowlist에 `economic_tendency_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

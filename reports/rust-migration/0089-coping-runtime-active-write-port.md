# 0089 - coping runtime active-write port

## Commit
- `[rust-r0-189] Port coping runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `CopingRuntimeSystem` 추가.
  - `Coping` 컴포넌트의 실제 write 경로 구현:
    - `strategy_cooldowns` 감소
    - `active_strategy` clear/selection
    - `usage_counts` 증가
  - `body::coping_learn_probability`, `body::coping_softmax_index`를 전략 선택 로직에 연동.
  - 단위 테스트 2건 추가:
    - `coping_runtime_system_decrements_strategy_cooldowns`
    - `coping_runtime_system_selects_strategy_and_updates_usage`
- `rust/crates/sim-bridge/src/lib.rs`
  - `coping_system` 지원 키 추가.
  - 런타임 등록 경로에 `CopingRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `coping_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `coping_system` 추가.
- `reports/rust-migration/README.md`
  - 0089 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `coping_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `coping_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (239 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `23 / 46 = 50.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `22 / 46 = 47.83%`
- State-write 잔여율: `50.00%`
- Owner transfer 잔여율: `52.17%`

## 메모
- 이번 단계로 coping 경로가 no-op이 아닌 실제 컴포넌트 write 시스템으로 전환됐다.
- 다음 단계는 owner-ready allowlist에 `coping_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

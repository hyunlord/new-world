# 0077 - mental-break runtime active-write port

## Commit
- `[rust-r0-177] Port mental-break runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `MentalBreakRuntimeSystem` 추가.
  - `Stress.active_mental_break`, `Stress.mental_break_remaining`, `Stress.mental_break_count`에 대한 실제 write 경로 구현.
  - break 종료 시 카타르시스 경감(`stress.level` 감소) 및 상태 재계산 반영.
  - 단위 테스트 2건 추가:
    - `mental_break_runtime_system_triggers_break_and_sets_runtime_fields`
    - `mental_break_runtime_system_clears_active_break_after_countdown`
- `rust/crates/sim-bridge/src/lib.rs`
  - `mental_break_system` 지원 키 추가.
  - 런타임 등록 경로에 `MentalBreakRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `mental_break_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `mental_break_system` 추가.
- `reports/rust-migration/README.md`
  - 0077 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `mental_break_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `mental_break_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (227 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `17 / 46 = 36.96%`
- Owner transfer 완료율 (`exec_owner=rust`): `16 / 46 = 34.78%`
- State-write 잔여율: `63.04%`
- Owner transfer 잔여율: `65.22%`

## 메모
- 이번 단계는 멘탈 브레이크의 핵심 runtime 상태를 Rust에서 직접 관리하도록 전환한 포팅이다.
- 다음 단계는 owner-ready allowlist에 `mental_break_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

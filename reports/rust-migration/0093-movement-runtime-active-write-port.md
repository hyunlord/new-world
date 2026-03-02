# 0093 - movement runtime active-write port

## Commit
- `[rust-r0-193] Port movement runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `MovementRuntimeSystem` 추가.
  - 실제 write 경로 구현:
    - `Position` 이동(step movement with passable tile check)
    - `Behavior.action_timer` 감소
    - 액션 완료 시 `Behavior.current_action/action_target_*` 정리
    - 도착 효과에 따른 `Needs` write (`Hunger/Thirst/Warmth/Safety/Belonging/Sleep`)
  - 단위 테스트 2건 추가:
    - `movement_runtime_system_moves_toward_target_on_passable_tile`
    - `movement_runtime_system_completes_action_and_applies_drink_restore`
- `rust/crates/sim-bridge/src/lib.rs`
  - `movement_system` 지원 키 추가.
  - 런타임 등록 경로에 `MovementRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신(`runtime_supports_expected_ported_systems`).
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `movement_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `movement_system` 추가.
- `reports/rust-migration/README.md`
  - 0093 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `movement_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `movement_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (243 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `25 / 46 = 54.35%`
- Owner transfer 완료율 (`exec_owner=rust`): `24 / 46 = 52.17%`
- State-write 잔여율: `45.65%`
- Owner transfer 잔여율: `47.83%`

## 메모
- 이번 단계로 movement 경로가 no-op이 아닌 실제 상태 변경(write) 시스템으로 전환됐다.
- 다음 단계는 owner-ready allowlist에 `movement_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

# 0133 - behavior runtime active-write port

## Commit
- `[rust-r0-233] Port behavior runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `BehaviorRuntimeSystem` 추가.
  - 행동 점수 평가/히스테리시스/타겟/타이머 할당 로직 구현.
  - `Behavior.current_action`, `action_target_*`, `action_timer`, `action_duration` write 경로 연결.
  - 행동 선택 이벤트(`SocialEventOccurred`) 방출 추가.
  - `behavior_runtime_system_assigns_forage_and_emits_event_for_hungry_adult` 테스트 추가.
  - `behavior_runtime_system_skips_migrate_and_active_timer_entities` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `behavior_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트를 `behavior_system=true`로 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `behavior_system`의 `simbridge_offload=no -> yes`, `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `behavior_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0133 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `behavior_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `Behavior.current_action`
  - `Behavior.action_target_x/y`
  - `Behavior.action_timer`
  - `Behavior.action_duration`

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `43 / 46 = 93.48%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `6.52%`

## 메모
- 이번 단계는 `behavior_system` active-write 구현까지 완료.
- `exec_owner=rust` 전환(allowlist 반영)은 다음 단계에서 처리한다.

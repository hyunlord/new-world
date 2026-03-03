# 0132 - stat threshold runtime active-write port

## Commit
- `[rust-r0-232] Port stat threshold runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-engine/src/engine.rs`
  - `SimResources`에 `stat_threshold_flags` 필드 추가.
- `rust/crates/sim-systems/src/runtime.rs`
  - `StatThresholdRuntimeSystem` 추가.
  - 임계치 판정(`body::stat_threshold_is_active`) 및 상태 플래그 유지 구현.
  - threshold 효과를 `Behavior.current_action` write(`Forage/Rest/Idle`)로 연결.
  - `stat_threshold_runtime_system_applies_hunger_effect_action` 테스트 추가.
  - `stat_threshold_runtime_system_hysteresis_clears_effect_after_recovery` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `stat_threshold_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트를 `stat_threshold_system=true`로 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stat_threshold_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `stat_threshold_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0132 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `stat_threshold_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `SimResources.stat_threshold_flags`
  - `Behavior.current_action`

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `45 / 46 = 97.83%`
- Owner transfer 완료율 (`exec_owner=rust`): `43 / 46 = 93.48%`
- State-write 잔여율: `2.17%`
- Owner transfer 잔여율: `6.52%`

## 메모
- 이번 단계는 `stat_threshold_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

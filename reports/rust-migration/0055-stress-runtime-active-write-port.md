# 0055 - stress runtime active-write port (Phase 5 start)

## Commit
- `[rust-r0-155] Port stress runtime to active-write and re-enable strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `StressRuntimeSystem` 재도입 (no-op 제거 후 active-write 구현).
  - `Needs + Stress + Emotion` 입력을 기반으로 `Stress.level/reserve/allostatic_load/state`를 실제 갱신.
  - `stress_runtime_system_updates_stress_state_and_components` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `stress_system` 재등록.
  - `runtime_supports_expected_ported_systems` 테스트를 새 지원 목록에 맞게 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stress_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `stress_system` 추가.
- `reports/rust-migration/README.md`
  - 0055 항목 및 누적 전환률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템: `resource_regen_system`, `needs_system`, `upper_needs_system`, `stress_system`.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `4 / 46 = 8.70%`
- Owner transfer 완료율 (`exec_owner=rust`): `0 / 46 = 0.00%`
- 잔여율: `91.30%`

## 메모
- 본 커밋은 Phase 5의 첫 active-write 시스템(stress) 착수다.
- 다음 우선순위는 `emotion -> reputation -> social_event -> morale` 순서로 active-write 전환.

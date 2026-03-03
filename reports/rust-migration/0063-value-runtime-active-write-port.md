# 0063 - value runtime active-write port

## Commit
- `[rust-r0-163] Port value runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ValueRuntimeSystem` 추가.
  - `Values` 컴포넌트(협력/공정/가족/우정/법/권력/경쟁/평화 축)에 age plasticity + personality/needs/stress/social 신호 기반 드리프트를 실제 적용.
  - `body::value_plasticity` 커널 사용.
  - `value_runtime_system_updates_value_axes_with_context` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `value_system` 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `value_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `value_system` 추가.
- `reports/rust-migration/README.md`
  - 0063 항목 추가 및 누적 전환률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템에 `value_system` 추가.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `9 / 46 = 19.57%`
- Owner transfer 완료율 (`exec_owner=rust`): `8 / 46 = 17.39%`
- State-write 잔여율: `80.43%`
- Owner transfer 잔여율: `82.61%`

## 메모
- 다음 후보: `network_system` 또는 `job_satisfaction_system` active-write 포팅.

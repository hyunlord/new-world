# 0057 - reputation runtime active-write port

## Commit
- `[rust-r0-157] Port reputation runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ReputationRuntimeSystem` 추가.
  - `Social.reputation_local`, `Social.reputation_regional`, `Social.reputation_tags`를 emotion/stress/needs/action 신호 기반으로 실제 업데이트.
  - `body::reputation_event_delta`, `body::reputation_decay_value` 커널을 직접 사용.
  - `reputation_runtime_system_updates_reputation_state_and_tags` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `reputation_system` 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `reputation_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `reputation_system` 추가.
- `reports/rust-migration/README.md`
  - 0057 항목 추가 및 누적 전환률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템: `resource_regen_system`, `needs_system`, `upper_needs_system`, `stress_system`, `emotion_system`, `reputation_system`.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `6 / 46 = 13.04%`
- Owner transfer 완료율 (`exec_owner=rust`): `0 / 46 = 0.00%`
- 잔여율: `86.96%`

## 메모
- 다음 우선순위는 `social_event -> morale` active-write 전환.

# 0059 - morale runtime active-write port

## Commit
- `[rust-r0-159] Port morale runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `MoraleRuntimeSystem` 추가.
  - morale 계산(affect/life-satisfaction + maslow + hygiene + warr proxy) 후 `Behavior.job_satisfaction`, `Behavior.occupation_satisfaction`, `Needs.meaning`, `Needs.transcendence`를 실제 업데이트.
  - `body::morale_behavior_weight_multiplier`, `body::morale_migration_probability` 커널 사용.
  - `morale_runtime_system_updates_behavior_and_upper_needs` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `morale_system` 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `morale_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `morale_system` 추가.
- `reports/rust-migration/README.md`
  - 0059 항목 추가 및 누적 전환률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템: `resource_regen_system`, `needs_system`, `upper_needs_system`, `stress_system`, `emotion_system`, `reputation_system`, `social_event_system`, `morale_system`.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `8 / 46 = 17.39%`
- Owner transfer 완료율 (`exec_owner=rust`): `0 / 46 = 0.00%`
- 잔여율: `82.61%`

## 메모
- 다음 우선순위는 `owner transfer`(allowlist 기반 실행 소유권 이관) 시작.

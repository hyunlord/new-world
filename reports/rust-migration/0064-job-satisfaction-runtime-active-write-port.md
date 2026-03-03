# 0064 - job-satisfaction runtime active-write port

## Commit
- `[rust-r0-164] Port job-satisfaction runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `JobSatisfactionRuntimeSystem` 추가.
  - `body::job_satisfaction_score`를 사용해 `Behavior.job_satisfaction`, `Behavior.occupation_satisfaction`를 실제 업데이트.
  - 프로필 매핑(builder/miner/lumberjack/hunter/default)과 skill/value/personality/need 기반 평가 로직 추가.
  - `job_satisfaction_runtime_system_updates_behavior_scores` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `job_satisfaction_system` 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `job_satisfaction_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `job_satisfaction_system` 추가.
- `reports/rust-migration/README.md`
  - 0064 항목 추가 및 누적 전환률 갱신.

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템에 `job_satisfaction_system` 추가.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `10 / 46 = 21.74%`
- Owner transfer 완료율 (`exec_owner=rust`): `8 / 46 = 17.39%`
- State-write 잔여율: `78.26%`
- Owner transfer 잔여율: `82.61%`

## 메모
- 다음 후보: `network_system` active-write 또는 owner transfer 추가 확장.

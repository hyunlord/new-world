# 0073 - job-assignment runtime active-write port

## Commit
- `[rust-r0-173] Port job-assignment runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `JobAssignmentRuntimeSystem` 추가.
  - `Behavior.job`에 대한 실제 write 경로 구현:
    - 유아/걸음마: `none` 강제
    - 아동/청소년 규칙 반영
    - 비할당 엔티티 비율 기반 배정
    - 분포 편차 시 1명 재배치(rebalance)
  - 단위 테스트 2건 추가:
    - `job_assignment_runtime_system_assigns_jobs_with_age_rules`
    - `job_assignment_runtime_system_rebalances_one_idle_surplus_job`
- `rust/crates/sim-bridge/src/lib.rs`
  - `job_assignment_system` 지원 키 추가.
  - 런타임 등록 경로에 `JobAssignmentRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `job_assignment_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `job_assignment_system` 추가.
- `reports/rust-migration/README.md`
  - 0073 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `job_assignment_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `job_assignment_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (224 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `15 / 46 = 32.61%`
- Owner transfer 완료율 (`exec_owner=rust`): `14 / 46 = 30.43%`
- State-write 잔여율: `67.39%`
- Owner transfer 잔여율: `69.57%`

## 메모
- 이번 단계는 no-op이 아닌 실제 `Behavior.job` 변경 경로를 Rust에 이관한 포팅이다.
- 다음 단계는 owner-ready allowlist에 `job_assignment_system`을 추가해 실행 소유권(`exec_owner`)을 Rust로 전환하는 것이다.

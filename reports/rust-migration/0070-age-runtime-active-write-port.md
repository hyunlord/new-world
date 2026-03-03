# 0070 - age runtime active-write port

## Commit
- `[rust-r0-170] Port age runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `AgeRuntimeSystem` 추가.
  - `Age.ticks/years/stage`를 실제 갱신하고, `Identity.growth_stage` 동기화.
  - elder 단계에서 `Behavior.job == builder`이면 `none`으로 정리.
  - `age_runtime_system_updates_stage_identity_and_elder_builder_job` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `age_system` 추가.
  - `register_supported_rust_system(...)`에 `AgeRuntimeSystem` 등록 분기 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `age_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `age_system` 추가.
- `reports/rust-migration/README.md`
  - 0070 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- 추가(Active-write): `age_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템에 `age_system` 추가.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `14 / 46 = 30.43%`
- Owner transfer 완료율 (`exec_owner=rust`): `12 / 46 = 26.09%`
- State-write 잔여율: `69.57%`
- Owner transfer 잔여율: `73.91%`

## 메모
- 이번 단계는 age 진행/stage 반영의 핵심 write 경로를 Rust로 이전했다.
- 성장 이벤트 발행/세부 body 재계산 parity는 후속 확장 대상이다.

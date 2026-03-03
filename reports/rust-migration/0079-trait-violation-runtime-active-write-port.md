# 0079 - trait-violation runtime active-write port

## Commit
- `[rust-r0-179] Port trait-violation runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `TraitViolationRuntimeSystem` 추가.
  - `Stress.level`, `Stress.reserve`, `Stress.allostatic_load`에 대한 실제 write 경로 구현.
  - trait-context/facet 기반 위반 스트레스 증가, desensitize/PTSD 누적 히스토리, intrusive-thought 재주입 경로 반영.
  - 단위 테스트 2건 추가:
    - `trait_violation_runtime_system_increases_stress_on_violation`
    - `trait_violation_runtime_system_ptsd_path_amplifies_repeat_delta`
- `rust/crates/sim-bridge/src/lib.rs`
  - `trait_violation_system` 지원 키 추가.
  - 런타임 등록 경로에 `TraitViolationRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `trait_violation_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `trait_violation_system` 추가.
- `reports/rust-migration/README.md`
  - 0079 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `trait_violation_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `trait_violation_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (229 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `18 / 46 = 39.13%`
- Owner transfer 완료율 (`exec_owner=rust`): `17 / 46 = 36.96%`
- State-write 잔여율: `60.87%`
- Owner transfer 잔여율: `63.04%`

## 메모
- 이번 단계는 trait_violation 핵심 스트레스 경로를 Rust에서 직접 업데이트하도록 전환한 실포팅이다.
- 다음 단계는 owner-ready allowlist에 `trait_violation_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

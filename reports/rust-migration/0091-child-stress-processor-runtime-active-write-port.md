# 0091 - child-stress-processor runtime active-write port

## Commit
- `[rust-r0-191] Port child-stress-processor runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ChildStressProcessorRuntimeSystem` 추가.
  - child stage(`infant/toddler/child/teen`)에서만 `Stress` 컴포넌트 write 수행:
    - `stress.reserve`
    - `stress.level`
    - `stress.allostatic_load`
  - `body::child_social_buffered_intensity`, `body::child_stress_type_code`,
    `body::child_stress_apply_step`를 런타임 write 경로에 연동.
  - 단위 테스트 2건 추가:
    - `child_stress_processor_runtime_system_updates_child_stress_fields`
    - `child_stress_processor_runtime_system_skips_non_child_stages`
- `rust/crates/sim-bridge/src/lib.rs`
  - `child_stress_processor` 지원 키 추가.
  - 런타임 등록 경로에 `ChildStressProcessorRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신(`runtime_supports_expected_ported_systems`).
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `child_stress_processor`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `child_stress_processor` 추가.
- `reports/rust-migration/README.md`
  - 0091 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `child_stress_processor`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `child_stress_processor`가 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (241 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `24 / 46 = 52.17%`
- Owner transfer 완료율 (`exec_owner=rust`): `23 / 46 = 50.00%`
- State-write 잔여율: `47.83%`
- Owner transfer 잔여율: `50.00%`

## 메모
- 이번 단계로 child stress 처리 경로가 no-op이 아닌 실제 컴포넌트 write 시스템으로 전환됐다.
- 다음 단계는 owner-ready allowlist에 `child_stress_processor`를 추가해 실행 소유권을 Rust로 승격하는 것이다.

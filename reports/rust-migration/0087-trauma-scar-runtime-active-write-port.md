# 0087 - trauma-scar runtime active-write port

## Commit
- `[rust-r0-187] Port trauma-scar runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `TraumaScarRuntimeSystem` 추가.
  - `Memory.trauma_scars` 기반으로 `Emotion.baseline`을 실제 갱신하는 write 경로 구현.
  - scar 타입별 baseline drift 규칙 추가 및 `body::trauma_scar_sensitivity_factor` 연동.
  - baseline 값 clamp(0..1) 보장.
  - 단위 테스트 2건 추가:
    - `trauma_scar_runtime_system_applies_baseline_drift_from_scars`
    - `trauma_scar_runtime_system_clamps_baseline_with_repeated_updates`
- `rust/crates/sim-bridge/src/lib.rs`
  - `trauma_scar_system` 지원 키 추가.
  - 런타임 등록 경로에 `TraumaScarRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `trauma_scar_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `trauma_scar_system` 추가.
- `reports/rust-migration/README.md`
  - 0087 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `trauma_scar_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `trauma_scar_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (237 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `22 / 46 = 47.83%`
- Owner transfer 완료율 (`exec_owner=rust`): `21 / 46 = 45.65%`
- State-write 잔여율: `52.17%`
- Owner transfer 잔여율: `54.35%`

## 메모
- 이번 단계로 trauma scar 경로가 no-op이 아닌 실제 감정 baseline write 시스템으로 전환됐다.
- 다음 단계는 owner-ready allowlist에 `trauma_scar_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

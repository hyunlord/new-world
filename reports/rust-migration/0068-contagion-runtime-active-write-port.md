# 0068 - contagion runtime active-write port

## Commit
- `[rust-r0-168] Port contagion runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `ContagionRuntimeSystem` 추가.
  - AoE/정착지 네트워크 전파 커널을 적용해 `Emotion.primary`와 `Stress.level`을 실제 갱신.
  - `contagion_runtime_system_propagates_emotion_and_stress` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `contagion_system` 추가.
  - `register_supported_rust_system(...)`에 `ContagionRuntimeSystem` 등록 분기 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `contagion_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `contagion_system` 추가.
- `reports/rust-migration/README.md`
  - 0068 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- 추가(Active-write): `contagion_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템에 `contagion_system` 추가.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `13 / 46 = 28.26%`
- Owner transfer 완료율 (`exec_owner=rust`): `10 / 46 = 21.74%`
- State-write 잔여율: `71.74%`
- Owner transfer 잔여율: `78.26%`

## 메모
- 본 단계는 감정/스트레스 전파의 핵심 state write를 Rust 실행 경로로 이전했다.
- refractory/메타 상태 기반 세부 동작은 후속 parity 단계에서 확장 대상이다.

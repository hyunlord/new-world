# 0065 - network runtime active-write port

## Commit
- `[rust-r0-165] Port network runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `NetworkRuntimeSystem` 추가.
  - `Social.edges`의 tie 강도/bridge 여부 + 평판 점수로 `Social.social_capital`을 실제 갱신.
  - `network_runtime_system_recomputes_social_capital_from_edges` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust runtime 지원/등록 대상에 `network_system` 추가.
  - `register_supported_rust_system(...)`에 `NetworkRuntimeSystem` 등록 분기 추가.
  - `runtime_supports_expected_ported_systems` 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `network_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - strict rule에 `network_system` 추가.
- `reports/rust-migration/README.md`
  - 0065 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- 추가(Active-write): `network_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime registration policy 변경:
  - Rust 지원 시스템에 `network_system` 추가.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `11 / 46 = 23.91%`
- Owner transfer 완료율 (`exec_owner=rust`): `8 / 46 = 17.39%`
- State-write 잔여율: `76.09%`
- Owner transfer 잔여율: `82.61%`

## 메모
- 이번 단계는 `network_system`의 social-capital write 경로를 Rust로 이관한 active-write 포팅이며,
  authority type/revolution 이벤트 full parity는 후속 단계에서 settlement 데이터 확장과 함께 이관 대상이다.

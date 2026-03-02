# 0075 - mortality runtime active-write port

## Commit
- `[rust-r0-175] Port mortality runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `MortalityRuntimeSystem` 추가.
  - `body::mortality_hazards_and_prob` 기반으로 월간/연간 체크를 수행하고 `Age.alive`를 실제로 write.
  - 고위험 입력에서 사망 상태가 반영되는 단위 테스트 추가:
    - `mortality_runtime_system_marks_high_risk_entity_dead`
- `rust/crates/sim-bridge/src/lib.rs`
  - `mortality_system` 지원 키 추가.
  - 런타임 등록 경로에 `MortalityRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `mortality_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `mortality_system` 추가.
- `reports/rust-migration/README.md`
  - 0075 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `mortality_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `mortality_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (225 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `16 / 46 = 34.78%`
- Owner transfer 완료율 (`exec_owner=rust`): `15 / 46 = 32.61%`
- State-write 잔여율: `65.22%`
- Owner transfer 잔여율: `67.39%`

## 메모
- 이번 단계는 사망 판정의 핵심 상태(`Age.alive`)를 Rust 경로에서 직접 갱신한다.
- 다음 단계로 owner-ready allowlist에 `mortality_system`을 추가하면 실행 소유권도 Rust로 전환 가능하다.

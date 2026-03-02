# 0083 - intelligence runtime active-write port

## Commit
- `[rust-r0-183] Port intelligence runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `IntelligenceRuntimeSystem` 추가.
  - `Intelligence.values`, `Intelligence.g_factor`, `Intelligence.ace_penalty`, `Intelligence.nutrition_penalty` 실제 write 경로 구현.
  - 연령 곡선(Fluid/Crystallized/Physical), 활동도 보정, ACE/영양 페널티를 body 커널(`intelligence_effective_value`, `cognition_activity_modifier`, `cognition_ace_fluid_decline_mult`, `intelligence_g_value`)과 연결.
  - 단위 테스트 2건 추가:
    - `intelligence_runtime_system_applies_nutrition_penalty_in_critical_window`
    - `intelligence_runtime_system_applies_ace_penalty_and_fluid_decline`
- `rust/crates/sim-bridge/src/lib.rs`
  - `intelligence_system` 지원 키 추가.
  - 런타임 등록 경로에 `IntelligenceRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `intelligence_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `intelligence_system` 추가.
- `reports/rust-migration/README.md`
  - 0083 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `intelligence_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `intelligence_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (233 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `20 / 46 = 43.48%`
- Owner transfer 완료율 (`exec_owner=rust`): `19 / 46 = 41.30%`
- State-write 잔여율: `56.52%`
- Owner transfer 잔여율: `58.70%`

## 메모
- 이번 단계는 지능 실효값 재계산을 Rust ECS 내부에서 직접 수행하도록 전환한 실포팅이다.
- 다음 단계는 owner-ready allowlist에 `intelligence_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

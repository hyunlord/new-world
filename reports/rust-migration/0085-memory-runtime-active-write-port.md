# 0085 - memory runtime active-write port

## Commit
- `[rust-r0-185] Port memory runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `MemoryRuntimeSystem` 추가.
  - `memory_system.gd` 핵심 루프를 Rust state-write로 이관:
    - working memory decay (`body::memory_decay_batch`)
    - low-intensity forgetting (`< 0.01`)
    - capacity eviction (`MEMORY_WORKING_MAX`)
    - old-entry compression (`MEMORY_COMPRESS_MIN_GROUP`, `MEMORY_COMPRESS_INTERVAL_TICKS`)
    - permanent promotion (`MEMORY_PERMANENT_THRESHOLD` + canonical permanent 이벤트 타입)
  - 단위 테스트 2건 추가:
    - `memory_runtime_system_decays_evicts_and_promotes_entries`
    - `memory_runtime_system_compresses_old_entries_into_summary`
- `rust/crates/sim-bridge/src/lib.rs`
  - `memory_system` 지원 키 추가.
  - 런타임 등록 경로에 `MemoryRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `memory_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `memory_system` 추가.
- `reports/rust-migration/README.md`
  - 0085 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `memory_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `memory_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (235 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `21 / 46 = 45.65%`
- Owner transfer 완료율 (`exec_owner=rust`): `20 / 46 = 43.48%`
- State-write 잔여율: `54.35%`
- Owner transfer 잔여율: `56.52%`

## 메모
- 이번 단계로 memory 경로가 no-op이 아닌 실제 상태 변경(write) 시스템으로 전환됐다.
- 다음 단계는 owner-ready allowlist에 `memory_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

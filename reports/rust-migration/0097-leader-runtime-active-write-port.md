# 0097 - leader runtime active-write port

## Commit
- `[rust-r0-197] Port leader runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `LeaderRuntimeSystem` 추가.
  - 실제 write 경로 구현:
    - `resources.settlements[*].leader_id` 갱신
    - `resources.settlements[*].leader_reelection_countdown` 갱신/감산
  - 리더 선출 로직:
    - adult/elder 후보군만 대상으로 점수 산출
    - 기존 리더 유효성(생존 + 동일 settlement) 검증 후 무효 시 즉시 재선출
    - 동점 허용 구간에서는 평판/엔티티 ID 기준으로 deterministic tie-break
  - 단위 테스트 3건 추가:
    - `leader_runtime_system_elects_best_candidate_and_sets_countdown`
    - `leader_runtime_system_decrements_countdown_without_re_election`
    - `leader_runtime_system_replaces_invalid_leader`
- `rust/crates/sim-bridge/src/lib.rs`
  - `leader_system` 지원 키 추가.
  - 런타임 등록 경로에 `LeaderRuntimeSystem::new(...)` 연결.
  - 지원 시스템 테스트 갱신(`runtime_supports_expected_ported_systems`).
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `leader_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `leader_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0097 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- active-write 구현 추가: `leader_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime Rust 구현 범위 확장:
  - `leader_system`이 Rust state-write 시스템으로 승격.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅ (248 passed)
- `cd rust && cargo test -p sim-bridge` ✅ (28 passed)

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `27 / 46 = 58.70%`
- Owner transfer 완료율 (`exec_owner=rust`): `26 / 46 = 56.52%`
- State-write 잔여율: `41.30%`
- Owner transfer 잔여율: `43.48%`

## 메모
- 이번 단계로 `leader_system`은 no-op이 아닌 실제 settlement 상태 변경(write) 경로로 Rust 전환됐다.
- 다음 단계는 owner-ready allowlist에 `leader_system`을 추가해 실행 소유권을 Rust로 승격하는 것이다.

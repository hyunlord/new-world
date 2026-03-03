# 0103 - tension runtime active-write port

## Commit
- `[rust-r0-203] Port tension runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-engine/src/engine.rs`
  - `SimResources`에 `tension_pairs` 저장소(`HashMap<String, f64>`) 추가.
- `rust/crates/sim-systems/src/runtime.rs`
  - `TensionRuntimeSystem` 추가.
  - 정착지 쌍별로 거리/식량부족/자연감쇠를 반영해 `SimResources.tension_pairs`를 갱신.
  - `tension_runtime_system_updates_pair_tension_from_scarcity_and_decay` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `tension_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `tension_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `tension_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `tension_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0103 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `tension_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime data schema 변경:
  - `SimResources.tension_pairs` 추가 (`"min_settlement_id:max_settlement_id" -> tension`).
- 시그널/이벤트 공개 계약 변경 없음.

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `30 / 46 = 65.22%`
- Owner transfer 완료율 (`exec_owner=rust`): `29 / 46 = 63.04%`
- State-write 잔여율: `34.78%`
- Owner transfer 잔여율: `36.96%`

## 메모
- 이번 단계는 `tension_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

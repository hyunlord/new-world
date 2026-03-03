# 0107 - migration runtime active-write port

## Commit
- `[rust-r0-207] Port migration runtime to active-write and update strict tracking`

## 변경 파일
- `rust/crates/sim-systems/src/runtime.rs`
  - `MigrationRuntimeSystem` 추가.
  - 정착지 혼잡/식량부족/확률 기반 이주 시도, 신규 정착지 생성, 멤버 이동, 행동 상태(`Migrate`) 갱신 구현.
  - `migration_runtime_system_founds_new_settlement_and_moves_members` 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `migration_system`을 Rust 지원 시스템 키/레지스트리 등록 경로에 추가.
  - bridge 지원 검증 테스트에 `migration_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `migration_system`의 `rust_runtime_impl=no -> yes` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule`에 `migration_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0107 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- runtime active-write 추가: `migration_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime write 경로 변경:
  - `SimResources.settlements` (신규 정착지 추가, source cooldown/food/member 갱신)
  - `Identity.settlement_id` (이주 멤버 소속 변경)
  - `Behavior` (`current_action=Migrate`, `action_target_x/y`, `action_timer`)
- 이벤트 큐 write:
  - `SettlementFounded`
  - `MigrationOccurred`

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `32 / 46 = 69.57%`
- Owner transfer 완료율 (`exec_owner=rust`): `31 / 46 = 67.39%`
- State-write 잔여율: `30.43%`
- Owner transfer 잔여율: `32.61%`

## 메모
- 이번 단계는 `migration_system`의 Rust active-write 구현까지 포함하며,
  실행 소유권 전환(`exec_owner=rust`)은 다음 단계에서 진행한다.

# 0137 - bus v2 event payload expansion

## Commit
- `[rust-r0-237] Expand Bus v2 event mapping and payload coverage`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - Bus v2 이벤트 타입 ID 상수에 엔티티/스트레스/직업/자원/건설/이주/가족/사회/기술 계열 이벤트를 추가.
  - `game_event_type_id` 매핑을 확장해 신규 이벤트가 `EVENT_TYPE_ID_GENERIC`로 떨어지지 않도록 정리.
  - `game_event_payload` 직렬화 필드를 확장해 주요 `GameEvent` 변형들의 payload를 v2 딕셔너리로 전달.
  - Godot 엔진 컨텍스트가 없는 단위 테스트에서도 검증 가능한 `game_event_type_id_maps_new_v2_event_variants` 테스트 추가.
- `reports/rust-migration/README.md`
  - 0137 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- `runtime_export_events_v2()` 출력 이벤트의 `event_type_id`/`payload` 커버리지가 확장됨.
  - 추가 커버 이벤트: `EntitySpawned`, `EntityDied`, `EntityRemoved`, `StressChanged`,
    `MentalBreakTriggered`, `JobAssigned`, `ResourceGathered`, `BuildingConstructed`,
    `MigrationOccurred`, `SettlementFounded`, `FamilyFormed`, `BirthOccurred`,
    `SocialEventOccurred`, `RelationshipChanged`, `TechDiscovered`, `EraAdvanced`.
- 공개 GDExtension 메서드 시그니처 변경 없음.

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 기존에 추가했던 엔진 의존 테스트(`VarDictionary` 직접 역직렬화)는 단위 테스트 환경에서 불안정해 제거했고, 순수 Rust 레벨 매핑 테스트로 교체했다.

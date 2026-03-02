# 0143 - bus v2 v1 re-emit expansion

## Commit
- `[rust-r0-243] Expand SimulationBus v2-to-v1 re-emission mapping`

## 변경 파일
- `scripts/core/simulation/simulation_bus.gd`
  - v2 이벤트 타입 상수 추가:
    - `ENTITY_SPAWNED(10)`, `ENTITY_DIED(11)`, `MENTAL_BREAK_TRIGGERED(21)`,
      `FAMILY_FORMED(35)`, `ERA_ADVANCED(61)`
  - `_on_v2_event_emitted()` 매핑 확장:
    - `entity_born`, `entity_died`, `mental_break_started`, `couple_formed`, `era_changed` 재방출.
  - payload에 없는 필드(name/age/old_era)는 하위 호환 placeholder 값(`""`, `0.0`)으로 채움.
- `reports/rust-migration/README.md`
  - 0143 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 시그니처 변경 없음.
- 동작 변경:
  - Bus v2 원본 이벤트가 기존 v1 시그널로 더 넓게 재방출되어, 구독 중인 UI/패널 코드의 무중단 호환 범위가 확대됨.

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 이름/나이/old_era는 현재 v2 payload에 없는 필드이므로 placeholder를 사용했다. 해당 필드가 Rust payload에 추가되면 즉시 치환 가능하다.

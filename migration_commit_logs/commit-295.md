# Commit 295 - MovementSystem 연령별 skip 판정 Rust 브리지 이관

## 커밋 요약
- `movement_system`의 연령별 이동 skip 판정 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `movement_should_skip_tick(skip_mod, tick, entity_id) -> bool`
  - 단위 테스트 추가:
    - 모듈로 조건 기반 skip 판정 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_movement_should_skip_tick(...)`

- `scripts/systems/world/movement_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `execute_tick`의 연령별 이동 skip 판정을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 이동 시스템 hot-path 분기 판정이 Rust 경로로 이동해 tick 루프 분기 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `24/56` 적용, 잔여 `32/56`.

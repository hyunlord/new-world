# Commit 280 - ReputationSystem 수식 Rust 브리지 이관

## 커밋 요약
- `reputation_system`의 이벤트 delta/감쇠 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `reputation_event_delta(...)`
    - `reputation_decay_value(...)`
  - 단위 테스트 추가:
    - negativity bias 반영 delta 검증
    - 부호별 decay 적용 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_reputation_event_delta(...)`
    - `body_reputation_decay_value(...)`

- `scripts/systems/social/reputation_system.gd`
  - SimBridge 연결 캐시 추가.
  - `_apply_reputation_event`의 delta 계산을 Rust-first 호출로 전환(fallback 유지).
  - `_decay_all`의 domain decay 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 평판 이벤트 반영/감쇠 수식이 Rust 경로로 이동해 반복 연산 비용을 줄일 기반 확보.
- 브리지 실패 시 기존 GDScript 경로를 유지해 동작 안정성을 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `9/56` 적용, 잔여 `47/56`.

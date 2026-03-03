# Commit 307 - MoraleSystem 수식 Rust 브리지 이관

## 커밋 요약
- `morale_system`의 행동 가중치 배수 및 이주 확률 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `morale_behavior_weight_multiplier(morale, ...) -> f32`
    - `morale_migration_probability(morale_s, k, threshold_morale, patience, patience_resistance, max_probability) -> f32`
  - 단위 테스트 추가:
    - `morale_behavior_weight_multiplier_follows_band_rules`
    - `morale_migration_probability_increases_when_morale_drops`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_morale_behavior_weight_multiplier(...)`
    - `body_morale_migration_probability(...)`

- `scripts/systems/psychology/morale_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `get_behavior_weight_multiplier`를 Rust-first 호출로 전환(fallback 유지).
  - `get_migration_probability`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 모랄 기반 행동 배수/이주 확률 계산의 핵심 수식이 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 계산 경로 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 실측 기준)**: `40/56` 적용, 잔여 `16/56`.

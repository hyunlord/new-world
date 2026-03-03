# Commit 292 - MigrationSystem 의사결정 수식 Rust 브리지 이관

## 커밋 요약
- `migration_system`의 식량 부족 판정과 migration 시도 게이트 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `migration_food_scarce(nearby_food, population, per_capita_threshold) -> bool`
    - `migration_should_attempt(overcrowded, food_scarce, chance_roll, migration_chance) -> bool`
  - 단위 테스트 추가:
    - threshold 기반 식량 부족 판정 검증
    - 압력/확률 기반 시도 게이트 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_migration_food_scarce(...)`
    - `body_migration_should_attempt(...)`

- `scripts/systems/world/migration_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - 식량 부족 판정을 Rust-first 호출로 전환(fallback 유지).
  - migration 시도 게이트 판정을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 이주 시스템 초기 의사결정 수식이 Rust 경로로 이동해 반복 판단 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `21/56` 적용, 잔여 `35/56`.

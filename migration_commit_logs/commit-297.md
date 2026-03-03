# Commit 297 - BuildingEffectSystem 수식 Rust 브리지 이관

## 커밋 요약
- `building_effect_system`의 campfire/social boost 및 capped add 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `building_campfire_social_boost(is_night, day_boost, night_boost) -> f32`
    - `building_add_capped(current, delta, cap) -> f32`
  - 단위 테스트 추가:
    - `building_campfire_social_boost_selects_by_time`
    - `building_add_capped_limits_to_cap`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_building_campfire_social_boost(...)`
    - `body_building_add_capped(...)`

- `scripts/systems/work/building_effect_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_apply_campfire`의 day/night social boost 선택을 Rust-first 호출로 전환(fallback 유지).
  - `_apply_campfire`/`_apply_shelter`의 capped add(energy/warmth/safety/social) 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 건물 효과 처리에서 반복되는 분기/캡 연산 일부가 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 계산 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `26/56` 적용, 잔여 `30/56`.

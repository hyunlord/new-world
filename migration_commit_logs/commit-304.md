# Commit 304 - IntelligenceSystem 유효값 계산 Rust 브리지 이관

## 커밋 요약
- `intelligence_system`의 유효 지능 계산 수식(나이/활동/ACE/환경 패널티)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `intelligence_effective_value(potential, base_mod, age_years, is_fluid, activity_mod, ace_fluid_mult, env_penalty, min_val, max_val) -> f32`
  - 단위 테스트 추가:
    - `intelligence_effective_value_applies_fluid_decline_modifiers`
    - `intelligence_effective_value_uses_base_when_not_declining`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_intelligence_effective_value(...)`

- `scripts/systems/cognition/intelligence_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_update_effective_intelligence` 루프의 per-key effective 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 반복 루프에서 수행되는 지능 유효값 계산 핵심 수식이 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 계산 경로 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `33/56` 적용, 잔여 `23/56`.

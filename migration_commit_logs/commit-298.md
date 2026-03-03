# Commit 298 - TechPropagation 수식 Rust 브리지 이관

## 커밋 요약
- `tech_propagation_system`의 문화 보정/캐리어 보너스/최종 전파 확률 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `tech_propagation_culture_modifier(knowledge_avg, tradition_avg, ...) -> f32`
    - `tech_propagation_carrier_bonus(max_skill, ...) -> f32`
    - `tech_propagation_final_prob(base_prob, lang_penalty, ...) -> f32`
  - 단위 테스트 추가:
    - `tech_propagation_culture_modifier_applies_weights_and_clamp`
    - `tech_propagation_carrier_bonus_scales_by_skill`
    - `tech_propagation_final_prob_multiplies_and_clamps`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_tech_propagation_culture_modifier(...)`
    - `body_tech_propagation_carrier_bonus(...)`
    - `body_tech_propagation_final_prob(...)`

- `scripts/systems/world/tech_propagation_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `attempt_cross_propagation`의 최종 전파 확률 계산을 Rust-first 호출로 전환(fallback 유지).
  - `_calculate_culture_modifier`를 Rust-first 호출로 전환(fallback 유지).
  - `_calculate_carrier_bonus`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 교차 정착지 기술 전파 확률의 핵심 수식 연산 일부가 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 계산을 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `27/56` 적용, 잔여 `29/56`.

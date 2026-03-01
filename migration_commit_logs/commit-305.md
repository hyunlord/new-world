# Commit 305 - IntelligenceGenerator g 계산 Rust 브리지 이관

## 커밋 요약
- `intelligence_generator`의 `g` 계산 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `intelligence_g_value(has_parents, parent_a_g, parent_b_g, heritability_g, g_mean, openness_mean, openness_weight, noise) -> f32`
  - 단위 테스트 추가:
    - `intelligence_g_value_uses_parental_blend_when_available`
    - `intelligence_g_value_applies_openness_shift_and_noise`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_intelligence_g_value(...)`

- `scripts/systems/cognition/intelligence_generator.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_generate_g`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 지능 생성 시 `g` 계산의 핵심 수식(부모 유전/개방성/노이즈 합성)이 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 수식 경로 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `34/56` 적용, 잔여 `22/56`.

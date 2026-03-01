# Commit 051 - stress personality scale 수식 Rust 이관

## 커밋 요약
- 이벤트 기반 stress 주입 경로의 성격 스케일 계산(`_calc_personality_scale`) 핵심 수식을 Rust helper로 이관.
- HEXACO 축/facet 편차와 trait multiplier 곱셈 경로를 네이티브 계산으로 통일.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_personality_scale(values, weights, high_amplifies, trait_multipliers) -> f32`
  - 동작:
    - 방향(`high_amplifies`)에 따라 편차 계산
    - `1 + weight * deviation` 누적 곱
    - trait multiplier 누적 곱
    - 결과 clamp(0.05..4.0)
  - 단위 테스트 2개 추가:
    - 축/facet + trait 배수 적용 검증
    - 결과 clamp 범위 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_personality_scale(values, weights, high_amplifies, trait_multipliers) -> f32`
  - `PackedByteArray -> Vec<u8>` 변환 helper 추가
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_personality_scale(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_personality_scale(...)`
  - Rust 우선 + 기존 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_personality_scale(...)` 변경:
    - 기존 GDScript 내부 곱셈 수식을 Packed 입력 구성 + `StatCurveScript.stress_personality_scale(...)` 호출로 치환
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_personality_scale` 호출 추가

## 기능 영향
- 이벤트 stress 계산의 성격 배수 수식이 Rust 경로로 일원화됨.
- 기존 스케일 의미(방향/가중치/trait 곱/클램프)는 동일하게 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (41 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=273.3`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지

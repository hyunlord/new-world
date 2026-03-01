# Commit 155 - child stress parent-transfer 수식 Rust 이관

## 커밋 요약
- `child_stress_processor`의 부모→아동 스트레스 전이 수식을 Rust 함수로 이관하고, GDScript는 Rust 우선 + 기존 fallback 구조로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_parent_stress_transfer(...)` 추가.
    - attachment 코드 기반 계수(`secure/anxious/avoidant/disorganized`) 적용
    - caregiver social buffer 반영
    - contagion 결합 후 `[0,1]` clamp
  - unit test 2개 추가:
    - attachment profile 차이에 따른 전이량 비교
    - social buffer 활성 시 전이량 감소 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_child_parent_stress_transfer(...)` export 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_child_parent_stress_transfer(...)` wrapper 추가.

- `scripts/systems/development/child_stress_processor.gd`
  - execute_tick parent transfer 구간을 Rust 우선 호출로 전환.
  - `_attachment_type_to_code(attachment_type)` helper 추가.
  - Rust 미지원 시 기존 `_calculate_parent_stress_transfer(...)` fallback 유지.

## 기능 영향
- parent stress transfer 계산 의미는 기존과 동일.
- child stress tick 경로의 해당 수식이 Rust에서 계산되어 스크립트 hot path 비용을 절감.
- bridge 미지원 환경은 기존 동작 보장.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 81 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=471.6`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=204.9`, `checksum=29743414.00000`

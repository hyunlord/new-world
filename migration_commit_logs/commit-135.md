# Commit 135 - needs 에너지 소모/회복 수식 Rust 이관

## 커밋 요약
- `needs_system`의 행동 에너지 소모/휴식 회복 수식을 Rust `body` 모듈로 이관하고 bridge 경로를 연결.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `action_energy_cost(base_cost, end_norm, end_cost_reduction)` 추가.
  - `rest_energy_recovery(base_recovery, rec_norm, rec_recovery_bonus)` 추가.
  - 테스트 추가:
    - endurance 증가 시 action energy cost 감소
    - recovery stat 증가 시 rest recovery 증가

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_action_energy_cost(...) -> f32` export 함수 추가.
  - `body_rest_energy_recovery(...) -> f32` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_action_energy_cost(...)` wrapper 추가.
  - `body_rest_energy_recovery(...)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `compute_action_energy_cost(end_norm)` 추가:
    - Rust bridge 우선 호출, 미지원 시 기존 GDScript 수식 fallback.
  - `compute_rest_energy_recovery(rec_norm)` 추가:
    - Rust bridge 우선 호출, 미지원 시 기존 GDScript 수식 fallback.

- `scripts/systems/psychology/needs_system.gd`
  - 행동 중 에너지 소모 계산을 `BodyAttributes.compute_action_energy_cost`로 전환.
  - 휴식 중 에너지 회복 계산을 `BodyAttributes.compute_rest_energy_recovery`로 전환.

## 기능 영향
- 에너지 소모/회복 수식 의미와 clamp 흐름은 기존과 동일.
- bridge 지원 환경에서 needs tick의 해당 수학 연산이 Rust 경로를 우선 사용.
- bridge 미지원 환경은 기존 GDScript 계산으로 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 67 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=447.4`, `checksum=13761358.00000`

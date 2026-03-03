# Commit 125 - Body training gain Rust 이관 (브리지 연결)

## 커밋 요약
- `BodyAttributes.calc_training_gain` 수식을 Rust로 이관하고 브리지 경유 호출을 추가.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `calc_training_gain(potential, trainability, xp, training_ceiling, xp_for_full_progress)` 추가.
  - 기존 GDScript 수식과 동일한 계산:
    - `max_gain`, `xp_progress`, `xp_factor`, `train_factor`, clamp 및 int 변환.
  - 단위 테스트 3개 추가:
    - zero xp, trainability 스케일링, 상한 범위 검증.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_calc_training_gain(...) -> i32` Godot export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_calc_training_gain(...)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `calc_training_gain(axis)`에서 Rust 경로 우선 호출:
    - `_call_sim_bridge("body_calc_training_gain", [...])`
  - Rust 결과가 없을 때 기존 GDScript 수식 fallback 유지.
  - 중간 변수(`pot`, `training_ceiling`, `trainability_value`, `xp`)를 명시적으로 정리.

## 기능 영향
- 훈련 gain 계산의 수학 의미는 동일.
- bridge 사용 가능 환경에서는 training gain 계산이 네이티브 Rust 경로를 우선 사용.
- bridge 미연결/미지원 환경은 기존 GDScript 계산으로 안전 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 58 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.3`, `checksum=13761358.00000`

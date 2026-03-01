# Commit 129 - Body training gain 배치 브리지 경로 추가

## 커밋 요약
- 신체 훈련 gain 5축(`str/agi/end/tou/rec`) 계산을 단일 Rust bridge 호출로 받을 수 있는 배치 API를 추가하고, 연간 `realized` 재계산 경로를 배치 조회로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `calc_training_gains(...) -> Vec<i32>` 추가.
  - 입력 슬라이스 길이의 최소값 기준으로 배치 계산.
  - 기존 `calc_training_gain` 단건 수식을 내부 재사용.
  - 테스트 추가:
    - `batch_training_gains_match_single_calls`
    - 배치 결과가 단건 호출과 동일한지 검증.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_calc_training_gains(...) -> PackedInt32Array` export 함수 추가.
  - Packed 배열 입력(`potentials/trainabilities/xps/training_ceilings`)을 Rust 수학 함수로 전달.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_calc_training_gains(...)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `_TRAINING_GAIN_AXES`, `_TRAINING_GAIN_AXIS_COUNT` 상수 추가.
  - `calc_training_gain_batch()` 추가:
    - axis별 potential/trainability/xp/ceiling을 PackedArray로 구성.
    - Rust bridge `body_calc_training_gains` 우선 호출.
    - bridge 미지원/실패 시 기존 `calc_training_gain(axis)` 단건 계산으로 fallback.
    - 반환 형태 `{axis: gain}` Dictionary.

- `scripts/systems/biology/age_system.gd`
  - 연간 body 재계산에서 `training_gains = entity.body.calc_training_gain_batch()` 1회 계산 후 5축 루프에서 재사용.
  - 축별 `entity.body.calc_training_gain(...)` 반복 호출 제거.

## 기능 영향
- 훈련 gain 수식/결과 의미는 기존과 동일.
- bridge 지원 환경에서 연간 body 재계산 시 training gain 계산이 다중 FFI 호출에서 단일 호출로 축소.
- bridge 미지원 환경은 기존 GDScript 계산 fallback 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 63 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.5`, `checksum=13761358.00000`

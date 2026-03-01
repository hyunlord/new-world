# Commit 130 - 연간 body realized 계산 단일 Rust 호출 통합

## 커밋 요약
- 연간 신체 `realized` 재계산을 단일 Rust bridge 호출(`body_calc_realized_values`)로 통합하고, batch training gain 경로의 `trainability` 누락 동작을 기존 단건 로직과 동일하게 보정.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `calc_training_gains`에 `trainability < 0` sentinel 처리 추가(해당 축 gain=0).
  - `calc_realized_values(...) -> Vec<i32>` 추가:
    - 출력 순서: `str, agi, end, tou, rec, dr`
    - 5축은 `(potential + gain) * age_curve` 후 clamp(0~15000)
    - `dr`는 `potential * age_curve` 후 clamp(0~10000)
  - 테스트 추가:
    - `batch_training_gains_skip_negative_trainability`
    - `realized_values_match_manual_formula`

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_calc_realized_values(...) -> PackedInt32Array` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_calc_realized_values(...)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `calc_training_gain_batch()`에서 trainability 누락 축은 sentinel(`-1`)을 전달하도록 수정해 단건 함수와 동일하게 gain 0을 보장.
  - `calc_realized_values_batch(age_years)` 추가:
    - Rust bridge `body_calc_realized_values` 우선 호출.
    - bridge 미지원/실패 시 기존 `compute_age_curve_batch + calc_training_gain_batch` 조합으로 fallback.

- `scripts/systems/biology/age_system.gd`
  - 연간 재계산 경로가 `calc_realized_values_batch` 결과를 사용하도록 전환.
  - 기존 `age_curves` + `training_gains` 이중 배치 호출을 제거해 경계 호출을 1회로 축소.
  - 미사용 `BodyAttributes` preload 제거.

## 기능 영향
- `realized` 수치 의미/이벤트 방출 기준(변화량 50 이상)은 유지.
- bridge 지원 환경에서 연간 body 재계산 경로의 bridge 왕복이 추가로 축소.
- trainability 누락 축의 gain 동작이 기존 단건 함수와 정합하게 맞춰짐.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 65 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=454.4`, `checksum=13761358.00000`

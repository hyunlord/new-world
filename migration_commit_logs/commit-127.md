# Commit 127 - Body age curve 배치 브리지 경로 추가

## 커밋 요약
- 신체 나이 커브 6축(`str/agi/end/tou/rec/dr`) 계산을 단일 Rust bridge 호출로 받을 수 있는 배치 API를 추가하고, 주요 호출부를 배치 경로로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `compute_age_curves(age_years) -> [f32; 6]` 추가.
  - 축 순서를 고정(`str, agi, end, tou, rec, dr`)해 bridge 측에서 PackedFloat32Array로 직렬화 가능한 형태로 제공.
  - 테스트 추가:
    - `batch_curve_order_matches_single_curve_calls`
    - 배치 결과 인덱스별 값이 기존 `compute_age_curve` 단건 호출과 동일한지 검증.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_compute_age_curves(age_years) -> PackedFloat32Array` export 함수 추가.
  - `sim-systems::body::compute_age_curves` 결과를 packed 배열로 반환.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_compute_age_curves(age_years)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `_AGE_CURVE_AXES`, `_AGE_CURVE_AXIS_COUNT` 상수 추가.
  - `compute_age_curve_batch(age_years)` 추가:
    - Rust bridge `body_compute_age_curves`를 우선 호출.
    - bridge 미지원/실패 시 기존 `compute_age_curve` 단건 계산으로 fallback.
    - 반환 형태는 `{axis: curve}` Dictionary.

- `scripts/systems/biology/age_system.gd`
  - 연간 `realized` 재계산 구간에서 배치 커브를 1회 계산(`age_curves`) 후 재사용.
  - 5축 루프 및 `dr` 계산에서 단건 호출 대신 배치 결과 조회 사용.

- `scripts/core/entity/entity_manager.gd`
  - 엔티티 스폰 시 초기 `realized` 계산에서 배치 커브 1회 계산 후 재사용.

## 기능 영향
- 계산식과 결과 의미는 기존과 동일.
- bridge 지원 환경에서 `age_system`/`entity_manager` 경로의 body age curve 계산이 다중 FFI 호출에서 단일 호출로 축소.
- bridge 미지원 환경은 기존 GDScript 계산으로 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 61 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=481.3`, `checksum=13761358.00000`

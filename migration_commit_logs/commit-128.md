# Commit 128 - Age trainability modifier 배치 브리지 경로 추가

## 커밋 요약
- 연령 기반 훈련 효율 계산(`str/agi/end/tou/rec`)을 단일 Rust bridge 호출로 받을 수 있는 배치 API를 추가하고, 채집/건설 XP 누적 경로를 배치 조회로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `age_trainability_modifiers(age_years) -> [f32; 5]` 추가.
  - 축 순서를 고정(`str, agi, end, tou, rec`)하고 기존 단건 함수(`age_trainability_modifier`)를 내부 재사용.
  - 테스트 추가:
    - `batch_trainability_order_matches_single_calls`
    - 배치 결과가 단건 호출과 동일한지 검증.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_age_trainability_modifiers(age_years) -> PackedFloat32Array` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_age_trainability_modifiers(age_years)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `_AGE_TRAINABILITY_AXES`, `_AGE_TRAINABILITY_AXIS_COUNT` 상수 추가.
  - `get_age_trainability_modifier_batch(age_years)` 추가:
    - Rust bridge `body_age_trainability_modifiers` 우선 호출.
    - bridge 미지원/실패 시 기존 단건 함수로 fallback.
    - 반환 형태 `{axis: modifier}` Dictionary.

- `scripts/systems/work/gathering_system.gd`
  - 채집 XP 누적 구간에서 연령별 trainability 배치를 1회 계산(`_age_mods`) 후 `food/wood/stone` 분기에서 재사용.

- `scripts/systems/work/construction_system.gd`
  - 건물 완공 XP 보너스 구간에서 `str/agi` 연령 배수를 단건 2회 호출 대신 배치 1회 조회로 변경.

## 기능 영향
- trainability 수치 의미와 XP 적립 결과는 기존과 동일.
- bridge 지원 환경에서 채집/건설 경로의 trainability 계산이 다중 FFI 호출에서 단일 호출로 축소.
- bridge 미지원 환경은 기존 GDScript 계산 fallback 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 62 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=468.8`, `checksum=13761358.00000`

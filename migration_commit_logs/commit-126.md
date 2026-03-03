# Commit 126 - Body age trainability modifier Rust 이관

## 커밋 요약
- `BodyAttributes.get_age_trainability_modifier` 분기표를 Rust로 이관하고 브리지 경로를 연결.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `age_trainability_modifier(axis, age_years)` 추가.
  - 기존 GDScript 분기표와 동일한 연령 구간/값(`str/end/agi/tou/rec`)을 유지.
  - 테스트 추가:
    - unknown axis 기본값(1.0)
    - `str` 축의 연령 증가에 따른 감소 형태 검증.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_age_trainability_modifier(axis, age_years) -> f32` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_age_trainability_modifier(axis, age_years)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `get_age_trainability_modifier(axis, age_years)` 시작 시 Rust 호출 경로 추가:
    - `_call_sim_bridge("body_age_trainability_modifier", [axis, age_years])`
  - Rust 결과가 없으면 기존 GDScript 분기표 fallback 유지.

## 기능 영향
- 나이 기반 훈련 효율 수치 의미는 기존과 동일.
- bridge 사용 가능 환경에서는 trainability modifier 계산이 Rust 경로를 우선 사용.
- bridge 미연결/미지원 환경은 기존 GDScript 계산으로 안전 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 60 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=476.7`, `checksum=13761358.00000`

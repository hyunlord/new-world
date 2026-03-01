# Commit 124 - Body age curve Rust 이관 (브리지 연결)

## 커밋 요약
- `BodyAttributes.compute_age_curve` 수식을 Rust(`sim-systems`)로 이관하고, `sim-bridge`/`SimBridge`를 통해 GDScript에서 Rust 우선 호출 + fallback 구조로 연결.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs` (신규)
  - `compute_age_curve(axis, age_years)` 구현.
  - GDScript와 동일한 축별 파라미터(`str/agi/end/tou/rec/dr`) 및 수식:
    - `grow(logistic) * decl1 * decl2`, clamp `[0.02, 1.0]`
    - `dr` 축 maternal bonus 추가
  - 단위 테스트 추가:
    - unknown axis fallback, range 보장, STR 성장/감쇠 형태, DR maternal bonus 검증.

- `rust/crates/sim-systems/src/lib.rs`
  - `pub mod body;` 추가.

- `rust/crates/sim-bridge/src/lib.rs`
  - bridge export 함수 추가:
    - `body_compute_age_curve(axis: GString, age_years: f32) -> f32`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가:
    - `body_compute_age_curve(axis, age_years)`
    - native method 없을 때 `null` 반환하는 기존 fallback 정책 유지.

- `scripts/core/entity/body_attributes.gd`
  - `compute_age_curve` 시작 시 Rust 호출 경로 추가:
    - `_call_sim_bridge("body_compute_age_curve", [axis, age_years])`
    - 결과가 있을 때 Rust 값 사용, 없으면 기존 GDScript 수식 fallback.
  - SimBridge 조회 캐시 헬퍼(`_get_sim_bridge`, `_call_sim_bridge`) 추가.

## 기능 영향
- 나이 커브 계산 의미(수식/상수/clamp)는 기존과 동일.
- Rust bridge 사용 가능 환경에서는 body age curve 연산이 네이티브 경로를 우선 사용.
- bridge 미연결/미지원 환경은 기존 GDScript 계산으로 안전 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 55 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=507.8`, `checksum=13761358.00000`

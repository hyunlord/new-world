# Commit 144 - REC 전용 trainability 브리지 경로 추가

## 커밋 요약
- 휴식 XP 경로에서 문자열 축 전달 오버헤드를 줄이기 위해 `rec` 전용 age-trainability bridge 메서드를 추가하고 needs 시스템에 적용.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `body_age_trainability_modifier_rec(age_years)` export 함수 추가.
  - 내부에서 `sim-systems::body::age_trainability_modifier("rec", age_years)` 호출.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_age_trainability_modifier_rec(age_years)` wrapper 추가.

- `scripts/core/entity/body_attributes.gd`
  - `get_rec_age_trainability_modifier(age_years)` helper 추가.
  - Rust 전용 메서드 우선 호출, 미지원 시 기존 `get_age_trainability_modifier("rec", age_years)` fallback.

- `scripts/systems/psychology/needs_system.gd`
  - 휴식 XP 적립 시 `BodyAttributes.get_rec_age_trainability_modifier(...)` 사용으로 전환.

## 기능 영향
- REC trainability 값 의미는 기존과 동일.
- 휴식 경로에서 축 문자열 전달/변환 오버헤드를 줄인 전용 경로 사용.
- bridge 미지원 환경은 기존 함수 fallback 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 72 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=465.3`, `checksum=13761358.00000`

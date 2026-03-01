# Commit 162 - child stress 잔여 단순 수식 Rust 이관

## 커밋 요약
- child stress 경로의 잔여 단순 수식(부모 전이 스트레스 반영, deprivation damage 누적)을 Rust로 이관하고, GDScript를 Rust 우선 + fallback 구조로 정리.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_parent_transfer_apply_step(...) -> f32` 추가.
    - `transferred > threshold` 조건에서만 child stress를 반영하고 clamp 적용.
  - `child_deprivation_damage_step(...) -> f32` 추가.
    - developmental damage 누적 수식 이관.
  - unit test 2개 추가:
    - threshold 반영 여부 검증
    - deprivation 누적 검증(부동소수 허용 오차 기반)

- `rust/crates/sim-bridge/src/lib.rs`
  - export 추가:
    - `body_child_parent_transfer_apply_step(...)`
    - `body_child_deprivation_damage_step(...)`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가:
    - `body_child_parent_transfer_apply_step`
    - `body_child_deprivation_damage_step`

- `scripts/systems/development/child_stress_processor.gd`
  - parent stress transfer 반영 구간에서 Rust apply-step 우선 사용.
  - `_accumulate_deprivation_damage(...)`에서 Rust 누적 step 우선 사용.
  - bridge 미사용 시 기존 GDScript 수식 fallback 유지.

- `rust/crates/sim-test/src/main.rs`
  - `--bench-needs-math`에 child 수식 호출 추가:
    - `child_parent_transfer_apply_step`
    - `child_deprivation_damage_step`
  - checksum 합산 항목 확장.

## 기능 영향
- child stress 경로의 수치 의미(전이 threshold, stress clamp, deprivation 선형 누적)는 유지.
- bridge 가능 환경에서 해당 수식이 Rust 경로로 실행됨.
- bridge 미사용 환경에서도 fallback으로 동일 동작 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 93 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=808.1`, `checksum=13767388.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=538.1`, `checksum=38434752.00000` (child 잔여 수식 항목 포함으로 기준 업데이트)

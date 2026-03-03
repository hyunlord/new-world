# Commit 146 - ERG frustration 계산 Rust 경로 이관

## 커밋 요약
- needs 시스템의 ERG frustration tick 계산(성장/관계 좌절 누적 및 회귀 전이 판정)을 Rust 배치 함수로 이관하고, GDScript는 결과 적용 + 이벤트 방출만 담당하도록 정리.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `erg_frustration_step(...) -> [i32; 6]` 추가.
    - 출력: `[growth_ticks, relatedness_ticks, growth_regressing, growth_started, relatedness_regressing, relatedness_started]`.
    - 기존 GDScript 로직과 동일하게:
      - 좌절 시 `+1`
      - 비좌절 시 `-10` 후 `>=0` clamp
      - `ERG_FRUSTRATION_WINDOW` 기준 회귀 상태/시작 여부 계산
  - unit test 2개 추가:
    - 임계 진입 시 started 플래그 검증
    - 비좌절 시 tick 회복/플래그 해제 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_erg_frustration_step_packed(scalar_inputs, flag_inputs)` 추가.
  - packed 입력 디코딩 후 `sim-systems::body::erg_frustration_step` 호출, 결과를 `PackedInt32Array`로 반환.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_erg_frustration_step_packed(...)` wrapper 추가.

- `scripts/systems/psychology/needs_system.gd`
  - scratch packed 버퍼 추가:
    - `_ERG_FRUSTRATION_SCALAR_COUNT = 10`
    - `_ERG_FRUSTRATION_FLAG_COUNT = 2`
    - `_erg_frustration_scalar_inputs`, `_erg_frustration_flag_inputs`
  - `execute_tick`에서 버퍼 resize 재사용 처리 추가.
  - `_update_erg_frustration`를 Rust 우선 경로로 전환:
    - packed 입력 1회 구성 → bridge 호출 → tick/회귀 상태/시작 플래그 반영
    - bridge 미지원 시 기존 GDScript 계산 fallback 유지
    - 스트레스 주입/`erg_regression_started` 이벤트는 시작 플래그 기반으로 동일 의미 유지

## 기능 영향
- ERG frustration 수치/이벤트 의미는 유지.
- entity 루프에서 해당 계산의 분기/산술을 Rust로 위임해 hot path 계산 비용과 스크립트 오버헤드를 완화.
- bridge 미지원 환경에서도 기존 fallback 로직으로 동작 보장.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 74 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=461.9`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=167.7`, `checksum=29719684.00000`

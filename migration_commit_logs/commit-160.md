# Commit 160 - child stress apply-step 수식 Rust 이관

## 커밋 요약
- `child_stress_processor.process_stressor`의 positive/tolerable/toxic 상태 업데이트 수식을 Rust 함수로 이관하고, GDScript는 이벤트/메타 반영만 유지하도록 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_stress_apply_step(...)` 추가.
    - 입력: 현재 resilience/reserve/stress/allostatic + intensity 계수들 + stress_type_code
    - 출력: `[next_resilience, next_reserve, next_stress, next_allostatic, developmental_damage_delta]`
  - 관련 수식 테스트 2개 추가(분기 shape 및 toxic damage delta).
  - 보완 함수 추가:
    - `child_shrp_step(...)` (SHRP 수식)
    - `child_stress_type_code(...)` (분류 코드)
  - 관련 테스트 3개 추가.

- `rust/crates/sim-bridge/src/lib.rs`
  - export 추가:
    - `body_child_shrp_step(...)`
    - `body_child_stress_type_code(...)`
    - `body_child_stress_apply_step(...)`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가:
    - `body_child_shrp_step`
    - `body_child_stress_type_code`
    - `body_child_stress_apply_step`

- `scripts/systems/development/child_stress_processor.gd`
  - `_apply_shrp(...)`가 Rust 결과(`[adjusted_intensity, override_flag]`) 우선 적용.
  - `_classify_stress_type(...)`가 Rust 분류 코드 우선 적용.
  - `process_stressor(...)`의 상태 업데이트가 Rust apply-step 결과 우선 반영.
  - fallback으로 기존 분기 수식 유지.
  - helper 추가: `_stress_type_to_code(...)`, `_stress_type_from_code(...)`.

- `rust/crates/sim-test/src/main.rs`
  - `--bench-needs-math`에 child 수식 호출 추가:
    - `child_shrp_step`
    - `child_stress_type_code`
    - `child_stress_apply_step`
  - checksum 합산 항목 확장.

## 기능 영향
- child stress 분기별 수치 의미는 유지.
- child stress 핵심 분기 계산 경로의 네이티브화 범위 확대.
- Chronicle 이벤트/로그 부작용은 GDScript에 남겨 기존 동작을 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 89 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=494.6`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=228.9`, `checksum=33378700.00000` (child apply-step 항목 포함으로 기준 업데이트)

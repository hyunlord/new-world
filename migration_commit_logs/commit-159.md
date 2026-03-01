# Commit 159 - child SHRP/스트레스 분류 수식 Rust 이관

## 커밋 요약
- `child_stress_processor`의 SHRP 적용 수식과 stress type 분류 로직을 Rust 함수로 이관하고, GDScript는 이벤트 부작용 처리만 유지하도록 정리.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_shrp_step(intensity, shrp_active, shrp_override_threshold, vulnerability_mult)` 추가.
    - 반환: `[adjusted_intensity, override_flag]`
  - `child_stress_type_code(intensity, attachment_present, attachment_quality)` 추가.
    - 반환 코드: `0=positive`, `1=tolerable`, `2=toxic`
  - unit test 3개 추가:
    - SHRP threshold 미만 차단
    - SHRP override 플래그/배수 적용
    - stress type 분류 threshold 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_child_shrp_step(...) -> PackedFloat32Array` export 추가.
  - `body_child_stress_type_code(...) -> i32` export 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가:
    - `body_child_shrp_step(...)`
    - `body_child_stress_type_code(...)`

- `scripts/systems/development/child_stress_processor.gd`
  - `_apply_shrp(...)`가 Rust 결과(`[intensity, override_flag]`)를 우선 사용.
  - `_classify_stress_type(...)`가 Rust 분류 코드를 우선 사용.
  - toxic onset Chronicle 이벤트 로깅은 기존과 동일하게 GDScript에서 유지.
  - `_stress_type_from_code(...)` helper 추가.

## 기능 영향
- SHRP/분류 결과 의미는 기존과 동일.
- child stress 경로에서 해당 계산의 네이티브 실행 비중 증가.
- bridge 미지원 시 기존 GDScript fallback 경로 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 87 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=575.2`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=245.3`, `checksum=29781070.00000`

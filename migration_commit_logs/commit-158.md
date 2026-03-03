# Commit 158 - child social buffer 수식 Rust 경로 이관

## 커밋 요약
- `child_stress_processor`의 social buffer 감쇠 수식을 Rust 함수로 이관하고, GDScript는 Rust 우선 + 기존 fallback 구조로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_social_buffered_intensity(intensity, attachment_quality, caregiver_present, buffer_power)` 추가.
  - caregiver 부재 시 원값 반환, 존재 시 `attachment_quality * buffer_power` 감쇠 적용.
  - unit test 1개 추가(지원 존재 시 감쇠 검증).

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_child_social_buffered_intensity(...)` export 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_child_social_buffered_intensity(...)` wrapper 추가.

- `scripts/systems/development/child_stress_processor.gd`
  - `_apply_social_buffer(...)`가 Rust bridge를 우선 호출.
  - bridge 미지원 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- social buffering 계산 의미는 유지.
- child stress 처리 경로의 감쇠 수식이 Rust 실행 경로를 사용하도록 확장.
- bridge 미지원 환경은 기존 동작 보장.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 84 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=589.9`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=192.8`, `checksum=29781070.00000`

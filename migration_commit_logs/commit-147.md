# Commit 147 - anxious attachment stress 수식 Rust 이관

## 커밋 요약
- needs 시스템의 anxious attachment 저강도 스트레스 누적 수식을 Rust 함수로 이관하고 GDScript에서 Rust 우선 호출 + fallback 구조로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `anxious_attachment_stress_delta(social, social_threshold, stress_rate)` 추가.
  - 로직: `social < threshold`일 때만 `stress_rate` 반환, 그 외 `0.0`.
  - unit test 2개 추가:
    - threshold 미만일 때 delta 적용
    - threshold 이상/동일 시 delta 0 유지

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_anxious_attachment_stress_delta(...) -> f32` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_anxious_attachment_stress_delta(...)` wrapper 추가.

- `scripts/systems/psychology/needs_system.gd`
  - anxious attachment 분기에서 `SimBridge.body_anxious_attachment_stress_delta(...)` 호출.
  - bridge 미지원 시 기존 `social < threshold` 비교 fallback 유지.
  - 적용 시 `entity.emotion_data.stress` clamp 동작은 기존과 동일.

## 기능 영향
- anxious attachment 스트레스 증가 의미/조건은 기존과 동일.
- needs tick hot path에서 해당 분기 수식이 Rust 경로로 이동.
- bridge 미지원 환경에서도 fallback으로 기존 결과 보장.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 76 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=464.9`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=168.0`, `checksum=29719684.00000`

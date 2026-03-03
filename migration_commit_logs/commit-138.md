# Commit 138 - needs 기본 소모 계산 배치 Rust step 추가

## 커밋 요약
- hunger/energy/social + (옵션) thirst/warmth 소모를 한 번에 계산하는 Rust 배치 스텝(`body_needs_base_decay_step`)을 추가하고, needs tick에서 단일 결과를 재사용하도록 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `needs_base_decay_step(...) -> [f32; 5]` 추가.
    - 반환 순서: `[hunger_decay, energy_decay, social_decay, thirst_decay, warmth_decay]`
    - 기존 `thirst_decay`/`warmth_decay` 수식을 내부 재사용.
  - 테스트 추가:
    - `base_decay_step_matches_manual_formula`

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_needs_base_decay_step(...) -> PackedFloat32Array` export 추가.
  - Godot 파라미터 수 제한 회피를 위해 입력을 `PackedFloat32Array` + `PackedByteArray`로 인코딩해 수신.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_base_decay_step(...)` wrapper 추가.
  - wrapper 내부에서 scalar/flag 입력을 packed 배열로 인코딩해 bridge에 전달.

- `scripts/systems/psychology/needs_system.gd`
  - needs tick 초반에서 `body_needs_base_decay_step`를 1회 호출.
  - 성공 시 hunger/energy/social decay와 thirst/warmth decay를 배치 결과에서 직접 적용.
  - 미지원 시 기존 GDScript 수식 fallback 유지.
  - 하위 호환을 위해 base step 미지원 환경에서는 기존 `body_needs_temp_decay_step` fallback도 유지.

## 기능 영향
- needs 기본 소모 수식 의미는 기존과 동일.
- bridge 지원 환경에서 needs tick의 기본 소모 계산이 단일 배치 호출 중심으로 통합.
- bridge 미지원 환경은 기존 GDScript 계산 경로로 안전 fallback.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 71 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=454.0`, `checksum=13761358.00000`

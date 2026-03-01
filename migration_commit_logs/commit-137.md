# Commit 137 - needs 온도 소모 계산 단일 Rust 호출 통합

## 커밋 요약
- 갈증/체온 온도 소모 계산을 단일 Rust 호출(`body_needs_temp_decay_step`)로 통합해 needs tick의 bridge 왕복 수를 줄임.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `needs_temp_decay_step(...) -> [f32; 2]` 추가.
    - 반환 순서: `[thirst_decay, warmth_decay]`
    - 내부에서 기존 `thirst_decay`/`warmth_decay` 함수를 재사용.
  - 테스트 추가:
    - `combined_temp_decay_matches_individual_functions`

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_needs_temp_decay_step(...) -> PackedFloat32Array` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_temp_decay_step(...)` wrapper 추가.

- `scripts/systems/psychology/needs_system.gd`
  - 엔티티별 needs tick에서 온도 소모 계산 전에 `body_needs_temp_decay_step`를 1회 호출.
  - 갈증/체온 소모 블록에서 해당 결과를 재사용.
  - bridge 미지원 시 기존 GDScript 분기 수식 fallback 유지.

## 기능 영향
- 갈증/체온 온도 소모 수식 의미는 기존과 동일.
- bridge 지원 환경에서 needs tick의 온도 소모 계산이 2회 호출에서 1회 호출로 축소.
- bridge 미지원 환경은 기존 GDScript 계산 fallback 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 70 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=463.2`, `checksum=13761358.00000`
